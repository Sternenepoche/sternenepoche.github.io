# Anmeldung, Testkonten und Authenticator

Stand: 8. Oktober 2026. Die Kontoverwaltung läuft im Rust-Spielserver, lokal am PC und später auf dem VPS. Die bestehende Welt wird beim Update fortgesetzt.

## Lokale Dateien und Spielerregister

Auf Karls PC liegen die privaten Laufzeitdaten unter `D:\projekte_ki\Sternepoche\data\online`:

| Datei | Inhalt |
|---|---|
| `spiel.sqlite3` | Konten, Passwort-Hashes, Sitzungen, gemeinsame Welt, Spielerstände, Konto- und Befehlsprotokolle |
| `TESTKONTEN-ZUGAENGE.txt` | Namen und neue Passwörter der fünf lokalen Testkonten; ausschließlich für den Betreiber |
| `email-code-key.txt` | Privater Schlüssel für verschlüsselte Authenticator-Geheimnisse und E-Mail-Codeprüfung |
| `mail-private.json` | Mailtransport; zunächst `enabled: false` |
| `admin-token.txt` | Privater Dashboardzugang |
| `backups/` | Konsistente Datenbankbackups und zugehörige `.auth-key.txt`-Schlüssel |

Das private Dashboard öffnest du auf dem Server-PC unter **http://127.0.0.1:8891**. Unter **Spieler & Konten** kannst du nach Name oder E-Mail suchen, letzte Anmeldungen und Authenticator-Status prüfen und mit **Details & Logs** den aktuellen Reichsstand sowie die jüngsten Ereignisse und Befehle als JSON exportieren. Der öffentliche Listener besitzt keine Verwaltungsroute.

SQLite enthält die folgenden lesbaren Strukturen:

| Tabelle oder View | Bedeutung |
|---|---|
| `accounts` | ID, Name, Argon2-Passwort-Hash, E-Mail/Bestätigung, Testkonto-Flag, Reichzuordnung, Spielweise, Sperre, Anmeldestatistik; verschlüsselte TOTP-Schlüssel |
| `registered_players` | Gemeinsame Übersicht über Konto und letzten Spielerstand, ohne Passwort-Hashes und TOTP-Geheimnisse |
| `player_states` | JSON-Reichsstand und Zusammenfassung; nach Änderungen an der Reichzuordnung sofort, sonst spätestens nach ungefähr 60 Sekunden gespeichert |
| `account_events` | Anmeldung, Abmeldung, Fehlversuche, Testkontoänderungen, Authenticator-Verwaltung und Spielerereignisse |
| `commands` | Angenommene/abgelehnte Spielbefehle und Antworten für die Wiederholsicherheit |
| `sessions` | Gehashte Sitzungstokens und Ablaufzeit |
| `email_challenges` | Kurzlebige E-Mail-Codeprüfung und einmalige Registrierungsgenehmigung |
| `recovery_codes` | Hashes der einmaligen Wiederherstellungscodes und ihr Verbrauchszeitpunkt |
| `checkpoint` | Verbindlicher gemeinsamer Welt- und Runtime-Spielstand |

Der JSON-Spielerstand ergänzt den verbindlichen Checkpoint. Die Detailansicht im Dashboard liest den aktuellen Stand unmittelbar aus der Welt. Bei älteren Konten ist das ursprüngliche Erstellungsdatum unbekannt und wird als Strich angezeigt; seit dem Update werden Anmeldungen und Ereignisse protokolliert. Die Migration ergänzt Tabellen und Spalten und erhält Kontonummern, Passwort-Hashes, vorhandene Sitzungen und Reichbesitz. Ein gezielter Passwortwechsel widerruft die bisherigen Sitzungen.

## Fünf Testkonten ohne E-Mail

**LiveDemo plus vier Testkonten** können sich direkt mit Name und Passwort anmelden. Das private Zugangsdokument enthält die tatsächlichen Kennwörter. LiveDemo behält sein bestehendes Reich. Die vier neuen Konten wählen bei ihrem ersten Beitritt Volk und Spielweise. Es sind insgesamt fünf Teilnehmerplätze freigegeben; die aktuelle Belegung zeigt der Spielbrowser.

Über die authentifizierte private Adminaktion `test_account` lässt sich ein Testkonto anlegen oder dessen Passwort setzen. Höchstens fünf Konten können als Testkonto gekennzeichnet werden. Die Daten gehören ausschließlich in den privaten Datenordner und werden vom Pages-Packager nicht aufgenommen. Ein Spieler kann seinen eigenen Authenticator anschließend freiwillig verbinden.

## Authenticator lokal testen

1. Spiel unter **http://127.0.0.1:8890** öffnen und mit einem Testkonto anmelden. Auch der öffentliche HTTPS-Spielzugang funktioniert.
2. **Spielerprofil → Anmeldung mit Authenticator schützen** öffnen, aktuelles Passwort eingeben und **QR-Code erzeugen** wählen.
3. In einer TOTP-App **Konto hinzufügen → QR-Code scannen** benutzen. Alternativ App-Link auf demselben Gerät öffnen oder Schlüssel manuell übernehmen.
4. Den sechsstelligen App-Code im Spiel eingeben und **Verbindung bestätigen** wählen. Erst diese Bestätigung aktiviert den zusätzlichen Schutz.
5. Die acht Wiederherstellungscodes als Text speichern. Sie werden einmal angezeigt und sind jeweils einmal verwendbar.
6. Abmelden. Beim nächsten Login Name, Passwort und aktuellen App-Code eingeben. Ein gerade verwendeter Code wird bis zum nächsten Wechsel abgewiesen.

Der Server erzeugt QR und `otpauth://`-Link selbst. Für die Einrichtung muss das Handy den QR-Code sehen können; es benötigt keine Netzwerkverbindung zum lokalen PC. Danach berechnet die App die Codes offline. Verwendet werden RFC 6238, SHA-1, sechs Stellen, 30 Sekunden und ein zufälliges 160-Bit-Geheimnis. Das Zeitfenster toleriert einen Schritt in beide Richtungen; automatische Uhrzeit auf Handy und Server verwenden. TOTP bestätigt den Besitz des verbundenen Geheimnisses. Die optionale E-Mail-Registrierung bestätigt zusätzlich die Mailadresse.

TOTP-Geheimnisse sind in SQLite mit ChaCha20-Poly1305 verschlüsselt und an die Kontonummer gebunden. Passwort, gültiger App-Code und einmaliger Wiederherstellungscode werden nicht als Klartext protokolliert. Kontobezogene und IP-bezogene Limits begrenzen Fehlversuche. Die Einrichtung läuft nach zehn Minuten oder fünf falschen Bestätigungen ab. Sicherheitsänderungen benötigen das aktuelle Passwort und widerrufen andere Sitzungen. Beim Entfernen sind zusätzlich ein gültiger App- oder Wiederherstellungscode erforderlich.

## E-Mail-Registrierung auf dem VPS einschalten

Die Registrierung bleibt zunächst aus. Es wird kein Mailanbieter benötigt und keine echte E-Mail verschickt. Der komplette Ablauf ist bereits eingebaut:

**E-Mail → sechsstelliger Code → Name, Passwort, Volk und Spielweise → Reich oder Warteliste.**

Nach Auswahl eines SMTP-Anbieters oder eigenen Mailservers `mail-private.json` im privaten Datenordner bearbeiten:

```json
{
  "enabled": true,
  "host": "smtp.deine-domain.example",
  "port": 587,
  "security": "starttls",
  "username": "DEIN_SMTP_BENUTZER",
  "password": "DEIN_SMTP_PASSWORT",
  "from": "Sternenepoche <spiel@deine-domain.example>"
}
```

Für direktes TLS üblicherweise `security: "tls"` und Port 465 verwenden; maßgeblich sind die Einstellungen deines Mailservers. STARTTLS ist verpflichtend, ein Rückfall auf Klartext wird abgelehnt. Die Konfiguration wird pro Anfrage gelesen; Aktivieren benötigt keinen Neustart. `/api/registration/status` zeigt den Freigabestatus ohne Verbindungsdaten. Der Versand erfolgt außerhalb der Weltsperre, damit eine langsame Mailverbindung die Spielzeit nicht anhält.

Codes gelten zehn Minuten, maximal fünf Prüfversuche; erneuter Versand frühestens nach 60 Sekunden, höchstens fünf pro Adresse und Stunde. Nach erfolgreicher Prüfung ist die Registrierungsgenehmigung ebenfalls zehn Minuten und einmal gültig. In SQLite stehen nur geschützte Codeprüfwerte. Namen und E-Mail-Adressen sind eindeutig; weitere Anmeldungen nutzen bei voller Freigabe die bestehende Warteliste.

Vor Freigabe am tatsächlichen VPS eine eigene Mailadresse durch den ganzen Ablauf testen. Die lokale Prüfung deckt Codeversand, Codeprüfung, Profilanlage, Warteliste und Sicherheitsgrenzen ab; die Zustellung und Domainkonfiguration des später gewählten Anbieters lassen sich erst mit dessen Zugang prüfen.

## Backup und Umzug

Ein Dashboardbackup legt `spiel-….sqlite3` und den zugehörigen `spiel-….auth-key.txt`-Schlüssel an. **Beide gemeinsam sichern.** Zur Wiederherstellung in einem neuen Datenordner die Datenbank `spiel.sqlite3` und den Begleitschlüssel `email-code-key.txt` nennen. Ohne passenden Schlüssel lassen sich bestehende TOTP-Verbindungen nicht prüfen. Der Server verweigert einen Start mit aktivierten TOTP-Konten, wenn der Schlüssel fehlt.

Beim VPS-Umzug den Dienst stoppen und den privaten Datenordner konsistent übernehmen oder dieses Backuppaar verwenden. Die Mailkonfiguration separat übernehmen und auf dem VPS anpassen. Dedizierten Dienstbenutzer und ausschließlich für ihn lesbare private Dateien verwenden. Rust-Service und Caddy-Vorlagen liegen unter `deploy/`; das Dashboard bleibt über SSH-Tunnel zugänglich. Konten, Reichbesitz, Spielstände, Protokolle und verbundene Authenticator ziehen mit um. Öffentliche HTTPS-Adresse und erlaubte Origins anpassen. Die fünf Testkonten funktionieren weiterhin ohne Mailbestätigung.

## Prüfungen

```powershell
cargo test --locked --release -p sternenepoche-server
python tools/identity_smoke.py
python tools/identity_smoke.py --browser
```

Bei einem separaten Build `--server-exe <Pfad-zum-Server>` angeben. Der erste Python-Test braucht nur die Standardbibliothek und verwendet eine eigene Welt sowie einen lokalen SMTP-Empfänger auf Loopback. Die Browseroption benutzt zusätzlich Playwright, Chrome und OpenCV: Sie liest den tatsächlich angezeigten QR-Code, bestätigt TOTP, lädt Wiederherstellungscodes herunter und prüft die mobile E-Mail-Registrierung und das private Dashboard. `identity_migration_check.py --server-exe <Pfad>` prüft eine isolierte SQLite-Kopie der bestehenden Welt; die Quelldatenbank wird nicht verändert.
