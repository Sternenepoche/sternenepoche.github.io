# Dokumentation der Sternenepoche

**Aktueller Einstieg, 8. Oktober 2026:** Die gemeinsame Onlinewelt läuft auf dem PC-Server,
erreichbar über [TLS](https://desktop-3dei636.taila4f584.ts.net/). GitHub Pages veröffentlicht den
Browserclient. 30 Skriptbots spielen neben bis zu 20 Teilnehmern; zunächst sind drei Plätze freigegeben.
Die Browseroberfläche richtet sich an Menschen, Agenten und Mischbetrieb. Das lokale Dashboard verwaltet
den gemeinsamen Dienst. Grundlage sind das aktive Onlineprofil und der gemeinsame Rust-Spielkern.

Das Forschungslabor mit persistenten Spielerbüros und der native Menschenmodus sind zusätzliche
Betriebsarten. Ihre älteren Beschreibungen dürfen nicht mit dem Takt und den Platzregeln der Onlinewelt
gleichgesetzt werden. Das Labor hält die Welt während Modellentscheidungen an; der Onlinehost läuft weiter.

| Dokument | Inhalt | Für wen |
|---|---|---|
| [Sternenepoche Spielguide](../Sternenepoche-Start.html) | Bebilderte Singlepage: erste Spielsitzung, Bedienwege für alle 17 Bereiche, durchsuchbare Spielregeln mit Beispielen und einfacher Serverstart | Spieler und Betreiber |
| [HANDBUCH.md](HANDBUCH.md) · [HTML zum Öffnen](../Sternenepoche-Handbuch.html) | Anmeldung, Völker, Mensch/Bot/Agent/Mischbetrieb, Bedienung, Spielregeln mit Beispielen und Serververwaltung | Spieler und Betreiber |
| [ONLINE-KONZEPT.md](ONLINE-KONZEPT.md) | Verbindliche Onlinearchitektur, Informationsgrenze, Anmeldung, Aufklärung, Kampf, Saven und Ausscheiden | Spiel, Entwicklung |
| [SERVER-BETRIEB.md](SERVER-BETRIEB.md) | PC-Betrieb, TLS, Verwaltung, Backups, Epochenprüfung und VPS-Vorbereitung | Betreiber |
| [ONLINE-REGELN.md](ONLINE-REGELN.md) | Erzeugte Referenz des Onlineprofils; der Server liefert das tatsächlich aktive Profil | Nachschlagen |
| [LABOR-BACKEND.md](LABOR-BACKEND.md) | V4: native Rust-Büros ohne Docker, faire Entscheidungsfenster, Hauptmodelle, Ollama-only, Kolonieplanung, Besitzwechsel und versioniertes Lernen | Backend, Betrieb, Forschung |
| [LABOR-ABNAHME.md](LABOR-ABNAHME.md) | Ausgeführte Tests, echte Modellproben und offene Abnahmen | Prüfung, Forschung |
| [LABOR-REVIEW-SOL-HIGH.md](LABOR-REVIEW-SOL-HIGH.md) | Angeforderte Gegenprüfung mit gpt-6.1-sol high | Entwicklung |
| [SPIELER-SANDBOX-KONZEPT.md](SPIELER-SANDBOX-KONZEPT.md) | Neue Zielarchitektur: dauerhafte Spieler, anpassbare Rollen/Skills, Tool-Schnittstelle, Ollama/OpenRouter-Broker, Paarstarts, Audit, Lernen und Backend-Abnahme | Backend, Spieldesign, Forschung |
| [SPIELEN.md](SPIELEN.md) | Selbst spielen: Start, Bildschirm, Zeit, ein Fahrplan für die ersten Tage, jeder Bereich mit Bild, Stufen, Völker, Punkte, Speichern, Hilfe bei Problemen | alle, die spielen wollen |
| [LIVE-TEST.md](LIVE-TEST.md) | Mit echten Modellen live: selbst gegen KI-Reiche spielen (Spieler-Oberfläche) oder einem Lauf im Browser zuschauen; Checkliste | alle, die es ausprobieren wollen |
| [SPEZIFIKATION.md](SPEZIFIKATION.md) | Die Spielregeln: Zeit, Galaxie, Völker, Wirtschaft, Gebäude, Forschung, Stufen, Militär, Diplomatie, Markt, Wertung, Determinismus, Abweichungen vom Konzept | Spieldesign, Prüfung |
| [REGELWERK.md](REGELWERK.md) | Alle Zahlen als Tabellen, erzeugt aus `regeln/regelwerk.ron` | Nachschlagen |
| [REGELTEXT.md](REGELTEXT.md) | Der Regeltext, den die Modelle je Rolle bekommen, erzeugt aus dem Regelwerk | Prompt-Arbeit, Prüfung |
| [AGENTEN-SCHNITTSTELLE.md](AGENTEN-SCHNITTSTELLE.md) | Was ein Modell sieht, darf und antworten muss: Rollen, Antwortschema, alle Aktionen und Abfragen, Ablauf eines Aufrufs, Wecker, Informationsgrenze, Fehlerverhalten | Modellanbindung |
| [ARCHITEKTUR.md](ARCHITEKTUR.md) | Aufbau: Crates, Orchestratoren, Oberflächen, Zeitmodell, Determinismus, Brücke, Datenformate, Erweitern | Entwicklung |
| [BETRIEB.md](BETRIEB.md) | Bauen, Testen, alle Befehle, Konfigurationen, OpenRouter und Kosten, Fortsetzen, lange Läufe, Auswertung, Wissensdatenbank, Fehlerbehebung | Betrieb |
| [BALANCE.md](BALANCE.md) | Messung mit Skriptbots, Abnahmekriterien, Ergebnis, Änderungen, wie man Schieflagen findet | Balance |
| Echter Lauf | Der Lauf mit echten Modellen über OpenRouter: Modellwahl, Verlauf, Ergebnis gegen die Bots, Kosten, Befunde — steht im Projekt-`README.md`, Abschnitt „Echter Lauf" (die frühere Verweisdatei `docs/ECHTER-LAUF.md` existiert nicht) | Auswertung |

## Lesereihenfolge

- **Online spielen und verwalten:** HANDBUCH, ONLINE-KONZEPT und SERVER-BETRIEB.
- **Forschungslabor:** SPIELER-SANDBOX-KONZEPT, danach ARCHITEKTUR und AGENTEN-SCHNITTSTELLE als Bestandsreferenz.
- **Nativ selbst spielen:** der ältere Teil von SPIELEN (Doppelklick auf `Spielen.cmd`), gegen Sprachmodelle dann LIVE-TEST.
- **Modelle beobachten:** LIVE-TEST, dann SPEZIFIKATION in Auszügen.
- **Ein Modell anbinden oder Prompts ändern:** AGENTEN-SCHNITTSTELLE, REGELTEXT, dann BETRIEB (Konfiguration).
- **Am Code arbeiten:** ARCHITEKTUR, dann SPEZIFIKATION; vor dem Abschluss die Prüfungen aus ARCHITEKTUR 8.4.
- **Regeln oder Balance ändern:** SPEZIFIKATION, REGELWERK, BALANCE.

## Einstiegsseite und Handbuch bearbeiten

**Öffentliche Website, Stand 8. Oktober 2026:** Die Landingpage liegt als Vorlage in
**docs/landingpage.html**. **docs/site-theme.css** und **docs/site-motion.js** verbinden Landingpage,
Spielguide und Handbuch gestalterisch. **python tools/build_website.py** erzeugt alle drei Seiten
und die maschinenlesbaren Fassungen gemeinsam. Die bestehenden Spielgrafiken werden eingebettet;
es gibt keine fremden Schrift-, Skript- oder Bilddienste.

Die 17 aktuellen Browseransichten liegen unter **docs/bilder/online/**. Sie zeigen eine isolierte,
pausierte Rust-Beispielwelt vom 8. Oktober 2026, keine Live-Spielerdaten. Bildunterschriften und
die Zuordnung von Bildern zu Mechaniken werden in **tools/website_content.py** gepflegt.
**python tools/capture_website.py** nimmt diese Ansichten mit dem vorhandenen Chrome und einer
separaten lokalen Testwelt auf D: erneut auf. Die Originalgrafiken bleiben unverändert;
dunkle Hintergründe dekorativer Schiffe und Asteroiden werden nur im Browser ausgeblendet.

Für Browserprüfungen die Website lokal mit **python -m http.server 18888 --bind 127.0.0.1**
ausliefern und **python tools/verify_website.py** starten. Die Prüfung umfasst Desktop/Handy,
Bilder, Anker, Suche, Navigation, Vergrößerung, reduzierte Bewegung und Lesen ohne JavaScript.

Die Gestaltung des bebilderten Spielguides wird in **docs/startseite.html** gepflegt.
Die praktische erste Sitzung, alle 17 Bedienwege und die Spielregeln kommen aus **docs/HANDBUCH.md**.
**python tools/build_startpage.py** erzeugt daraus **Sternenepoche-Start.html** mit eingebetteten
Spielmotiven, Bereichsnavigation und Suche. Die fertige Datei lässt sich allein per Doppelklick öffnen;
zum Lesen und Suchen braucht sie keine Internetverbindung. Die ersten 22 Handbuchkapitel sind direkt
enthalten; die Projektdateiübersicht aus Kapitel 23 bleibt im separaten Handbuch.

Die ausführliche Anleitung steht in **docs/HANDBUCH.md**.
**python tools/build_handbook.py** erzeugt **Sternenepoche-Handbuch.html**.
Beide Seiten verweisen aufeinander.

**Server per Doppelklick:** Im Hauptordner liegen **Server-starten.cmd**, **Dashboard-oeffnen.cmd**,
**Spiel-oeffnen.cmd**, **Server-status.cmd** und **Server-stoppen.cmd**. Ablauf und Voraussetzungen
stehen im Spielguide unter **Den Server per Doppelklick starten** und in Handbuchkapitel 20.

## Was erzeugt ist und was geschrieben

`REGELWERK.md` und `REGELTEXT.md` erzeugt die Engine:

```bash
./target/release/sternenepoche.exe doku
```

Ein Test (`doku_passt_zum_regelwerk` in `crates/lauf`) schlägt an, sobald sich das Regelwerk ändert und die beiden
Dateien nicht neu erzeugt wurden. Alle übrigen Dokumente sind geschrieben und gegen den Code geprüft (Stand
4. Okt. 2026); sie nennen zu jeder Mechanik die Datei und meist die Funktion, damit sich jede Aussage nachprüfen
lässt.

## Wissensdatenbank

Alle Dokumente, Quelltexte, das Regelwerk, der Inhaltskatalog und die Einträge aus `wissen/knowledge.json`
(Anforderungen, Entscheidungen, Befunde, offene Punkte) stehen außerdem durchsuchbar in einer DuckDB-Datenbank:

```bash
./target/release/sternenepoche-wissen.exe build
```

```bash
./target/release/sternenepoche-wissen.exe query wissen/current.json search "{\"q\":\"Weckregel\"}"
```

Lesend als HTTP-API (`serve`, Port 8197, nur 127.0.0.1) oder als MCP-Server (`mcp`). Einzelheiten in
BETRIEB, Abschnitt 9.
