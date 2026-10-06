---
layout: default
title: "Labor: Prüfbelege und verbleibende Abnahme"
---

# Labor: Prüfbelege und verbleibende Abnahme

Stand: 5. Oktober 2026. Windows-Host, native Rust-Isolation; frühere Docker-Belege bleiben historisch.

## Abschluss: Kapazität, kleine Tools und Forschungsberichte

Der geplante Backend-Umfang B0–B7 ist als ausführbare Infrastruktur umgesetzt. Die letzten Ergänzungen
sind konkrete V4-Abfragewerkzeuge statt einer verpflichtenden Varianten-Sammelabfrage, ein quellengebundener
Kapazitätsplaner, eine deterministische Karten-/Taktikauswertung und gekreuzte Modell-Harness-Vergleiche.
UI, frei ausführbarer Agentencode und eine behauptete optimale Strategie gehören nicht zu dieser Abnahme.

**33 gezielte Labortests bestanden**, ein alter Docker-Umgebungstest wurde ausgelassen.
Beleg: `laeufe/labor-evidence/tests-v4-research-final.txt`. Darunter V3-Kompatibilität, V4-Fairness,
Wiederaufnahme, Sicht-/Lerngrenzen, Abfragevalidierung, ungeeignete Kapazitätsbelege, Sitzzuordnung,
Research-Resume und Manipulation der Herkunftsläufe. Die zuvor geprüften 23 Kern-/Kolonisationsszenarien
wurden nicht erneut ausgeführt; der Spielkern wurde in dieser Abschlussrunde nicht verändert.

| Kurzer realer oder nativer Nachweis | Ergebnis |
|---|---|
| Qwen3.5 2B, alte Sammelabfrage, vier Spieler | 8 Calls, 38 ungültige Tools, keine Aktionen: negativer Befund, kein Strategienachweis. |
| Qwen3.5 2B, konkrete Tools, vier Spieler | 16 Calls, 20 Tools, davon 8 Fehler; private Notizen funktionieren, noch keine ausgeführte Spielaktion. Kein Wirksamkeitsvergleich aus nur diesen beiden unterschiedlich budgetierten Proben. |
| Qwen3 8B, zehn native Spieler, ein Fenster | **40 Calls, 14 ausgeführte Aktionen, 11 Speicheränderungen; alle zehn Spieler speichern und handeln.** Fünf ungültige Fusionskraftwerk-Versuche wurden abgewiesen. **96,445 Sekunden** vom Fensterbeginn bis zum gespeicherten Abschluss. 162 verifizierte Audit-Ereignisse; fairer Vergleich und Echtzeitziel erfüllt. |
| Replay und Export dieses echten Laufs | Welthash `9133fc54c08f59ba3f33fd1c7219c3adfbf388ccde3318e6280ac56c8d378583`; privater Datensatzexport für zehn Spieler erfolgreich. |
| Native Forschungs-CLI, 2 Mockmodelle × 2 Harnesses × 3 Seeds | 12 unabhängige Kurzläufe mit vollständigen Sitzrotationen; Bericht enthält Seedmittel, Streuung, Kostenstatus, Tool-/Gedächtnismetriken und verifizierte Herkunft. Keine Modellgewinner behauptet. |
| Balancebericht | 18 Karten für 2/3/10/11/20/50 Spieler plus kurze Versorgungs- und Schildschwellen. `PASS_WITH_LIMITS`; geografische Asymmetrien bleiben ausdrücklich sichtbar. |

Reale Belege liegen unter `v4-calibration-small`, `v4-calibration-flat`, `v4-calibration-qwen8` und
`v4-qwen8-ten` in `laeufe/labor-evidence`. Forschungsbericht:
`v4-factorial-control/report.json`; Balancebericht: `v4-balance-report.json`.
Die GPU-Proben verwendeten vorhandene Gewichte. Nur selbst geladene Ollama-Modelle wurden danach entladen;
fremde GPU-Arbeit wurde nicht beendet.

Der Kapazitätsplan nutzt 44 echte Qwen-8B-Calls einschließlich beobachteter Verlangsamung. Bei voller
Ausnutzung von 8K Kontext und 1024 Ausgabetoken ist er deutlich vorsichtiger als der kurze frühe Lauf:
Das erzeugte Profil `konfig/labor-native-local-v4-kapazitaet.json` begrenzt zwei Spieler auf je vier Calls.
Für zehn Spieler mit demselben Maximalbudget erteilt er **keine** konservative Kapazitätsfreigabe, obwohl
der gemessene frühe Ablauf in 96 Sekunden fertig wurde. Diese Unterscheidung ist beabsichtigt: kurze
Werkzeugantworten sind kein Beleg für spätere volle Planungsantworten oder 50 starke Modellspieler.
Das Forschungssystem ist einsatzbereit; empirische Stärke, Langzeitbalance und maximaler später
Durchsatz sind Ergebnisse zukünftiger Experimente, keine als fertig ausgegebenen Garantien.

Die folgenden Abschnitte sind historische Prüfstände.

## V4: Abschluss der Integrationsprüfung am 5. Oktober

Der Sol-6.1-High-Partner schloss fünf weitere Kernlücken: Integritätsreste nach vollständigem Abriss,
Laborprüfung beim Start einer Forschung nach Besitzverlust, alte Bau-/Fertigungsereignisse nach Capture,
mehrfaches Gefecht bei identischem Fensterabschluss sowie Empfängerbindung bereits gekaufter Marktware.
Die neue Marktlieferungsvariante ist am Ende des Ereignis-Enums angefügt; bestehende Diskriminanten bleiben
erhalten. Eigene Flottenansichten prüfen Transportempfänger, Rückkehrziel und Warteorbit mit.

Nachgewiesen: **16 V4-Kernszenarien und 7 bestehende Kolonisationstests** beim Partner. Ein Testvergleich
benötigte die reguläre Bestandsabrechnung; nach Korrektur bestand die gezielte Wiederholung. **28 Labor-
Integrationsprüfungen** bestanden: 10 Runtime, 8 V3-Kompatibilität, 10 V4. Beleg:
`laeufe/labor-evidence/tests-v4-integration-2026-10-05.txt`. Der alte Docker-Umgebungstest blieb explizit
übersprungen. Testzeiten ohne Kompilierung: 2,66 s + 10,48 s + 1,15 s.

Neu geprüft: Absturz nach gespeicherten Audit-Ereignissen vor dem Checkpoint mit bytegleicher
Wiederaufnahme und ohne neue Calls; Manipulation des dauerhaften Zeitabschlusses; regelversionierte
Lernpakete und Importgrenzen; Vergleichsqualität im Export; dringende Erinnerungen außerhalb der ersten
30 Datensätze; private Marktlieferungen über das echte Gateway; Initialkontext ohne verlorene Reaktion;
konfigurierte statt fest eingebauter Kapazitätszeit. CLI und Bibliotheksinventur erzeugen beide V4.

Zusätzlich startete die fertige CLI zwei echte native Rust-Spielerbüros für **ein Mockfenster**:
4 Mockantworten, 22 Audit-Ereignisse, `comparison_valid=true`, `real_time_target_met=true`; keine LLM-Inferenz.
Beleg: `laeufe/labor-evidence/native-v4-integration-2026-10-05.txt`. Replay und privater V2-Datensatzexport
funktionieren. Der frühere echte V3-Lauf bleibt unter der finalen Binärdatei mit unverändertem Hash
wiederholbar. Das lokale V4-Profil ist validiert. Die Projekt-Wissensbasis wird mit diesem Stand neu indiziert;
ihre Metadaten erteilen ausdrücklich keine eigene GPU-Freigabe.

Dies sind begrenzte Funktions- und Kompatibilitätsprüfungen mit Mockmodellen beziehungsweise lokalen
HTTP-Stubs. Es wurde kein Forschungs-Match mit echten Modellen durchgespielt und keine GPU-Messung
wiederholt. Balance und Durchsatz einer vollen späten Epoche werden dadurch weiterhin nicht behauptet.
Die folgenden Abschnitte dokumentieren die vorherigen Abnahmestände.

## V4: gezielte Abnahme der fertigen Regelentscheidungen

Nach der Konzeptprüfung wurden ausschließlich kurze Prüfungen der geänderten Logik ausgeführt:

- 12 Kernszenarien in `crates/kern/tests/flotten_ankunft_v4.rs` bestanden beim Sol-6.1-High-Partner.
  Sie prüfen Empfängerbindung, gezielten Nachschub, Einzel-/Verbandsentsatz, geänderte Allianzen,
  Rückkehrstrecken/Warteorbit, echte Verteidigungskämpfe während einer Besetzung, Kolonieschiffverlust,
  Kapazität, Bau-/Reparaturkonkurrenz sowie V2/V3-Bytes und V4-Snapshots.
- 7 bestehende Kolonisationstests bestanden ebenfalls. Die abschließenden gezielten Wiederholungen für
  beidseitige Abbruchmeldungen und einen inzwischen geschützten Zielbesitzer waren erfolgreich.
- 5 V4-Labortests bestanden in 0,79 Sekunden Testlaufzeit, ohne Modellaufrufe. Beleg:
  `laeufe/labor-evidence/tests-v4-labor.txt`. Geprüft sind Karten mit 2/10/50 Spielern, private unverändernde
  Kolonieplanung, Commit/Resume/Replay, Kapazitätsstopp gegenüber beeinträchtigtem Durchsatzlauf sowie
  Nachholen nach einer bereits gespeicherten Antwort ohne deren Änderung oder zusätzliches Spielerbudget.
  Fristablauf wird durch eine deterministische Fristvorgabe ausgelöst; es wird nicht 15 Minuten gewartet.
- Der neue Host baut erfolgreich; `konfig/labor-native-local-v4.json` ist ohne Netzwerkzugriff validiert.
  Der vorhandene echte V3-Lauf `v3-native-final-qwen8` lässt sich mit drei gespeicherten Fenstern unter
  der neuen Binärdatei wiederholen; Welthash `bda53369c54af02fd883b8dca9a3f70e1f2af4c5d39e805c67c889db713527e7`.

Das ist eine gezielte Funktionsabnahme, keine erneute Gesamttestsuite und kein Nachweis optimaler
Spielbalance oder des 50-Spieler-LLM-Durchsatzes. Native Isolation und echte Modellmessungen wurden in
dieser V4-Runde nicht erneut ausgeführt; die folgenden V3-Belege gelten für deren damaligen Prüfstand.
Die Übersicht dieser Runde liegt unter `laeufe/labor-evidence/LATEST-V4.json`.


## V3: neuer Backend- und Regelvertrag

98 reguläre Rust-Tests bestanden nach Einführung der neuen Kolonisations- und V3-Laufzeitregeln.
Beleg: `laeufe/labor-evidence/tests-v3-final.txt`. Darunter Sonden-Erkundung und versorgte Expedition,
echter Kampf mit mehreren Planetenschild-Einheiten, ursprünglicher Heimatschutz, zwei Reaktionsfenster,
Rückruf/Verteidigungswiederaufbau im letzten Fenster, Kapazitätsprüfung bei Übernahme, Reparatur und
Snapshot-Roundtrip. Die V2-Spielsnapshot-Tests laufen weiterhin durch.

V3-Laufzeittests prüfen Rollenkoordination, kompakte Werkzeugauswahl und Rechte, Ollama-only gegen
Cloud-/Remote-Modelle, abgelaufene Fristen ohne tatsächlichen Versand, Audit-Manipulation, Replay/Resume
sowie eigenes Gedächtnis über zwei abgeschlossene/neu begonnene Epochen mit verschiedenen Seeds.
Zusätzliche HTTP-Stubtests belegen: Fallback bewahrt explizite lokale Modell-/CPU-/Kontextvorgaben,
Inventarfehler führen nicht zu Remote-Aufrufen, ein leeres Inventar erlaubt den konfigurierten Ersatz.
Die Hauptmodellzuordnung rotiert in Vergleichsmatrizen zusammen mit dem Harness.
Die native Isolation wird zusätzlich mit tatsächlich gestarteten Windows-Workern geprüft; die genauen
Belege stehen in der Worker-Anleitung und im Review.

### Kurze freigegebene GPU-Messungen

Hardware: RTX 3080 mit 10 GiB VRAM, etwa 63 GiB RAM. Beim ersten Test belegte andere Arbeit bereits
rund 4,6 GiB VRAM. Es wurden keine Modelle heruntergeladen und keine fremden GPU-Prozesse beendet.

| Probe | Gemessener Befund |
|---|---|
| Mistral 7B, 8K, kleines natives Werkzeug | 5.110.360.964 Bytes Modellresidenz vollständig in VRAM; erster Call 43.843 ms, zweiter 338 ms, je 41 Ausgabetoken und ein Tool. `native-v3-mistral-profile.json`. |
| Mistral 7B, vollständiges frühes Spielbriefing | Keine Tools, nur wiederholter/übersetzter Lagebericht; auf 384 Token abgeschnitten. Die Läufe `v3-mistral-full-tools` und `v3-mistral-focused-tools` sind keine erfolgreichen Spielagenten. |
| Qwen3 8B, 8K, explizite Notizrevisionen | Sechs native Toolaufrufe, aber sechs Versionsfehler. Unveränderter Fehlversuch `v3-qwen8-full-tools`. |
| Qwen3 8B, 8K, vom Harness verwaltete Notizrevisionen | Zwei Spieler speichern je einen Plan, wählen Werkzeuge und bauen je eine Erzmine. Sechs Calls, keine Toolfehler, etwa 6,7 s gesamte warme Ausführung im Entwicklungsmodus `trusted`. Beleg `v3-qwen8-managed-memory`. |
| Qwen3 8B Residenz | `/api/ps`: 5.729.293.434 Bytes, vollständig VRAM, Kontext 8192; `v3-qwen8-residency.json`. |

Alle Pfade liegen unter `laeufe/labor-evidence/`. Native Isolation und echte Inferenz sind getrennt
nachzuweisen; der erfolgreiche `trusted`-Lauf allein belegt keine Sandbox. Er belegt auch keine gute
Langzeitstrategie: Die Agenten wählten Minenausbau trotz knapper Versorgung. Der Vergleich der kurzen
Aufrufe zeigt einen konkreten Harness-Fehler und dessen Beseitigung, keine statistisch abgesicherte
Überlegenheit über ein stärkeres Modell. `capacity` gibt nur eine gekennzeichnete Planungsschätzung.

### Kombinierter Nachweis: native Büros und echte lokale Spieler

`v3-native-qwen8` verbindet den nachgewiesenen Windows-Adapter mit zwei echten Qwen3:8b-Spielern.
Zwei Fenster inklusive Startprüfungen, Modellladen, sechs Calls und Checkpoints dauerten **12.089 ms**.
Beide Spieler speicherten private Pläne und ließen gültige Bauaktionen ausführen. Anschließendes Resume
auf Fenster 3 benötigte keine weiteren Modellaufrufe; Replay und Export zweier privater JSONL-Dateien
bestanden. Dateien: `v3-native-qwen8-config.json`, `v3-native-qwen8-summary.json`,
`v3-native-qwen8-timing.json`, `v3-native-qwen8-capacity.json` und `v3-native-qwen8-export/`.

Die separate Skalierungsprobe startete **50 native Mockspielerbüros**: Import/Start 3,199 s, anschließend
50 einfache Metadaten-RPCs 14 ms. Gemessener privater Prozessspeicher insgesamt 47.067.136 Bytes
(44,9 MiB), WorkingSet 244.527.104 Bytes (geteilte Seiten mehrfach gezählt). Zwei Weltfenster 13,789 s,
Resume um ein Fenster 2,412 s; Audit und Replay korrekt. Messung:
`laeufe/labor-tests/native-fifty-runtime-777840-1791147784434163300/native-measurement.json`.
Diese 50-Spieler-Probe misst Büro- und Simulationskosten, **keinen 50-Spieler-LLM-Durchsatz**.

## Historischer V2-Abnahmestand

Die folgenden Zahlen und Docker-/CPU-Läufe gehören zum früheren Vertrag und werden nicht als
V3-Benchmark ausgegeben.
Die Abstimmung erfolgte mit **gpt-6.1-sol high**, nicht mit dem wegen OAuth-Ausfall unerreichbaren Opus.

## Technische Prüfung

Ausgeführte Befehle:

```powershell
cargo test -p sternenepoche-agenten -p kern --lib --tests
cargo test -p sternenepoche-agenten --test labor_gateway --test labor_runtime -- --ignored
```

Die regulären Prüfungen decken den bestehenden Spielkern und Legacy-Runner sowie die neue Laufzeit ab.
**Abschlussergebnis: 83 reguläre Tests bestanden, 0 fehlgeschlagen; zusätzlich alle 3 expliziten
Docker-Prüfungen bestanden.** Die drei Docker-Tests sind im regulären Lauf als umgebungsabhängig markiert
und wurden anschließend separat ausgeführt.
Zu den neuen Tests gehören 96 Kartenkombinationen (6 Spielerzahlen × 16 Seeds), Rollenrechte ohne
`Rolle::Alle`-Bypass, echte private SQLite, Versionskonflikte, gemeinsame Rollenbudgets, feste Laufverträge,
Auditmanipulation, identische Welt bei serieller/paralleler Zustellung und Wiederaufnahme nach Abstürzen
vor Commit beziehungsweise nach gespeicherter Modellantwort. Außerdem: bestätigte Ablehnung mit
explizitem Retry, Matrix-Resume und gemeinsames Lernpaket mit gesperrtem Trainings-Seed.

Die drei expliziten Docker-Prüfungen wurden tatsächlich ausgeführt: negative Isolationsproben,
privater persistenter Worker inklusive Quoten und vollständiger Match-Neustart mit demselben Welt-Hash
wie im Trusted-Referenzlauf. Die Gateway-Gegenprüfung steht in [LABOR-REVIEW-SOL-HIGH.md](LABOR-REVIEW-SOL-HIGH.md).
Letzte Gesamtprotokolle: `laeufe/labor-evidence/tests-final.txt` und `tests-docker-final.txt`.

Der echte Windows/Linux-Resume-Test fand und beseitigte einen Fehler: Der Host verwendet JSON-Objekte mit
Einfügereihenfolge, der Worker sortierte Schlüssel. Logische DB-Inhalte werden jetzt vor dem Hashen
rekursiv kanonisiert. Ein anderer Absturztest fand ungesendete Vormerkungen, die irrtümlich als bereits
gesendet galten. Beide Zustände sind jetzt getrennt; unbekannte gesendete Calls bleiben gesperrt.

## Echter lokaler Modelllauf

Artefakte: `laeufe/labor-evidence/cpu-qwen35-office/`, zugehörige `cpu-qwen35-office-config.json`,
`cpu-qwen35-status.json`, `cpu-residency.json` und `cpu-qwen35-office-export/`.

| Messung | Ergebnis |
|---|---|
| Modell | `qwen3.5-2b:latest`, 1.9B Q4_K_M |
| Gewichtsdigest | `ec353d37c63d994e94fcf1ebefb51ed8b785f4424745d53ac74921c760687d4a` |
| Lauf-ID | `ccd23517a9dc8147ab42d1a54b00dbbe3d1e045df4d2d6fa9f80349fb60a612c` |
| Spieler/Isolation | 2 echte Modellspieler, 2 getrennte Docker-Büros |
| Inferenz | CPU, 32768 Kontext; `/api/ps` meldete `size_vram:0` |
| Calls | 2, keine Wiederholung beim Resume |
| Tokens | 8002 Eingabe, 306 Ausgabe; keine abgeschnittene Ausgabe |
| Summe Call-Dauer | 34,295 Sekunden, keine allgemeine Geschwindigkeitsmessung |
| Gedächtnis | je ein privater Plan `energie`, Revision 1 |
| Werkzeuge | 2× `memory_write`, 2× `game_bauen` |
| Weltwirkung | 2 akzeptierte Solarkraftwerk-Bauaufträge |
| Replay | Welt-Hash des ersten Fensters identisch |
| Resume | zweites Fenster, weiterhin insgesamt 2 Calls; Audit 16 Ereignisse / 3 Checkpoints |
| Export | zwei getrennte private JSONL-Trajektorien, Herkunft und Prüfsummen |

Der Auftrag an die Modelle war absichtlich ein kleiner **Integrationstest**: Plan speichern und
Solarkraftwerk bauen. Er ist kein Beleg für autonome Strategiequalität oder Überlegenheit eines kleinen
Modells. Die Plantexte enthalten Modellannahmen; sie werden als Pläne, nicht als Engine-Fakten geführt.
Der Test lief mit dem damals eingefrorenen Worker-Digest; die spätere Quotenhärtung erhält einen eigenen
Image-Digest und wurde zusätzlich geprüft. Alte Images wurden nicht aus vorhandenen Verträgen umgebogen.

## Offen dokumentierte Gegenbefunde

- `cpu-smoke/`: Ollama HTTP 500 wegen eines nativen Werkzeugs mit `anyOf` an der Schemawurzel.
  Einzelproben isolierten `world_query`. Der Providerkatalog nutzt nun Objektschemas; die genaue
  Variantenprüfung bleibt im Gateway erhalten. Der angehaltene Versuch wurde nicht gelöscht oder
  stillschweigend als erfolgreich wiederholt.
- `cpu-smoke-fixed-tools/`: echte `world_query`-/`memory_search`-Calls, auch ungültige Abfragen des Modells,
  anschließend erneuter HTTP 500. Der gesendete Aufruf bleibt ungeklärt; kein vollständig gewertetes Match.
- `cpu-qwen3/`: zwei echte CPU-Aufrufe von `qwen3:4b` lieferten lange Texte mit jeweils 1024 Ausgabetokens,
  aber keine nativen Tools. Der technische Fensterabschluss gelang; kein Plan wurde geschrieben und keine
  Aktion ausgelöst. Das zeigt ein Harness-/Modell-/Budgetproblem unter dieser konkreten Konfiguration,
  keine allgemeine Rangfolge von 2B und 4B. Die neue Laufzeit kennzeichnet abgeschnittene Ausgaben eigens.

## Vollständige Kontroll-Epochen und Inventar

`laeufe/labor-evidence/full-epoch-matrix/` enthält vier abgeschlossene **Mock-Kontrollmatches**:
zwei Spieler, Seeds 42/43, beide Sitzrotationen, je ein Spieltag mit 96 Fenstern.
Alle vier erreichten das Epochenende. `full-epoch-matrix-summary.json` enthält die maschinenlesbaren
Ergebnisse. Familie: `20780d21cecbcf36173b24dad06dbd88e0a7d3dd450890eaeeb954cd06213b67`.
Das prüft den vollständigen Ablauf; Mock-Siege sind keine Forschungsergebnisse über LLMs.

Ollama-Inventar: 47 installierte Aliase. Der tatsächlich ausgeführte `local-config --players 10 --cpu`
ergab 33 nach Digest deduplizierte, als toolfähig gemeldete Modelle mit mindestens 32K Kontext.
`installed-models-config.json` belegt die Auswahl. Keine Gewichte heruntergeladen oder durch das
Inventar geladen. Toolfähigkeit aus Metadaten ist noch kein bestandener Verhaltenstest jedes Modells.

## Was diese Abnahme noch nicht beweist

Keine vollständigen langen LLM-Turniere mit 10/50 Spielern, keine statistisch abgesicherte Harness-
Überlegenheit, kein OpenRouter-Livetest und keine gemeinsame GPU-Kapazitätskalibrierung.
Keine automatische kausale Konzeptentdeckung, keine freie Code-Skill-Ausführung und kein Firecracker-
Nachweis. Die getestete Erstversion liefert die ausführbare Grundlage und getrennte Messdaten dafür.
Die UI wurde nicht weiterentwickelt.
