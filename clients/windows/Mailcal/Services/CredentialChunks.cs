// How one value too large for a single Credential Manager entry is kept across several, and
// replaced without a moment at which it reads as neither the old value nor the new one.
//
// Windows caps one credential's blob at CRED_MAX_CREDENTIAL_BLOB_SIZE (2560 bytes), and a Microsoft
// refresh-token config can exceed that, so a value is split. Writing the pieces over the previous
// ones in place is not safe: a refusal after the first piece leaves a value made of new and old
// halves, which is a broken sign-in, or a rotated token lost with the old one already overwritten.
// So a split value is written beside the current one and published by a single write of the base
// entry, which Windows performs whole or not at all.
//
// Three layouts can be found under a base target:
//
//   plain     the base entry holds the whole value. What a value that fits is written as, so a
//             small value reads the same to an earlier build.
//   manifest  the base entry holds a record naming a generation, a piece count, a length and a
//             SHA-256; the pieces sit at "<base>:gen<1|2>:<n>". Writes alternate between the two
//             generations, so the pieces a manifest names are never the ones being written.
//   legacy    the base entry holds the first piece and "<base>:1", "<base>:2", … the rest. Earlier
//             builds wrote this; it is read, and replaced by the next write, but never written.
//
// WinUI- and P/Invoke-free on purpose, so the replacement is tested against a store that refuses a
// write part-way (CredentialChunksTests). CredentialStore is the advapi32 half.

using System.Security.Cryptography;
using System.Text;

namespace Allodia.Mailcal.Services;

/// <summary>The three Credential Manager operations the chunked layout needs, by target name.</summary>
internal interface ICredentialSlots
{
    /// <summary>The entry's bytes, or <c>null</c> when there is no such entry.</summary>
    byte[]? Read(string target);

    /// <summary>Creates or replaces the entry whole. <c>false</c> means nothing changed.</summary>
    bool Write(string target, ReadOnlySpan<byte> bytes);

    /// <summary>Deletes the entry. <c>false</c> when it could not be, or was not there.</summary>
    bool Delete(string target);
}

internal static class CredentialChunks
{
    /// <summary>Comfortably under the 2560-byte cap, so a piece never trips it.</summary>
    internal const int MaxChunkBytes = 2048;

    // Neither TOML nor JSON can begin with a control character, so no plain value is mistaken for
    // a manifest. A value that does begin with one is always written as a manifest.
    private const byte Marker = 0x01;
    private const string ManifestTag = "mailcal-chunks";

    /// <summary>The stored value, or <c>null</c> when there is none, or when the pieces a manifest
    /// names do not add up to what it recorded.</summary>
    public static string? Read(ICredentialSlots slots, string baseTarget)
    {
        var head = slots.Read(baseTarget);
        if (head is null || head.Length == 0)
        {
            return null;
        }
        if (Manifest.Parse(head) is { } manifest)
        {
            return manifest.Assemble(slots, baseTarget);
        }
        if (head[0] == Marker)
        {
            return null;   // a manifest this build cannot parse; its pieces cannot be trusted
        }
        var all = new List<byte>(head);
        for (var n = 1; slots.Read(LegacyTarget(baseTarget, n)) is { } more; n++)
        {
            all.AddRange(more);
        }
        return Encoding.UTF8.GetString(all.ToArray());
    }

    /// <summary>Replaces the stored value. <c>false</c> means the write was refused and the value
    /// that was stored before is still the one that reads back.</summary>
    public static bool Write(ICredentialSlots slots, string baseTarget, string value)
    {
        var bytes = Encoding.UTF8.GetBytes(value);
        var current = slots.Read(baseTarget) is { } head ? Manifest.Parse(head) : null;
        var legacyPieces = slots.Read(LegacyTarget(baseTarget, 1)) is not null;

        // A plain write is one entry, so it is atomic by itself. It is safe only when no legacy
        // piece is left for a reader to append to it.
        if (bytes.Length <= MaxChunkBytes && (bytes.Length == 0 || bytes[0] != Marker) && !legacyPieces)
        {
            if (!slots.Write(baseTarget, bytes))
            {
                return false;
            }
            if (current is not null)
            {
                DeleteRun(slots, n => PartTarget(baseTarget, current.Generation, n), from: 0);
            }
            return true;
        }

        var generation = current?.Generation == 1 ? 2 : 1;
        var count = Math.Max(1, (bytes.Length + MaxChunkBytes - 1) / MaxChunkBytes);
        for (var n = 0; n < count; n++)
        {
            var offset = n * MaxChunkBytes;
            var length = Math.Min(MaxChunkBytes, bytes.Length - offset);
            if (!slots.Write(PartTarget(baseTarget, generation, n), bytes.AsSpan(offset, length)))
            {
                DeleteRun(slots, k => PartTarget(baseTarget, generation, k), from: 0);
                return false;
            }
        }
        var record = new Manifest(generation, count, bytes.Length, Convert.ToHexString(SHA256.HashData(bytes)));
        if (!slots.Write(baseTarget, record.Encode()))
        {
            DeleteRun(slots, k => PartTarget(baseTarget, generation, k), from: 0);
            return false;
        }

        // Published. What is left is housekeeping, and a failure in it costs an orphaned entry,
        // never the value: nothing reads the other generation or a legacy piece past a manifest.
        DeleteRun(slots, k => PartTarget(baseTarget, generation, k), from: count);
        DeleteRun(slots, k => PartTarget(baseTarget, 3 - generation, k), from: 0);
        DeleteRun(slots, k => LegacyTarget(baseTarget, k), from: 1);
        return true;
    }

    /// <summary>Deletes the value and every piece of it. <c>true</c> when nothing is stored under
    /// <paramref name="baseTarget"/> any more.</summary>
    public static bool Delete(ICredentialSlots slots, string baseTarget)
    {
        // The base entry first: once it is gone the value is gone, and a piece left behind by a
        // refused delete is an orphan nothing reads.
        if (!slots.Delete(baseTarget) && slots.Read(baseTarget) is not null)
        {
            return false;
        }
        DeleteRun(slots, k => PartTarget(baseTarget, 1, k), from: 0);
        DeleteRun(slots, k => PartTarget(baseTarget, 2, k), from: 0);
        DeleteRun(slots, k => LegacyTarget(baseTarget, k), from: 1);
        return true;
    }

    private static string LegacyTarget(string baseTarget, int n) => $"{baseTarget}:{n}";

    private static string PartTarget(string baseTarget, int generation, int n) =>
        $"{baseTarget}:gen{generation}:{n}";

    // Deletes consecutive entries until the first that is not there, or that will not go.
    private static void DeleteRun(ICredentialSlots slots, Func<int, string> target, int from)
    {
        for (var n = from; slots.Delete(target(n)); n++)
        {
        }
    }

    private sealed record Manifest(int Generation, int Count, int Length, string Sha256)
    {
        public byte[] Encode()
        {
            var text = $"{ManifestTag};{Generation};{Count};{Length};{Sha256}";
            return [Marker, .. Encoding.ASCII.GetBytes(text)];
        }

        public static Manifest? Parse(byte[] head)
        {
            if (head.Length < 2 || head[0] != Marker)
            {
                return null;
            }
            var fields = Encoding.ASCII.GetString(head, 1, head.Length - 1).Split(';');
            if (fields.Length != 5
                || fields[0] != ManifestTag
                || !int.TryParse(fields[1], out var generation) || generation is not (1 or 2)
                || !int.TryParse(fields[2], out var count) || count < 1
                || !int.TryParse(fields[3], out var length) || length < 0)
            {
                return null;
            }
            return new Manifest(generation, count, length, fields[4]);
        }

        // The bytes are joined before the UTF-8 decode, so a character straddling two pieces still
        // decodes.
        public string? Assemble(ICredentialSlots slots, string baseTarget)
        {
            var all = new List<byte>(Length);
            for (var n = 0; n < Count; n++)
            {
                if (slots.Read(PartTarget(baseTarget, Generation, n)) is not { } piece)
                {
                    return null;
                }
                all.AddRange(piece);
            }
            var bytes = all.ToArray();
            return bytes.Length == Length && Convert.ToHexString(SHA256.HashData(bytes)) == Sha256
                ? Encoding.UTF8.GetString(bytes)
                : null;
        }
    }
}
