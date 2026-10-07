# Sternenepoche: zuerst PC, später VPS

Stand: 7. Oktober 2026. Der neue Dienst ist `crates/server`, der gemeinsame Browser
`web-client`. Der ältere Python-Prototyp im GitHub-Checkout ist kein zweiter Produktionsserver.

## Am PC starten

`Server-starten.cmd` doppelklicken oder im Projektverzeichnis:

```powershell
.\start-server.ps1
# Nach Quellcodeänderungen:
.\start-server.ps1 -Restart -Build
```

Spiel: **http://127.0.0.1:8890**. Lokale Verwaltung: **http://127.0.0.1:8891**.
Standard: eine gemeinsame Welt, 30 Skriptbots, 20 mögliche Teilnehmerplätze; zunächst drei freigegeben, 1× Echtzeit.
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

Der PC ist über **Tailscale Funnel mit gültigem TLS** erreichbar:
[Online spielen und anmelden](https://desktop-3dei636.taila4f584.ts.net/).
Die [GitHub-Website](https://sternenepoche.github.io/) verlinkt diesen Zugang; ihr eigener
Browserclient hat dieselbe HTTPS-API-Adresse vorkonfiguriert. Mitspieler brauchen keine
Tailscale-Installation. Der öffentlich freigegebene Listener ist ausschließlich 8890;
das Dashboard auf 8891 bleibt auf Loopback.

```powershell
.\tailscale-spielzugang.ps1 -Action Status
# Erneut aktivieren, nur wenn noch höchstens drei Plätze freigegeben sind:
.\tailscale-spielzugang.ps1 -Action Enable
# Ausschließlich diese Webfreigabe abschalten:
.\tailscale-spielzugang.ps1 -Action Disable
```

Funnel läuft mit `--bg` und nimmt seine Freigabe nach einem Tailscale-/PC-Neustart wieder
auf. Der Rust-Server muss nach dem PC-Start über `Server-starten.cmd` gestartet werden;
ein Windows-Autostart wurde nicht eingerichtet. PC, Tailscale und Rust-Dienst müssen
laufen. Im Ruhezustand/offline ist das Spiel nicht erreichbar; ausgeschaltete Zeit wird
nicht nachgerechnet. `data/online/public-url.txt` speichert die HTTPS-Adresse für die
Origin-Freigabe beim nächsten Serverstart. Das Hilfsscript überschreibt keine fremde
Serve-/Funnel-Belegung und veröffentlicht weder Dateiverzeichnisse noch Adminports.
Siehe [offizielle Funnel-Anleitung](https://tailscale.com/docs/features/tailscale-funnel)
und [CLI, Hintergrundbetrieb und Abschalten](https://tailscale.com/docs/reference/tailscale-cli/funnel).

Der geprüfte öffentliche DNS-Relaypfad und die direkte HTTPS-Browseransicht funktionieren.
Auf Karls Tailscale-PC löst MagicDNS die Adresse intern auf eine Tailscale-IP auf: Der
Pages-Client kann dann die lokale Netzwerkfreigabe des Browsers benötigen. Der Link
„Spiel direkt auf diesem Server öffnen“ nutzt dieselbe sichere Adresse und denselben
Server mit einer gemeinsamen Browser-Origin. Es wurde keine Zertifikatsprüfung umgangen.

Der aktuelle Funnelbetrieb verwendet **kein `--trusted-proxy`** und vertraut keinem vom
Spieler gesendeten IP-Header. Anmelde- und IP-Limits gelten deshalb gemeinsam für die
Funnelverbindung. Der vorbereitete VPS-Caddy setzt `X-Real-IP` selbst und kann mit dem
expliziten Proxyflag arbeiten. Der öffentliche und der private Listener haben getrennte
Limit-Tabellen; öffentlicher Druck füllt keine private Admin-Limit-Tabelle.

## Drei Plätze und Warteliste

Konto anlegen oder anmelden, dann Spielweise und Volk wählen. Solange weniger als drei
Teilnehmer aktiv sind, wird atomar ein Platz vergeben. Danach bedeutet der Button
„Auf Warteliste anmelden“ eine gespeicherte Anmeldung mit persönlicher Position, Volk
und Spielweise. Es wird noch kein Reich aktiviert. Zuschauer brauchen keinen Spielplatz.
Die verbleibenden 17 möglichen Plätze sind zunächst gesperrt, keine simulierten Spieler.

Im Dashboard bestimmt **Freigegebene Teilnehmerplätze (0–20)** die Kapazität. Eine Erhöhung
lässt die Warteliste automatisch nach Reihenfolge nachrücken; das war beim Eintragen
angekündigt und startet das Reich auch bei abgemeldetem Konto. Wiederholte Anmeldung
aktualisiert die Wahl ohne neue Position. Verlassen und spätere Neuanmeldung stellt ans Ende.
Nur eigener Status und Gesamtanzahl sind öffentlich; Namen/Präferenzen der Wartenden sind
privat im Dashboard. Dort lassen sich Einträge entfernen. Das Limit der Warteliste ist
1.000 Konten. Eine niedrigere Freigabe entfernt aktive Spieler nicht; gesperrte Spieler
behalten ihr Reich und belegen weiter einen Platz. Gesperrte Wartende werden entfernt.

Freigabe und Liste überstehen Prozessneustart und Backup. Reset leert die Liste, setzt
Konten auf Zuschauer und behält die eingestellte Freigabe. Eine neue Epoche erfordert
eine neue Spieleranmeldung. Alte Runtime-Daten ohne Freigabefeld bleiben kompatibel mit
20 Plätzen; Karls bestehende Welt wurde ohne Reset auf drei begrenzt.

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
Zugriff auf den PC-Loopback blieb im eingebauten Prüf-Browser im Timeout. Die direkte
HTTPS-Spielansicht ist inzwischen geprüft; Pages-Zugriff auf MagicDNS-Adressen kann
dagegen ebenfalls lokale Browserfreigabe verlangen; lokales Ollama benötigt
weiter seine eigene Freigabe. Es wurde keine Browser-Sicherheitswarnung umgangen.

Auf Karls PC ist jetzt auch die genaue Tailscale-Spielwebsite in `OLLAMA_ORIGINS`
freigegeben. Ollama bleibt auf `127.0.0.1:11434`, nicht im Funnel. Andere unbekannte
Websites werden weiter mit 403 abgelehnt. Die vorhandenen 54 Modelle bleiben erhalten.
Der idle Ollama-Server wurde dafür neu gestartet, die Ollama-App blieb geöffnet.
Für andere Windows-Mitspieler oder nach Änderung der Spieladresse:

```powershell
.\ollama-spielzugang.ps1 -GameOrigin https://desktop-3dei636.taila4f584.ts.net -Restart
```

Das Script ergänzt genaue Origins, sichert die vorige Benutzereinstellung und stoppt
nur einen eindeutig erkannten lokalen `ollama.exe serve` ohne bestehende Verbindungen.
Es installiert keine Modelle und ändert keinen Modellspeicher. Ein Browser kann zusätzlich
seine lokale Netzwerkfreigabe verlangen. [Ollama-Dokumentation zur Origin-Freigabe](https://docs.ollama.com/faq#how-can-i-allow-additional-web-origins-to-access-ollama).

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

### Spielstandkompatibilität ab 8. Oktober 2026

Der Server liest bestehende V6-Spielstände weiter. Exakte Bauzeiten und der Beginn
laufender Forschungen werden in einer V7-Erweiterung gespeichert; der unveränderte
V6-Kern bleibt darin eingebettet. Krisen, Niederlagen und geänderte Rettungsfristen
liegen zusätzlich in einer V8-Erweiterung; V6 und V7 werden weiterhin gelesen.
Dafür ist kein Weltreset nötig. Vor dem Update
ein Datenbankbackup erstellen. Ältere Serverversionen können die neue Erweiterung
nicht lesen: Ein Rückwechsel erfordert das passende Backup und verliert dann alle
Spielaktionen seit dessen Erstellung.

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
VPS-/Proxybetrieb auf dem späteren Zielhost, echter 24-Stunden-Betrieb und Online-Balancing.

Für den beschleunigten 180-Tage-Botlauf ein **neues** Datenverzeichnis wählen:

```powershell
cargo run -p sternenepoche-server --example bot_soak --release --offline -- laeufe/mein-neuer-botlauf
```

Der Lauf verändert keine bestehende Welt und schreibt alle 30 Spieltage einen Diagnosebericht.
Die Verwaltungsansicht zeigt Versorgungsprobleme als Hinweise; fallende Nahrung während
eines Ausbaus ist kein Beweis für einen defekten Bot. Fehler sind mit Befehlsverlauf,
Beständen und nächstem Zug zusammen zu beurteilen.

### Ausscheiden und Epochenwechsel

Im Dashboard sind zwei Rettungsfristen von 1 bis 720 **Spielstunden** einstellbar.
Standard sind 72 Stunden für durchgehend weniger als 50 Prozent lebensnotwendige
Versorgung auf allen bewohnten eigenen Planeten und 48 Stunden für eine nicht
wiederanlaufbare Rohstoffwirtschaft. Bei Syntheten zählt Energie statt Nahrung.
Pausieren hält diese Fristen an; das Tempo beschleunigt sie wie die übrige Spielzeit.
Eine gesunde Kolonie, bezahlbare Reparaturen, erreichbarer grundlegender Wiederaufbau,
funktionsfähiger Handel oder eine eigene beziehungsweise ankommende Hilfsflotte
verhindern einen fälschlich festgestellten wirtschaftlichen Stillstand. Ein bloß
unbezahlter Bauauftrag genügt dafür nicht. Die Versorgungskrise wird unabhängig geprüft.

Erholung beendet die jeweilige Krise. Nach einer Niederlage bleiben Konto,
ursprüngliche Heimatwelt und historische Platzbelegung bestehen. Produktion,
Befehle, Bau-/Forschungsaufträge, Bots und Agenten stoppen dauerhaft für diese Epoche.
Auch eine spätere Lieferung hebt eine Niederlage nicht auf. Die ausgegraute Heimatwelt
bleibt gegen Kolonisation und Übernahme geschützt; andere Kolonien bleiben eroberbar.
Ein besiegter Platz wird nicht an die Warteliste neu vergeben. Erst ein vorbereiteter
Epochenreset löscht Niederlagen und Krisen; die eingestellten Rettungsfristen bleiben erhalten.

Für einen kontrollierten Browsernachweis ein **neues, noch nicht vorhandenes**
Datenverzeichnis verwenden:

```powershell
cargo run -p sternenepoche-server --example defeat_demo --release --offline -- laeufe/meine-neue-ausscheiden-pruefung
```

Die getrennte Prüfwelt enthält `UiNotstand` und `UiRettung` mit dem Testpasswort
`nur-ein-test-passwort`. Sie startet pausiert, mit Tempo 60 und einer Stunde Frist.
`UiRettung` kann sich durch den Bau einer Farm retten; `UiNotstand` hat keine
wiederherstellbare Wirtschaft. Dieses Werkzeug verweigert vorhandene Verzeichnisse
und darf niemals für das öffentliche Datenverzeichnis gestartet werden.
