# Offer to save the Sent copy of a message the Outbox sent

Platforms: all
Bump: patch

> The inline send already raised the unfiled-copy question; a queued send delivered by a drain
> (reconnecting, a sync pass or Send now) did not, because nothing read the drain's
> `SentNotFiled` outcome. The drain now reads the queued messages before it runs, since a settled
> op leaves the outbox, and raises the same question. Only IMAP can reach this: the other
> transports file the copy on the server.

**English**

```
When a message from the Outbox goes out without a copy in Sent, the app now offers to save one instead of leaving it missing unnoticed.
```

**Nederlands**

```
Als een bericht uit Postvak UIT wordt verzonden zonder kopie in Verzonden, biedt de app nu aan er een op te slaan, in plaats van dat die kopie ongemerkt ontbreekt.
```

**Deutsch**

```
Wird eine Nachricht aus dem Postausgang ohne Kopie in „Gesendet“ verschickt, bietet die App jetzt an, eine zu speichern, statt dass die Kopie unbemerkt fehlt.
```

**Français**

```
Lorsqu’un message de la boîte d’envoi part sans copie dans Envoyés, l’application propose désormais d’en enregistrer une, au lieu de la laisser manquer sans rien dire.
```

**Español**

```
Cuando un mensaje de la bandeja de salida se envía sin copia en Enviados, la aplicación ahora ofrece guardarla, en lugar de que falte sin avisar.
```

**Italiano**

```
Quando un messaggio della Posta in uscita parte senza una copia in Inviati, l’app ora propone di salvarla, invece di lasciarla mancare senza avvisare.
```

**Português**

```
Quando uma mensagem da caixa de saída é enviada sem cópia em Enviados, a aplicação passa a propor guardá-la, em vez de a deixar em falta sem aviso.
```
