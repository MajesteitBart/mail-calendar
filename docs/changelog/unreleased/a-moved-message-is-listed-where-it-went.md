# A message you archive or delete is listed where it went

Platforms: all
Bump: patch

> The optimistic hide that takes a moved row off the list waited for the row's key to leave the
> store. JMAP, Gmail and Graph keep the key across a move, so on those accounts the row stayed
> hidden everywhere, its new folder included, until it aged out of the loaded window or the app
> restarted: nothing could be restored from Trash or marked not-spam from Junk. Each hide now
> carries its destination and also ends once the row is filed there. The same applies to a spam
> report and its correction, which the note leaves out because not every client offers them.

**English**

```
A message you archive or delete now appears in Archive or Trash straight away, rather than staying hidden there until the app is restarted.
```

**Nederlands**

```
Een bericht dat je archiveert of verwijdert, staat nu meteen in Archief of Prullenbak, in plaats van daar verborgen te blijven tot de app opnieuw start.
```

**Deutsch**

```
Eine Nachricht, die Sie archivieren oder löschen, erscheint jetzt sofort im Archiv oder Papierkorb, statt dort bis zum Neustart der App verborgen zu bleiben.
```

**Français**

```
Un message que vous archivez ou supprimez apparaît désormais aussitôt dans Archives ou Corbeille, au lieu d'y rester masqué jusqu'au redémarrage de l'application.
```

**Español**

```
Los mensajes que archiva o elimina aparecen ahora de inmediato en Archivo o Papelera, en lugar de quedar ocultos allí hasta reiniciar la aplicación.
```

**Italiano**

```
Un messaggio che archivi o elimini ora compare subito in Archivio o nel Cestino, invece di restarvi nascosto fino al riavvio dell'app.
```

**Português**

```
Uma mensagem que arquiva ou elimina aparece agora de imediato em Arquivo ou Reciclagem, em vez de ficar lá oculta até a aplicação ser reiniciada.
```
