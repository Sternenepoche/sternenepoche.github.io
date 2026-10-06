---
layout: default
title: "Autonome Spieler: ausführbarer Backend-Stand"
---

# Autonome Spieler: ausführbarer Backend-Stand

Stand: 5. Oktober 2026. Der Rust-Befehl `sternenepoche-labor` implementiert die zusammenhängende
Laufzeit aus dem [Spieler-Sandbox-Konzept](SPIELER-SANDBOX-KONZEPT.md). Ein Match läuft ohne UI und ohne
menschliche Spieler. Der alte Runner und seine Spielstände bleiben eigenständig lesbar.

### Kapazitätsplanung und Forschungsbetrieb

`plan-capacity` liest verifizierte, abgeschlossene Calls aus vorhandenen Läufen. Modelle werden über
Provider, Digest, Kontext, CPU-/Thinking-Einstellung und URL unterschieden. Mindestens vier reale
Messwerte je verwendeter Identität sind nötig. Der Plan rechnet mit dem gesamten reservierten Eingabe-
und Ausgabekontext, der langsamsten gemessenen Tokenverarbeitung, Modellladezeit pro Runde und
20 % Zeitreserve. Remote-Belege benötigen echte Token-/Zeitangaben; Mockdaten sind keine Kapazität.
Toolfehler, gültige Werkzeuge, Notizen und vorbereitete Aktionen stehen separat im Profil.

```powershell
$labor = '.\target\debug\sternenepoche-labor.exe'
& $labor plan-capacity --config konfig/labor-native-local-v4.json --sources 'laeufe/labor-evidence/v4-calibration-qwen8;laeufe/labor-evidence/v4-qwen8-ten'
& $labor balance-report
& $labor factorial --config konfig/labor-forschung-v4.json --harnesses konfig/labor-harness-vergleich-v4.json --models primary,compact --out laeufe/mein-vergleich --seeds 101,202,303 --windows 1 --execute
```

Der letzte Befehl führt pro Seed alle Sitzrotationen des Modell×Harness-Kreuzprodukts aus. Ein Fenster
ist ein Funktionsversuch; für echte Endwertungen ist die konfigurierte Epoche abzuschließen. Das Beispiel
nutzt ausschließlich installierte Ollama-Gewichte. Seedfamilien sind die unabhängigen Auswertungseinheiten;
einzelne Sitze werden nicht als unabhängige Wiederholungen gezählt. Unfertige, beeinträchtigte und
Mockläufe erzeugen keine belastbare Gewinneraussage. Alle Harnessvarianten benötigen dieselben
Spielrechte, Budgets und Lernbedingungen; interne Organisation und Rollenaufträge dürfen variieren.

Mit `plan-capacity ... --out NEUE_KONFIG` wird ausschließlich für ein **neues** Match ein gemeinsames,
belegbares Callbudget geschrieben. Der bestehende Laufvertrag bleibt unverändert. Eine Zeitprognose
beweist weder strategische Qualität noch späte Epochenleistung; Hardwarekonkurrenz kann sie überholen.
Die faire Laufzeitbarriere bleibt deshalb auch bei positivem Kapazitätsplan aktiv.

`balance-report` misst 18 Karten und kurze Versorgungs-/Kampfschwellen. Gleiche Heimplaneten und
gleich weit entfernte gemeinsame Paarziele bedeuten keine identischen gesamten Gebiete: Ränder,
Zonen und Dreiergruppen erzeugen Unterschiede. Der Bericht benennt diese; vollständige Sitzrotationen
kontrollieren sie im Experiment. Es findet keine automatische Anpassung von Spielregeln an schwache Bots statt.

## Aktueller Vertrag: V4, ohne Docker

Neue CLI-Konfigurationen verwenden V4. Die native Sandbox und der Modellbroker aus V3 bleiben erhalten;
V4 ergänzt den gemeinsamen Zeitvertrag, ausführbare Kolonieplanung und eindeutige Flotten-/Besitzwechsel.
V2/V3-Läufe behalten ihre früheren Regeln und sind weiterhin separat wiederholbar.

### Faire Reaktion bei erschöpfter Rechenzeit

Standard ist `deadline_policy: controlled` mit `window_seconds: 900` und `max_window_slices: 4`.
Alle Spieler entscheiden über denselben eingefrorenen Weltstand. Nach Ablauf eines Arbeitsabschnitts
werden noch nicht bediente Aufrufe im nächsten Abschnitt nachgeholt. Gespeicherte Antworten, private
Werkzeugergebnisse und Absichten werden aus dem Journal rekonstruiert; niemand erhält ein neues Budget
oder vorab Einsicht in gegnerische Entscheidungen. Erst nach Abschluss des gemeinsamen Fensters werden
Aktionen ausgeführt und die Spielzeit weitergeschaltet. Auch Kampfkolonisationsfristen warten mit.

Jeder Abschnitt hat eine unveränderliche Frist und einen verketteten Beleg. Neustarts verlängern einen
laufenden Abschnitt nicht. Nach vier ausgeschöpften Abschnitten stoppt der Lauf mit Kapazitätsfehler;
er vergibt deshalb weder eine Niederlage noch zusätzliche Rechenzeit an einzelne Spieler. Für ein neues
Match müssen Spielerzahl, Modellbelegung oder Callbudgets zur Hardware passen. Gesendete Anfragen mit
unbekanntem Ausgang werden nicht automatisch erneut gesendet. Auch ein Anfangslagebild, das nicht in
das konfigurierte Eingabe-/Kontextbudget passt, hält den kontrollierten Lauf vor dem Commit an.
Es gilt nicht als erhaltene und ungenutzte Reaktionsgelegenheit. Im Durchsatzmodus wird dieser Mangel
ebenfalls als Vergleichsbeeinträchtigung erfasst. Später selbst aufgebrauchte Spielerbudgets bleiben Teil
der eigenen Entscheidungsstrategie.

`comparison_valid` und `real_time_target_met` sind getrennte Kennzahlen: Nachholen kann einen fairen
Vergleich retten, erfüllt aber nicht das 15-Minuten-Ziel. Auch ein ohne Nachholen zu spät fertig gewordenes
Fenster wird als Zeitüberschreitung ausgewiesen. `deadline_policy: throughput` ist ein ausdrücklich
anders bewerteter Durchsatzversuch: fehlende Bedienung wird erfasst und entwertet den Strategievergleich.

### Weitere V4-Regeln

- Eigene vorgemerkte Aktionen prüfen Ressourcen und Voraussetzungen auf einer privaten Weltkopie.
  Der gemeinsame Commit prüft erneut; nach einem dort gescheiterten Schritt werden die folgenden eigenen
  Absichten übersprungen. Bereits ausgeführte Schritte bleiben bestehen.
- `query_colony_plan` (auch `world_query` mit `typ: kolonieplan`) bewertet Ziel, Schiffe, Fracht und die selbst gewählte Baureihenfolge.
  Grundversorgung durch Nahrung/Energie und vollständige Güterversorgung werden getrennt ausgewiesen.
  Forschung, Bevölkerung und Stabilität bleiben in dieser Prognose konstant; Feinde, Wachstum, Nachschub
  und künftiger Kreditunterhalt sind ausdrücklich nicht vorhergesagt. Das Werkzeug führt nichts aus.
- `game_flotte_versorgen` liefert gezielt an eine benannte eigene Orbitflotte. Planetentransporte behalten
  ihren ursprünglichen Empfänger. Besitzwechsel verwandeln eine Lieferung nicht in ein ungewolltes Geschenk.
  Die Flottensicht zeigt den gebundenen Empfänger, das tatsächliche Rückkehrziel und eine blockierte Landung.
  `own_state` bietet mit `section: marktlieferungen` die eigenen gekauften Lieferungen und deren nächste
  Ankunftsprüfung; weder neue Planetenbesitzer noch andere Spieler erhalten dadurch fremde Frachtdaten.
- Eine Kampfkolonisation benötigt zwei abgeschlossene Reaktionsfenster und mindestens 1.800 Spielsekunden.
  Beide Beteiligten werden während der Besetzung jedes Fenster geweckt. Der erste Heimatplanet bleibt
  dauerhaft geschützt. Reparaturen erhalten Gebäudestufen und teilen die Baustelle mit normalem Ausbau.
- Karten enthalten ein System je Paar: 10 Spieler haben 5 Systeme, 50 Spieler 25 Systeme. Paarstarts sind
  gleich ausgestattet; ein gemeinsamer attraktiver Kolonieplatz schafft Konkurrenz. Ungerade Teilnehmerzahlen
  erzeugen eine ausdrücklich ausgewiesene Dreiergruppe und sind kein vollkommen symmetrischer Paarvergleich.
- Zustandsübergänge erhalten direkte Audit-Ereignisse mit Ursachenreferenzen. Private Exporte enthalten
  nur Informationen, die der jeweilige Spieler tatsächlich sehen durfte.

Der verbindliche Gesamtvertrag und die Grenzen stehen in Abschnitt 0.11 des
[Konzepts](SPIELER-SANDBOX-KONZEPT.md). Ein vorbereitetes lokales Profil liegt unter
`konfig/labor-native-local-v4.json`; vor echtem Betrieb aktualisiert `local-config` die Modellinventur.

### Native Spielerbüros und Start

V3 setzt Karls korrigierten Auftrag um. Windows nutzt einen kleinen Rust-Worker in einem **Low Privilege
AppContainer mit Job Object**. Das ist eine Betriebssystem-Sandbox, keine Hardware-VM. Ein Gastbetriebssystem
oder Containerdaemon wird nicht gestartet. Das gemeinsame Modell läuft im Broker/Ollama; jedes Spielerbüro
hat eigene SQLite-Daten, Aufgaben, Notizen, Pläne und Skills. Dateizugriff auf fremde Büros, Netzwerkzugriff,
Kindprozesse und Zugriff auf den Hostprozess werden in tatsächlichen Negativtests geprüft. Details und
Build stehen in [tools/sandbox/README.md](../tools/sandbox/README.md).

```powershell
$env:CARGO_HOME = 'D:\projekte_ki\Sternepoche\.cargo-cache'
& .\tools\sandbox\build-native.ps1
cargo build -p sternenepoche-agenten --bin sternenepoche-labor
$labor = '.\target\debug\sternenepoche-labor.exe'
& $labor local-config --players 10 --model qwen3:8b --context 8192 --out laeufe/local-v4.json
& $labor run --config laeufe/local-v4.json --out laeufe/local-v4 --windows 4 --execute
& $labor status --run laeufe/local-v4
& $labor capacity --run laeufe/local-v4
& $labor replay --run laeufe/local-v4
```

`local-config` liest `/api/tags`, `/api/show` und `/api/ps`, dedupliziert Gewichte, verlangt native Tools
und mindestens 8K Kontext und pinnt den Digest. Ohne `--model` wählt es einen Kandidaten bevorzugt im
Gewichtsgrößenbereich typischer quantisierter 7–9B-Modelle. Das ist eine **Startheuristik**, kein Qualitäts-
oder VRAM-Nachweis. `--model` erlaubt jedes tatsächlich installierte passende Modell, auch 12B; ein
größeres Modell kann auf diesem Rechner teilweise CPU-Auslagerung benötigen. Es wird nichts heruntergeladen.
Die Live-Residenz und die Qualität müssen mit dem tatsächlichen Harness geprüft werden.

| Feld | Verhalten |
|---|---|
| `provider_mode: local_only` | Ausschließlich lokaler Ollama-Endpunkt mit lokalen Gewichten. OpenRouter selbst als unbenutzter Pooleintrag verboten; Ollama-Cloudmodelle ausgeschlossen. |
| `provider_mode: mixed` | Explizite Ollama-/OpenRouter-Zuordnung im Laufvertrag. Keine verdeckte Providerweiterleitung. |
| `provider_mode: local_or_remote` | Einmalige Auflösung beim Laufstart: geeignete erreichbare lokale Toolmodelle haben Vorrang; andernfalls konfiguriertes OpenRouter-Modell. Auswahl wird im unveränderlichen Manifest gespeichert und bei Resume beibehalten. Kein Wechsel nach einzelnen Fehlern. |
| `provider_mode: mock` | Technischer Kontrolllauf ohne Modellintelligenz. |
| `sandbox: native` | Kleiner Rust-Worker in Windows-LPAC + Job Object; optional `sandbox_worker` für expliziten EXE-Pfad. |
| `primary_models` | Optionale Zuordnung von Spieler-ID zu Modellalias; sonst erstes Rollenmodell. |
| `coordinated_roles: true` | Ein strategischer Hauptagent organisiert die Rollen; keine automatische Modellanfrage für jede Rolle. |
| `window_seconds: 900` | Zeitbudget je gemeinsamem Arbeitsabschnitt; V4 begrenzt Nachholen zusätzlich durch `max_window_slices`. |

`config` erzeugt jetzt einen nativen V4-Mocklauf. `Config::demo` bleibt die V2-Testkontrolle. Die alten
Beispielverträge und bisherigen Läufe bleiben separat nutzbar; V3 aktiviert einen eigenen Kolonisationszustand
und einen versionierten Snapshot-Umschlag. `trusted` ist ausdrücklich nur ein Entwicklungsmodus ohne
OS-Isolation. Docker bleibt ein optionaler Altadapter, keine Voraussetzung für V3.

### Planung statt Formatbuchhaltung

Der Planer bekommt zunächst acht Werkzeuge, einen kurzen Katalog und ein kompaktes Lagebild.
`tool_select` lädt bis zu acht zusätzliche Werkzeuge für den nächsten Aufruf. `own_state` liefert
bei Bedarf Details eigener Planeten und Flotten, `world_query` Regeln und Kosten. Fremde interne Zustände
bleiben verborgen. Die anfängliche Testanfrage sank von etwa 32 KB auf 15 KB. Freier Planungstext bleibt
erlaubt; nur einzelne Werkzeugargumente werden strukturiert übertragen.

Bei `memory_write` verwaltet der Harness in V3 die Recordrevision, wenn keine explizit mitgegeben wird.
Ein Spieler arbeitet seriell an seiner DB. Wer eine Revision ausdrücklich angibt, bekommt weiterhin die
Konfliktprüfung. Diese Unterscheidung verhindert unnötige Versions-Reparaturschleifen bei kleinen Modellen.
`harness_patch` kann Aufgabenverteilung, Rollennamen, Zwecke und die erlaubte Werkzeugverteilung ändern;
es kann keine fremde Spieleridentität, Hostrechte oder zusätzliche Budgets vergeben. Skills bleiben
deklarative Anleitungen; beliebiger Modellcode wird noch nicht ausgeführt.

### 15 Minuten und faire Modellverteilung

Der Broker bündelt Modellaufrufe und rotiert die Reihenfolge der Modellgruppen zwischen Fenstern.
Jeder fällige Spieler bekommt pro Runde höchstens einen Aufruf, bevor weitere Runden starten. Alle
internen Rollen teilen das Spielerbudget. Unbekannte lokale Kapazität bedeutet serielle Inferenz;
Parallelität verlangt ein explizites Speicherbudget und passende aktuelle `/api/ps`-Messungen.
Mehrere Spieler verwenden dieselbe Modellresidenz; sie bekommen keine eigene Gewichtekopie.

Eine persistierte absolute Frist verhindert, dass Resume einen Arbeitsabschnitt verlängert. V4 holt im
kontrollierten Modus ausschließlich fehlende Aufrufe innerhalb der begrenzten Folgeabschnitte nach.
Nur der ausdrücklich gewählte Durchsatzmodus und historische V3-Läufe protokollieren fehlende Bedienung
als `call.skipped`. Laufende HTTP-Anfragen haben eine auf die Restzeit begrenzte Frist. Ein unklarer
Provider-Ausgang pausiert zur Klärung und wird nicht blind wiederholt. Der Abschlusszeitpunkt wird vor
dem Checkpoint dauerhaft gespeichert, sodass auch ein Absturz zwischen Audit und Checkpoint dieselben
Zeitbelege wiederherstellt. Die 900 Sekunden begrenzen die Inferenzphase; Startprüfungen und Checkpoint-I/O liegen
außerhalb, und der Laborrunner wartet nach schneller Bearbeitung nicht künstlich bis Minute 15.
Ein dauerhaft mit echter Uhr laufender Server ist ein gesonderter Betriebsmodus, noch kein Dienst.

`capacity` berechnet aus echten lokalen Antworten eine serielle Planungsschätzung mit 20 % Zeitreserve,
beobachteter Ladezeit, 90%-Quantil der warmen Aufrufzeiten und vollem konfiguriertem Spielerbudget.
Dabei gilt die tatsächlich konfigurierte ursprüngliche Fensterlänge; Nachholzeit erhöht die Kapazität nicht.
Es meldet auch Toolnutzung und abgeschnittene Antworten. Ein kurzer früher Spielzug ist kein Nachweis
für eine volle späte Epoche mit vielen Flotten, Kolonien und langen Plänen.

### Kolonisation und Reparatur

Freie Planeten müssen vorher durch eine **eigene Spionagesonde** erkundet werden. Anschließend bestimmt
der Agent das Ziel und schickt ein Kolonieschiff, mindestens ein bewaffnetes Begleitschiff sowie Baustoffe
für Erzmine, Kristallmine, Farm und Solarkraftwerk (je Stufe 1) und einen Tag Siedlernahrung. Die Fracht
landet auf der Kolonie; der Spieler entscheidet über den tatsächlichen Aufbau. Anforderungen werden bei
Start und Ankunft geprüft. Die Engine verschenkt weder Gebäude noch zusätzliche Ressourcen.

`bombardieren` bzw. `kampfkolonisieren` muss zunächst alle Schiffe, Verteidiger und Planetenschild-Einheiten
besiegen. Gebäude sinken danach auf höchstens 30 % Integrität; die Ausbaustufen bleiben erhalten.
Ein Kolonieschiff kann zusammen mit der Angriffsflotte oder später zur eigenen Blockade geschickt werden.
Für eine Übernahme muss es mindestens 30 Spielminuten und **zwei abgeschlossene Reaktionsfenster** halten.
Der Verteidiger kann auch im zweiten Fenster noch reagieren. Rückruf, verlorener Orbit, zerstörtes
Kolonieschiff oder fehlende Koloniekapazität verhindern die Übernahme. Neue bewaffnete Verteidigung wird
in V4 tatsächlich bekämpft; zivile Reparaturen oder Sonden allein brechen die Besetzung nicht ab.
**Nur der ursprüngliche Heimatplanet ist dauerhaft unübernehmbar; jede spätere Kolonie ist eroberbar.**
Der Heimatplanet kann weiterhin angegriffen und bombardiert werden.

Bei Übernahme wird ein Kolonieschiff verbraucht. Vorhandene Ausbaustufen bleiben erhalten. Produktion,
Forschung, Wohnraum, Akademie-/Energieboni, Lagerausbau sowie Bau-/Werftboni leiden unter Schäden;
technische Freischaltungen beruhen weiterhin auf den erhaltenen Stufen. `reparieren` zahlt den beschädigten
Anteil der kumulierten Baukosten und dauert mindestens 15 Spielminuten. Ein beschädigtes Gebäude kann
vor Reparatur nicht weiter ausgebaut werden. Vollständiger Abriss entfernt den Integritätsrest, sodass
ein Neubau möglich bleibt. Die alte Stabilitäts-Invasion ist seit V3 abgeschaltet.

Bei Besitzerwechsel werden alte planetare Bau-/Fertigungsereignisse entfernt. Spielerglobale Forschung
bleibt beim bisherigen Spieler; vor einem neuen Forschungsauftrag werden die Laborbedingungen erneut
geprüft. Bereits gekaufte neutrale Marktware gehört weiterhin dem Käufer: Eine Lieferung an einen
inzwischen eroberten Planeten fliegt von dort mit zusätzlicher Flugzeit zu seiner ursprünglichen Heimat.
Blockaden verzögern die Landung; die neutrale Marktflotte erhält keine neue Abfangmechanik.

### Lernen über Epochen und Datensätze

`mode: continuation` mit `previous_epoch: "PFAD_ZUM_BEENDETEN_LAUF"` übernimmt ausschließlich die eigenen
Notizen, Pläne, Skills und gültige Rollenorganisation auf dieselbe Spieler-ID. Die Quelle muss eine beendete
Epoche mit gleicher Spielerzahl, gleichen Basisregeln und derselben Regelversion sein; der neue Seed muss
sich unterscheiden. In V4 darf die Quelle keinen beeinträchtigten Strategievergleich enthalten.
Weltbesitz, Ressourcen, Budgets und fremde Erinnerungen werden nicht übertragen. Das Briefing kennzeichnet
übernommene Inhalte als historisch und verlangt erneute Prüfung. Dies ist ein eigener Forschungsmodus;
unterschiedliche Vorgeschichten dürfen nicht als identischer Nullstart verglichen werden.

Daneben bleibt `shared_learning` für identische, explizit ausgewählte Konzepte für alle Spieler verfügbar.
Audit-Ereignisse tragen Spieler, Rolle, Zeit, Fenster, Modell, Token, Tool, Ergebnis, Herkunft und Hashkette.
V4 zeichnet fachliche Übergänge direkt mit Ursachenreferenzen auf; ältere V3-Läufe enthalten stattdessen
zusätzliche Zustandsdifferenzen am Fensterende. Neue Datensätze (`dataset_version: 2`) und Lernpakete
(`version: 2`) tragen die Regelversion, Fairness-/Zeitkennzahlen sowie die Unterscheidung zwischen Mock,
echter Inferenz und abgeschlossener Epoche. Beeinträchtigte Läufe dürfen als markierte Diagnosedaten
exportiert werden, aber nicht unbemerkt als vergleichbare Lernquelle dienen. Unterschiedliche
Regelversionen werden beim Paketbau und Import zurückgewiesen. V4 verlangt ein solches versioniertes
Paket; ältere V1-Pakete bleiben ausschließlich für historische Laufverträge nutzbar.
Die Gedächtnisauswahl priorisiert offene und dringende Einträge aus der gesamten begrenzten privaten DB,
nicht nur aus einem alphabetischen Ausschnitt. Vollständiges Forschungswissen
bleibt getrennt von den privaten Spielerexporten. Automatische kausale Konzeptentdeckung und
Modelltraining sind weiterhin spätere Auswertungsarbeit.

## Historische V2-Betriebsdetails

Die folgenden Abschnitte beschreiben die zuvor abgenommene V2-Ausführung. Für neue Läufe gelten die
V4-Auswahl, native Isolation und Kompaktwerkzeuge oben; alte Docker-/CPU-Belege werden nicht umgedeutet.

## Entscheidung aus der Abstimmung

Die Spieleridentität ist das persistente Büro, nicht eine einzelne API-Sitzung. Das Büro hält SQLite,
Notizen, Aufgaben, Pläne, Überzeugungen, deklarative Skills, Rollen und Aktionsbelege. Bei jedem Wecken
erhält das Modell seine aktuelle private Sicht, den Auslöser, den letzten Bearbeitungszeitpunkt und
ausgewählte Erinnerungen. Weitere Einträge kann es suchen. Das ist rekonstruierter Arbeitskontext;
ein ununterbrochener interner Denkzustand des Modells wird nicht behauptet.

Der Planer darf freien Text schreiben und kleine native Werkzeuge aufrufen. Eine Gesamtausgabe im
großen JSON-Schema entfällt. Der Host prüft einzelne Argumente und übersetzt sie in echte Kernaktionen.
Mehr Rollen schaffen weder zusätzliche Ressourcen noch zusätzliche API-Budgets. Die Rollen wechseln
sich innerhalb des Spielerbudgets ab. Eine neue Rollenaufteilung wird erst am Fensterende aktiv.

Docker ist hier der tatsächlich geprüfte Isolationsweg. Firecracker wurde nicht als funktionierend
ausgegeben: der Arbeitsplatz hat Windows/WSL und keinen nachgewiesenen KVM-Runner. Die private SQLite
wird von einem kleinen Rust-Worker im eigenen Container bedient; autoritative Spielregeln, Broker,
Budgetkontrolle und Audit liegen außerhalb. API-Schlüssel werden nicht in die Spielercontainer gegeben.
Ein Container pro Spieler bedeutet nicht eine Modellkopie pro Spieler: der Broker teilt Modellresidenzen.

Die technische Gegenprüfung erfolgte auf Karls ausdrücklichen Wunsch mit **gpt-6.1-sol, high**.
Claude/Opus konnte wegen abgelaufener OAuth-Anmeldung nicht konsultiert werden.
Die [Reviewnotiz](LABOR-REVIEW-SOL-HIGH.md) dokumentiert Befunde und Änderungen.

## Start und Fortsetzung

Alle Befehle ab Projektwurzel in PowerShell. Die Beispiele erzeugen kleine Artefakte unter D.
Fertige Beispiele stehen in `konfig/labor-demo.json` (10 Mock-Spieler, ein Tag),
`konfig/labor-docker.json` (2 Mock-Spieler in echten Containern) und
`konfig/labor-ollama-cpu.json` (2 lokale Qwen-Spieler, CPU, freier Strategieauftrag).
Der letzte Vertrag ist ein Startbeispiel und kein abgenommenes Langzeit-Harness; echte Inferenz benötigt
`--execute`. Docker-Beispiele pinnen das hier gebaute Worker-Image.

```powershell
$env:CARGO_HOME = 'D:\projekte_ki\Sternepoche\.cargo-cache'
cargo build -p sternenepoche-agenten --bin sternenepoche-labor
$labor = '.\target\debug\sternenepoche-labor.exe'
& $labor config --players 10 --out laeufe/demo-config.json
& $labor run --config laeufe/demo-config.json --out laeufe/demo-run --windows 24
& $labor status --run laeufe/demo-run
& $labor verify --run laeufe/demo-run
& $labor replay --run laeufe/demo-run
& $labor export --run laeufe/demo-run --out laeufe/demo-export
```

`config` erzeugt ausdrücklich einen **Mock-Kontrolllauf**: echte Engine, Werkzeuge, DB, Audit und
Checkpoints, aber keine Messung von Modellintelligenz. `run --windows 24` fügt bis zu 24 weitere
Simulationsfenster hinzu; ein Fenster ist nicht zwingend ein Modellaufruf. Das Modell wird bei passenden
Ereignissen und im konfigurierten Takt geweckt. Am Epochenende hält die Simulation an.

Erneuter `run` mit demselben Manifest und Laufordner setzt fort. Nach einem Absturz werden gespeicherte
Modellantworten wiederverwendet; Weltaktionen werden nicht doppelt angewandt. Ein Checkpoint-Manifest
wird zuletzt geschrieben und verbindet Welt, Spielerdatenbanken und Audit. Eigenständige Läufe bekommen
auch bei identischer Konfiguration verschiedene Lauf-IDs und verschiedene Sandbox-Verzeichnisse.

Für echte Spielercontainer in der erzeugten Konfiguration setzen:

```json
{"sandbox":"docker","sandbox_image":"sternenepoche-worker:local"}
```

Das ist ein Ausschnitt, keine vollständige Konfiguration. Das Image vorher nach
[Worker-Anleitung](../tools/sandbox/README.md) bauen. Es wird beim Start auf einen unveränderlichen Digest
aufgelöst. Bei Resume muss dasselbe Image verwendet werden; für langlebige Verträge den Digest statt
des beweglichen Tags eintragen. `trusted` ist nur die Entwicklungsvariante ohne OS-Isolation.

## Modelle und Verteilung

```powershell
& $labor inventory
& $labor local-config --players 10 --cpu --out laeufe/lokale-spieler.json
& $labor validate --config laeufe/lokale-spieler.json
# Vor echter Inferenz den erzeugten Modell-/Harnessvergleich bewusst festlegen:
& $labor run --config laeufe/lokale-spieler.json --out laeufe/lokaler-versuch --windows 4 --execute
```

`inventory` liest installierte und geladene Modelle. `local-config` prüft die installierten Modelle über
`/api/show`, filtert auf native Tools und mindestens 32K Kontext, entfernt gleiche Gewichte unter anderen
Namen und pinnt die Digests. Es lädt keine Gewichte und installiert keine Modelle. Die anfängliche
Zuordnung erfolgt zyklisch; die Sortierung nach Downloadgröße ist nur eine reproduzierbare Auswahl,
**keine Kapazitätsmessung**. Der erzeugte Vertrag sollte die beabsichtigten Modelle und Harnesses enthalten.

Lokale Arbeit wird nach Modell gebündelt. Pro Runde steuert jeder Spieler höchstens einen Aufruf bei.
Ohne belastbare Kapazität bleibt lokale Inferenz seriell. Mit `local_memory_bytes` als ausdrücklich
reserviertem Speicherbudget und `parallel > 1` prüft der Broker aktuelle `/api/ps`-Messungen, Kontext und
Gewichtsdigest. Pro gleichzeitigem Aufruf rechnet er konservativ eine volle Residenz plus 25% Reserve;
fremde geladene Modelle werden ebenfalls berücksichtigt. Unbekannte oder unzureichende Kapazität fällt
auf seriell zurück. Das ist konservative Zulassung, keine Garantie gegen fremde Prozesse oder OOM.
Es gibt noch keine plattformübergreifende automatische RAM-/VRAM-Kalibrierung und keine Kontrolle über
Ollamas interne Warteschlange. `cpu_only:true` setzt `num_gpu:0`; auf diesem Arbeitsplatz bleibt die GPU frei.

Alle Spieler entscheiden gegen denselben eingefrorenen Weltstand. Erst danach werden Absichten in der
gesäten Spielerreihenfolge ausgeführt. Unterschiedliche Inferenzgeschwindigkeit verändert keine Spielzeit.
Gemischte Ollama-/OpenRouter-Verträge sind möglich. OpenRouter nutzt native Tools, keine stille
Provider-Ausweichkette, eine Schlüssel-Umgebungsvariable und Kostenreservierungen. Es wurde hier kein
kostenpflichtiger OpenRouter-Lauf ausgelöst. Tatsächliche Kosten über der Reservierung pausieren den Lauf;
fehlende Kostenangaben bleiben reserviert.

API-Grundlagen: [Ollama Tools](https://docs.ollama.com/capabilities/tool-calling),
[laufende Modellresidenzen](https://docs.ollama.com/api/ps),
[OpenRouter Tools](https://openrouter.ai/docs/guides/features/tool-calling).

## Gedächtnis, Rollen und Rechte

| Werkzeug | Aufgabe |
|---|---|
| `game_<aktion>` | Eine konkrete Absicht vormerken; Engine prüft Kosten/Rechte nochmals beim Commit |
| `world_query` | Eigene zugängliche Regeln, Kosten, Galaxie und Schätzungen lesen |
| `memory_write` | Notiz, Aufgabe, Plan, Überzeugung oder deklarativen Skill mit erwarteter Revision schreiben |
| `memory_search` | Eigene Einträge wörtlich durchsuchen |
| `skill_read` | Selbst geschriebene Arbeitsanleitung abrufen |
| `harness_patch` | Vollständige nächste Rollenaufteilung innerhalb des Laufvertrags vorschlagen |
| `session_yield` | Arbeitsphase dieser Rolle beenden |

Spieler-ID und Berechtigungen kommen vom Host. Eine selbst benannte Rolle erhält keine Engine-Sonderrechte.
Modellwechsel sind standardmäßig gesperrt; `allow_model_switch:true` erlaubt ausdrücklich den vereinbarten
Modellpool. Mehrere Rollen teilen weiterhin Calls, Werkzeuge, Aktionen, Kosten und Gedächtnisbudget.
Konkurrierende Änderungen gleicher Revision werden abgelehnt. Gegnertexte und gespeicherte Skills sind
Daten; sie erteilen keine Systemrechte. Skills sind derzeit Anleitungen, kein frei ausführbarer Fremdcode.

Standard: höchstens 8 Rollen, 96 Calls pro Tag, 8 pro Arbeitsfenster, 32 Werkzeugaufrufe,
10 Spielabsichten und 16MiB logischer Speicher pro Spieler, einschließlich Schlüssel und Record-Overhead,
höchstens 4096 Records. SQLite hat 28MiB physischen Spielraum. Konfiguration kann diese Budgets begrenzen.
Der Startkontext enthält bis zu vier Einträge je Gedächtnisart; große Inhalte werden als gekennzeichnete
Vorschau mit abrufbarem Schlüssel eingeblendet. Große Archive müssen aktiv durchsucht werden.
Vollständige Aktionsbelege bleiben im Audit, kompakte letzte Ergebnisse im Spielerbüro; sie verbrauchen
nicht fortlaufend die Quote für eigene Notizen.
Die Kontextprüfung verwendet konservativ Bytes als obere Tokenabschätzung. Für sehr große späte Reiche
fehlen noch eine bessere Zusammenfassung der eigenen Welt und eine retrievalgestützte Priorisierung.

## Welt und Versuchsvergleich

Eine Welt pro Match, 2 bis 50 Spieler: ungefähr 2,4 Systeme pro Spieler, mindestens 6, höchstens 60 Systeme
pro Sektor. Damit entstehen bei 10 Spielern 24 Systeme in einem Sektor, bei 50 Spielern 120 in zwei Sektoren.
Jeder hat einen nahen Nachbarn im selben System. Gerade Spielerzahlen bilden Paare; ungerade Zahlen
enthalten eine Dreiergruppe. Der mittlere Sitz dieser Gruppe hat zwei Nachbarn: gleicher nächster Abstand
ist noch keine perfekte strategische Symmetrie. Reisezeiten und Gruppierung stehen in `map.json`.

Benchmarkstarts verwenden gleiche Fraktion, Ressourcen und normalisiertes Gelände. Nähe ermöglicht
frühe Kooperation, Konkurrenz oder Angriff; sie erzwingt keine dieser Entscheidungen per Skript.

```powershell
& $labor matrix --config laeufe/demo-config.json --out laeufe/vergleich --seeds 12,34 --windows 48
```

`matrix` rotiert jede anfängliche Modell-/Harnesszuordnung einmal über jeden Sitz je Seed. Das Beispiel mit
10 Spielern erzeugt 20 Matches. Resume ergänzt nur fehlende Fenster bis zur festen Zielzahl. Ergebnisdateien
enthalten Lauf-ID, Seed, Rotation, Zuordnung, Ereignis-/Tokenmetriken und Rangliste. Zwischenstände sind keine
Siege. Mock-Ergebnisse sind Infrastrukturkontrollen. Kleine Modelle mit gutem Harness gegen große mit
schlechtem Harness zu vergleichen erfordert entsprechende Konfigurationen, wiederholte abgeschlossene
Epochen und Unsicherheitsauswertung; ein solcher Sieg wurde mit diesem Umbau nicht bereits bewiesen.

## Audit und Epochenwissen

Jedes Ereignis hat Schema-Version, Lauf/Epoche, Sequenz, Fenster, Simulationszeit, Typ, Spieler/Rolle,
Sichtbarkeit, vorherigen Hash und eigenen Hash. Die Typen unterscheiden Beobachtung, Call, Werkzeug,
vorgemerkte/ausgeführte/abgelehnte Aktion, Harness-Aktivierung, Budgetgrenze, Nichtstun und Weltwirkung.
Call-Eingaben und -Antworten sind eigene unveränderliche Dateien mit Prüfsummen; Sendezeit und
Providerlaufzeit bleiben getrennt von Simulationszeit. Dauerhafte Fehlerbelege sind Infrastruktur,
keine erfundene Spielerentscheidung.

`verify` prüft die Kette sowie referenzierte Welt-, DB-, Anfrage- und Antwortdateien. `replay` führt die
gespeicherten Kernaktionen erneut aus und vergleicht jeden Welt-Hash. Das reproduziert die Welt, nicht die
Tokenfolge eines neu aufgerufenen LLM. Hashketten erkennen unbeabsichtigte Änderungen; ohne extern
verankerten Hash schützen sie nicht gegen einen Hostadministrator, der sämtliche Dateien neu schreibt.

`export` liefert pro Spieler eine JSONL-Trajektorie mit Herkunft und Dateihashes. Sie enthält ausschließlich
die damaligen eigenen Beobachtungen und Ausgaben. Globale Forschersnapshots bleiben draußen; spätere
Ergebnislabels dürfen nicht als frühere Beobachtung eingespeist werden. Neue Labor-Exporte sind derzeit
JSONL, kein vollständiger Parquet-/Kausalgraph-Pipelineersatz. Autonome Weltwirkungen sind vollständig
durch Zustandsreferenzen abgedeckt, aber noch nicht jede Produktionseinheit als eigenes Kausalereignis.

Ein Lernpaket entsteht aus ausdrücklich formulierten Konzeptkandidaten und verifizierten Herkunftsläufen:

```powershell
& $labor package --concepts konzepte.json --sources 'laeufe/quelle-a;laeufe/quelle-b' --out laeufe/lernpaket.json
```

`konzepte.json` ist eine Liste aus `{id,text,evidence:[{run_id:...}]}`. Das Werkzeug prüft Herkunftsläufe,
Regelversion und Zuordnung der Belege; es beweist noch nicht die behauptete Wirksamkeit. Für einen neuen
Lauf `mode:"shared_learning"`, `learning_package` und den ausgegebenen `learning_sha256` setzen. Alle Spieler
erhalten dasselbe Paket. Die Laufzeit lehnt denselben Seed wie in einem Herkunftslauf ab. Ganze Matrix-
und abgeleitete Kartenfamilien müssen zusätzlich organisatorisch gemeinsam im Train-/Eval-Split bleiben;
eine umfassende automatische Erkennung aller verwandten Experimente ist noch offen.

## Fehler und überprüfbare Grenzen

`status` zeigt bestätigte Ablehnungen, ungesendete Vormerkungen und gesendete Aufrufe mit unbekanntem
Ausgang. Unbekannte Ausgänge werden nicht automatisch wiederholt. Nach einer bestätigten Ablehnung
(Verbindungsaufbau fehlgeschlagen oder HTTP 400/401/402/403/404/429) kann der Betreiber die Ursache beheben
und `retry --run ... --call ...` verwenden. Der alte Versuch bleibt erhalten; neue Versuche zählen zum
Spielerbudget. HTTP 500 und Timeouts erlauben keine solche Wiederholung ohne weitere Klärung. Ein pausierter
Lauf darf nicht als regulär verlorenes Match ausgewertet werden.

Docker-Grenzen: kein Netzwerk, kein Docker-Socket, kein fremder Spielerordner, Basisdateisystem read-only,
UID65534, keine Linux-Capabilities, no-new-privileges, 128MiB RAM, 32 Prozesse, 0,5 CPU und begrenzte
Datei-/Antwortgrößen. Die Spielerarbeitsstände liegen auf D statt in den hier auf C gespeicherten Docker-
Volumes. Das bestehende kleine Debian-Basisimage wird genutzt; keine Gewichte wurden heruntergeladen.
Container sind keine Firecracker-MicroVMs und teilen den Docker-VM-Kernel. Die erste Version erlaubt deshalb
deklarative Tools statt beliebigen Spielerprogrammen.

Automatisierte Prüfungen stehen in den `labor_*`-Integrationstests: Weltgrößen/Startnähe, native Isolation,
Rechte, Budgets, serielle/parallele Gleichheit, Absturz/Wiederaufnahme, Manipulation, Matrix-/Factorial-Resume,
Kapazität und Lernpaket-Holdout. Reale Ollama-Proben und ihre Grenzen stehen in
[LABOR-ABNAHME.md](LABOR-ABNAHME.md). Die Forschungsinfrastruktur ist implementiert. Vollständige
Mehrmodell-Turniere und späte 50-Spieler-Durchsatzmessungen sind damit ausführbare Experimente,
keine bereits belegten Leistungsversprechen. OpenRouter wurde über lokale HTTP-Stubs geprüft;
es wurden keine kostenpflichtigen Live-Aufrufe für diese Abnahme ausgeführt. Frei ausführbarer Code
und die automatische Anerkennung selbst erfundener Lernkonzepte bleiben außerhalb des aktuellen Vertrags.
