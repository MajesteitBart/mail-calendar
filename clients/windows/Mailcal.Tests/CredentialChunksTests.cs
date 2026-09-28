using System.Text;
using Allodia.Mailcal.Services;
using Xunit;

namespace Allodia.Mailcal.Tests;

/// <summary>
/// The Credential Manager layout for a value larger than one entry. What it guards is silent on
/// screen until the next launch: a refused write that left new and old pieces side by side reads
/// back as a config nobody wrote, which is a Microsoft account that no longer signs in, or a
/// rotated refresh token lost with the old one already overwritten.
/// </summary>
public sealed class CredentialChunksTests
{
    private const string Base = "app:account:alice@example.com@graph.microsoft.com";

    /// <summary>An in-memory Credential Manager that can be told to refuse a write.</summary>
    private sealed class FakeSlots : ICredentialSlots
    {
        public readonly Dictionary<string, byte[]> Entries = new();

        /// <summary>Writes that succeed before every later one is refused; null for never.</summary>
        public int? WritesAllowed;

        public int Writes;

        public byte[]? Read(string target) => Entries.TryGetValue(target, out var v) ? v : null;

        public bool Write(string target, ReadOnlySpan<byte> bytes)
        {
            if (WritesAllowed is { } allowed && Writes >= allowed)
            {
                return false;
            }
            Writes++;
            Entries[target] = bytes.ToArray();
            return true;
        }

        public bool Delete(string target) => Entries.Remove(target);
    }

    private static string Config(int bytes, char fill) =>
        "refresh_token = \"" + new string(fill, bytes) + "\"\n";

    [Fact]
    public void A_value_that_fits_is_one_plain_entry()
    {
        var slots = new FakeSlots();
        Assert.True(CredentialChunks.Write(slots, Base, "user = \"a\""));
        // Plain, so an earlier build reads it too.
        Assert.Equal("user = \"a\"", Encoding.UTF8.GetString(slots.Entries[Base]));
        Assert.Single(slots.Entries);
        Assert.Equal("user = \"a\"", CredentialChunks.Read(slots, Base));
    }

    [Fact]
    public void A_large_value_round_trips_across_pieces()
    {
        var slots = new FakeSlots();
        var value = Config(5000, 'x');
        Assert.True(CredentialChunks.Write(slots, Base, value));
        Assert.True(slots.Entries.Count > 2);
        Assert.All(slots.Entries.Values, v => Assert.True(v.Length <= CredentialChunks.MaxChunkBytes + 128));
        Assert.Equal(value, CredentialChunks.Read(slots, Base));
    }

    [Fact]
    public void A_character_straddling_two_pieces_still_decodes()
    {
        var slots = new FakeSlots();
        // Three-byte characters, so a piece boundary falls inside one.
        var value = new string('€', 3000);
        Assert.True(CredentialChunks.Write(slots, Base, value));
        Assert.Equal(value, CredentialChunks.Read(slots, Base));
    }

    /// <summary>The contract: a refused write, at whichever write it is refused, leaves the old
    /// value readable, and a write that reported success reads back as the new one.</summary>
    [Theory]
    [InlineData(5000, 7000)]   // split over split
    [InlineData(5000, 100)]    // split replaced by plain
    [InlineData(100, 5000)]    // plain replaced by split
    public void A_refused_write_leaves_the_old_value_whole(int oldSize, int newSize)
    {
        var oldValue = Config(oldSize, 'o');
        var newValue = Config(newSize, 'n');
        for (var allowed = 0; ; allowed++)
        {
            var slots = new FakeSlots();
            Assert.True(CredentialChunks.Write(slots, Base, oldValue));
            slots.Writes = 0;
            slots.WritesAllowed = allowed;

            var landed = CredentialChunks.Write(slots, Base, newValue);

            Assert.Equal(landed ? newValue : oldValue, CredentialChunks.Read(slots, Base));
            if (landed)
            {
                break;
            }
        }
    }

    [Fact]
    public void Repeated_replacements_leave_no_pieces_behind()
    {
        var slots = new FakeSlots();
        Assert.True(CredentialChunks.Write(slots, Base, Config(7000, 'a')));
        var afterFirst = slots.Entries.Count;
        Assert.True(CredentialChunks.Write(slots, Base, Config(7000, 'b')));
        Assert.True(CredentialChunks.Write(slots, Base, Config(7000, 'c')));
        Assert.Equal(afterFirst, slots.Entries.Count);

        Assert.True(CredentialChunks.Write(slots, Base, "small"));
        Assert.Single(slots.Entries);
    }

    /// <summary>What an earlier build left: the first piece at the base, the rest at ":1", ":2".
    /// It must still read, and the next write must not leave those pieces for a reader to append
    /// to the new value.</summary>
    [Fact]
    public void A_value_an_earlier_build_split_reads_and_is_replaced_cleanly()
    {
        var slots = new FakeSlots();
        var value = Config(5000, 'l');
        var bytes = Encoding.UTF8.GetBytes(value);
        slots.Entries[Base] = bytes[..2048];
        slots.Entries[Base + ":1"] = bytes[2048..4096];
        slots.Entries[Base + ":2"] = bytes[4096..];
        Assert.Equal(value, CredentialChunks.Read(slots, Base));

        Assert.True(CredentialChunks.Write(slots, Base, "small"));
        Assert.Equal("small", CredentialChunks.Read(slots, Base));
        Assert.False(slots.Entries.ContainsKey(Base + ":1"));
        Assert.False(slots.Entries.ContainsKey(Base + ":2"));
    }

    [Fact]
    public void Replacing_an_earlier_builds_split_value_is_atomic_too()
    {
        var value = Config(5000, 'l');
        var bytes = Encoding.UTF8.GetBytes(value);
        for (var allowed = 0; ; allowed++)
        {
            var slots = new FakeSlots { WritesAllowed = allowed };
            slots.Entries[Base] = bytes[..2048];
            slots.Entries[Base + ":1"] = bytes[2048..4096];
            slots.Entries[Base + ":2"] = bytes[4096..];

            var landed = CredentialChunks.Write(slots, Base, "small");

            Assert.Equal(landed ? "small" : value, CredentialChunks.Read(slots, Base));
            if (landed)
            {
                break;
            }
        }
    }

    [Fact]
    public void Pieces_that_do_not_match_their_record_read_as_nothing()
    {
        var slots = new FakeSlots();
        Assert.True(CredentialChunks.Write(slots, Base, Config(5000, 'x')));
        var piece = slots.Entries.Keys.First(k => k.EndsWith(":1", StringComparison.Ordinal) && k != Base + ":1");
        slots.Entries[piece] = Encoding.UTF8.GetBytes(new string('y', CredentialChunks.MaxChunkBytes));
        Assert.Null(CredentialChunks.Read(slots, Base));
    }

    [Fact]
    public void Delete_removes_every_piece_and_reports_an_absent_value_as_gone()
    {
        var slots = new FakeSlots();
        Assert.True(CredentialChunks.Write(slots, Base, Config(7000, 'x')));
        Assert.True(CredentialChunks.Delete(slots, Base));
        Assert.Empty(slots.Entries);
        Assert.Null(CredentialChunks.Read(slots, Base));
        Assert.True(CredentialChunks.Delete(slots, Base));
    }
}
