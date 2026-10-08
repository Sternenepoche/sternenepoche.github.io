# Sternenepoche

**8. Oktober 2026 – gemeinsame Onlinewelt:** 30 serverseitige Skriptbots und 20 mögliche
Spielerplätze, davon zunächst drei freigegeben; weitere Anmeldungen kommen auf die Warteliste.
Menschen, lokale Ollama-Agenten, OpenRouter-Agenten und Mischbetrieb spielen dieselbe Rust-Welt.
Vor dem Start werden Volk und Spielweise gewählt; die Vor- und Nachteile kommen aus dem aktiven Regelprofil.

[Online anmelden und spielen](https://desktop-3dei636.taila4f584.ts.net/) ·
[Website](https://sternenepoche.github.io/) · [Spielanleitung](SPIELEN.md) ·
[Architektur und Regeln](ONLINE-KONZEPT.md) · [Betrieb und VPS-Umzug](SERVER-BETRIEB.md).

GitHub Pages liefert Website und Browserclient. Der autoritative Rust-Dienst läuft zunächst auf Karls PC;
Tailscale Funnel liefert den öffentlichen TLS-Zugang ohne Tailscale-Pflicht für Mitspieler.
`Server-starten.cmd` startet Spiel (`http://127.0.0.1:8890`) und getrenntes lokales Dashboard
(`http://127.0.0.1:8891`). Das Dashboard wird nicht öffentlich ausgeliefert.

Der Menschenclient verbindet bebilderte Bau-/Forschungskacheln mit servergestützten Fortschrittsanzeigen
und einem räumlichen Sternenatlas: Galaxie → Sektor → Sonnensystem → Planet.
Systeme erscheinen als Sternwolken, Planeten laufen visuell um die zentrale Sonne.
Typ, Bewohner und Ressourcen bleiben bis zum jeweiligen Sondenbericht unbekannt.
Pause, Niederlage und Epochenende sperren Befehle einheitlich; Ansichten und Berichte bleiben lesbar.
Der Online-Regelsatz `regeln/online-v1.ron` umfasst Kolonisation, Aufklärung, Saven und dauerhaftes Ausscheiden.
Das geschlossene Forschungslabor und der native Einzelspielermodus bleiben eigene Betriebsarten.

## Entwicklungshistorie der weiteren Betriebsarten

**5. Oktober 2026 – Menschenmodus und Bildernacht:** Der Menschenmodus wird parallel zum Forschungslabor gepflegt: sofortige menschliche Befehle, KI-Rollen alle 15 Spielminuten, standardmäßig 1× Echtzeit und aktuelle Kolonisationsregeln für neue Partien. Qwen Image 2.1 erzeugt lokal über ComfyUI 400 Motive. Laufender Stand: `content/night-status.json`; Abnahme und Fortsetzung: [Nachtlauf](NACHTLAUF-2026-10-05.md).

**Neue Zielrichtung (4. Oktober 2026):** Zunächst eine vollständig autonome Forschungsumgebung ohne
menschliche Spieler. Ein Spieler wird als persistente Sandbox mit eigenem Gedächtnis, Pinwand und
veränderbarem Harness betrachtet. Strategische Planung, faire Modellverteilung, passende Weltgrößen und
belastbare Lerndaten haben Vorrang; die UI folgt später.
Das [Spieler-Sandbox-Konzept](SPIELER-SANDBOX-KONZEPT.md) beschreibt die Zielarchitektur.
**Die autonome Rust-Laufzeit V4 ist implementiert:** `sternenepoche-labor` mit kleinen nativen
Windows-Spielerbüros (LPAC + Job Object, ohne Docker), privater SQLite und Pinwand, strategischem
Hauptmodell, nachladbaren Werkzeugen, Ollama-only/Mischbetrieb, 900-Sekunden-Aufruffenstern,
Epochen-Gedächtnis, Paarstarts, Replay und Datensatzexport. Neue Kolonien brauchen Sondenaufklärung,
bewaffnete Eskorte und Startfracht. Kampfkolonisierung braucht vollständig besiegte Verteidigung,
30 % Gebäudeintegrität und zwei Reaktionsfenster. **Nur der ursprüngliche Heimatplanet ist geschützt.**
V4 holt bei Rechenzeitmangel fehlende Calls im selben eingefrorenen Fenster nach, ohne neue Spielerbudgets.
Der faire Strategievergleich und das 15-Minuten-Durchsatzziel werden getrennt bewertet. Hinzu kommen
Kolonieplanung, empfängergebundener Nachschub und knappere gemeinsame Expansionsräume.
Start, genaue Funktionen und verbleibende Grenzen stehen in [LABOR-BACKEND.md](LABOR-BACKEND.md),
einschließlich `plan-capacity`, `balance-report` und gekreuzten Modell×Harness-Versuchen über `factorial`.
Der kurze reale V4-Nachweis: zehn native Qwen-8B-Spieler, 40 Calls und 14 Aktionen in rund 96 Sekunden.
Ein konservativ aus echten Messwerten abgeleitetes Startprofil liegt in
`konfig/labor-native-local-v4-kapazitaet.json` (zwei Spieler, je vier Calls).
Die Prüfbelege stehen in [LABOR-ABNAHME.md](LABOR-ABNAHME.md). Die folgenden Abschnitte dokumentieren den
bisherigen Runner einschließlich seiner bereits vorhandenen UI.

Eine geschlossene Welt, in der 50 Zivilisationen ein Spieljahr lang um die höchste Punktzahl konkurrieren.
Jede Zivilisation wird von vier Sprachmodellen in vier Rollen regiert: Stratege, Verwalter, Feldherr, Diplomat.
Die Engine ist deterministisch, jede Entscheidung wird samt Wirkung gespeichert.

Die Modelle laufen lokal (vLLM, llama.cpp, Ollama), über **OpenRouter** oder gemischt.

Die ausführliche Dokumentation liegt in [`docs/`](README.md): Spezifikation der Spielregeln,
Agenten-Schnittstelle, Architektur, Betriebshandbuch, Balance, Live-Test und der echte Lauf. Alle Zahlen des
Regelwerks als Tabellen stehen in [`docs/REGELWERK.md`](REGELWERK.md), erzeugt aus dem Regelwerk.

## Selbst spielen

Doppelklick auf `Spielen.cmd` (nach einmaligem `cargo build --release`): eine Partie gegen 49 Skriptbots, ganz
ohne Netz. Die Oberfläche erklärt sich selbst (in jedem Bereich „So funktioniert es“, in der Übersicht „Was jetzt
ansteht“, Tooltips an allen Zahlen und grauen Knöpfen); das Handbuch mit Bildern jedes Bereichs und einem
Fahrplan für die ersten Tage ist [`docs/SPIELEN.md`](SPIELEN.md).

![Die Übersicht der Spieleroberfläche](bilder/spiel-uebersicht.jpg)

## Live-Test über die Oberfläche

Selbst gegen KI-Reiche spielen: die Spieler-Oberfläche öffnen (oder `Live-Test.cmd` per Doppelklick), Bereich
**Live-KI**, „Partie mit KI-Reichen starten“. Einige Reiche regieren dann Sprachmodelle über OpenRouter, je mit
vier Rollen; jede ihrer Entscheidungen steht mit Begründung, Befehlen, Urteil des Spielkerns und Kosten im
Protokoll. Den Schlüssel als `OPENROUTER_API_KEY` setzen oder im Programm eintragen (er wird nirgends
gespeichert); ohne Schlüssel geht alles mit der Attrappe.

```bash
./target/release/sternenepoche-spieler.exe --screen Live-KI
```

Einem laufenden Modelllauf im Browser zuschauen (oder `Betrachter.cmd`), mit Punkteverlauf, Galaxie und jeder
Entscheidung der Modelle, alle 30 Sekunden aktualisiert:

```bash
.venv/Scripts/python.exe -m sternenepoche zeigen laeufe/openrouter-test
```

Schritt für Schritt, mit Checkliste: [`docs/LIVE-TEST.md`](LIVE-TEST.md).

## Schnellstart

```bash
cargo build --release
```

```bash
cargo test --release
```

Eine Epoche mit Skriptbots, mit Determinismusprüfung:

```bash
./target/release/sternenepoche.exe epoche --startwert 1 --pruefen
```

Python-Umgebung für den Orchestrator (liegt im Projekt auf D:):

```bash
uv venv .venv --python 3.13
```

```bash
uv pip install --python .venv/Scripts/python.exe -e orchestrator
```

Der ganze Ablauf ohne Modell und ohne Kosten (Attrappe, drei Reiche, fünf Spieltage):

```bash
.venv/Scripts/python.exe -m sternenepoche lauf konfig/probe.toml
```

Ein neuer Lauf in einen Ordner, in dem schon einer liegt, wird abgelehnt, damit nichts Altes überschrieben
wird oder liegen bleibt. Für einen zweiten Probelauf `laeufe/probe` vorher löschen oder in der Konfiguration
`ausgabe` ändern.

Den Lauf aus Startwert und Aktionsprotokoll nachspielen und den Zustandshash prüfen:

```bash
./target/release/sternenepoche.exe replay laeufe/probe
```

## OpenRouter

OpenRouter ist ein Anbieter neben lokalen Servern. Jede Rolle bekommt in der Konfiguration ihren
Anbieter, lokal und OpenRouter lassen sich frei mischen.

Der Schlüssel steht nur in der Umgebung, nie in einer Datei:

```powershell
$env:OPENROUTER_API_KEY = "sk-or-..."
```

Vor dem ersten Aufruf prüfen, was er kostet und ob Modelle und Schlüssel passen. Beide Befehle rufen
kein Modell auf; die Modellliste von OpenRouter ist öffentlich:

```bash
.venv/Scripts/python.exe -m sternenepoche pruefen konfig/openrouter-test.toml
```

```bash
.venv/Scripts/python.exe -m sternenepoche kosten konfig/openrouter-test.toml
```

Dann ein Probetag und danach der Lauf (ein Reich mit den getesteten Modellen gegen 19 Bots, 90 Spieltage):

```bash
.venv/Scripts/python.exe -m sternenepoche lauf konfig/openrouter-probe.toml
```

```bash
.venv/Scripts/python.exe -m sternenepoche lauf konfig/openrouter-test.toml
```

| Konfiguration | Inhalt | Schätzung am 3. Okt. 2026 |
|---|---|---|
| `konfig/probe.toml` | Attrappe, kein Modell | 0 USD |
| `konfig/openrouter-test.toml` | ein Reich mit getesteten günstigen Modellen gegen 19 Bots, 90 Tage | rund 0,02 USD je Spieltag, gemessen |
| `konfig/openrouter-probe.toml` | dasselbe, ein Spieltag | rund 0,02 USD |
| `konfig/openrouter-klein.toml` | ein Modell, 5 Reiche, 30 Tage | rund 2 USD |
| `konfig/hybrid.toml` | Verwalter lokal, drei Rollen über OpenRouter, volle Epoche | rund 100 USD |
| `konfig/openrouter.toml` | vier Modelle über OpenRouter, volle Epoche | rund 230 USD |
| `konfig/vergleich.toml` | Modellvergleich: jedes Modell steuert 12 oder 13 Reiche in allen Rollen | je nach Modellen |
| `konfig/lokal.toml` | vier lokale Server | Strom |

Die Schätzung rechnet mit dem Mengengerüst des Konzepts und den Preisen der Modellliste. Den größten
Posten stellt der Verwalter: er hat die meisten Aufrufe und den längsten Regelauszug. Alle 6 statt alle
4 Spielstunden (`takt_stunden` im Regelwerk) spart ein Drittel davon.

Was die Anbindung leistet:

- **Erzwungenes Antwortschema.** Das JSON-Schema der Antwort erzeugt die Engine aus ihrer eigenen
  Aufzählung der Aktionen, je Rolle. OpenRouter bekommt es als `response_format` mit `strict`, dazu
  `provider.require_parameters`, damit nur Endpunkte antworten, die es wirklich erzwingen. Lokal wird
  daraus eine Grammatik (llama.cpp) oder `guided_json` (vLLM).
- **Gleiche Gewichte für alle.** OpenRouter verteilt sonst auf Provider mit verschiedener
  Quantisierung. `provider.order`, `allow_fallbacks = false` und `quantizations` pinnen das.
  Der tatsächliche Provider jeder Antwort steht im Datensatz (`upstream`).
- **Denken.** `denken = "budget"` begrenzt das Denken auf `denk_tokens`, `"niedrig"`, `"mittel"` und
  `"hoch"` setzen eine Denkstufe, `"aus"` schaltet es ab. Nicht jedes Modell hält ein Budget ein:
  DeepSeek V4 denkt dann bis zum Tokenlimit und antwortet nie. `pruefen` liest aus der Modellliste,
  was ein Modell kann, und meldet solche Einstellungen vorab. Denktokens zählen als Ausgabe.
- **Neuversuche mit Verstand.** Endet eine Antwort am Tokenlimit (leer oder mitten im JSON), wird
  derselbe Aufruf nicht noch einmal geschickt: der nächste Versuch bekommt doppelten Platz und denkt
  weniger (Budget wird zu "aus", eine Stufe zu "niedrig"). Was gescheiterte Versuche gekostet haben,
  steht bei der Antwort im Datensatz.
- **Aufrufe und Wecker.** Jede Rolle kommt im Regeltakt dran (Stratege 24, Verwalter 4, Feldherr 12,
  Diplomat 24 Stunden) und bei Ereignissen ihres Bereichs. Ein Wecker oder ein nicht dringendes
  Ereignis weckt frühestens nach dem halben Takt (`agenten.frueh_anteil`); dringende Ereignisse wie
  eine anfliegende Flotte sofort. Ohne diese Grenze stellten Modelle bei jedem Aufruf einen Wecker auf
  eine Stunde, und ein Lauf kostete ein Vielfaches.
- **Kosten und Budgetgrenze.** Die Kosten jeder Antwort kommen aus `usage.cost`. Bei
  `budget_usd` hält der Lauf zwischen zwei Fenstern an, speichert den Stand und läuft mit
  `--fortsetzen` exakt weiter. Die Grenze gilt für den ganzen Laufordner: beim Fortsetzen zählen die
  bisherigen Kosten mit. Ein Test belegt: angehalten und fortgesetzt ergibt denselben Endzustand.
- **Absturz.** Endet der Prozess hart (Rechner aus, Sitzung beendet), setzt `--fortsetzen` beim
  letzten Schnappschuss wieder ein und kürzt Protokoll, Tabellen und Entscheidungen genau auf ihn;
  Nachspielen trifft danach denselben Zustandshash. Mit `schnappschuss_tage = 1` geht höchstens ein
  Tag verloren. Ein Test bricht einen Lauf hart ab und vergleicht mit dem ununterbrochenen.
- **Fehler.** Ratenlimit und Serverfehler werden mit Wartezeit wiederholt. Bleibt eine Antwort aus,
  setzt die Rolle diesen Aufruf aus und der Datensatz vermerkt es. Fehlendes Guthaben (402), falscher
  Schlüssel und unbekannte Modelle halten den Lauf sauber an.
- **Datenschutz.** `data_collection = "deny"` lässt nur Provider zu, die Eingaben nicht speichern.

Die Weltuhr steht, während die Modelle antworten. Latenz und Ausfälle bei OpenRouter verändern
also die Dauer eines Laufs, nie das Spiel. Die Antworten selbst sind nicht wiederholbar, das Spiel
schon: nachgespielt wird aus dem Aktionsprotokoll.

### Rust-Orchestrator

`crates/agenten` fährt dieselbe Engine ohne Python. Jeder Aufruf wird vorher im Journal reserviert und
nachher abgeschlossen; ein Lauf setzt nach Abbruch genau dort fort und schickt keine Anfrage doppelt.
Eindeutige Fehler (HTTP-Status, leere oder abgeschnittene Antwort, keine Verbindung) werden mit ihrem
Grund abgeschlossen und unter eigenem Schlüssel erneut versucht (`versuche`, Standard 2); scheitert auch
das, setzt die Rolle diesen Aufruf aus, ohne Korrekturanfrage. Lehnt der Anbieter ab (401, 402, 403:
Schlüssel, Guthaben, Schlüssellimit), hält der Lauf an; der Anbieter hat nichts verarbeitet, der Aufruf wird
beim Fortsetzen neu gestellt. Ein unklarer Ausgang (Anfrage gesendet, Antwort abgerissen) hält den Lauf
ebenfalls an und wird nie automatisch wiederholt, weil der Anbieter ihn berechnet haben kann. Je Aufruf
liefert der Orchestrator einen Bericht mit Befehlen, Urteil des Kerns und Kosten; die KI-Reiche der
Oberfläche laufen über genau diesen Weg. Optionale Felder je Anbieter: `denken`
(`aus`, `niedrig`, `mittel`, `hoch`), `schema` (Antwortschema erzwingen), `provider` (etwa `order`,
`data_collection`).

```bash
./target/release/sternenepoche-agenten.exe validate --config konfig/rust-agenten-openrouter.json
```

```bash
./target/release/sternenepoche-agenten.exe run --config konfig/rust-agenten-openrouter.json --out laeufe/rust-openrouter --windows 96 --execute
```

## Was aus welchem Entwurf stammt

Grundlage ist das Konzept „Sternenepoche“: Zeitmodell, Galaxie, vier Völker, Zivilisationsstufen,
Rollen, Töpfe, Rust-Engine, Zahlen. Aus dem zweiten Entwurf kommen diese Teile dazu:

| Übernommen | Umsetzung |
|---|---|
| Blockade als dritte Konfliktform neben Plünderung und Eroberung | Mission `blockade`: Flotte hält den Orbit, Transporte kehren um, Marktlieferungen warten |
| Kolonisierung vervielfacht nichts | Siedler kommen aus der Heimatbevölkerung, die Habitatmodule des Schiffs werden der Wohnraum der Kolonie, sie beginnt nur mit der Ladung |
| Kautionen für Verträge | beide Seiten hinterlegen Credits, bei Bruch gehen beide an den Geschädigten |
| Keine unbegrenzt zahlende Außenwelt | kein Händler außerhalb der Spieler, Marktgebühren verschwinden aus dem Spiel |
| Kein kostenloser Punktgewinn | Abriss erstattet nichts, Wertung zählt nur Bestehendes |
| Beobachtung und Weltzustand getrennt | Entscheidungen enthalten nur, was der Agent sah; der Weltzustand liegt in Schnappschüssen |
| Prognose je Entscheidung | Feld `prognose` in jeder Antwort, für spätere Kalibrierungsauswertung |
| Verdacht des Modells lässt sich nicht ausschließen | wird gemessen: Feld `verdacht`, wenn Begründung, Notiz oder Denken auf einen Versuch anspielen |

Nicht übernommen: Wasser als weiteres Gut (mehr Buchhaltung, keine neue Entscheidung), fünf Völker
(die vier sind ausgearbeitet, die Syntheten sind der beabsichtigte Lesetest), 800 statt 1.440 Plätze,
Inventar in der Wertung (Horten soll nicht zählen), die andere Rollenteilung.

## Aufbau

```
regeln/regelwerk.ron       alle Zahlen des Spiels; der Regeltext der Agenten wird daraus erzeugt
crates/kern                Spielkern ohne Ein- und Ausgabe: Zustand, Regeln, Ereignisse, Kampf, Wertung
crates/lauf                Skriptbots, Balance, Nachspielen, Brücke zum Orchestrator
orchestrator/sternenepoche Lagebild, Rollen, Modellanbindung, Datenerfassung
konfig/                    Läufe: Attrappe, OpenRouter, Hybrid, lokal, Vergleich, Live-Test (spieler-live.json)
crates/agenten             Rust-Orchestrator mit Journal, auch für die KI-Reiche der Oberfläche
crates/spieler             native Oberfläche: selbst spielen, mit Skriptbots und KI-Reichen
crates/inhalt, content/    Inhaltskatalog, Planetenbilder, Bildaufträge
crates/wissen, wissen/     Wissensdatenbank (DuckDB) mit Quellen, Regeln, Inhalten, Entscheidungen, Befunden
betrachter/                Betrachter für Läufe, eine HTML-Datei; live über `python -m sternenepoche zeigen`
docs/                      Dokumentation, REGELWERK.md und REGELTEXT.md erzeugt mit `sternenepoche doku`
```

Der Kern rechnet mit Ganzzahlen in Tausendsteln, festen Arrays mit Aufzählungen als Index und
eigenem Zufallsstrom je Kampf. Die Engine redet mit dem Orchestrator über zeilenweises JSON auf
Standardein- und -ausgabe; `pyo3` ist dafür nicht nötig.

## Daten eines Laufs

| Datei | Inhalt |
|---|---|
| `entscheidungen.jsonl.gz` | je Aufruf: Lagebild, alle Runden mit roher Antwort, Tokens, Latenz, Kosten, Provider, Abfragen, Aktionen mit Ergebnis, Notiz, Prognose |
| `protokoll.jsonl` | Aktionsprotokoll der Engine, daraus und aus dem Startwert entsteht der Lauf neu |
| `tageswerte.jsonl` | täglich je Reich: Teilpunkte, Bevölkerung, Stabilität, Flottenwert, Produktion |
| `kampfberichte.jsonl`, `register.jsonl`, `nachrichten.jsonl`, `handel.jsonl` | Kämpfe, Vertragsregister, Diplomatie, Markt |
| `schnappschuesse/` | vollständiger Weltzustand alle paar Tage, zum Abzweigen |
| `prompts.json` | Systemtext je Rolle, in den Entscheidungen nur als Hash |
| `regelwerk.ron` | das Regelwerk des Laufs, bytegenau; `replay` nimmt es von hier, auch nach späteren Regeländerungen |
| `schluss.json` | Rangliste, Statistik, Kosten, Zustandshash |

Folgen an jede Entscheidung hängen (Punktedifferenz nach 1, 7 und 30 Spieltagen, Endrang):

```bash
.venv/Scripts/python.exe -m sternenepoche etiketten laeufe/probe
```

Dateien sind JSONL statt Parquet, komprimiert mit gzip statt zstd. Das spart Abhängigkeiten;
die Umwandlung ist ein Einzeiler mit `pyarrow`.

## Stand

Geprüft:

- 88 Rust-Tests im ganzen Workspace (`cargo test --release`), 39 Python-Tests des Orchestrators sowie
  10 Python- und 9 JS-Tests für `content/`, alle bestanden (Stand 4. Okt. 2026). Darunter prüft
  `crates/kern/tests/schema.rs`, dass Antwortschema, Kern und Werkzeug für jede Aktion und Abfrage
  zusammenpassen.
- Determinismus: zwei Läufe mit gleichem Startwert und das Nachspielen aus dem Protokoll ergeben
  denselben Zustandshash, auch für Läufe des Orchestrators und nach Anhalten und Fortsetzen.
- Die Formeln des Konzepts: Flugzeit 40 Minuten ins Nachbarsystem und knapp zwei Stunden in den
  anderen Sektor, Nebelabstand der Startplätze höchstens 20 Prozent verschieden, Völker 13, 13, 12, 12.
- Plünderung, Blockade, Belagerung und Eroberung, Halten bei Verbündeten, Vertragsbruch mit
  Kaution, Markt, Tribut, Abbau, Recycling, Spionage, Desertion: je ein Test über den ganzen Weg.
- Die Anfrage an OpenRouter gegen einen nachgebauten Server: Schema, Provider, Denkbudget,
  Wiederholung bei Ratenlimit, Kosten, Abbruch bei fehlendem Guthaben.
- Die vier Modelle der Konfiguration gibt es bei OpenRouter, sie unterstützen das Antwortschema
  (Stand 3. Okt. 2026, `pruefen`).

Echt geprüft am 4. Okt. 2026: Läufe über OpenRouter mit dem Python- und dem Rust-Orchestrator,
Modellwahl am echten Prompt, Kosten und Verlauf im Abschnitt „Echter Lauf“.

Gemessen mit Skriptbots, 24 Epochen mit je 50 Spielern über 365 Spieltage, Stufenzeiten als Median
(`sternenepoche balance --abnahme --laeufe 24`, Stand 4. Okt. 2026):

| Bottyp | Punkte im Mittel | Stufe II | Stufe III | Stufe IV | Stufe V (erreicht) |
|---|---|---|---|---|---|
| Ökonom | 156.233 | Tag 13,6 | Tag 36,4 | Tag 61,4 | Tag 160,8 (100 %) |
| Räuber | 171.472 | Tag 13,6 | Tag 36,4 | Tag 61,6 | Tag 174,0 (99 %) |
| Igel | 164.668 | Tag 13,6 | Tag 36,5 | Tag 64,8 | Tag 167,3 (100 %) |
| Händler | 156.321 | Tag 13,7 | Tag 36,5 | Tag 64,2 | Tag 164,8 (99 %) |

| Volk | Punkte im Mittel | Stufe IV | Stufe V |
|---|---|---|---|
| Aurelianer | 162.437 | Tag 64,2 | Tag 163,5 |
| Krath | 164.918 | Tag 61,4 | Tag 167,4 |
| Veyari | 160.717 | Tag 56,5 | Tag 152,9 |
| Syntheten | 160.651 | Tag 71,4 | Tag 219,9 |

Alle Zielzeiten des Konzepts sind getroffen und alle Abnahmekriterien erfüllt (Abschnitt Balance).
Eine volle Bot-Epoche rechnet hier rund 28 Sekunden, die Abnahme mit 12 Startwerten gut 4 Minuten.

Offen:

- **Verteidigung zählt ohne Unterhalt voll als Punkte.** Der Igel steckt bis zur Hälfte seines
  Gebäudewerts hinein und liegt trotzdem nicht vorn; nötig ist keine Änderung. Ob das so gewollt
  ist, entscheidet Karl.
- **Seit dem ersten Stand ergänzt (Codex):** Verbandsangriff, Raketensilo, drei Großprojekte der
  Stufe V, eine native Rust-Umgebung (`kern::umgebung`) und Parquet-Ausgabe in `crates/agenten`.
  Die übrigen Punkte der alten Liste (geteilte Galaxieansicht der Allianz, Registrierung als
  Gymnasium- oder PettingZoo-Umgebung) sind nicht nachgeprüft.

## Balance

Die Balance wird mit Skriptbots gemessen, die über dieselben Aktionen spielen wie die Agenten:
Ökonom, Räuber, Igel und Händler, dazu der Kontrolltyp Wehrlos (ein Ökonom, der sich nie schützt).

```bash
./target/release/sternenepoche.exe balance --abnahme
```

Die Abnahme fährt drei Aufstellungen über je 12 Startwerte (Mischung aller vier Typen, Räuber gegen
Wehrlose, Räuber gegen Igel) und prüft die mit Codex vereinbarten Kriterien:

- **Keine Dominanz:** Der Räuber liegt höchstens 25 Prozent über dem besten anderen Typ.
- **Raub lohnt sich:** Gegen wehrlose Ökonomen gewinnt der Räuber über 80 Prozent der Kämpfe und liegt vorn.
- **Schutz wirkt:** Der Igel verliert mindestens 80 Prozent weniger an Plünderung als der wehrlose Ökonom.
- **Entwicklung:** Stufe II bis IV erreichen mindestens 90 Prozent, Median in Tag 10–20, 30–50 und
  60–110; Stufe V erreichen bei Räuber und Igel mindestens 80 Prozent, Median in Tag 150–250.
- **Völker ausgeglichen:** Das beste Volk liegt höchstens 15 Prozent über dem schwächsten.
- **Keine Plateaus:** Alle Typen wachsen in den letzten 60 Tagen.

Woran Bots an Stufe V scheitern, zeigt `sternenepoche stufe5`. Mit `epoche --spur SPIELER` druckt
die Engine alle 15 Tage den Zustand eines Spielers, je Planet mit dem nächsten Bau und dem Gut, das
ihm dafür fehlt.

Geändert gegenüber dem ersten Regelwerk (alle Werte in `regeln/regelwerk.ron`, kommentiert):

| Wert | vorher | jetzt | Grund |
|---|---|---|---|
| Beutequote / Krath | 0,5 / 0,6 | 0,4 / 0,5 | Raub lohnte zu sehr |
| Unterhalt je Schiff und Tag | 0,1 % | 0,5 % des Bauwerts in Credits | Flotten waren eine Punktesenke ohne Kosten |
| Bunker je Stufe | 3.000 je Gut | 20.000 je Gut | Schutz, der zu den Lagergrößen passt |
| Stabilitätsverlust durch Plünderung | höchstens 20 | höchstens 10 | Mehrfachplünderung legte Reiche lahm |
| Sechs Verteidigungsanlagen | Grundwerte | Struktur, Schild, Angriff ×3 | Verteidigung hielt keiner Flotte stand |
| Stufe IV | 20.000 Einwohner | 32.000 | Median lag bei Tag 53 |
| Stufe V | 80.000 Einwohner | 700.000 | mit ordentlicher Wirtschaft zu früh |
| Strombedarf der Syntheten | 30 je 1000 Einwohner | 25 | Syntheten lagen 15 Prozent zurück |

Was die Bots dafür können mussten, lohnt sich auch für Agenten: Stufe-V-Kolonien brauchen
Habitatmodule, und die kosten Konsumgüter, die eine große Bevölkerung vollständig verbraucht.
Wer bei Stufe IV zuerst die Orbitalwerft baut und die Module fertigt, bevor er weiterwächst,
kommt an; wer erst wächst, nicht. Außerdem: Kolonien mit eigenem Lager und Strom versorgen,
Rohstoffe nicht im Kreis schicken, Kraftwerke nach Kosten je Stromeinheit wählen, nach der
ersten Plünderung Bunker und Verteidigung bauen.

## Echter Lauf

Am 4. Oktober 2026 mit einem OpenRouter-Schlüssel, der auf 2 USD begrenzt war. Die Modelle sind nicht nach
Ruf gewählt, sondern am echten Prompt verglichen: dasselbe Lagebild und derselbe Regeltext, Einzelaufrufe
für Bruchteile eines Cents (Skript im Verlauf der Sitzung, Ergebnisse hier).

| Rolle | Modell | Denken | Warum |
|---|---|---|---|
| Stratege | `qwen/qwen3.5-flash-02-23` | aus | schrieb als einziges günstiges Modell auf Anhieb eine durchdachte Doktrin |
| Verwalter | `deepseek/deepseek-v4-flash` | aus | vollständige, sinnvolle Bauaufträge ohne unnötige Abfragen |
| Feldherr | `deepseek/deepseek-v4-flash` | aus | lief im Probelauf fehlerfrei |
| Diplomat | `qwen/qwen3.5-flash-02-23` | aus | schnell, sauberes Deutsch |

Verworfen nach Messung:

- **DeepSeek V4 mit Denken:** hält weder ein Tokenbudget noch die Stufe „low“ ein, denkt bis zum Limit
  (3.000 bis 6.000 Token) und antwortet nie. Ohne Denken ist es gut und günstig.
- **gpt-oss-120b:** fragt Kosten ab, obwohl sie im Lagebild stehen, leert die eben gefüllte Bauschleife,
  schreibt als Stratege fünfmal `stufenaufstieg` statt einer Doktrin.
- **Gemini 2.5 Flash Lite:** brauchbar, fragt aber ebenfalls unnötig ab; teurer je Aufruf.
- **gpt-6-luna:** kein Provider, der Eingaben nicht speichert (`data_collection = "deny"`).

Was der Test am Spiel und am Orchestrator gefunden hat, ist behoben:

- Wecker im Stundentakt (Weckregel, siehe OpenRouter), der Regeltext nannte den Takt gar nicht.
- Vier Aktionen und zwei Abfragefelder fehlten im Antwortschema und wurden immer abgelehnt.
- Leere oder abgeschnittene Antworten wurden identisch wiederholt; jetzt mit mehr Platz und weniger Denken.
- Dem Verwalter fehlten Baukosten im Lagebild; jede Kostenabfrage kostete einen zweiten Modellaufruf.
- Provider ohne Vorgabe: OpenRouter verteilte DeepSeek auf sechs Provider, darunter solche mit
  zehnfachem Ausgabepreis. `provider.order` auf einen günstigen Provider halbiert die Kosten je Aufruf.

Gemessen: rund 0,001 USD je Modellaufruf, etwa 40 Aufrufe je Reich und Spieltag in der Aufbauphase
(der Verwalter lässt sich alle zwei Stunden wecken), also rund 0,02 USD je Reich und Spieltag und
7 bis 9 Minuten Rechenzeit je Spieltag, weil jede Entscheidung auf die Modellantwort wartet.

<!-- ERGEBNIS-LAUF -->

## Befunde aus dem Bau

Diese Stellen des Konzepts gingen nicht auf und sind im Regelwerk geändert:

- **Wachstum 5 Prozent pro Tag** und die Zielzeiten der Stufen passen nicht zusammen: logistisch
  gebremst erreicht ein Reich 2.000 Einwohner erst um Tag 39. Mit 12 Prozent stimmen die Zeiten.
- **Zwei freie Systeme zwischen zwei Spielern** passen bei 25 Spielern auf 60 Systeme nicht. Jetzt
  liegt mindestens ein System dazwischen.
- **Plünderungen** summierten ihren Stabilitätsverlust unbegrenzt; ein oft geplündertes Reich fiel
  auf Stabilität 0 und konnte nichts mehr bauen. Der Verlust ist jetzt auf 10 Punkte gedeckelt.
- **Stufe V mit 80.000 Einwohnern** fiel bei ordentlicher Wirtschaft weit vor Tag 150. Mit 700.000
  liegt sie im Zielfenster; die Bevölkerung taktet die Stufen, Forschungsbedingungen verschieben sie kaum.
- **Arbeitskräfte ohne Prioritätenliste** gingen zuerst an die Minen. Bei den Syntheten stand dann
  das Kraftwerk leer, die Bevölkerung hungerte, das Reich starb. Ohne Vorgabe werden jetzt Farm und
  Kraftwerke zuerst besetzt.
- **Kolonien ohne mitgebrachte Nahrung** hungern vom ersten Tag an. Das ist gewollt; die Bots
  laden deshalb Nahrung ins Kolonieschiff.
