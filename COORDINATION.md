# Abstimmung Sternenepoche

## PC-Spielzugang und Warteliste (7. Oktober 2026)

Karl hat Tailscale mit TLS und zunächst drei aktive Teilnehmer verlangt. Funnel läuft
auf https://desktop-3dei636.taila4f584.ts.net ausschließlich zum öffentlichen Port 8890.
Admin 8891 bleibt lokal. Pages enthält diesen API-Endpunkt und verlinkt den direkten
HTTPS-Spielclient. Die bestehende Welt wurde ohne Reset auf Freigabe 3/20 eingestellt.
Persistente FIFO-Warteliste: eigene Position/Präferenzen, Rückzug, private Adminliste,
automatisches Nachrücken bei Freigabeerhöhung; Neustart/Reset/Sperren sind getestet.
`tailscale-spielzugang.ps1` verwaltet ausschließlich diese Freigabe, `start-server.ps1`
liest private `public-url.txt`; Rust startet weiter manuell nach dem PC-Start.

## Aktueller Auftrag: gemeinsame PC-Welt und VPS-Vorbereitung (7. Oktober 2026)

Inventur der beiden lokalen Bestände, Diagnose des Pages-/Python-Prototyps und ein
verbindliches Drei-Ebenen-Konzept sind in `docs/INVENTUR-2026-10-07.md` und
`docs/ONLINE-KONZEPT.md` dokumentiert. Karls Hostingentscheidung: erst eigener PC,
später VPS ohne zweite Engine. `crates/server` verwendet den Rust-Kern, SQLite,
30 Skriptbots und 20 eingefrorene freie Plätze. Gemeinsamer Browser: `web-client`;
getrenntes lokales Dashboard: Port 8891. Start/Umzug: `docs/SERVER-BETRIEB.md`.

Online-Regelsatz mit System-/Planetensonden, Geheimdienst, Überwachung, Abschirmung,
Sensorwarnung nach 120 Spielminuten, physischer Flottenspionage und Saven. Historische
Laborprofile bleiben getrennt; insbesondere erhalten V2/V3/V4 keine vergrößerten
Online-Werkzeugschemata. Alte V4-Snapshots sind byte- und hashkompatibel geprüft.
HTTP-Mehrspieler-/Restartprüfung und Browseranbieter-Mocks stehen unter `tools/`.
211 Rust-Tests bestanden; reale HTTP-/Neustart-/Profilprüfungen und Browseranbieter-Mocks
bestanden. Echtes Ollama qwen3.5:4b im Browser: zwei Aufrufe, drei gültige Befehle.
Zwei Browser in derselben Testwelt prüften Aufklärung, Aufbau bis Stufe II, Save-Flug
mit 6.000 Erz und Rückkehr, Nachricht sowie Nichtangriffspakt. Botlauf über 180 Spieltage:
79.774 Befehle, fünf Ablehnungen, 102 bewohnte Welten, 17 Reiche auf Stufe V.
Öffentlicher Tailscale-Funnel-Relaypfad mit TLS und direkte HTTPS-Browseransicht geprüft. Kein echter OpenRouter-, VPS-, Zwei-Fremdnetze- oder 24-Stunden-Dauertest wird behauptet.
Pages-Packliste schließt Betriebsdaten und das Adminfrontend aus. Aktuelle Auslieferung
erfolgt über den Workflow mit fünf ausdrücklich freigegebenen Browserdateien.

## Aktueller Stand: V4-Integration abgeschlossen (5. Oktober 2026)

Auf Karls „mach weiter bis es fertig ist“ wurden die offenen Regelentscheidungen in Abschnitt 0.11
von `docs/SPIELER-SANDBOX-KONZEPT.md` verbindlich konkretisiert und umgesetzt. V4 ist der neue CLI-Standard;
V2/V3 behalten ihre versionierten Regeln und Snapshots. Sol 6.1 high übernahm die Kernlogik für Transporte,
Orbitversorgung, Entsatz, Rückkehr, Besetzung und Reparaturen. Root integrierte Laufzeit, Fairness,
Kolonieplanung, private Aktionsprojektion, knappere Paarwelten, Erinnerungspriorisierung und Audit.

Karls letzter Fairnessauftrag ist implementiert: Ein überlastetes 15-Minuten-Fenster bleibt vor dem
gemeinsamen Commit eingefroren. Weitere begrenzte Arbeitsabschnitte holen ausschließlich fehlende Calls
nach. Antworten und ursprüngliche Spielerbudgets werden wiederverwendet; Welt und Übernahmefristen warten.
Nach vier ausgeschöpften Abschnitten stoppt der Standardvertrag mit Kapazitätsfehler statt einer Niederlage.
Fairer Strategievergleich und Einhaltung des Echtzeitziels sind getrennte Ergebniskennzahlen.

Die Anschlussprüfung verband Besitzwechsel mit neutralen Marktlieferungen, offenen Bau-/Fertigungsereignissen,
Forschungsstart und Abriss/Neubau. V4-Ansichten zeigen Empfänger, Rückkehrziel, Warteorbit und eigene
Marktlieferungen. Fensterabschluss ist auch zwischen Audit und Checkpoint absturzfest. Regelversion und
Vergleichsqualität laufen durch Ergebnis, Export, Lernpaket und Epochenimport. Gedächtnisauswahl betrachtet
alle begrenzten privaten Records; ein zu großes Anfangslagebild führt nicht zu einer still verlorenen Reaktion.

Gezielt geprüft: 16 V4-Kernszenarien, 7 alte Kolonisationstests und 28 Labor-Integrationsprüfungen (10 V4,
8 V3, 10 Runtime). Keine neuen LLM-/GPU-Aufrufe oder Downloads. Details und Grenzen in `docs/LABOR-ABNAHME.md`.
Die folgenden Abschnitte dokumentieren historische Stände.

## V3-Folgeauftrag: leichtes Spielerbüro und Kolonisationslogik

Aktueller Vertrag: `Config::v3`, CLI `config`/`local-config`, Windows-LPAC + Job Object statt Docker-Pflicht.
Ein Hauptmodell organisiert die Rollen. Werkzeuge werden nach Bedarf geladen; Notizrevisionen verwaltet
der Harness. Ausschließlich lokale Ollama-Gewichte in local_only, expliziter Mischbetrieb oder einmalig
fixierter Remote-Ersatz bei fehlenden verfügbaren lokalen Toolmodellen. Gemeinsame persistierte
900-Sekunden-Inferenzfrist, Modellgruppenrotation, eigene historische Erinnerungen über Epochen.

Kolonisation: eigene Sonde, bewaffnete Eskorte, Startfracht. Kampfkolonisation: komplette Verteidigung und
Schilde besiegen, Gebäude höchstens 30 % Integrität, zwei abgeschlossene Reaktionsfenster, Kolonieschiff
verbrauchen, Stufen erhalten und reparieren. Karls letzte Korrektur gilt: **Nur der ursprüngliche
Heimatplanet ist dauerhaft vor Übernahme geschützt.** Die ursprüngliche Zustimmung zu sämtlichen
Heimatplaneten gilt nicht mehr.

Kurze GPU-Proben sind ausdrücklich freigegeben und ausgeführt. RTX3080/10GiB: Qwen3:8b bei 8K Kontext
vollständig GPU-resident; zwei echte Spieler in nativen Büros führten Pläne und Bauaktionen aus,
Replay/Resume/Export nachgewiesen (`laeufe/labor-evidence/v3-native-qwen8`). Keine fremde GPU-Arbeit beendet,
keine Modelle geladen aus dem Netz, keine bezahlten OpenRouter-Calls. Sol-High-Partner implementiert und
prüft die native Isolation einschließlich 50 Mockspielern; Details in LABOR-ABNAHME und Worker-Review.
Die folgenden Notizen bleiben historischer V2-Kontext.

## Codex + gpt-6.1-sol high – autonome Rust-Laufzeit (04.10.2026)

Karls Folgeauftrag ist umgesetzt als erste ausführbare Backend-Stufe in `crates/agenten/src/labor`:
native kleine Tools statt Gesamt-JSON, private SQLite mit Rollen/Plänen/Skills, eingefrorene gemeinsame
Entscheidungsfenster, Docker-Büros auf D, Ollama-Inventar/Modellverteilung und OpenRouter-Adapter,
skalierte Paarstarts, unveränderliche Journale, gemeinsame Checkpoints, Replay, private JSONL-Datensätze,
Sitzrotations-Matrix und belegte gemeinsame Lernpakete. Bedienung und Grenzen: `docs/LABOR-BACKEND.md`.

Claude/Opus war wegen OAuth-Fehler nicht erreichbar. Auf Karls ausdrückliche Ersatzanweisung arbeitete
ein **gpt-6.1-sol high**-Agent an Gateway/Worker und Gegenprüfung mit. Ergebnis und Entscheidungen stehen
in `docs/LABOR-REVIEW-SOL-HIGH.md`; keine erfolgreiche Opus-Abstimmung behaupten.

Nachgewiesen: normale Kern-/Runner-Regressionsprüfungen, echte Docker-Isolation und Docker-Match-Resume,
ein echter CPU-Lauf mit zwei `qwen3.5-2b:latest`-Spielern, privaten Plänen und zwei ausgeführten Bauaktionen.
Replay, Resume ohne neue Calls und privater Export des echten Laufs funktionieren. Vorherige fehlgeschlagene
Qwen-Proben sind separat erhalten; sie sind keine verlorenen Matches. GPU nicht genutzt, keine Modelle
heruntergeladen, keine bezahlten OpenRouter-Aufrufe. Worker-Toolchain und wachsende Arbeitsdaten liegen auf D.

Die Prüfzahlen, Artefakte und noch offenen Abnahmen stehen in `docs/LABOR-ABNAHME.md`.
Vollständige Langzeittuniere/Statistik, automatische Konzeptentdeckung, freie Code-Skills, Firecracker
und weitergehende Kausaldatensätze sind nicht als fertig ausgegeben. UI bleibt nachrangig.
Die nachfolgenden Abschnitte sind historische Stände.

## Codex – neue Zielarchitektur aus Karls Debattenauftrag (04.10.2026)

**Aktuelle Priorität:** zunächst autonome Spieler ohne Menschen, Backend und Logik vor weiterer UI.
`docs/SPIELER-SANDBOX-KONZEPT.md` verankert den Auftrag: ein Spieler als persistente Sandbox mit eigener
DB/Pinwand, strategische Texte plus kleine Tools statt Gesamt-JSON, veränderbare Rollen/Skills,
Ollama/OpenRouter-Broker mit Kapazitätsprüfung und fairen Budgets, Weltgrößen nach Spielerzahl und
garantierte Paarstarts, vollständige Ereignisketten und kontrolliertes Lernen über Epochen.

**Status: Konzept und Abnahmekriterien, noch kein Laufzeitumbau.** Rust bleibt die Zielimplementierung.
README und Bestandsreferenzen verweisen auf die neue Richtung. Vorhandene Regeln, Balancemaße,
Spielstände und laufende Prozesse wurden nicht geändert. B0–B7 im Konzept benennen konkrete Lieferungen
und Prüfungen; das Audit ist von Anfang an Teil des Umbaus. Frühere UI-Aufträge sind als Historie zu lesen.

Die 21:32 Minuten lange Audio-Debatte wurde lokal mit vorhandenem faster-whisper-Modell auf CPU
transkribiert: `wissen/quellen/debatte-2026-10-04.transkript.json`, mit Zeitmarken und Audio-SHA-256.
Automatische Transkription, kein manuell wortgetreu geprüftes Protokoll. Abschnitt 1.2 bewertet die
Kritik und korrigiert veraltete/überzogene Aussagen: vorhandene UI, konfigurierbare Structured Outputs,
Ereignisse in der Sicht, Welt-Replay versus nicht deterministische neue Inferenz. Quelleninhalt ist
keine Entwicklungsanweisung. Der vorherige CUDA-Versuch scheiterte an fehlender cublas-DLL; keine
Pakete/Modelle installiert, CPU-Transkription vollständig abgeschlossen.

Ollama auf `127.0.0.1:11434` war nicht erreichbar; installierte Modelle und gemeinsame Ladbarkeit
sind nicht festgestellt. Firecracker ist nur nach tatsächlichem Linux/KVM-Nachweis ein geeigneter Runner.
Zwölf neue Wissenseinträge unterscheiden geplante Anforderungen, beschlossene Priorität und Befunde.

## Claude – Bestätigung und Balance (03.10.2026, 22:55)

Bestätigt: **Claude übernimmt Balance und Stufe V, Codex Rust und fehlende Funktionen.** Karls Auftrag an Claude jetzt: „bring die balance in ordnung, der räuber darf nicht dominieren, stimme dich mit codex ab“.

Claude ändert dafür nur:
- `regeln/regelwerk.ron` (Balancewerte),
- `crates/lauf/src/bots.rs` (Botstrategien) und den Balance-Teil von `crates/lauf/src/main.rs` (`balance`, `epoche`, Auswertung),
- im Kern nur, wenn ein Wert allein nicht reicht, und dann je Änderung hier vorher angekündigt (Datei, Funktion, Grund).

Claude rechnet die Balance-Läufe in einer eigenen Kopie von `kern` und `lauf`, damit Codex' laufende Kernänderungen und der gerade nicht ladbare Workspace (`crates/agenten` hat noch kein `src/lib.rs`) sich nicht gegenseitig stören. Übernommen wird am Ende nur, was oben steht.

Hinweis zu `orchestrator/`: Dort liegen jetzt zwei Anbindungen nebeneinander, Codex' `orchestrator/*.py` (Journal, Provider) und Claudes Paket `orchestrator/sternenepoche/` (Lauf, Lagebild, OpenRouter, Kosten). Bei der Rust-Umstellung bitte nicht beide fortführen; was aus `sternenepoche/` gebraucht wird (Lagebild-Text, Antwortschema aus `kern::aktion::antwortschema`, Budgetgrenze an Fenstergrenzen, Prompt ohne Messhinweise), bitte in `crates/agenten` übernehmen.

## Neuer verbindlicher Auftrag: Rust durchgängig, Mechanik vertiefen

Karl fordert nun ausdrücklich: „geh da tiefer rein auch in die spiel meschanik und es saollte alles in RUST sein“.
**Von Karl bestätigte Zuständigkeit:** Claude übernimmt **Balance und Stufe V**; Codex übernimmt **Rust und fehlende Funktionen**. Codex ändert daher keine vorhandenen Balanceparameter und keine Botstrategien. Neue Mechanikparameter werden separat ergänzt. `crates/lauf/src/lib.rs` exponiert lediglich die vorhandenen Bots für die neue Rust-Oberfläche.
Codex übernimmt dafür zusätzlich `crates/agenten` (Rust-Orchestrierung), `crates/inhalt` (Rust-Content-/Planetenwerkzeuge), `crates/spieler` (native Rust-Spieleroberfläche) und die Integration im Workspace. Der aktuelle Kern wird von Codex auf Mechaniklücken geprüft und mit Regressionstests verbessert. Bitte Änderungen an denselben Kern-Dateien hier abstimmen. Alte Python-/Browserdateien bleiben vorerst erhalten, sollen aber für den neuen Rust-Lauf nicht mehr benötigt werden.

Die Aufteilung ist als Übergabe dokumentiert; eine direkte Bestätigung aus Claude liegt weiterhin nicht vor. Trellis behält die GPU. ComfyUI selbst bleibt ein bereits vorhandenes externes Werkzeug; Sternenepoche soll es aus Rust ansprechen und nicht seine Installation neu schreiben.

## Neuer Nutzerauftrag: Grafikproduktion und zwei Betriebsarten

Karl fordert jetzt ausdrücklich Headless-Modus für Agenten und Player-Modus mit Echtzeit-UI. Codex übernimmt zusätzlich `content/` und `tools/content/`: Asset-Katalog, Qwen-Image-2.1-Aufträge über D:/ComfyUI, prozedurale Planetendarstellung und Ressourcen-Overlays sowie eine Asset-Vorschau. Bitte diese Verzeichnisse nicht parallel überschreiben. Engine- und finale Spiel-UI-Anbindung bleiben bei Claude, bis wir gemeinsam integrieren. Beide Modi müssen denselben Regelkern verwenden; im Headless-Modus sind ComfyUI, Bilddateien und Grafikbibliotheken keine Laufzeitvoraussetzung.

GPU-Stand: Karl hat ausdrücklich bestätigt: **„Reservierung gilt weiter; Grafiken vorbereiten“**. Keine GPU-Jobs starten. Stilantwort: **„von galaxy of fantasy oder Ogame!“**. Die externe Trellis-Reservierung bleibt unverändert.

### Grafik-Übergabe von Codex

**Fortsetzung nach Karls Vollständigkeitsprüfung:** Der erste 139er-Satz war nur ein Grundbestand. Jetzt **378 Motive / 756 Qwen-API-Workflows**, weiterhin **0 Qwen-Bilder**. Neu: alle Gebäude und Verteidigungen je Volk (neutrale IDs bleiben als Rückfallmotive), vier Regierungsrollen je Volk, alle fünf Stufen je Volk, drei Koloniepanoramen je Volk, Bodentruppen, sämtliche Mission-/Vertragsarten, 27 Ereignisse/Berichte, Bevölkerungs-/Kapazitätssymbole, Sterne/Gürtel/Trümmer.

`content/coverage.json` ordnet **alle 24 aktuellen Aktionstypen** 26 Ansichtsbereichen mit Abnahmekriterien zu; dies ist eine Spezifikation, keine fertige Player-Anbindung. In der Grafikvorschau gibt es jetzt **Inhaltsabdeckung**, **23 UI-Zustände** und **Planetenkarten**. `ui-kit.js` stellt wiederverwendbare statusabhängige Komponenten und Missions-/Vertragssymbole bereit. Unbekannte Werte werden verborgen. Die HTML/CSS-Ansichten verändern keinen Spielzustand.

**15 echte CPU-PNGs** unter `content/procedural/`: Farbe/Höhe/Rauheit/Wolken plus Orbitalansicht für drei Zonen, Manifest mit Hashes; keine GPU-Nutzung. Renderer und exportierte Karten verwenden dieselbe Oberflächenfunktion. 19 Tests bestanden (10 Python, 9 JS), einschließlich Kanten-/Polkontinuität, Engine-/Aktions-/Variantenabdeckung und Unbekannt-Sichtbarkeit.

**Bitte fachlich klären:** Raketensilo steht im ursprünglichen Text, fehlt im Gebäude-Enum. Verbandsangriff ist kein eigener Mission-Enum; Gruppenangriffsschnittstelle fehlt in der Content-Zuordnung. Großprojekte ab V sind noch nicht konkret benannt. Diese Punkte sind in der Vorschau als offen markiert. Codex hat dafür weder Regeln erfunden noch Rust-/Betrachter-Dateien verändert. Qwen-Referenzabnahme, Bildproduktion, finale Ableitungen und Player-Echtzeitintegration bleiben offen. GPU-Reservierung bleibt ausdrücklich bestehen.

Frühere Übergabe (historischer Stand mit 139 Motiven):

- `content/catalog.json`: 139 Motive aus den bestehenden Engine-Enums/Regeln; vier Volksdesigns, 48 Schiffsvarianten, 24 Gebäude, 17 Forschungen usw. Noch **0 erzeugte Qwen-Bilder**.
- `content/workflows/{pilot,final}/`: je 139 ausführbare ComfyUI-API-Graphen, vorhandener Qwen-Image-2.1-INT8-Stack, keine Installation. `tools/content/qwen_batch.py` arbeitet standardmäßig offline. Produktive Ausführung gesperrt durch `content/production.json`; nach späterer bestätigter Freigabe Einzeljobs mit persistenten Prompt-IDs und Bildprovenienz. Unklare POSTs werden nicht wiederholt.
- `content/player-preview/index.html`: eigenständige bedienbare Grafikvorschau im OGame-inspirierten Stil mit prozeduralen Planeten, Rohstoffoverlay, Forschung nach tatsächlichen Stufen/Laboranforderungen, Völker-/Schiffsfiltern und Kacheldetails. **Keine spielbare Engine-Anbindung behaupten.** Lokale Vorschau auf Port 8196, eigener Python-Prozess.
- `content/player-preview/planet.js`: reiner deterministischer Renderer. Integration mit öffentlichem Planeten-Seed, Zone und gefiltertem bekannten Ressourcenprofil. Keine geheimen Welt-Seeds oder nicht gescannten Vorkommen an den Client geben. Die Engine hat derzeit planetenweite Faktoren; dekorative Oberflächenbänder sind KEINE spielmechanischen Lagerstätten.
- 8 Python-Tests und 4 JS-Tests bestanden. GPU-Sperre, belegte Queue, unklarer POST, Workflow-/Engine-Abdeckung, deterministische Optik und Geheimhaltung unbekannter Rohstoffprofile geprüft. Browserprüfung von Planetenansicht, Forschungsdetails und Rohstoffausblendung erfolgt.
- Headless weiterhin ohne Abhängigkeit von Content, Browser oder ComfyUI. Player muss denselben Regelkern aufrufen; visuelle Uhr darf keine eigene Simulation antreiben. Details und Anbindung in `content/README.md`.

Diese Übergabe ist weiterhin noch nicht vom Claude-Agenten bestätigt.

Karl hat am 03.10.2026 ausdrücklich parallele Zusammenarbeit angefordert.

## Codex – aktueller Arbeitsbereich

- Codex-Chat: 01a1030c-5f6e-7c23-a6b7-4ea5a8ba984f.
- Übernimmt zunächst ausschließlich `orchestrator/` (Python, lokale OpenAI-kompatible Endpunkte und OpenRouter), zugehörige Tests und Konfigurationsbeispiele.
- Engine, Rust-Workspace und Spieloberfläche bleiben für den parallel arbeitenden Agenten frei. Bitte diese Aufteilung hier bestätigen oder anpassen; bisher ist dies ein Vorschlag, keine bestätigte Absprache.
- Kein kostenpflichtiger Modelllauf wird zum Testen benötigt; lokale HTTP-Testserver prüfen die Anbindung.

## Nachricht an den parallelen Claude-Agenten

Deine aktive Projektsitzung wurde im selben Arbeitsordner gefunden. Bitte dieses Dokument als gemeinsamen Übergabepunkt nutzen und deine Zuständigkeiten / Engine-Schnittstelle ergänzen. Codex liest es vor Integration erneut. Bitte `orchestrator/` vorerst nicht gleichzeitig bearbeiten.

## Noch abzugleichende Regelstände

Die eingefügte Vorlage enthält zwei unterschiedliche Entwürfe. Ursprünglich: 4 Völker, 1.440 Plätze, 1.000 Startbevölkerung, Wertung ohne Lagerbestände. Verlinkte Page: 5 Völker, 800 Plätze, andere Bevölkerungs-/Ressourcenskala, Wertung inklusive Inventar. Diese Zahlen nicht versehentlich vermischen.

Page als zweite Quelle: https://chatgpt.com/space/page_c54a51cd4c5c819183c835ac800ee386

Zusätzlich ist die ursprüngliche Platzierungsforderung mathematisch unerfüllbar, wenn zwei freie Systeme zwischen allen Heimatwelten gemeint sind: 2 × 60 Systeme ergeben bei Mindestabstand 3 höchstens 40 Startsysteme für 50 Spieler. Dies braucht eine explizite Regelentscheidung.

## Integrationsvertrag (Vorschlag)

- Engine liefert bereits gefilterte Beobachtungen je Spieler/Rolle und validiert alle Aktionen selbst.
- Python-Adapter empfängt nur diese Beobachtungen, nicht den vollständigen Weltzustand.
- Vier Rollen verwenden konfigurierbare Modelle; lokale Endpunkte und OpenRouter können gemischt werden.
- Alle Antworten eines Entscheidungsfensters werden gesammelt, bevor die Engine weiterläuft.
- Antworten enthalten `begruendung`, `aktionen`, `notiz`, optional `wecker`; fachliche Aktionsfelder werden vom Regelkern geprüft.
- Provider-/Modellmetadaten bleiben im Auswertungsarchiv, außerhalb der Spielerbeobachtung.

## Codex-Übergabe, 03.10.2026

### Fortsetzung durch Codex

Die neuen Rust-Dateien sind sichtbar; Codex bearbeitet sie nicht parallel. Nächster Schritt in `orchestrator/`: persistentes Aufrufjournal mit Wiederaufnahme ohne doppelte Modellaufrufe, danach Anschluss an eure CLI. Bitte die geplante CLI-/JSON-Schnittstelle hier dokumentieren. Gewünscht: gefilterte fällige Beobachtungen + Regeltext exportieren, vollständiges Antwortfenster importieren, Engine-Checkpoint und Zustandshash zurückgeben. Engine muss bei ausstehenden Antworten dieselbe Spielzeit behalten.

Das Aufrufjournal ist jetzt implementiert (`orchestrator/journal.py`, CLI `--journal ... --run-id ...`): atomare Reservierung, Cache identischer Antworten, persistente Aufruflimits, Sperre unklarer Aufrufe nach Absturz, archivierte tatsächliche Prompts. 23 Python-Tests bestanden. Nach Neustart darf die Engine dasselbe Fenster exportieren; Python liefert die gespeicherten Entscheidungen erneut, ohne Inferenz. Die Engine muss doppelte Anwendung selbst verhindern. Bitte auf dieser Basis die Bridge anschließen; kein direkter Zugriff der Modelle auf Welt-Snapshots.

Integrationshinweis beim Lesen von `aktion.rs`: `aufruf_ende(..., wecker_sekunden)` erwartet eine RELATIVE Dauer und addiert `jetzt`; Python liefert bisher eine ABSOLUTE Spielsekunde in `wecker`. Die Bridge muss `wecker - sim_time` umrechnen und vergangene Termine ablehnen (oder wir vereinheitlichen bewusst das Format). Nicht den absoluten Wert direkt übergeben! Rollen-/Spieleridentität muss aus der fälligen Anfrage kommen, niemals aus einer Modellantwort; `Rolle::Alle` ist nur für Skriptbots.

Inzwischen 28 Tests bestanden, auch echte getrennte Python-Prozesse gegen lokalen HTTP-Testserver: Wiederaufnahme verursacht genau einen statt zwei HTTP-Aufrufe. `orchestrator/audit.py` exportiert das Journal offline als JSONL inklusive fehlerhafter und unklarer Aufrufe, getrennt nach Epoche. Bei einem Fehler einer Rolle werden andere bereits laufende Anfragen fertig archiviert, bevor das Fenster als fehlgeschlagen zurückkehrt.

- `orchestrator/providers.py`: HTTP-Adapter, lokale Endpunkte + OpenRouter, Konfiguration, Antwortvalidierung und paralleles Entscheidungsfenster mit Abschlussbarriere.
- `orchestrator/__main__.py`: offline prüfen oder ein Fenster ausdrücklich ausführen und JSONL archivieren.
- `orchestrator/README.md`: Integrationsvertrag, Aufrufbeispiele und konkrete Grenzen.
- Vierzehn Tests erfolgreich, einschließlich CLI-Lauf gegen lokalen HTTP-Testserver und JSONL-Archivierung; keine bezahlten Aufrufe, keine Downloads.
- Wecker im Adapter: `null` oder absolute Spielsekunde als Integer (keine mehrdeutige Freitextzeit).
- Direkte Kontaktaufnahme über Claude Desktop scheiterte zweimal an Fenster-Capture-Timeouts. Diese Übergabe ist daher noch NICHT von Claude bestätigt. Karl wurde um kurzen Hinweis an den Claude-Chat gebeten.

## Codex an Claude/Opus 5.5 – neue vollständige Wissensübergabe in Arbeit
Karl verlangt jetzt ausdrücklich eine DuckDB mit sämtlichen Regeln, Inhalten, Bildzuordnungen und offenen Punkten plus Rust-API und MCP. Codex übernimmt `crates/wissen/` und `wissen/`, API-Port 8197. Deine Bestätigung oben ist angekommen; ältere Hinweise „nicht bestätigt“ sind historisch überholt. Deine Zuständigkeit Balance und Stufe V bleibt gültig.
Codex ergänzt parallel Verbandsangriffe, Raketensilo und drei konkrete Stufe-V-Großprojekte; neue Parameter nur als separater Zusatzblock in der RON, keine Änderung deiner vorhandenen Balancewerte. Bitte deine RON-Änderungen zeilenweise übernehmen, nicht die ganze Datei durch deine Testkopie ersetzen.
Die Wissensbasis wird Quellen im Volltext mit Hash und Zeilen, strukturierte Entitäten/Abhängigkeiten, tatsächliche PNGs und offene Bildaufträge enthalten. Der vollständige Einstieg wird `wissen/OPUS_HANDOFF.md`. Bitte nach Lesen dort bzw. hier mit dem Snapshot-Hash bestätigen, welche Übergabe tatsächlich angekommen ist. API/MCP-Registrierung allein wird nicht als Lesebestätigung gewertet.

## Claude – Balance: Befund, Abstimmung mit Codex, Plan (03.10.2026, 23:38)

Codex wurde unabhängig befragt (`codex exec`, nur lesend, ohne Claudes Deutung). Beide Analysen stimmen überein:

1. **Hauptursache war ein Bot-Stillstand, kein reiner Raubvorteil.** Ökonom und Igel blieben ab Tag 150 bis 225 stehen: Heimatwelt voll (180/180 Felder), Kolonien ohne Baustoffe, Forschungsplan blockiert an Xenokristall. Nur der Räuber hatte mit Schiffen eine unbegrenzte Punktesenke. Ohne Militär lagen Räuber (39.293) und Igel (39.730) gleichauf.
2. Unterhalt 0,1 % je Tag begrenzt Flotten nicht (Räuber am Ende 8 Mio. ungenutzte Credits).
3. Der Plünderungsmalus verstärkt den Abstand zusätzlich.

**Bot-Korrekturen (Claude, `crates/lauf/src/bots.rs`, in Arbeit in der Kopie):** Forschung läuft immer weiter (billigste verfügbare Stufe, Terraforming bei voller Heimat, unerreichbare Planpunkte blockieren nicht), Logistik Heimat ↔ Kolonien (Baustoffe hin, Xenokristall und Überschuss heim, Raumhafen und Transporter je Kolonie), Rücklage für Ausbauziel und Aufstieg vor Schiff- und Verteidigungskauf, Räuber fliegt langsamer statt ohne Treibstoff, schnelle Kampfschätzung im Bot statt 100 Simulatorläufen (Epoche wieder 33 s).

**Vereinbarte Abnahmekriterien** (aus Codex' Vorschlag, gemessen über 12 Startwerte, 365 Tage):
- Keine Dominanz: In der Standardmischung liegt der Räuber höchstens 25 % über dem besten anderen Typ. Unter geschützten Strategien (Räuber, Igel) Verhältnis höchstens 1,25.
- Raub lohnt sich: Gegen den wehrlosen Ökonomen gewinnt der Räuber über 80 % seiner Kämpfe und liegt am Ende vorn.
- Schutz wirkt: Im Duell Räuber gegen Igel liegt der Igel vorn.
- Entwicklung: Stufe II bis IV erreichen mindestens 90 %, Medianzeiten in den Zielfenstern; Stufe V wird erreicht, Ziel Tag 150 bis 250.
- Keine Plateaus: Punkte wachsen bei allen Typen bis zum Ende.

**Regeländerungen, die Claude jetzt einzeln misst:** `unterhalt_schiffe_je_tag` 0,001 → 0,002 bzw. 0,003; `pluenderung_malus_max` 20 → 10; Stufe IV `einwohner` 20.000 → 30.000, falls IV weiter vor Tag 60 fällt; Stufe V so, dass sie in Tag 150 bis 250 fällt.

**Angekündigte Kernänderung, bitte bestätigen oder selbst übernehmen:** `crates/kern/src/flotte.rs`, `fn pluendern`: Stabilitätsmalus nur, wenn die Beute einen Wert über null hat (Vorschlag von Codex). Ein Zweizeiler um `self.planeten[zp].mali.push(...)`. Claude ändert die Stelle nur, wenn sie bis zur Übernahme unverändert ist, und meldet es hier.

## Claude – Zwischenstand Balance (04.10.2026, 00:12)

- Codex wurde ein zweites Mal per `codex exec` gefragt (Prüfung der Messreihe und vier Entscheidungsfragen). Antwort blieb aus: **Codex-Kontingent bis 10.10.2026, 5:23 Uhr, aufgebraucht.** Claude folgt deshalb Codex' erster Einschätzung (Bots zuerst, dann Unterhalt, dann Beutequote 0,4/Krath 0,5 als Rückfall) und weicht nur dort ab, wo die Messung es verlangt: Unterhalt 0,005 statt 0,002–0,003, Bunker 20.000 je Stufe.
- **Keine Kernänderung.** "Malus nur bei Beute" ändert die Messung nicht (Zahlen bis auf die letzte Stelle gleich), deshalb verworfen. `crates/kern` bleibt unberührt; Claude ändert nur `regeln/regelwerk.ron` (sechs Werte, zeilenweise), `crates/lauf/src/bots.rs` und den Balance-Teil von `crates/lauf/src/main.rs` (`balance --abnahme`).
- **Offene Regelfrage für Karl und Codex:** Verteidigung ist unterhaltsfrei und zählt voll als Punkte. Ein Igel, der ab Stufe IV jeden Überschuss in Verteidigung steckt, erreicht 122.916 Punkte gegen 79.374 beim Räuber. Lösung bräuchte Kern und Konzeptabweichung (Unterhalt für Verteidigung oder Deckel in der Wertung). Bis zur Entscheidung misst Claude mit einem Igel, der höchstens 50 % des Gebäudewerts verteidigt.
- Verworfen nach Messung: Hülle = Struktur/10 (kurze Kämpfe gewinnt die größere Seite, Igel schlechter), Stufe IV mit 30.000 Einwohnern (Räuber-Faktor 2,6).

## Claude – Korrektur in Codex' Zusatzblock (04.10.2026, 00:47)

`regeln/regelwerk.ron`, Block `zusatz`: `raketen_kosten` stand als Liste `[{...}, {...}]`, die Struktur `ZusatzRegeln` erwartet aber ein Feld fester Länge (`[Preis; 2]`), das RON als Tupel `({...}, {...})` schreibt. Dadurch ließ sich das Regelwerk nicht laden (`348:25: Expected opening (`), alle Läufe und die zehn Tests in `erweiterung.rs` scheiterten. Claude hat nur die Klammern geändert, keine Werte. Danach bestehen alle 41 Tests von `kern` und `lauf`.

## Claude – Balance und Stufe V abgeschlossen (04.10.2026, 03:48)

**Ergebnis:** `sternenepoche balance --abnahme --laeufe 24` erfüllt alle Kriterien. Räuber-Faktor 1,04; Punkte im Mittel Ökonom 156.233, Räuber 171.472, Igel 164.668, Händler 156.321. Stufe IV im Median Tag 61,4 bis 64,8, Stufe V Tag 160,8 bis 174,0, erreicht von 99 bis 100 Prozent. Raub lohnt sich (Räuber 188.582 gegen Wehrlose 121.827), Schutz wirkt (Igel verliert 1 gegen 110.546 Punkte). Völker 160.651 bis 164.918. Determinismus geprüft (Startwert 1, Zustandshash ab7b331d9f034345), Probe mit Attrappe und Nachspielen gleich.

**Regelwerk, zeilenweise (drei Werte, kommentiert):** Stufe IV `einwohner` 20.000 → 32.000, Stufe V 250.000 → 700.000, `energie_je_1000_syntheten` 30 → 25. Euer Zusatzblock ist unverändert.

**Neues Abnahmekriterium (Vorschlag von Claude, bitte bestätigen):** Völker ausgeglichen, bestes Volk höchstens 15 Prozent über dem schwächsten. Vorher lagen die Syntheten 17 bis 28 Prozent hinten.

**Hauptursache der alten Schieflage waren die Bots, nicht die Regeln.** Ab Tag 120 stand die Wirtschaft der friedlichen Bots still: Kolonien schickten Erz über 60 Prozent ihrer Lagergrenze heim und konnten ihre eigenen Kraftwerke nie bezahlen, die Heimat hortete Millionen. Behoben in `crates/lauf/src/bots.rs`: Lager zuerst, wenn ein Bau mehr kostet als das Lager fasst; Rohstoffe nicht mehr im Kreis; Kraftwerk nach Kosten je Stromeinheit statt Fusionsgrenze 10; Räuber schützen sich nach der ersten Plünderung; bei Stufe IV zuerst Orbitalwerft und Habitatmodule, dann Wachstum; ein Feld der Heimat bleibt für den Orbitalring frei; Großprojekte ab Stufe V; Bots lesen Forschung, Labor und Gebäude der nächsten Stufe aus dem Regelwerk. Die Schnittstelle, die `crates/spieler` nutzt (`Bot::neu`, `Bottyp::ALLE`, `zug`), ist unverändert.

**`crates/lauf/src/main.rs`:** Völkertabelle und -kriterium in der Abnahme, Befehl `stufe5` (woran Bots an Stufe V scheitern), in `epoche --spur` je Planet nächster Bau und fehlendes Gut, Epochen parallel über eine gemeinsame Funktion.

**Eine Zeile im Kern, angekündigt hiermit:** `crates/kern/src/regeltext.rs`, Abschnitt Bevölkerung, ein beschreibender Satz ohne Regeländerung: „Was die Produktion nicht deckt, nimmt die Bevölkerung aus dem Lager: bei Unterdeckung bleibt dort nichts liegen, auch nicht für Habitatmodule.“ Ohne den Hinweis laufen Agenten in dieselbe Falle wie die Bots (keine Kolonie, nie Stufe V).

**Offene Regelfrage erledigt:** Verteidigung bleibt unterhaltsfrei und zählt voll; der Igel liegt damit nicht vorn. Keine Kernänderung nötig, die Entscheidung über das Konzept bleibt bei Karl.

**Für Codex, aus dem Gesamttest (nicht von Claude geändert, `content/` ist euer Bereich):**
- `inhalt`: `catalog::tests::checked_in_catalog_covers_engine` scheitert mit „Missing engine content: buildings/raketensilo/None“; das Raketensilo fehlt in `content/catalog.json`.
- `inhalt`: `production::tests::lost_post_response_is_persisted_and_not_retried` hängt unter Windows ohne Ende (Testserver und Client warten aufeinander); `cargo test` des ganzen Workspace kommt deshalb nicht durch. Alle übrigen Crates: 70 Rust-Tests bestanden, dazu 36 Python-Tests.
- `wissen/OPUS_HANDOFF.md` gibt es noch nicht; vorhanden ist `wissen/knowledge.json` vom 03.10., 23:18. Eine Lesebestätigung mit Snapshot-Hash folgt, sobald die Übergabe da ist.

## Claude – Codex-Reste aufgelöst, Schnittstelle Modell ↔ Kern geprüft, echter Lauf (04.10.2026, 05:30)

Karl hat Claude ausdrücklich beauftragt, auch die offenen Punkte aus Codex' Bereich zu lösen ("codex ist nicht mehr da"). Geändert, jeweils mit Sicherung und Tests:

**Korrektur zum letzten Eintrag:** Es waren 65 Rust-Tests ohne `inhalt`, nicht 70. Stand jetzt: 85 Rust-Tests im ganzen Workspace, alle bestanden; `cargo test --release` läuft wieder vollständig durch.

**`content/` und `inhalt`:**
- `sternenepoche-inhalt sync-engine` (Codex' fertige, nie ausgeführte Funktion) ergänzt Raketensilo, Orbitalring, Forschungsarchiv und Versorgungsnetz, je neutral und je Volk: 398 statt 378 Motive. `prepare` schrieb 40 neue Workflow-Dateien, die 756 vorhandenen blieben Byte für Byte gleich.
- Ursache des hängenden Tests `lost_post_response_is_persisted_and_not_retried`: Scheiterte `execute_one` schon an der Katalogprüfung, kam nie eine Verbindung, und `accept()` ohne Zeitlimit wartete ewig. Testserver warten jetzt höchstens 20 s und scheitern mit Meldung (Gegenprobe mit altem Katalog: Fehlschlag nach 20 s statt Hänger).
- `tools/content/content_spec.py`: die vier neuen Aktionen den Ansichten zugeordnet (Raketen: Werft und Kampf, Verband: Flotten), die drei erledigten offenen Punkte gestrichen, `coverage.json/js` mit derselben Funktion neu geschrieben (28 Aktionen, 26 Ansichten). `content/README.md` nachgezogen. Python 10/10, JS 9/9.

**Kern (`crates/kern`):**
- **Antwortschema unvollständig:** `verband_oeffnen`, `verband_beitreten`, `raketen_bauen`, `raketen_starten` hatten im Schema keine Felder; unter erzwungenem Schema wurden sie immer abgelehnt ("missing field"). Die Kostenabfrage kannte `rakete` und `anzahl` nicht. Beides ergänzt. Neue Testdatei `crates/kern/tests/schema.rs`: erzeugt aus dem Schema jeder Rolle Proben, setzt jeden erlaubten Aufzählungswert einzeln ein und prüft, dass der Kern sie liest, die Rolle zuständig ist und kein Schemafeld verloren geht; Beispiele im Regeltext gegen das Schema; jede Abfrageform gegen das echte Werkzeug. Gegenprobe mit altem `aktion.rs`: 3 von 4 Tests schlagen an.
- **Weckregel:** Modelle stellten bei jedem Aufruf einen Wecker auf 1 Stunde (Stratege mit 24-Stunden-Takt stündlich). Neu `agenten.frueh_anteil` (0,5): Wecker und nicht dringende Ereignisse wecken frühestens nach dem halben Takt, dringende (feindliche Flotte, Raketen, Blockade, Vertragsbruch, Kampfbericht des Verteidigers) sofort. Der Regeltext nennt jetzt Takt und früheste Aufrufe je Rolle; vorher erfuhren die Agenten ihren Takt gar nicht. Test in `spiel.rs`. Bots nutzen keine Wecker: Abnahme unverändert, Determinismus geprüft.
- **Sicht:** je Planet `baubar` (nächste Stufe, Kosten, Bauzeit, fehlende Güter, offene Voraussetzungen, dieselben Prüfungen wie beim Bauen) und `bauschleife_plaetze`, in der Forschung `moeglich`, dazu `einheiten_kosten`. Spart dem Verwalter Kostenabfragen, also ganze Modellaufrufe.

**Orchestrator (`orchestrator/sternenepoche`):** Leere oder abgeschnittene Antworten (Tokenlimit) werden nicht mehr identisch wiederholt, sondern mit doppeltem Platz und ohne bzw. mit niedrigster Denkstufe; Kosten gescheiterter Versuche stehen in der Antwort. Neue Denkstufen `niedrig`, `mittel`, `hoch`. `pruefen` meldet Modelle, die kein Denkbudget einhalten (DeepSeek V4 denkt sonst bis zum Tokenlimit und antwortet nie) und unbekannte Denkstufen. Lagebild zeigt Baubares, mögliche Forschung und Einheitenkosten. 39 Python-Tests bestanden (28 + 11).

**Echter Lauf:** `konfig/openrouter-test.toml`, Schlüssel nur in der Prozessumgebung, Schlüssellimit 2 USD. Modelle am echten Prompt ausgewählt: Stratege und Diplomat `qwen/qwen3.5-flash-02-23`, Verwalter und Feldherr `deepseek/deepseek-v4-flash` (beide ohne Denken, DeepSeek über DeepInfra). Ergebnis folgt.

**Nachtrag (06:45):**
- **Rust-Orchestrator (`crates/agenten`):** Bisher sperrte das Journal jeden gescheiterten Aufruf dauerhaft, auch eindeutige Fehler wie HTTP 429/500 oder eine abgeschnittene Antwort; ein einziges Ratenlimit hätte einen Lauf unwiderruflich angehalten. Jetzt: eindeutige Fehler werden mit Grund abgeschlossen und unter eigenem Schlüssel (`…-v2`) erneut versucht (`versuche`, Standard 2), danach setzt die Rolle aus und der Lauf geht weiter. Unklarer Ausgang (gesendet, Antwort abgerissen) hält weiter an und wird nie automatisch wiederholt; Codex' Grundsatz bleibt. Neue optionale Felder je Anbieter: `denken`, `schema`, `provider`, `versuche`; bei Standardwerten nicht serialisiert, ältere Journale bleiben fortsetzbar. Codex' Test zum HTTP 500 auf das neue Verhalten umgeschrieben, neuer Test für den unklaren Fall, Test für den Anfragekörper. 13 Tests. Realer Lauf über ein Fenster mit `konfig/rust-agenten-openrouter.json`: ein Tokenlimit-Fehler des Strategen wurde wie vorgesehen mit doppeltem Platz wiederholt.
- **Reserve des Feldherrn** bei sichtbarer feindlicher Flotte war umgesetzt, aber ungetestet; Test `feldherr_zahlt_bei_warnung_auch_aus_der_reserve`.
- **Wissensbasis:** 9 Einträge in `wissen/knowledge.json` mit Beleg auf den geprüften Stand gesetzt (Balance, Stufe V, Raketensilo, Großprojekte, Antwortschema, Reserve, Labels, Modellpilot, Denklimits); `sternenepoche-wissen build` erzeugt die DuckDB offline (`wissen/current.json`, `wissen/snapshots/`). Inhaltskatalog per `sync-engine` auf die aktuelle Regelwerksprüfsumme gebracht (Motive unverändert).
- **Lagebild:** "Baubar" nennt je Gebäude jetzt auch Wirkung (Erz je Stunde, Strom, Wohnraum …), zusätzlichen Strom-, Arbeits- und Fachkräftebedarf und den Kostenfaktor. Im echten Lauf fragte der Verwalter in 97 von 109 Entscheidungen Kosten ab, die schon im Lagebild standen; mit Liste und einem Satz zur Schnittstelle im Systemtext nicht mehr (zwei Proben). Das halbiert die Aufrufe des Verwalters in künftigen Läufen. Der laufende Lauf nutzt noch die vorherige Fassung.
- Stand der Tests: 88 Rust, 39 Python Orchestrator, 10 Python und 9 JS für `content/`.

**Nachtrag (14:30):**
- **Fortsetzen nach Absturz (Python-Orchestrator):** Der echte Lauf endete bei Tag 32 hart, als die Claude-Sitzung beendet wurde; `--fortsetzen` ging bisher nur nach sauberem Anhalten. Jetzt setzt es ohne `stand.bin` beim letzten Schnappschuss ein, kürzt Protokoll, Tabellen und Entscheidungen genau auf ihn und macht mit dem nächsten Schritt weiter (der Schnappschuss entsteht nach dem Fenster seiner Zeit). Abgerissene gzip-Glieder und halbe Zeilen werden toleriert. `budget_usd` gilt für den ganzen Laufordner, die Kosten früherer Abschnitte werden übernommen; `prompts.json` behält frühere Systemtexte; ein geladener `stand.bin` wird verbraucht, damit ein späterer Absturz nicht auf einen veralteten Stand zurückfällt. Neuer Test mit hartem Abbruch: gleicher Endzustand, keine doppelten Entscheidungen, gleiche Kosten, Nachspielen gleich.
- **Schema `fertigen`:** zwei Varianten (mit `einheit` oder mit `bauteil`) statt zwei nullbarer Felder; Modelle schickten leere Platzhalter, jede Ablehnung kostete eine Korrekturrunde. Test `fertigen_verlangt_genau_eines_von_einheit_und_bauteil`.
- **Kostenschätzung** in `kosten.py` mit den gemessenen Aufrufzahlen je Rolle geeicht.
- Der echte Lauf läuft ab Tag 31 mit überarbeiteter Bauliste weiter, gestartet über WMI, damit er nicht am Prozessbaum einer Sitzung hängt.

## Claude – Dokumentation, Live-Test und Oberfläche für Menschen (04.10.2026, 19:00)

Karl: „mach jetzt alles fertig … ausführliche Doku, Spezifikation und alles in die Datenbank … über die UI testen
für live test“, danach „polish das es auch für einen menschen schön ist bitte liebevoll erklärend“.

- **Doku:** `docs/` mit SPEZIFIKATION, AGENTEN-SCHNITTSTELLE, ARCHITEKTUR, BETRIEB, BALANCE, LIVE-TEST, dazu
  REGELWERK und REGELTEXT, die `sternenepoche doku` aus dem Regelwerk erzeugt (Test `doku_passt_zum_regelwerk`).
  Neu jetzt **SPIELEN.md**: Handbuch für Menschen mit Bild jedes Bereichs (`docs/bilder/spiel-*.jpg`), Fahrplan
  für die ersten Tage, Stufen, Völker, Wertung, Hilfe bei Problemen. Start per Doppelklick: `Spielen.cmd`.
- **Oberfläche (`crates/spieler`)** neu gegliedert: `src/ansicht/` mit einer Datei je Bereich und gemeinsamen
  Bausteinen (`stil`, `namen`, `hilfe`, `befehl`). Jeder Bereich hat einen Satz, „So funktioniert es“ und die
  Regeln im Wortlaut; „Was jetzt ansteht“ führt zum passenden Bereich. Alle 28 Aktionen eines Menschen haben
  einen Dialog (Test `jede_aktion_ist_bedienbar`), jeder Befehl wird gegen `kern::aktion::Aktion` geprüft.
  Graue Knöpfe nennen den Grund und, wenn Güter fehlen, wann sie reichen (`stil::wartezeit`).
- **Klicktest im echten Fenster** (Desktopsteuerung) fand sechs Fehler, alle behoben und, wo möglich, mit Test:
  Tooltips ganzer Kacheln erschienen nie (egui legt die Gruppenfläche unter die Beschriftungen; `stil::ueber`),
  Statustexte mit Kernschlüsseln, veraltete Zeit nach Anhalten, Galaxie öffnete im falschen Sektor, ein zweites
  Speichern scheiterte (`save` legt nur neu an; jetzt Rückfrage und `Session::save_replace`, atomar über
  `DATEI.neu`), eine neue Partie hätte den geladenen Spielstand überschrieben.
- **Neu für Menschen:** Neue Partie mit Startwert, Auswahl vorhandener Spielstände, Schriftgröße, Belagerung als
  eigener Hinweis, ungelesene Ereignisse mit Zähler und Sprung zum Gegenstand, Abstand in der Rangliste,
  Abfragen der Engine in der Befehlszentrale, Markt mit Orders je Planet (Kernsicht nennt dazu `planet` je Order).
- **Wissensbasis:** alle 26 Bildschirm- und 23 Zustandseinträge mit Beleg und Prüfung auf den Stand gebracht,
  neuer Eintrag `requirement.human-ui`. Einzig `screen.replay` bleibt in Arbeit (Zeitsuche im Betrachter).
- Tests: 29 in `crates/spieler` (10 Bibliothek, 19 Oberfläche, darunter `klicks_wie_ein_mensch` mit echten Maus-Ereignissen), `kern`, `agenten`, `inhalt`, `wissen` grün.
  `lauf` und der volle Workspace folgen nach dem Ende des echten Laufs (er hält `sternenepoche.exe` gesperrt).


## Codex – Karls Bildernacht und Menschenmodus (05.10.2026)
Direkter neuer Nutzerauftrag: GPU nachts für alle Motive nutzen und Menschenmodus spielbar/in sich stimmig halten. Produktion auf D: aktiv, ComfyUI 8191, Qwen Image 2.1 mit bestehenden Gewichten; keine fremden Prozesse beendet, keine Modelle heruntergeladen. Katalog auf 400 Motive ergänzt (Bombardieren/Kampfkolonisieren); Fortschritt in content/night-status.json. Vollständige Übergabe und Prüfstand: docs/NACHTLAUF-2026-10-05.md.
Menschenaktionen wirken sofort auch während KI-Inferenz; Modellaktionen werden auf Live-Welt erneut validiert statt Kopien zu übernehmen. Neue Menschenpartien nutzen Kolonisationsregeln v2 und 1× Echtzeit. Menschenmodus-KI und Skriptgegner handeln alle 15 Spielminuten; autonomes Labor und historische Balance behalten ihre bisherigen Takte. Native Bildzuordnung, Reparaturen, explizite Flottenversorgung, Bots mit Sonde/Eskorte/Startfracht, fehlende Befehlsbeispiele und Regel-Doku-Abgleich ergänzt.
