# Sternenepoche: zuerst PC, später VPS

Stand: 7. Oktober 2026. Der neue Dienst ist `crates/server`, der gemeinsame Browser
`web-client`. Der ältere Python-Prototyp im GitHub-Checkout ist kein zweiter Produktionsserver.

## Am PC starten

`Server-starten.cmd` doppelklicken oder im Projektverzeichnis:

```powershell
.\start-server.ps1
# Nach Quellcodeänderungen:
.\start-server.ps1 -Build
```

Spiel: **http://127.0.0.1:8890**. Lokale Verwaltung: **http://127.0.0.1:8891**.
Standard: eine gemeinsame Welt, 30 Skriptbots, 20 freie Plätze, 1× Echtzeit.
Anmelden ohne Platz bleibt Zuschauer. Erst „Platz belegen“ aktiviert ein Reich. Spätere
Anmeldungen behalten ihren Platz. Ein Wechsel Mensch/Agent/Gemischt erzeugt kein zweites Reich
und ändert das gewählte Volk nicht. Keine neue Welt beim gewöhnlichen Serverneustart.

Private Daten: `data/online/spiel.sqlite3`, SQLite-WAL, Admin-Token, Backups und Logs.
Diese Dateien werden weder auf Pages gepackt noch ins Git-Repository aufgenommen.
Für einen Vordergrundprozess `-Foreground` benutzen und zum Beenden Strg+C.
Beim Hintergrundprozess steht die Prozessnummer in `data/online/server.pid`; beenden über
Task-Manager oder `Stop-Process -Id <geprüfte Server-Prozessnummer>`.
Vor einem Update den eigenen Server stoppen und dann mit `-Build` starten.

Die Welt läuft nur, während PC und Dienst laufen. PC-Ausfall, Prozessneustart und lange
Suspend-Phasen holen keine verpassten Stunden als nachträgliche Überfälle nach. Während
der Dienst läuft, hält ein langsames Sprachmodell die Welt nicht an. Alle bestätigten
Spielbefehle werden mit dem Weltzustand in derselben SQLite-Transaktion gespeichert.
Skriptbots und offene Ereignisse werden ebenfalls wieder aufgenommen.

## Von außen auf den PC zugreifen

GitHub Pages liefert den Browser, führt aber keine laufende Welt aus.
[GitHub Pages](https://docs.github.com/en/pages/getting-started-with-github-pages/what-is-github-pages)

Für weltweiten Zugriff braucht der PC eine erreichbare **HTTPS-Adresse**: eigenen
DNS-Namen mit HTTPS-Reverse-Proxy oder einen HTTPS-Tunnel zum Spielport 8890.
Nur der Spielport wird vermittelt. Adminport 8891 bleibt lokal.
Die Ports sind absichtlich an Loopback gebunden; eine einfache Routerweiterleitung
direkt auf den Rust-Port stellt keinen vollständigen Internetbetrieb her.

Die konkrete öffentliche Adresse ist noch nicht eingerichtet. Im Pages-Browser wird sie
unter „Spielserver“ eingetragen. Alternativ `web-client/config.js` mit der öffentlichen
API-Adresse konfigurieren. `--origin` lässt genau die tatsächlich verwendete Website zu;
die Origin `https://sternenepoche.github.io` ist bereits freigegeben.

Für Caddy siehe `deploy/Caddyfile`. Domain ersetzen, Server mit `--trusted-proxy` und
`--origin https://DEINE-DOMAIN` starten. Caddy begrenzt Bodygröße/Lesedauer, puffert
vollständige Requests und setzt `X-Real-IP` selbst. Bei einem anderen Tunnel/Proxy dessen
Timeouts, Requestgrenzen und vertrauenswürdige Client-IP-Konfiguration entsprechend setzen.
Ohne vertrauenswürdige IP-Weitergabe gilt das Anmeldelimit gemeinsam für den Tunnel.
[Caddy Reverse Proxy](https://caddyserver.com/docs/caddyfile/directives/reverse_proxy)

## Agenten und Mischbetrieb

Im Spiel Agent oder Gemischt wählen. Unter „Mein Agent“ Anbieter, Modell, Rollen und
Aufruflimit einstellen. Der Server vergibt pro Reich eine exklusive Freigabe mit 90 Sekunden
Gültigkeit; der Browser verlängert sie alle 20 Sekunden. Der Browser schickt Modellanfragen
direkt an Ollama auf **dem Spieler-PC** oder an OpenRouter. Der OpenRouter-Schlüssel bleibt
im Arbeitsspeicher des Tabs, wird nicht in Spielserver, localStorage oder Pages gespeichert.
Ein Neuladen erfordert erneute Eingabe. Ein abgebrochener Tab verliert seine Freigabe spätestens
nach 90 Sekunden; „Agent stoppen“ oder Dashboard-Stop wirkt sofort.

Das Tab muss geöffnet bleiben. Browser-Hintergrunddrosselung kann einen Agenten stoppen.
Ein unbeaufsichtigter nativer Multiplayer-Agentenläufer ist noch nicht Teil dieses Dienstes.
Der vorhandene native Laborläufer arbeitet weiter in seinem eigenen Laborvertrag.

Für Ollama auf dem eigenen PC `OLLAMA_ORIGINS` gezielt auf die Browser-Origin setzen und
Ollama neu starten; Loopback-Port 11434 reicht. Unter PowerShell beispielsweise:

```powershell
$env:OLLAMA_ORIGINS = 'http://127.0.0.1:8890,https://sternenepoche.github.io'
ollama serve
```

Eine bereits laufende Ollama-App zuerst normal beenden, damit der Port frei ist.
Je nach Browser ist die lokale Netzwerkfreigabe erforderlich. Keine Portfreigabe für Ollama
ins Internet. [Ollama FAQ](https://docs.ollama.com/faq),
[Browserzugriff auf lokale Netze](https://developer.chrome.com/blog/local-network-access).

Auch Pages-Zugriff auf den PC-Spielserver braucht diese Browserfreigabe. Falls der Browser
das blockiert oder in einen Timeout läuft, den gleichen Spielbrowser direkt auf
`http://127.0.0.1:8890` öffnen. Die veröffentlichte Pages-Oberfläche wurde geladen; ihr
Zugriff auf den PC-Loopback blieb im eingebauten Prüf-Browser im Timeout. Eine öffentliche
HTTPS-Spieladresse vermeidet diese Grenze für den Weltzugang; lokales Ollama benötigt
weiter seine eigene Freigabe. Es wurde keine Browser-Sicherheitswarnung umgangen.

OpenRouter benötigt den Schlüssel des Spielers und den vollständigen Modellnamen.
Aufruflimit und Tokenzähler begrenzen/zeigen Browseraktivität; sie sind **kein garantierter
Gelddeckel**. Einen Gelddeckel beim Anbieter über den verwendeten Schlüssel setzen.
OpenRouter-Transport und Budget/Stop sind mit Mockantworten geprüft. Ein echtes vorhandenes
Ollama-Modell `qwen3.5:4b` hat im Browser drei gültige Spielbefehle ausgeführt.
Die Ergebnisse und Grenzen stehen im aktuellen Inventur-Prüfstand. Hier wurden keine
bezahlten Modellaufrufe gestartet.
[OpenRouter Authentifizierung](https://openrouter.ai/docs/guides/overview/auth/oauth).

## Verwaltung

Das separate lokale Dashboard öffnet seinen Adminzugang automatisch über dieselbe lokale
Origin. Der öffentliche Listener besitzt keine Adminroute. Im Dashboard: Pause, Tempo,
Epochenlänge, Botabstand, globale Botaktivierung, einzelne Botstrategie, Kontosperren,
private Spielerprüfung, Heimatressourcen/Credits, Agentstop, Backups und Reset. Dazu
kommen Botdiagnosen/Befehlsverlauf, Modellentscheidungen, private Systemkarte, Events,
Audit und öffentlicher Betriebshinweis. Ein bearbeitetes Regelprofil kann für die nächste
Epoche geprüft und mit konkreter Resetvorschau übernommen werden.

Vor Ressourcenänderung oder Reset wird ein vollständiges SQLite-Backup erzeugt. Der Reset
benötigt die genaue Bestätigung `RESET <aktuelle Weltkennung>`, setzt Plätze zurück,
widerruft Sitzungen und alte Agentantworten und behält die Konten als Zuschauer.
Der Startwert ist optional. Ein Reset wurde ausschließlich in isolierten Testwelten geprüft.
Normale Ausbauregeln stehen versioniert in `regeln/online-v1.ron`; Preise und Technologien
werden nicht mitten in laufende Bauaufträge hineineditiert. Weltgeschwindigkeit,
Epochenende und Botabstände lassen sich dagegen im Dashboard ändern.

Backups regelmäßig im Dashboard erstellen und zusätzlich auf ein anderes Gerät kopieren.
Zum Wiederherstellen **Dienst zuerst beenden**, den gesamten aktuellen Datenordner erhalten,
in einem neuen Datenordner die gewählte Backupdatei als `spiel.sqlite3` ablegen und mit
`--data <neuer Ordner>` starten. Niemals eine Datenbankdatei unter einem laufenden Prozess
oder zusammen mit einem alten fremden `-wal` ersetzen. Der Admin-Token wird im neuen Ordner
neu erzeugt. Wiederhergestellte Agentfreigaben werden beim Start widerrufen.

## VPS-Umzug

1. Rust-Binary für das Betriebssystem des VPS bauen (`cargo build -p sternenepoche-server --release`).
2. Dienst stoppen oder ein konsistentes Dashboardbackup erstellen; Backup als `spiel.sqlite3`
   in `/var/lib/sternenepoche` übernehmen. Gleiche veröffentlichte Code-/Regelversion verwenden.
3. Dedizierten Benutzer `sternenepoche` erstellen, Binary nach `/opt/sternenepoche` legen,
   `deploy/sternenepoche.service` auf Domain/Pfade anpassen und über systemd starten.
4. DNS und Caddy konfigurieren. Öffentlich nur HTTPS-Spielzugang; beide Rust-Listener bleiben
   Loopback. Diese Vorlagen sind vorbereitet, aber noch nicht auf einem echten VPS installiert.
5. Auf Karls PC einen SSH-Tunnel öffnen:

```powershell
ssh -N -L 8891:127.0.0.1:8891 -L 8890:127.0.0.1:8890 BENUTZER@DEIN-VPS
```

Dann öffnet das **Dashboard auf Karls PC** weiterhin `http://127.0.0.1:8891` und verwaltet
die VPS-Welt. Vorher den alten PC-Dienst beenden, damit die lokalen Ports frei sind.
Für Spieler nur die öffentliche API-Adresse in Pages/Browser ändern. Konten, Reichbesitz,
Spielstand und Botstrategie ziehen mit der Datenbank um; Agenten werden neu gestartet.

## Nachweise

```powershell
cargo test --workspace --release --offline
python tools/online_smoke.py
node tools/browser_agent_test.cjs
```

Der HTTP-Test startet/beendet eine eigene Welt auf 18990/18991 unter `laeufe/`.
Der Browseragententest verwendet ausschließlich Mockanbieter. Die 24-Spielstunden-Tests
prüfen eingefrorene Plätze und alle 30 aktiven Bots beschleunigt, einschließlich einer
identischen Fortsetzung aus einem Datenbankbackup; sie ersetzen keinen 24-Stunden-Dauerbetrieb.
Offen für öffentliche Abnahme: bezahlter OpenRouter-Nachweis, zwei externe Netze,
HTTPS-/Proxybetrieb auf dem Zielhost, längerer Last-/Botlauf und Online-Balancing.

Für den beschleunigten 180-Tage-Botlauf ein **neues** Datenverzeichnis wählen:

```powershell
cargo run -p sternenepoche-server --example bot_soak --release --offline -- laeufe/mein-neuer-botlauf
```

Der Lauf verändert keine bestehende Welt und schreibt alle 30 Spieltage einen Diagnosebericht.
Die Verwaltungsansicht zeigt Versorgungsprobleme als Hinweise; fallende Nahrung während
eines Ausbaus ist kein Beweis für einen defekten Bot. Fehler sind mit Befehlsverlauf,
Beständen und nächstem Zug zusammen zu beurteilen.
