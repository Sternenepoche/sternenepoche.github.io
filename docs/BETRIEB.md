---
layout: default
title: "Betriebshandbuch Sternenepoche"
---

# Betriebshandbuch Sternenepoche

Dieses Handbuch beschreibt, wie man Sternenepoche baut, testet, startet, konfiguriert, Kosten abschätzt,
Läufe fortsetzt und auswertet. Jede Aussage stützt sich auf Code oder Konfiguration im Projekt
(Stand 4. Oktober 2026); die Quelle steht jeweils dabei.

Alle Befehle laufen im Projektordner `D:\projekte_ki\Sternepoche`; der Python-Orchestrator findet Engine
und Ausgabe über relative Pfade (`orchestrator/sternenepoche/konfig.py`). Blöcke mit `bash` sind für Git
Bash, Blöcke mit `powershell` nur für Windows PowerShell 5.1; je Block steht ein Befehl. Schlüssel stehen
nie in einer Datei, auch nicht hier.

## 1. Voraussetzungen und Bauen

### Werkzeuge

| Werkzeug | Wofür | Beleg |
|---|---|---|
| Rust mit `cargo` | Workspace mit sechs Crates, Edition 2021, `resolver = "2"` | `Cargo.toml` |
| Python ab 3.11 (die Umgebung `.venv` nutzt 3.13) | Orchestrator, einzige Abhängigkeit `httpx>=0.27` | `orchestrator/pyproject.toml`, `.venv/pyvenv.cfg` |
| `uv` | legt `.venv` an und installiert den Orchestrator | `README.md`, Schnellstart |
| Node.js mit `node --test` | JS-Tests der Inhalte | `content/README.md` |
| DuckDB-Bibliothek 1.5.6 | nur zur Laufzeit der Wissensdatenbank, nicht zum Bauen | `crates/wissen/src/db.rs` |

### Speicherorte: große Downloads nie auf C:

Modelle, Gewichte, Datensätze, Python-Umgebungen, Pakete und Caches (Richtwert ab etwa 1 GB) gehören
nur auf D:. Die Benutzervariablen `HF_HOME`, `HF_HUB_CACHE`, `TORCH_HOME`, `OLLAMA_MODELS`, `UV_CACHE_DIR`
und `PIP_CACHE_DIR` zeigen auf D: und dürfen nicht nach C: umgebogen werden. D: ist fast voll: vor einem
großen Download den freien Platz prüfen; passt er nicht mit rund 10 GB Reserve, Karl fragen statt auf C:
auszuweichen.

```powershell
Get-PSDrive D
```

Im Projekt auf D: liegen `.venv`, `target/` und der Cargo-Cache `.cargo-cache/` (rund 430 MB). Die
Crate-READMEs (`crates/agenten`, `crates/inhalt`, `crates/spieler`) setzen dafür vor dem Bauen:

```powershell
$env:CARGO_HOME = 'D:\projekte_ki\Sternepoche\.cargo-cache'
```

Die lokalen Konfigurationen (`konfig/lokal.toml`, `konfig/hybrid.toml`) setzen Modellserver mit Gewichten
von 9 bis 70 Mrd. Parametern voraus. Deren Download gehört ebenfalls nach D:.

### Crates und Binärdateien

| Crate (Paketname) | Binärdatei in `target/release/` | Aufgabe |
|---|---|---|
| `crates/kern` (`kern`) | keine, Bibliothek | deterministischer Spielkern ohne Ein- und Ausgabe |
| `crates/lauf` (`lauf`) | `sternenepoche.exe` | Skriptbots, Balance, Nachspielen, Brücke zum Python-Orchestrator |
| `crates/agenten` (`sternenepoche-agenten`) | `sternenepoche-agenten.exe` | Rust-Orchestrator mit Journal |
| `crates/inhalt` (`inhalt`) | `sternenepoche-inhalt.exe` | Inhaltskatalog, Planetenkarten, gesperrte Bildproduktion |
| `crates/spieler` (`spieler`) | `sternenepoche-spieler.exe` | native Spieleroberfläche (egui): ein Mensch gegen Skriptbots, wahlweise mit KI-Reichen über OpenRouter (Live-Test, `docs/LIVE-TEST.md`) |
| `crates/wissen` (`wissen`) | `sternenepoche-wissen.exe` | Wissensdatenbank in DuckDB, HTTP und MCP |

Das Regelwerk `regeln/regelwerk.ron` wird beim Bauen in `sternenepoche.exe` und `sternenepoche-agenten.exe`
eingebettet (`include_str!` in `crates/lauf/src/main.rs` und `crates/agenten/src/lib.rs`). Eine Änderung
am Regelwerk wirkt erst nach dem nächsten Bauen, es sei denn, ein Befehl bekommt `--regeln PFAD`.

Alles bauen (Release, `opt-level = 3`); ein einzelnes Paket mit `-p PAKETNAME`, etwa `-p lauf`:

```bash
cargo build --release
```

Läuft gerade ein Lauf, ist dessen Binärdatei unter Windows gesperrt und `cargo` kann sie nicht ersetzen
(Abschnitt 10). Dann in ein eigenes Zielverzeichnis bauen:

```bash
cargo build --release --target-dir target-bau
```

### Python-Umgebung

Die Umgebung liegt im Projekt auf D: (`README.md`, Schnellstart):

```bash
uv venv .venv --python 3.13
```

```bash
uv pip install --python .venv/Scripts/python.exe -e orchestrator
```

Danach steht der Orchestrator als Modul `python -m sternenepoche` und als `.venv/Scripts/sternenepoche-lauf.exe`
bereit (`[project.scripts]` in `orchestrator/pyproject.toml`).

## 2. Tests

### Rust

Der ganze Workspace (laut `README.md` 88 Tests, alle bestanden am 4. Okt. 2026); ein einzelnes Paket mit
`-p PAKETNAME`, etwa `-p sternenepoche-agenten`:

```bash
cargo test --release
```

### Python-Orchestrator

Die Tests nutzen `unittest`; `pytest` ist weder Abhängigkeit noch in `.venv` installiert. Es gibt zwei
Gruppen, die getrennt gestartet werden:

- `orchestrator/tests/test_ablauf.py` (14 Tests) für das Paket `sternenepoche`. Die Tests starten die
  Engine `target/release/sternenepoche.exe`; sie muss also gebaut sein.
- `orchestrator/test_journal.py` und `orchestrator/test_providers.py` (zusammen 28 Tests) für die ältere
  Anbindung `orchestrator/*.py` (Journal, Provider), die als Referenz erhalten bleibt.

```bash
.venv/Scripts/python.exe -m unittest discover -s orchestrator/tests -v
```

```bash
.venv/Scripts/python.exe -m unittest discover -s orchestrator -p "test_*.py" -v
```

### Inhalte

10 Python-Tests und 9 JS-Tests (`content/README.md`):

```bash
.venv/Scripts/python.exe -m unittest discover -s tools/content -p "test_*.py" -v
```

```bash
node --test tools/content/test_planet.cjs tools/content/test_ui.cjs
```

### Was die wichtigsten Testdateien prüfen

| Datei | Prüft |
|---|---|
| `crates/kern/tests/schema.rs` | Antwortschema je Rolle und Kern passen zusammen: jede erlaubte Aktion und Abfrage versteht der Kern, die Beispiele im Regeltext passen zum Schema, `fertigen` verlangt genau eines von `einheit` und `bauteil`. |
| `crates/kern/tests/spiel.rs` | Formeln und Wege des Spiels: Flugzeiten, faire Startplätze, Mengenerhaltung, exakte Schnappschüsse, Stufenaufstieg nach 48 h, Plünderung, Blockade, Eroberung, Markt, Spionage, Unterhalt, Weckregel. |
| `crates/kern/tests/umgebung.rs` | Die native Mehrspielerumgebung `kern::umgebung` ist reproduzierbar, lehnt fehlerhafte gemeinsame Schritte atomar ab und zeigt jedem Spieler nur eigene Planetendetails. |
| `crates/kern/tests/verband.rs` | Allianzangriff: Zustimmung, gemeinsame Ankunft ohne Teleportation, ein Gefecht mit einem Beutepool, eigene Forschung je Teilnehmer, Rückruf. |
| `crates/kern/tests/vertiefung.rs` | Regressionen: Bauzeit bei Unruhen, überlaufende Mengen erzeugen nichts, die Weltuhr wartet auf jede fällige Rolle, das letzte Fenster endet genau am Epochenende. |
| `crates/agenten/tests/integration.rs` | Rust-Orchestrator: Fortsetzen ergibt denselben Zustand, offene Aufrufe werden nie doppelt gesendet, eindeutige Fehler werden wiederholt, ein unklarer Ausgang hält an, Netz nur mit `--execute`, echte Parquet-Dateien, Anfragekörper für OpenRouter. |
| `orchestrator/tests/test_ablauf.py` | Ganzer Lauf mit der Attrappe samt Nachspielen und Etiketten; Budgetgrenze und harter Abbruch, jeweils fortgesetzt mit gleichem Endhash und gleichen Kosten; Prompts ohne Messhinweise; OpenRouter-Anfrage gegen nachgebauten Server. |
| `crates/wissen/tests/service.rs` | Die Wissens-API nimmt nur begrenzte Leseoperationen an, wehrt fremde Hosts und Origins ab und spricht MCP korrekt. |

## 3. Engine-Befehle (`sternenepoche.exe`)

Quelle: Hilfetext und Argumente in `crates/lauf/src/main.rs`. Ohne Befehl druckt die Engine die Hilfe.
Fehler enden mit `Fehler: ...` und Exitcode 1. Überall gilt `--regeln PFAD` für ein anderes Regelwerk.
Bottypen: `oekonom`, `raeuber`, `igel`, `haendler`, dazu der Kontrolltyp `wehrlos` (ein Ökonom, der sich nie
schützt).

| Befehl | Optionen (Standard) | Wirkung |
|---|---|---|
| `epoche` | `--startwert` (1), `--spieler` (50), `--tage` (aus Regelwerk, 365), `--typen` (alle vier), `--pruefen`, `--spur SPIELER`, `--aus VERZEICHNIS` | eine Epoche mit Skriptbots; Rangliste, häufigste Ablehnungsgründe |
| `balance` | `--laeufe` (8), `--spieler` (50), `--tage` (150), `--typen` | mehrere Epochen, Vergleich der Bottypen |
| `balance --abnahme` | `--laeufe` (12), `--spieler` (50), `--tage` (365) | drei Aufstellungen, prüft die Abnahmekriterien; Exitcode 1, wenn eines fehlt |
| `stufe5` | `--laeufe` (12), `--spieler` (50), `--tage` (365), `--typen` | je Bottyp: wer Stufe V erreicht, wann, und woran die übrigen scheitern |
| `replay VERZEICHNIS` | `--regeln` (Standard: `regelwerk.ron` im Laufordner, sonst eingebaut) | spielt `protokoll.jsonl` aus Startwert nach, bis zur Zeit in `schluss.json`, und vergleicht mit dem Hash |
| `doku` | `--aus` (`docs`) | schreibt `REGELWERK.md` und `REGELTEXT.md` aus dem Regelwerk; ein Test prüft, dass beide aktuell sind |
| `regeltext` | `--rolle` (`stratege`, `verwalter`, `feldherr`, `diplomat`, Standard `alle`) | Regeltext der Agenten |
| `bruecke` | keine | zeilenweises JSON auf Standardein- und -ausgabe für den Python-Orchestrator |

`--pruefen` fährt die Epoche ein zweites Mal und spielt sie aus dem Protokoll nach; beide Hashes müssen
gleich sein. `--spur SPIELER` druckt alle 15 Tage je Planet den nächsten Bau und das fehlende Gut. `--aus`
schreibt `protokoll.jsonl`, `tageswerte.jsonl`, `kampfberichte.jsonl`, `register.jsonl`, `schluss.json`.

Eine Epoche mit Determinismusprüfung (rund 28 s laut `README.md`):

```bash
./target/release/sternenepoche.exe epoche --startwert 1 --pruefen
```

Eine Epoche mit Daten für den Betrachter (Abschnitt 8):

```bash
./target/release/sternenepoche.exe epoche --startwert 1 --aus laeufe/bots-001
```

Balanceabnahme (gut 4 Minuten laut `README.md`):

```bash
./target/release/sternenepoche.exe balance --abnahme
```

Regeltext einer Rolle:

```bash
./target/release/sternenepoche.exe regeltext --rolle verwalter
```

### Brückenprotokoll

Jede Zeile auf Standardeingabe ist ein JSON-Objekt mit `cmd`; jede Antwort trägt `"ok": true` oder
`"ok": false` mit `fehler` (`bruecke_befehl` in `crates/lauf/src/main.rs`).

| `cmd` | Felder | Antwort |
|---|---|---|
| `neu` | `startwert` (1), `spieler` (50), `tage`, `ki` (Liste), `bottypen`, `regeln` | Kopfdaten des Laufs |
| `laden` | `pfad` | Kopfdaten eines gespeicherten Stands |
| `stand` | – | Zeit, `faellig`, Regelwerk, Spieler, Bots |
| `weiter` | `bis_faellig` (true), `max_fenster` (38.400) | neue Zeit, Zahl der Fenster, `faellig` |
| `sicht` | `spieler`, `rolle` | Lagebild-Rohdaten |
| `regeltext` | `rolle` | Text, Aktionstypen, Antwortschema, Grenzen |
| `handeln` | `spieler`, `rolle`, `aktionen` | Ergebnis je Aktion |
| `aufruf_ende` | `spieler`, `rolle`, `notiz`, `wecker_sekunden`, `hinweise` | – |
| `werkzeug` | `spieler`, `abfrage` | Ergebnis einer lesenden Abfrage |
| `log`, `hash`, `rangliste`, `statistik`, `welt` | – | Protokolleinträge, Hashes, Wertung, Weltbild |
| `tabelle` | `name` (`tageswerte`, `kampfberichte`, `register`, `nachrichten`, `handel`), `ab` | Zeilen ab Index |
| `speichern` | `pfad` | – |
| `ende` | – | beendet die Brücke |

### Spieleroberfläche (`sternenepoche-spieler.exe`)

Für Menschen: `Spielen.cmd` (neue Partie gegen Skriptbots) oder `Live-Test.cmd` (Bereich Live-KI) im
Projektordner. Die Bedienung erklärt [SPIELEN.md](SPIELEN.md), Partien mit Modellreichen
[LIVE-TEST.md](LIVE-TEST.md).

| Option | Standard | Wirkung |
|---|---|---|
| `--seed N` | 42 | Startwert der Galaxie; bestimmt auch Name und Volk des Menschen |
| `--player N` | 0 | welches Reich der Mensch regiert |
| `--load DATEI` | – | Spielstand öffnen; „Speichern“ schreibt danach nach Rückfrage in dieselbe Datei |
| `--screen NAME` | Übersicht | in diesem Bereich beginnen (Name wie in der Navigation, etwa `Galaxie`, `Live-KI`) |
| `--vorlauf N` | – | N Fenster vorspielen, danach angehalten |
| `--autopilot TYP` | – | während `--vorlauf` führt ein Skriptbot (`oekonom`, `raeuber`, `igel`, `haendler`) das Reich des Menschen |
| `--screenshot DATEI.png` | – | ein echtes Bild rendern, speichern, schließen (für Dokumentation und Prüfung) |
| `--smoke-test` | – | einen Spieltag mit 50 Spielern ohne Fenster spielen |
| `--ki KONFIG`, `--attrappe`, `--ki-reiche N`, `--reiche N`, `--budget USD`, `--smoke-test-ki` | – | Partie mit Modellreichen, siehe LIVE-TEST |

Spielstände liegen unter `saves/` (relativ zum Arbeitsordner). Bilder für das Handbuch entstehen so:

```bash
./target/release/sternenepoche-spieler.exe --seed 7 --vorlauf 6000 --autopilot raeuber --screen Galaxie --screenshot galaxie.png
```

## 4. Python-Orchestrator (`python -m sternenepoche`)

Quelle: `orchestrator/sternenepoche/__main__.py`. Ablauf je Entscheidungsfenster (`lauf.py`): alle fälligen
Rollen antworten gleichzeitig, wer Abfragen stellt, bekommt die Ergebnisse und entscheidet erneut, die
Aktionen wirken in der von der Engine ausgelosten Reihenfolge, abgelehnte Aktionen dürfen einmal korrigiert
werden. Die Weltuhr steht, solange Modelle antworten.

| Befehl | Argumente | Wirkung | Exitcode |
|---|---|---|---|
| `lauf` | `KONFIG [--fortsetzen]` | fährt eine Epoche; `--fortsetzen` setzt einen angehaltenen oder abgestürzten Lauf fort | 0 fertig, 2 angehalten (fortsetzbar), 1 Konfigurations- oder Enginefehler |
| `pruefen` | `KONFIG` | prüft Konfiguration, Engine, Modelle und Schlüssel, ohne ein Modell aufzurufen | 1 bei einem Befund `[FEHLER]` |
| `kosten` | `KONFIG` | schätzt Aufrufe, Tokens und USD je Rolle und Anbieter | 0 |
| `etiketten` | `VERZEICHNIS` | hängt jeder Entscheidung ihre Folgen an (Abschnitt 8) | 0 |
| `zeigen` | `VERZEICHNIS [--port 8198] [--ohne-browser]` | Betrachter im Browser, auch für einen laufenden Lauf (Abschnitt 8) | 0 nach Strg+C, 1 wenn der Ordner fehlt |

```bash
.venv/Scripts/python.exe -m sternenepoche lauf konfig/probe.toml
```

```bash
.venv/Scripts/python.exe -m sternenepoche lauf konfig/openrouter-test.toml --fortsetzen
```

`pruefen` und `kosten` starten kurz die Engine (Regelwerk, Länge der Systemtexte) und brauchen deshalb die
gebaute `sternenepoche.exe`.

### Inhalt eines Laufordners

| Datei | Inhalt |
|---|---|
| `konfig.toml` | Kopie der Konfiguration beim Start |
| `regelwerk.ron` | das Regelwerk, mit dem die Engine rechnet (bytegenau); `replay` nimmt es von hier |
| `entscheidungen.jsonl.gz` | je Aufruf: Lagebild, alle Runden mit roher Antwort, Tokens, Latenz, Kosten, Provider, Abfragen, Aktionen mit Ergebnis, Notiz, Prognose, `verdacht` |
| `protokoll.jsonl` | Aktionsprotokoll der Engine; mit dem Startwert entsteht daraus der Lauf neu |
| `tageswerte.jsonl`, `kampfberichte.jsonl`, `register.jsonl`, `nachrichten.jsonl`, `handel.jsonl` | Tabellen der Engine |
| `schnappschuesse/tag-NNNN.bin` | Weltzustand alle `schnappschuss_tage` Tage |
| `prompts.json` | Systemtext je Rolle; Entscheidungen verweisen per Hash darauf |
| `fortschritt.json` | Zeiger auf die Tabellen und Zahl der Entscheidungen |
| `stand.bin` | nur nach sauberem Anhalten: Stand für `--fortsetzen` |
| `schluss.json` | Rangliste, Statistik, Kosten, Zustandshash (Abschnitt 8) |
| `etiketten.jsonl` | erst nach `etiketten` |

### Konfigurationsformat (TOML)

Quelle: `orchestrator/sternenepoche/konfig.py`. Unbekannte Schlüssel werden abgelehnt
(`[lauf]: unbekannte Felder [...]`). Eine Datei besteht aus `[lauf]`, `[grenzen]`, beliebig vielen
`[anbieter.NAME]` und entweder `[rollen]` oder `[vergleich]`.

#### `[lauf]`

| Schlüssel | Standard | Wertebereich | Bedeutung |
|---|---|---|---|
| `startwert` | 1 | Ganzzahl | Startwert der Welt; bestimmt Galaxie, Völker und Reihenfolge |
| `spieler` | 50 | 1 bis 60 mit dem Standardregelwerk | Zahl der Reiche; mehr meldet die Engine als „Spielerzahl N passt nicht in die Galaxie“ (2 Sektoren × 60 Systeme / `start_abstand` 2) |
| `tage` | 365 | positive Ganzzahl | Länge der Epoche in Spieltagen |
| `ki` | alle | `"alle"` oder Liste von Spielernummern 0 bis `spieler`−1 | Reiche mit Modellen; die übrigen spielen Skriptbots |
| `bottypen` | `["oekonom", "raeuber", "igel", "haendler"]` | dazu `"wehrlos"` | Typen der Skriptbots, reihum vergeben |
| `ausgabe` | `"laeufe/lauf"` | Pfad | Laufordner |
| `engine` | `"target/release/sternenepoche.exe"` | Pfad | Engine für die Brücke |
| `regeln` | eingebettetes Regelwerk | Pfad | anderes Regelwerk |
| `schnappschuss_tage` | 5 | 0 oder mehr; 0 schaltet Schnappschüsse ab | Abstand der Schnappschüsse; bestimmt, wie viel ein Absturz kostet |

#### `[grenzen]`

| Schlüssel | Standard | Bedeutung |
|---|---|---|
| `max_ausgabe_tokens` | 1200 | `max_tokens` je Anfrage |
| `denk_tokens` | 1000 | Denkbudget bei `denken = "budget"`; bei `budget` und Denkstufen wird `max_tokens` um diesen Wert erhöht |
| `temperatur` | 0.7 | Temperatur |
| `zeitlimit_sekunden` | 180.0 | Zeitlimit je HTTP-Anfrage (Verbindungsaufbau 15 s) |
| `versuche` | 3 | Versuche je Aufruf (Abschnitt 5, Neuversuche) |
| `budget_usd` | keine Grenze | Kostengrenze für den ganzen Laufordner, geprüft vor jedem Entscheidungsfenster |

#### `[anbieter.NAME]`

| Schlüssel | Standard | Wertebereich | Bedeutung |
|---|---|---|---|
| `art` | – (Pflicht) | `openrouter`, `openai`, `mock` | OpenRouter, lokaler OpenAI-kompatibler Server (vLLM, llama.cpp, Ollama) oder Attrappe ohne Modell |
| `modell` | – | Text | Modellkennung; Pflicht außer bei `mock` |
| `basis_url` | bei `openrouter`: `https://openrouter.ai/api/v1` | URL | Pflicht bei `openai` |
| `schluessel_env` | bei `openrouter`: `OPENROUTER_API_KEY` | Name einer Umgebungsvariablen | bei `openai` optional, dann als Bearer gesendet |
| `parallel` | 8 | ab 1 | gleichzeitige Anfragen an diesen Anbieter |
| `schema` | `json_schema` | `json_schema`, `guided_json` (vLLM), `json_object`, `aus` | wie das Antwortschema durchgesetzt wird |
| `denken` | `weglassen` | `weglassen`, `aus`, `budget`, `niedrig`, `mittel`, `hoch` | nur bei OpenRouter wirksam: `aus` sendet `effort: none`, `budget` begrenzt auf `denk_tokens`, Stufen senden `low`, `medium`, `high` |
| `provider` | leer | Tabelle | nur OpenRouter: Providervorgaben wie `order`, `allow_fallbacks`, `quantizations`, `only`, `data_collection`; bei `schema = "json_schema"` kommt `require_parameters = true` dazu |
| `zusatz` | leer | Tabelle | wird unverändert in den Anfragekörper übernommen, etwa `cache_prompt = true` für llama.cpp |
| `preis_ein`, `preis_aus` | 0.0 | USD je Million Tokens | Kosten lokaler Anbieter und der Attrappe; OpenRouter rechnet mit `usage.cost` |
| `titel`, `referer` | leer | Text | Kopfzeilen `X-OpenRouter-Title` und `HTTP-Referer` |

#### `[rollen]` und `[vergleich]`

`[rollen]` ordnet jeder der vier Rollen `stratege`, `verwalter`, `feldherr`, `diplomat` einen Anbieternamen
zu; alle vier sind Pflicht. Mit `[vergleich] anbieter = [...]` entfällt `[rollen]`: jedes genannte Modell
steuert seine Reiche in allen vier Rollen. Die Zuordnung ist
`anbieter[(Platz des Spielers in ki + startwert) mod Zahl der Anbieter]` (`Konfig.anbieter_fuer`), wechselt
also mit dem Startwert.

### Mitgelieferte Konfigurationen

| Datei | Startwert, Spieler, Tage, Modellreiche | Modelle | `budget_usd` | Ausgabe |
|---|---|---|---|---|
| `konfig/probe.toml` | 7, 6, 5, `[0, 1, 2]` | Attrappe in allen Rollen; Bots `oekonom`, `raeuber`, `igel`; Schnappschuss alle 2 Tage | – | `laeufe/probe` |
| `konfig/openrouter-probe.toml` | 3, 20, 1, `[1]` | Stratege und Diplomat `qwen/qwen3.5-flash-02-23`, Verwalter und Feldherr `deepseek/deepseek-v4-flash` (Provider `deepinfra`), alle ohne Denken, `data_collection = "deny"` | 0.10 | `laeufe/openrouter-probe` |
| `konfig/openrouter-test.toml` | 3, 20, 90, `[1]` | wie Probe; `max_ausgabe_tokens` und `denk_tokens` 1500, Temperatur 0.6, Schnappschuss täglich | 1.55 | `laeufe/openrouter-test` |
| `konfig/openrouter-klein.toml` | 1, 20, 30, `[0..4]` | `qwen/qwen3.5-9b` in allen Rollen, ohne Denken | 3.0 | `laeufe/openrouter-klein` |
| `konfig/openrouter.toml` | 1, 50, 365, alle | Stratege `meta-llama/llama-3.3-70b-instruct` (Quantisierung fp8, fp16, bf16), Verwalter `qwen/qwen3.5-9b`, Feldherr `google/gemma-4-31b-it`, Diplomat `mistralai/mistral-small-3.2-24b-instruct` | 250.0 | `laeufe/epoche-001` |
| `konfig/hybrid.toml` | 1, 50, 365, alle | Verwalter lokal `qwen3.5-9b` auf `http://127.0.0.1:8001/v1` mit `cache_prompt`, übrige Rollen mit den Modellen aus `openrouter.toml` (ohne Quantisierungsvorgabe) | 100.0 | `laeufe/hybrid-001` |
| `konfig/vergleich.toml` | 1, 50, 365, alle | `[vergleich]` mit den vier Modellen aus `openrouter.toml`, je 12 oder 13 Reiche | 200.0 | `laeufe/vergleich-001` |
| `konfig/lokal.toml` | 1, 50, 365, alle | vier lokale Server auf Port 8000 bis 8003, `schema = "guided_json"`, Zeitlimit 600 s | – | `laeufe/lokal-001` |

Anmerkungen:

- `openrouter-probe.toml` ist der Probetag zu `openrouter-test.toml`; der Kommentar im Kopf nennt die Befehle
  für `openrouter-test.toml`.
- `openrouter-test.toml` verbrauchte im echten Lauf rund 0,019 USD je Spieltag (`laeufe/openrouter-test.log`:
  0,65 USD nach Tag 35). Bei diesem Verbrauch reicht `budget_usd = 1.55` rechnerisch bis etwa Tag 80; dann
  hält der Lauf sauber an und lässt sich nach Erhöhen der Grenze fortsetzen.
- `orchestrator/models.example.toml` und `orchestrator/window.example.json` gehören zur älteren Anbindung
  `python -m orchestrator` und sind kein Format für `python -m sternenepoche`.

## 5. OpenRouter

### Schlüssel

Der Schlüssel steht nur in der Umgebungsvariablen `OPENROUTER_API_KEY` (oder der in `schluessel_env`
genannten), nie in einer Datei. Fehlt er, hält der Lauf beim ersten Aufruf mit Meldung an
(`OpenRouter._kopfzeilen` in `backends.py`).

PowerShell speichert getippte Befehle in der Verlaufsdatei von PSReadLine. Deshalb den Schlüssel abfragen
statt ihn in die Befehlszeile zu schreiben:

```powershell
$env:OPENROUTER_API_KEY = Read-Host "OpenRouter-Schlüssel"
```

In Git Bash ohne Echo und ohne Eintrag im Verlauf:

```bash
read -rs OPENROUTER_API_KEY && export OPENROUTER_API_KEY
```

### Vorab prüfen und schätzen

Beide Befehle rufen kein Modell auf. Sie laden die öffentliche Modellliste von OpenRouter; `pruefen` fragt
zusätzlich mit dem Schlüssel dessen Restguthaben ab (`/key`).

```bash
.venv/Scripts/python.exe -m sternenepoche pruefen konfig/openrouter-test.toml
```

```bash
.venv/Scripts/python.exe -m sternenepoche kosten konfig/openrouter-test.toml
```

`pruefen` meldet als `[FEHLER]`: unbekannte Modelle, fehlende Unterstützung für Antwortschema,
`response_format` oder `reasoning`, `denken = "aus"` bei Modellen, die immer denken, `denken = "budget"` bei
Modellen ohne Denkbudget (so DeepSeek V4, das sonst bis zum Tokenlimit denkt), unbekannte Denkstufen und
fehlende Umgebungsvariablen. Als `[Hinweis]`: Modelle, die von sich aus denken, und Anbieter ohne
`provider.order` oder `provider.only`.

### Kostenmodell

Quelle: `orchestrator/sternenepoche/kosten.py`, geeicht am echten Lauf vom 4. Okt. 2026 (28 Spieltage, ein
Reich).

| Rolle | Entscheidungen je Agent und Tag (`AUFRUFE_JE_TAG`) | Modellaufrufe je Entscheidung (`ZUSATZRUNDEN`) | Modellaufrufe je Agent und Tag | Ausgabetokens je Aufruf (`AUSGABE_JE_AUFRUF`) |
|---|---|---|---|---|
| Stratege | 2.0 | 1.55 | 3.1 | 730 |
| Verwalter | 10.1 | 2.25 | 22.7 | 390 |
| Feldherr | 3.3 | 2.0 | 6.6 | 320 |
| Diplomat | 2.0 | 1.1 | 2.2 | 630 |
| Summe | | | 34.6 | |

Weitere Konstanten: Lagebild 2.500 Tokens (`LAGEBILD_TOKENS`), 3,3 Zeichen je Token für den Systemtext,
Denkanteil von `denk_tokens` je Einstellung (`DENKANTEIL`): `budget` und `hoch` 1.0, `mittel` 0.5,
`niedrig` 0.25, sonst 0.

Rechnung je Rolle und Anbieter:

- Aufrufe = Entscheidungen je Tag × Modellaufrufe je Entscheidung × Agenten × Tage × Anteil
  (Anteil 1, im Vergleichsmodus 1 durch Zahl der Modelle).
- Eingabetokens = Aufrufe × (Zeichen des Systemtexts / 3,3 + 2.500).
- Ausgabetokens = Aufrufe × (Ausgabe je Aufruf + `denk_tokens` × Denkanteil).
- USD = Eingabe × Eingabepreis + Ausgabe × Ausgabepreis. Preise kommen aus der Modellliste, bei lokalen
  Anbietern aus `preis_ein` und `preis_aus`. Die Spalte „USD mit Cache“ rechnet den Systemtext zum
  Cache-Preis (`input_cache_read`), wo die Modellliste ihn nennt.

Gemessen: rund 0,001 USD je Modellaufruf und rund 0,02 USD je Reich und Spieltag mit den Modellen aus
`openrouter-test.toml` (`README.md`, Abschnitt „Echter Lauf“). Den größten Posten stellt der Verwalter.

### Budgetgrenze und Fortsetzen

`budget_usd` wird vor jedem Entscheidungsfenster geprüft, nie mitten in einem (`Kostenzaehler.pruefe`).
Ein Lauf kann die Grenze also um die Kosten eines Fensters überschreiten. Ist sie erreicht, schreibt der
Lauf Protokoll, Tabellen und `stand.bin`, setzt in `schluss.json` `"beendet": false` und den Grund unter
`angehalten` und endet mit Exitcode 2 und der Meldung
`Lauf angehalten (BudgetErreicht: ...). Fortsetzen mit --fortsetzen.`

Die Grenze gilt für den ganzen Laufordner: beim Fortsetzen werden die Kosten aller bisherigen
Entscheidungen übernommen (`Lauf._kosten_uebernehmen`). Weiterfahren heißt also: `budget_usd` erhöhen
(oder die Zeile entfernen) und mit `--fortsetzen` starten; `test_ablauf.py` belegt denselben Endzustand
wie ohne Unterbrechung. Beim Fortsetzen nur `[grenzen]` ändern: der geladene Stand bringt Welt und Bots
mit, und `ausgabe` muss derselbe Ordner bleiben.

Sauber angehalten wird außerdem bei fehlendem Guthaben (402), abgelehntem Schlüssel (401, 403), vom
Anbieter abgelehnter Anfrage (400, 404, 422) und bei Strg+C in der Konsole. In allen Fällen gilt derselbe
Weg: Ursache beheben, mit `--fortsetzen` weiter. Kam die erste Runde eines Fensters nicht zustande, läuft
dieses Fenster neu. Scheitert erst eine Abfrage- oder Korrekturrunde, wird das Fenster noch abgeschlossen,
die betroffenen Rollen bleiben in diesem Aufruf ohne Antwort, und danach hält der Lauf an
(`Lauf._nachrunde`).

### Fortsetzen nach einem Absturz

Endet der Prozess hart (Rechner aus, Sitzung beendet, Prozess beendet), fehlt `stand.bin`. Dann setzt
`--fortsetzen` beim letzten Schnappschuss ein (`Lauf._nach_absturz`): Die Engine lädt
`schnappschuesse/tag-*.bin` mit der höchsten Nummer, `protokoll.auf_schnappschuss_kuerzen` kürzt Protokoll,
Tabellen und Entscheidungen genau auf diesen Stand (abgerissene gzip-Glieder und halbe Zeilen werden
toleriert), und der Lauf macht mit dem nächsten Schritt weiter. Nachspielen trifft danach denselben
Zustandshash; ein Test bricht dafür einen Lauf hart ab und vergleicht.

Folgen:

- Verloren ist die Spielzeit seit dem Schnappschuss. Mit `schnappschuss_tage = 1` höchstens ein Tag.
- Ohne Schnappschuss (Absturz vor dem ersten oder `schnappschuss_tage = 0`) bricht `--fortsetzen` mit
  `kein gespeicherter Stand und kein Schnappschuss in ...` ab.
- Die Modellaufrufe seit dem Schnappschuss sind bezahlt, ihre Entscheidungen werden aber verworfen und
  zählen nicht mehr gegen `budget_usd`. Als harte Grenze dient deshalb zusätzlich ein Ausgabelimit am
  Schlüssel bei OpenRouter.
- Ein neuer Lauf ohne `--fortsetzen` in einen Ordner, in dem schon ein Lauf liegt (`protokoll.jsonl`,
  `stand.bin` oder `schnappschuesse/`), wird mit „In … liegt schon ein Lauf“ abgelehnt. Früher überschrieb er
  Protokoll und Tabellen und ließ alte Schnappschüsse liegen, auf die ein späteres `--fortsetzen`
  zurückgegriffen hätte. Für einen neuen Lauf also einen neuen Ausgabeordner wählen oder den alten selbst
  entfernen.

### Neuversuche

Quelle: `OpenAIKompatibel._frage` und `OpenRouter` in `orchestrator/sternenepoche/backends.py`. Je Aufruf
bis zu `versuche` Versuche (Standard 3), dazwischen `min(30, 1,5^Versuch)` Sekunden plus Zufall, bei einer
Kopfzeile `Retry-After` bis zu 60 s.

| Ergebnis eines Versuchs | Behandlung |
|---|---|
| Zeitüberschreitung, Verbindungsfehler | Wartezeit, nächster Versuch |
| HTTP-Status ab 400 außer den fatalen (etwa 429, 5xx), Providerfehler in der Antwort | Wartezeit, nächster Versuch |
| leere Antwort oder am Tokenlimit abgeschnittenes JSON | nächster Versuch mit doppeltem `max_tokens` (höchstens 4 × (`max_ausgabe_tokens` + `denk_tokens`)); OpenRouter denkt dann weniger: `budget` wird zu `effort: none`, eine Stufe zu `low` |
| 401, 403, 400, 404, 422, bei OpenRouter 402 | fataler Fehler: der Lauf hält sauber an |
| alle Versuche gescheitert | die Rolle setzt diesen Aufruf aus; der Datensatz trägt `nach N Versuchen aufgegeben: ...`, `schluss.json` zählt ihn unter `ohne_antwort` |

Was gescheiterte Versuche gekostet haben, wird der Antwort zugerechnet und zählt gegen das Budget.

### Provider und Datenschutz

Für eine Messung pinnen `provider.order`, `allow_fallbacks = false` und `quantizations` Provider und
Gewichte; der tatsächliche Provider steht je Antwort unter `upstream`. `data_collection = "deny"` lässt nur
Provider zu, die Eingaben nicht speichern (Kommentare in `konfig/openrouter.toml`).

## 6. Rust-Orchestrator (`sternenepoche-agenten.exe`)

Quelle: `crates/agenten/src/main.rs`, `config.rs`, `journal.rs`, `provider.rs`, `lib.rs`. Der
Rust-Orchestrator bindet den Kern direkt ein, ohne Python und ohne Brücke. Unterschiede zum Python-Weg:
alle Spieler werden von Modellen gesteuert (keine Skriptbots), die Temperatur ist fest 0, es gibt keine
USD-Grenze, sondern eine dauerhafte Anfragegrenze `max_anfragen`.

| Befehl | Wirkung |
|---|---|
| `validate --config DATEI` | prüft die Konfiguration, ohne Anfrage |
| `run --config DATEI --out ORDNER [--windows 96] [--execute]` | fährt `--windows` weitere 15-Minuten-Fenster (96 = ein Spieltag); ein vorhandener Laufordner wird fortgesetzt |
| `demo --out ORDNER [--windows 96]` | eingebaute Demo: 4 Spieler, Startwert 42, ein Tag, Attrappe, ohne Netz |
| `export --journal ORDNER --out NEUER_ORDNER` | schreibt `entscheidungen.parquet`, `aktionen.parquet`, `metriken.parquet` |

Netzwerkaufrufe verlangen `--execute`; ohne endet `run` mit „Netzwerkaufrufe benötigen --execute“.
Fehler enden mit Exitcode 2. `run` druckt am Ende Zeit, Tag, Zahl der Fenster und Anfragen, Hash, `beendet`
und Rangliste als JSON.

```bash
./target/release/sternenepoche-agenten.exe validate --config konfig/rust-agenten-openrouter.json
```

```bash
./target/release/sternenepoche-agenten.exe demo --out laeufe/rust-demo
```

```bash
./target/release/sternenepoche-agenten.exe run --config konfig/rust-agenten-openrouter.json --out laeufe/rust-openrouter --windows 96 --execute
```

```bash
./target/release/sternenepoche-agenten.exe export --journal laeufe/rust-openrouter --out laeufe/rust-openrouter-parquet
```

### JSON-Konfiguration

Unbekannte Felder werden abgelehnt (`deny_unknown_fields`).

| Feld | Pflicht | Wertebereich | Bedeutung |
|---|---|---|---|
| `startwert` | ja | Ganzzahl ≥ 0 | Startwert der Welt |
| `spieler` | ja | 1 bis 50 | Zahl der Reiche, alle mit Modellen |
| `tage` | ja | 1 bis 365 | Länge der Epoche |
| `parallel` | ja | 1 bis 200 | gleichzeitige Aufrufe je Barriere |
| `max_anfragen` | ja | > 0 | dauerhafte Obergrenze aller reservierten Aufrufe im Laufordner, einschließlich Abfrage-, Korrektur- und Wiederholungsaufrufen |
| `anbieter` | ja | Objekt Name → Anbieter | siehe unten |
| `rollen` | ja | genau `stratege`, `verwalter`, `feldherr`, `diplomat` | Rolle → Anbietername |

Je Anbieter:

| Feld | Standard | Wertebereich | Bedeutung |
|---|---|---|---|
| `art` | Pflicht | `mock`, `local`, `openrouter` | Attrappe, lokaler Server, OpenRouter |
| `modell` | Pflicht | nicht leer | Modellkennung |
| `url` | leer | URL ohne Zugangsdaten, Query, Fragment | Pflicht außer bei `mock`; `local`: HTTP(S) auf `127.0.0.1`, `localhost` oder `[::1]`, fremde Hosts nur mit HTTPS und `remote_erlaubt`; `openrouter`: genau `https://openrouter.ai/api/v1` |
| `api_key_env` | leer | Name einer Umgebungsvariablen | Pflicht bei `openrouter` |
| `remote_erlaubt` | Pflicht | `true`, `false` | erlaubt Aufrufe an fremde Hosts |
| `max_tokens` | Pflicht | > 0 | Tokenlimit je Aufruf; nach Abbruch am Limit verdoppelt |
| `timeout_sekunden` | Pflicht | > 0 | Zeitlimit je Aufruf |
| `denken` | nichts senden | `aus`, `niedrig`, `mittel`, `hoch` | nur OpenRouter: `reasoning.effort` `none`, `low`, `medium`, `high`; nach Tokenlimit `low` |
| `schema` | `false` | Wahrheitswert | `true` erzwingt das Antwortschema (`json_schema`, `strict`) und setzt `require_parameters`; sonst `json_object` |
| `provider` | keiner | JSON-Objekt | nur OpenRouter, ergänzt die Vorgabe `allow_fallbacks: false`, etwa `order`, `data_collection` |
| `versuche` | 2 | 1 bis 5 | Versuche je Aufruf bei eindeutigem Fehler |

Mitgelieferte Dateien:

| Datei | Inhalt |
|---|---|
| `konfig/rust-agenten-demo.json` | 50 Spieler, 365 Tage, Attrappe `regel-demo-v1`, `max_anfragen` 1.000.000, ohne Netz |
| `konfig/rust-agenten-local.json` | Vorlage mit Platzhaltern: alle Rollen auf `local` (`http://127.0.0.1:8000/v1`, Modell `MODELL_ID_DES_LOKALEN_SERVERS`), `max_anfragen` 200; der zusätzliche Eintrag `openrouter` hat `remote_erlaubt: false` und wäre so nicht nutzbar |
| `konfig/rust-agenten-openrouter.json` | ein Spieler, ein Tag, Startwert 3, `max_anfragen` 200; Stratege und Diplomat `qwen/qwen3.5-flash-02-23` (`max_tokens` 3000), Verwalter und Feldherr `deepseek/deepseek-v4-flash` (`max_tokens` 1500, Provider `deepinfra` mit `allow_fallbacks: true`); alle mit `schema: true`, `denken: "aus"`, `data_collection: "deny"` |

### Journal und Fortsetzen

Ein Laufordner enthält:

| Pfad | Inhalt |
|---|---|
| `writer.lock` | Dateisperre: genau ein schreibender Prozess je Ordner |
| `manifest.json` | Protokollversion `rust-agenten-v1`, Hash des eingebetteten Regelwerks, vollständige Konfiguration; unveränderlich |
| `calls/<Zeit>-<Spieler>-<Rolle>-<Phase>[-vN].request.json` | Anfrage, vor dem Senden gespeichert |
| `calls/...response.json` | Antwort oder eindeutiger Fehler, nach Abschluss gespeichert |
| `states/<Index>.bin` und `.json` | Checkpoint nach jedem Fenster mit SHA-256 und Zustandshash |
| `states/<Index>.actions.json` | Aktionsprotokoll des Fensters |

Fortsetzen heißt: denselben Befehl mit demselben `--out` wiederholen. Der Lauf lädt den letzten geprüften
Checkpoint; ein unvollständiges Fenster wird von dort mit den gespeicherten Antworten neu berechnet, ohne
eine Anfrage doppelt zu senden. Weil `manifest.json` die ganze Konfiguration enthält, lässt sich für einen
bestehenden Ordner nichts an der Konfiguration ändern, auch nicht `max_anfragen`; jede inhaltliche Änderung
und jedes Bauen mit geändertem Regelwerk führt zu `Journal-Konflikt: manifest.json; eigener Laufordner nötig`.
`--windows` und `--execute` stehen nicht im Manifest und dürfen wechseln.

### Fehlerklassen

Quelle: `provider::Fehler` und `call_with_retries` in `lib.rs`.

| Klasse | Fälle | Behandlung |
|---|---|---|
| Abgelehnt | HTTP 401, 402, 403 (Schlüssel falsch, Guthaben leer, Schlüssellimit erreicht) | der Anbieter hat nichts verarbeitet: die Reservierung wird zurückgezogen, der Lauf hält an; nach dem Beheben setzt `run` fort und stellt genau diesen Aufruf neu |
| Eindeutig | übrige HTTP-Status außerhalb 2xx (etwa 429, 5xx), Antwort kein JSON oder größer 2 MiB, Abbruch am Tokenlimit, leerer Text, keine Verbindung, fehlende Schlüsselvariable | Aufruf wird mit Grund abgeschlossen und unter eigenem Schlüssel (`-v2`, `-v3` ...) erneut versucht, nach 1,5 s × Versuch Pause; nach `versuche` setzt die Rolle aus (ohne Korrekturanfrage), der Lauf geht weiter |
| Unklar | Anfrage gesendet, Antwort nicht vollständig angekommen (Zeitüberschreitung nach dem Senden, abgerissene Antwort) | der Lauf hält an; die Anfrage bleibt ohne Antwort im Journal und wird nie automatisch wiederholt, weil der Anbieter sie berechnet haben kann |

Fehlendes Guthaben (402) oder ein abgelehnter Schlüssel (401, 403) halten den Rust-Lauf wie den Python-Lauf an
(seit 4. Okt. 2026; vorher setzten die Rollen nur aus). Eine fehlende Schlüsselvariable gilt dagegen als
eindeutiger Fehler: jeder Aufruf scheitert, die Rollen setzen aus und die Versuche zählen gegen
`max_anfragen`. Vor `run --execute` deshalb den Schlüssel prüfen, etwa mit
`python -m sternenepoche pruefen konfig/openrouter-probe.toml`, das dieselben Modelle nutzt wie
`konfig/rust-agenten-openrouter.json`.

Ein unklarer Aufruf blockiert jedes weitere Fortsetzen mit
`<Schlüssel>: offener Aufruf mit unbekanntem Ausgang; automatische Wiederholung gesperrt`. Einen Befehl
zum Auflösen gibt es nicht. `crates/agenten/README.md` verlangt, den Fall am Anbieter und im Journal zu
klären und Journaldateien nicht blind zu löschen. Wer nach dieser Klärung bewusst neu senden will: Erst
das Entfernen der betreffenden `calls/...request.json` gibt den Aufruf frei; er wird dann erneut gesendet
und unter Umständen ein zweites Mal berechnet.

## 7. Lange Läufe auf Windows

Ein Prozess, der aus einer Terminal- oder Agentensitzung gestartet wird, hängt an deren Prozessbaum und
endet mit ihr. So endete der echte Lauf am 4. Okt. 2026 bei Tag 32 hart (`COORDINATION.md`, Nachtrag
14:30). Läufe über Stunden deshalb über WMI starten: `Win32_Process.Create` erzeugt einen Prozess
außerhalb der Sitzung.

Der neue Prozess erbt die Umgebung der aufrufenden PowerShell nicht. Der Schlüssel wird deshalb in der
Befehlszeile von `cmd.exe` gesetzt. Zwischen Wert und `&&` steht kein Leerzeichen, sonst gehört es zum
Schlüssel. Beispiel mit Platzhalter:

```powershell
Invoke-CimMethod -ClassName Win32_Process -MethodName Create -Arguments @{ CommandLine = 'cmd.exe /c "set OPENROUTER_API_KEY=...&& .venv\Scripts\python.exe -m sternenepoche lauf konfig\openrouter-test.toml >> laeufe\openrouter-test.log 2>> laeufe\openrouter-test.err"'; CurrentDirectory = 'D:\projekte_ki\Sternepoche' }
```

Damit der Schlüssel nicht im PowerShell-Verlauf landet, die Befehlszeile aus der Umgebungsvariable der
aktuellen Sitzung bilden (vorher mit `Read-Host` gesetzt, Abschnitt 5):

```powershell
Invoke-CimMethod -ClassName Win32_Process -MethodName Create -Arguments @{ CommandLine = "cmd.exe /c `"set OPENROUTER_API_KEY=$env:OPENROUTER_API_KEY&& .venv\Scripts\python.exe -m sternenepoche lauf konfig\openrouter-test.toml >> laeufe\openrouter-test.log 2>> laeufe\openrouter-test.err`""; CurrentDirectory = 'D:\projekte_ki\Sternepoche' }
```

Hinweise:

- `ReturnValue` 0 heißt gestartet; `ProcessId` ist die Nummer des `cmd.exe`-Prozesses.
- Solange der Lauf läuft, steht der Schlüssel in der Befehlszeile dieses `cmd.exe` und ist in der
  Prozessliste des Rechners lesbar.
- `CurrentDirectory` muss der Projektordner sein, weil Engine und Ausgabe relativ angegeben sind.
- `>>` hängt an vorhandene Logs an; der Orchestrator schreibt jede Tageszeile sofort
  (`Tag N: E Entscheidungen, X USD, vorn NAME mit P Punkten`).

Fortschritt verfolgen:

```powershell
Get-Content laeufe\openrouter-test.log -Tail 20 -Wait
```

Prüfen, ob der Lauf noch rechnet:

```powershell
Get-CimInstance Win32_Process -Filter "Name = 'python.exe'" | Select-Object ProcessId, ParentProcessId, CreationDate
```

Fortsetzen nach Budgetgrenze, fatalem Fehler oder Absturz: derselbe Aufruf mit `--fortsetzen` hinter
der Konfiguration.

```powershell
Invoke-CimMethod -ClassName Win32_Process -MethodName Create -Arguments @{ CommandLine = "cmd.exe /c `"set OPENROUTER_API_KEY=$env:OPENROUTER_API_KEY&& .venv\Scripts\python.exe -m sternenepoche lauf konfig\openrouter-test.toml --fortsetzen >> laeufe\openrouter-test.log 2>> laeufe\openrouter-test.err`""; CurrentDirectory = 'D:\projekte_ki\Sternepoche' }
```

Ein über WMI gestarteter Lauf hat keine Konsole, Strg+C erreicht ihn nicht. Wer ihn beendet, beendet ihn
hart; `--fortsetzen` setzt dann beim letzten Schnappschuss ein (Abschnitt 5).

Der Rust-Orchestrator lässt sich nach demselben Muster starten, mit
`target\release\sternenepoche-agenten.exe run --config ... --out ... --windows N --execute` statt des
Python-Aufrufs; Fortsetzen ist dort derselbe Befehl.

Während ein Lauf rechnet, ist seine Binärdatei gesperrt. `cargo build --release` und `cargo test --release`
scheitern dann beim Ersetzen von `target/release/sternenepoche.exe` (Abschnitt 10).

## 8. Auswertung

### Etiketten

`etiketten` hängt jeder Entscheidung ihre Folgen an und schreibt `etiketten.jsonl` mit `zeit`, `spieler`,
`rolle`, `punkte`, `endrang`, `endpunkte` und `delta_1`, `delta_7`, `delta_30` (Punktedifferenz nach 1, 7
und 30 Spieltagen; `null`, wenn der Tag nicht mehr im Lauf liegt). Gebraucht werden `schluss.json` und
`tageswerte.jsonl` (`protokoll.etiketten`).

```bash
.venv/Scripts/python.exe -m sternenepoche etiketten laeufe/probe
```

### Nachspielen

`replay` spielt `protokoll.jsonl` aus dem Startwert nach und vergleicht mit `hash` in `schluss.json`. Es
nimmt das Regelwerk des Laufs aus `regelwerk.ron` im Laufordner (ohne diese Datei das eingebaute, mit
`--regeln PFAD` ein anderes) und prüft, dass es den Hash des Laufs hat. Nach dem letzten Protokolleintrag
rechnet es bis zur Zeit `zeit` in `schluss.json` weiter (`nachspielen` in `crates/lauf/src/main.rs`), bei
einem beendeten Lauf also bis zum Epochenende, bei einem angehaltenen bis zum Haltepunkt. Rust-Journale haben
kein `schluss.json` und lassen sich so nicht nachspielen.

```bash
./target/release/sternenepoche.exe replay laeufe/openrouter-probe
```

### Betrachter

`betrachter/index.html` ist eine einzelne HTML-Datei. Sie zeigt Punkte im Verlauf (gesamt, Wirtschaft,
Forschung, Militär, Einwohner, Flottenwert, Stabilität der Heimatwelt), die Galaxie, Rangliste, Kämpfe und
Vertragsregister, über `zeigen` auch die Entscheidungen der Modelle. Sie liest `schluss.json` und
`tageswerte.jsonl` (Pflicht) sowie `kampfberichte.jsonl` und `register.jsonl`; das passt zu Läufen des
Python-Orchestrators und zu `epoche --aus`.

Drei Wege:

1. Empfohlen, auch während ein Lauf rechnet: `zeigen` startet einen Server nur auf 127.0.0.1 (Port 8198)
   und öffnet den Browser. Ein laufender Lauf erscheint mit dem Stand seines neuesten Tagesschnappschusses
   und aktualisiert sich alle 30 Sekunden; der Abschnitt „Entscheidungen der Modelle“ zeigt jeden Aufruf mit
   Befehlen, Urteil des Kerns, Kosten, Begründung, Notiz und Prognose. Der Server liest nur.

```bash
.venv/Scripts/python.exe -m sternenepoche zeigen laeufe/openrouter-test
```

2. Datei direkt im Browser öffnen und über die Dateiauswahl die vier Dateien eines Laufordners wählen.
3. Über einen beliebigen Webserver mit `?lauf=PFAD`; der Pfad gilt relativ zur Seite oder ab der
   Serverwurzel. `.claude/launch.json` startet dafür einen Server auf Port 8765:

```bash
.venv/Scripts/python.exe -m http.server 8765 --bind 127.0.0.1
```

Danach im Browser `http://127.0.0.1:8765/betrachter/index.html?lauf=/laeufe/probe` öffnen.

### Felder von `schluss.json`

Der Python-Orchestrator schreibt (`Lauf.fahre` in `lauf.py`):

| Feld | Inhalt |
|---|---|
| `startwert`, `spieler`, `ki`, `tage` | Kopfdaten des Laufs |
| `regel_version`, `regel_hash` | Regelwerk |
| `bots` | Spielernummer → Bottyp |
| `anbieter`, `rollen`, `vergleich` | Art, Modell und Providervorgaben je Anbieter, Rollenzuordnung |
| `hash`, `log_hash`, `aktionen` | Zustandshash, Hash und Zahl der Protokolleinträge |
| `zeit`, `beendet`, `angehalten` | Spielzeit in Sekunden, `true` bei vollständiger Epoche, sonst Grund des Anhaltens |
| `rangliste` | Einträge `[Rang, Name, Punkte, Stufe]` |
| `statistik` | je Spieler: `id`, `name`, `volk`, `steuerung` (Bottyp oder `modell`), `stufe`, `rang`, `punkte` je Bereich, `statistik` (Aktionen, abgelehnt, Angriffe, Beute, Verluste ...), `stufenzeit` II bis V, `erste_kolonie`, `kolonien`, `credits` |
| `welt` | Galaxie, Planeten und Spieler für den Betrachter |
| `kosten` | `usd`, `aufrufe`, `ein_tokens`, `aus_tokens`, `je_anbieter` (Aufrufe, Tokens, Kosten, Fehler) |
| `entscheidungen`, `ohne_antwort` | Zahl der Entscheidungen; je Anbieter die Aufrufe, die nach allen Versuchen ohne Antwort blieben |
| `laufzeit_s` | Rechenzeit dieses Abschnitts in Sekunden |

`epoche --aus` schreibt eine kürzere Fassung mit `startwert`, `spieler`, `ki` (leer), `tage`,
`regel_version`, `regel_hash`, `hash`, `log_hash`, `bots`, `statistik`, `rangliste` und `welt`.

### Parquet aus dem Rust-Journal

`export` schreibt drei Dateien mit den Spalten `zeit`, `spieler`, `rolle`, `phase`, `eingabe`, `ergebnis`,
`tokens`: `entscheidungen.parquet` (je Aufruf die gefilterten Modellnachrichten und die Antwort; offene
Aufrufe als `pending_unknown`), `aktionen.parquet` (nur Fenster mit veröffentlichtem Checkpoint) und
`metriken.parquet` (Zustandskennzahlen je Checkpoint, als `label_only` markiert und nie Modelleingabe).
Der Zielordner muss neu sein; während ein Lauf den Ordner sperrt, scheitert der Export.

## 9. Wissensdatenbank (`sternenepoche-wissen.exe`)

Quelle: `crates/wissen/src/main.rs`, `index.rs`, `service.rs`, `db.rs`. Die Datenbank ist ein
unveränderlicher DuckDB-Schnappschuss unter `wissen/snapshots/sternenepoche-<Zeit>.duckdb`; der Zeiger
`wissen/current.json` nennt den gültigen (`database`, `snapshot_id`). Die DuckDB-Bibliothek wird zur
Laufzeit aus `wissen/native/duckdb-1.5.6/duckdb.dll` geladen oder aus dem Pfad in
`STERNENEPOCHE_DUCKDB_LIB`; das Archiv dazu liegt als `wissen/native/libduckdb-windows-amd64-1.5.6.zip` bereit.
Der Projektordner ist beim Bauen fest einkompiliert (`CARGO_MANIFEST_DIR`).

Aufruf: `sternenepoche-wissen build|mcp|serve|query [DATENBANK] [PORT|OPERATION] [JSON]`. Das Argument nach
dem Befehl ist immer der Datenbankpfad (Standard `wissen/current.json`); bei `query` steht er deshalb vor
der Operation.

| Befehl | Wirkung |
|---|---|
| `build` | baut einen neuen Schnappschuss und veröffentlicht ihn erst danach in `current.json`; ändert sich das Regelwerk während des Baus, bleibt der Schnappschuss unveröffentlicht |
| `serve [DATENBANK] [PORT]` | HTTP nur auf 127.0.0.1, Standardport 8197, nur lesend; lädt neu, wenn `current.json` wechselt |
| `mcp [DATENBANK]` | MCP-Server über Standardein- und -ausgabe (JSON-RPC, höchstens 64 KiB je Nachricht) |
| `query DATENBANK OPERATION [JSON]` | eine Abfrage, Ergebnis als JSON |

Indexiert werden `crates`, `regeln`, `content`, `konfig`, `wissen/quellen`, `orchestrator`, `tools`, `docs` (Kategorie `documentation`, mit Bildern), `betrachter`,
`README.md`, `COORDINATION.md`, `Cargo.toml`, `Cargo.lock`, `wissen/knowledge.json` und aus `laeufe/` die
Dateien `.md`, `.csv`, `schluss.json`, `fortschritt.json`, `manifest.json`; `.parquet` und `.bin` nur als
Bestandsliste. Bilder werden eingebettet. Jeder Bau legt einen weiteren Schnappschuss von derzeit rund
90 MB an; alte bleiben liegen und belegen Platz auf D:.

```bash
./target/release/sternenepoche-wissen.exe build
```

```bash
./target/release/sternenepoche-wissen.exe serve wissen/current.json 8197
```

```bash
./target/release/sternenepoche-wissen.exe query wissen/current.json status
```

```bash
./target/release/sternenepoche-wissen.exe query wissen/current.json search '{"q":"raketensilo","limit":5}'
```

PowerShell 5.1 entfernt innere Anführungszeichen beim Übergeben an native Programme; JSON-Argumente
deshalb in Git Bash übergeben.

### Operationen

| Operation | Parameter | Ergebnis |
|---|---|---|
| `status` | – | Metadaten (`snapshot_id`, `rule_sha256`, `database` ...), Zahl der Einträge je Tabelle, Arten, Asset-Status |
| `search` | `q` (Pflicht, bis 300 Zeichen, höchstens 10 Wörter, alle müssen vorkommen), `kind`, `limit`, `offset` | Treffer über ID, Titel und Text |
| `entities` | `q`, `kind`, `faction`, `status`, `limit`, `offset` | Spielobjekte und Regeln gefiltert |
| `entity` | `id` (Pflicht) | ein Objekt mit Parametern, Kosten und Beziehungen |
| `source` | `id` (Pflicht, Quellpfad wie `README.md`), `limit`, `offset` (in Zeilen) | Textausschnitt mit Zeilennummern |
| `assets` | `category`, `faction`, `status`, `limit`, `offset` | Grafikbestand, geplant oder vorhanden |
| `asset` | `id` (Pflicht), `include_image` | Metadaten, optional Bild als Base64 bis 10 MiB |
| `relations` | `id` (Pflicht), `direction` (`in`, `out`, `both`), `limit`, `offset` | Beziehungen eines Objekts |
| `issues` | `owner`, `status`, `limit`, `offset` | Einträge der Arten `issue`, `requirement`, `decision`, `handoff` aus `wissen/knowledge.json` |
| `handoff` | – | Status, `docs/README.md` (Index der Dokumentation), `COORDINATION.md` und `wissen/OPUS_HANDOFF.md`, falls vorhanden; offene Punkte, Leseanleitung |

`limit` liegt zwischen 1 und 200 (Standard 30), `offset` bis 1.000.000. Kennungen haben die Form
`faction:<volk>`, `building:<gebäude>`, `technology:<forschung>`, `ship:<einheit>`, `defense:<einheit>`,
`resource:<gut>`, `role:<rolle>`, `action:<typ>`, `mission:<mission>`, `progression:<stufe>`,
`rules:<abschnitt>`, `asset:<id>`, `image:<pfad>`.

### HTTP-Schnittstelle

`GET /v1/<operation>?schlüssel=wert` oder `POST /v1/query` mit `Content-Type: application/json` und genau
`{"operation": ..., "args": {...}}`. Der Host muss `127.0.0.1:<Port>` oder `localhost:<Port>` heißen,
ein `Origin` muss derselbe sein; Anfragen bis 64 KiB, Antworten bis 16 MiB.

```bash
curl -s "http://127.0.0.1:8197/v1/search?q=blockade&limit=5"
```

Der MCP-Server bietet die zehn Operationen als Werkzeuge, dazu die Ressourcen `sternenepoche://status`,
`sternenepoche://handoff`, `sternenepoche://issues`, Vorlagen für `entity`, `source`, `asset` und
`asset-image` sowie den Prompt `opus_handoff`.

## 10. Fehlerbehebung

### Bauen und Engine

| Meldung | Ursache | Abhilfe |
|---|---|---|
| `cargo` kann `target/release/sternenepoche.exe` (oder eine andere `.exe`) nicht ersetzen oder entfernen, Zugriff verweigert | die Datei läuft gerade, etwa als Brücke eines Python-Laufs, als Rust-Lauf oder als `serve` | warten, bis der Lauf endet, oder mit `--target-dir target-bau` in ein eigenes Verzeichnis bauen |
| `Fehler: Spielerzahl N passt nicht in die Galaxie` | mehr als 60 Spieler mit dem Standardregelwerk | `spieler` senken |
| `Fehler: das Regelwerk hat einen anderen Hash als im Lauf …` | `replay` eines Laufs ohne `regelwerk.ron` (vor dem 4. Okt. 2026 begonnen), dessen Regelwerk sich seitdem geändert hat | mit `--regeln` das Regelwerk des Laufs angeben |
| `In … liegt schon ein Lauf` | neuer Lauf in einen belegten Ausgabeordner | `--fortsetzen`, anderer Ausgabeordner oder den alten Ordner selbst entfernen |
| `... Zustandshash ... WEICHT AB`, `Nachspielen weicht ab` | Lauf nicht beendet (Nachspielen rechnet bis zum Epochenende) oder Protokoll beschädigt | nur beendete Läufe nachspielen (`"beendet": true`) |
| `Fehler: nicht alle Abnahmekriterien erfüllt` | `balance --abnahme` hat ein Kriterium verfehlt | Tabelle der Ausgabe lesen, mit `stufe5` und `epoche --spur` nach Bot-Blockaden suchen |

### Python-Orchestrator

| Meldung | Ursache | Abhilfe |
|---|---|---|
| `Engine nicht gefunden: target/release/sternenepoche.exe. Erst bauen: cargo build --release` | Engine nicht gebaut oder Befehl nicht im Projektordner gestartet | bauen; im Projektordner starten |
| `[lauf]: unbekannte Felder [...]`, `[anbieter.x]: modell fehlt`, `[rollen]: verwalter fehlt` | Konfiguration fehlerhaft (Exitcode 1) | Abschnitt 4 |
| `... Umgebungsvariable OPENROUTER_API_KEY ist nicht gesetzt ...` | Schlüssel fehlt in der Prozessumgebung | Schlüssel setzen, `--fortsetzen` |
| `OpenRouter meldet kein Guthaben mehr (402 ...)` | Guthaben oder Schlüssellimit aufgebraucht | aufladen oder Limit erhöhen, dann `--fortsetzen` |
| `Zugriff abgelehnt (401)` oder `(403)` | Schlüssel ungültig | Schlüssel prüfen, `pruefen` |
| `Anfrage für Modell '...' abgelehnt (400/404/422) ...` | Modell oder Provider unterstützt das Antwortschema nicht | `pruefen`; `schema = "json_object"` oder anderes Modell |
| `Lauf angehalten (BudgetErreicht: Budget von X USD erreicht ...)` | `budget_usd` erreicht | Grenze erhöhen, `--fortsetzen` |
| `Lauf angehalten (KeyboardInterrupt)` oder `(CancelledError)` | Strg+C in der Konsole | `--fortsetzen` |
| `nach 3 Versuchen aufgegeben: Status 429 ...` im Datensatz, Zähler `ohne_antwort` | Ratenlimit oder Serverfehler hielt an | ist kein Abbruch; bei vielen Fällen `parallel` senken |
| `kein gespeicherter Stand und kein Schnappschuss in ...` | Absturz vor dem ersten Schnappschuss oder `schnappschuss_tage = 0` | Lauf in neuem Ordner neu starten |
| `protokoll.jsonl hat N Zeilen, der Schnappschuss kennt M` | Schnappschuss passt nicht zu den Dateien, etwa aus einem älteren Lauf im selben Ordner | jeden Lauf in einem eigenen Ordner starten |
| `die Engine läuft nicht mehr`, `die Engine hat auf '...' nicht geantwortet` | Engine-Prozess abgestürzt | `laeufe/*.err` lesen; `--fortsetzen` setzt beim Schnappschuss ein |

### Rust-Orchestrator

| Meldung | Ursache | Abhilfe |
|---|---|---|
| `Netzwerkaufrufe benötigen --execute; demo arbeitet vollständig offline` | Anbieter nicht `mock`, `--execute` fehlt | `--execute` angeben |
| `Externe Modellaufrufe benötigen HTTPS und remote_erlaubt` | fremder Host ohne HTTPS oder ohne `remote_erlaubt: true` | Konfiguration korrigieren (neuer Ordner, Abschnitt 6) |
| `OpenRouter benötigt offiziellen HTTPS-Endpunkt und api_key_env` | `url` oder `api_key_env` falsch | `https://openrouter.ai/api/v1`, `OPENROUTER_API_KEY` |
| `Ein anderer Prozess verwendet dieses Journal` | zweiter Prozess am selben Ordner, auch `export` während eines Laufs | warten, bis der erste endet |
| `Journal-Konflikt: manifest.json; eigener Laufordner nötig` | Konfiguration oder eingebettetes Regelwerk geändert | alte Konfiguration und Binärdatei verwenden oder neuen Ordner |
| `Eingaben für <Schlüssel> haben sich geändert` | dieselbe Anfrage würde heute anders gestellt, etwa nach geändertem Lagebild im Code | mit der Binärdatei des Laufs fortsetzen oder neuen Ordner |
| `<Schlüssel>: offener Aufruf mit unbekanntem Ausgang; automatische Wiederholung gesperrt` | ein unklarer Aufruf aus einem früheren Abschnitt | am Anbieter klären, Abschnitt 6 |
| `HTTP-Aufruf fehlgeschlagen; Ausgang unbekannt, keine automatische Wiederholung`, `HTTP-Antwort unterbrochen` | unklarer Ausgang, der Lauf hält an | wie oben |
| `Persistente Anfragegrenze erreicht` | `max_anfragen` im Ordner verbraucht; die Grenze ist im Manifest festgeschrieben | Lauf ist an seinem Ende; für mehr einen neuen Ordner mit höherer Grenze |
| `Checkpoint-Prüfsumme falsch`, `Checkpoint-Zustandshash falsch` | Checkpoint verändert oder beschädigt | nicht fortsetzen, Ordner untersuchen |
| `HTTP-Status 402: der Anbieter lehnt ab – Schlüssel, Guthaben oder Schlüssellimit prüfen …` (auch 401, 403) | Guthaben, Schlüssel oder Schlüssellimit; der Lauf hält an, der abgelehnte Aufruf ist nicht im Journal | Ursache beheben, dann denselben `run`-Befehl erneut starten |
| `API-Key-Umgebungsvariable fehlt` in den Antworten | Schlüsselvariable nicht gesetzt; die Rollen setzen aus | Variable setzen; die betroffenen Aufrufe sind verloren, der Lauf läuft weiter |

### Kern, Spieler und Wissensdatenbank

| Meldung | Ursache | Abhilfe |
|---|---|---|
| `Entscheidungsfenster wartet auf KI-Antworten` (Spieler), `Weltuhr angehalten: KI-Entscheidungen fehlen` | im nativen Spieler ist ein Fenster fällig, dessen Modellrollen nicht abgeschlossen sind; die Weltuhr wartet | erst alle fälligen Aufrufe abschließen |
| `Entscheidungsfenster wartet noch auf Rollen-Antworten` (Kern, `schritt_wenn_bereit`) | in `kern::umgebung` oder im Spieler soll die Uhr weiterlaufen, bevor jede fällige Rolle ihren Aufruf beendet hat | alle fälligen Rollen handeln lassen, auch mit leerem Zug |
| `DuckDB library ...duckdb.dll: ...` | Bibliothek fehlt | Archiv in `wissen/native/` nach `wissen/native/duckdb-1.5.6/` entpacken oder `STERNENEPOCHE_DUCKDB_LIB` setzen |
| `Rules changed during index build; unpublished snapshot retained: ...` | Regelwerk während `build` geändert | `build` wiederholen |
| HTTP 403 `Host abgelehnt`, `Origin abgelehnt` | Anfrage nicht über `127.0.0.1:<Port>` oder `localhost:<Port>` | Adresse korrigieren |
| `Unknown operation ...`, `Unbekannte Operation` | Operation falsch geschrieben | Operationen aus Abschnitt 9 |
| DuckDB kann eine Datenbank `status`, `search` o. ä. nicht öffnen | bei `query` fehlt der Datenbankpfad; das erste Argument gilt immer als Pfad | `query wissen/current.json OPERATION` |
