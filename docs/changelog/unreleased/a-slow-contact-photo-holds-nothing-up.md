# A slow contact photo holds nothing up

Platforms: all
Bump: patch

> The photo pass fetched from a contacts provider while holding the account list's read lock.
> tokio's lock lets a waiting writer go first, so adding or removing an account queued behind the
> fetch, and every account read after it queued too: snapshot rebuilds, sends, edits, sync. The
> pass now takes its own handle to the account before the fetch. Separately, the republish that
> takes an archived or deleted row off the list ran the photo pass before the write was handed to
> the outbox; it now publishes only, and the rebuild that follows the write looks up the senders
> it left unresolved.

**English**

```
A contact photo that is slow to download no longer holds up archiving or deleting a message, or adding or removing an account.
```

**Nederlands**

```
Een contactfoto die traag binnenkomt, houdt het archiveren of verwijderen van een bericht en het toevoegen of verwijderen van een account niet langer op.
```

**Deutsch**

```
Ein Kontaktfoto, das langsam lädt, hält das Archivieren oder Löschen einer Nachricht und das Hinzufügen oder Entfernen eines Kontos nicht mehr auf.
```

**Français**

```
Une photo de contact lente à télécharger ne retarde plus l'archivage ou la suppression d'un message, ni l'ajout ou le retrait d'un compte.
```

**Español**

```
Una foto de contacto que tarda en descargarse ya no retrasa el archivado o la eliminación de un mensaje, ni añadir o quitar una cuenta.
```

**Italiano**

```
Una foto del contatto lenta da scaricare non rallenta più l'archiviazione o l'eliminazione di un messaggio, né l'aggiunta o la rimozione di un account.
```

**Português**

```
Uma fotografia de contacto que demora a transferir já não atrasa o arquivo ou a eliminação de uma mensagem, nem a adição ou remoção de uma conta.
```
