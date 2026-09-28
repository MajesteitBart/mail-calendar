# Windows: a message being written is not lost to a failed file picker or an unasked close

Platforms: windows
Bump: patch

> The composer's Attach picker and the signature editor's image picker threw into an async void
> handler, which ends the process; a second click while the picker was open was enough. Both now
> allow one picker at a time and report a failure on their own error line. A message withdrawn
> from the Outbox for editing counted as untouched, so opening another message closed it without
> the discard prompt, and that composer held the only copy.

**English**

```
On Windows, a message you are writing is no longer lost when attaching a file fails, and a message taken back from the Outbox to edit is no longer closed without asking.
```

**Nederlands**

```
Op Windows gaat een bericht dat je schrijft niet meer verloren als het toevoegen van een bestand mislukt, en een bericht dat je uit Postvak UIT terugneemt om te bewerken wordt niet meer zonder te vragen gesloten.
```

**Deutsch**

```
Unter Windows geht eine Nachricht, die Sie schreiben, nicht mehr verloren, wenn das Anhängen einer Datei fehlschlägt, und eine zum Bearbeiten aus dem Postausgang zurückgeholte Nachricht wird nicht mehr ohne Nachfrage geschlossen.
```

**Français**

```
Sous Windows, un message en cours de rédaction n'est plus perdu quand l'ajout d'un fichier échoue, et un message repris de la boîte d'envoi pour être modifié n'est plus fermé sans demande de confirmation.
```

**Español**

```
En Windows, un mensaje que estás escribiendo ya no se pierde si falla al adjuntar un archivo, y un mensaje que recuperas de la bandeja de salida para editarlo ya no se cierra sin preguntar.
```

**Italiano**

```
Su Windows un messaggio che stai scrivendo non va più perso se l'aggiunta di un file non riesce, e un messaggio ripreso dalla posta in uscita per modificarlo non viene più chiuso senza chiedere.
```

**Português**

```
No Windows, uma mensagem que está a escrever já não se perde quando falha a adição de um ficheiro, e uma mensagem retirada da caixa de saída para editar já não é fechada sem perguntar.
```
