# Windows: an account's sign-in survives a save that Windows refuses part-way

Platforms: windows
Bump: patch

> A Microsoft refresh-token config is larger than one Credential Manager entry, so it is split.
> The pieces used to be overwritten in place, and a write refused after the first piece left a
> config made of new and old halves. A split value is now written beside the current one and
> published by a single write of its base entry, so a refusal leaves the previous config readable.
> The credential failure log line no longer carries the account id, which is an address.

**English**

```
On Windows, an account stays signed in when Windows refuses to save its updated sign-in details part-way.
```

**Nederlands**

```
Op Windows blijft een account aangemeld als Windows het opslaan van bijgewerkte aanmeldgegevens halverwege weigert.
```

**Deutsch**

```
Unter Windows bleibt ein Konto angemeldet, wenn Windows das Speichern aktualisierter Anmeldedaten mittendrin verweigert.
```

**Français**

```
Sous Windows, un compte reste connecté quand Windows refuse en cours de route d'enregistrer ses identifiants mis à jour.
```

**Español**

```
En Windows, una cuenta sigue con la sesión iniciada cuando Windows rechaza a medias guardar sus datos de acceso actualizados.
```

**Italiano**

```
Su Windows un account resta connesso quando Windows rifiuta a metà il salvataggio delle credenziali aggiornate.
```

**Português**

```
No Windows, uma conta mantém a sessão iniciada quando o Windows recusa a meio guardar os dados de acesso atualizados.
```
