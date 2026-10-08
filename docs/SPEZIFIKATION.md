# Sternenepoche – fachliche Spielspezifikation

> **Aktuelles Onlineprofil, 8. Oktober 2026:** Für die gemeinsame Spielwelt gelten
> `regeln/online-v1.ron`, das beim Server gespeicherte aktive Profil und die daraus ausgelieferte
> Referenz unter `/api/rules`. Verbindlicher Onlinevertrag: [ONLINE-KONZEPT.md](ONLINE-KONZEPT.md),
> erzeugte Zahlen: [ONLINE-REGELN.md](ONLINE-REGELN.md). Die Onlinewelt hat 30 Skriptbots und 20
> mögliche Teilnehmerplätze, zunächst fünf freigegeben; Modelle halten ihre Uhr nicht an.
> Krisen, Rettungsfristen und dauerhaftes Ausscheiden werden einschließlich Neustarts gespeichert.
>
> **Nachfolgende Basis-/Legacy-Referenz, Regelversion V3:** Für neue Laborläufe gelten zusätzlich die tatsächlich implementierten
> [Kolonisations- und Reparaturregeln](LABOR-BACKEND.md#kolonisation-und-reparatur): eigene Sondenaufklärung,
> Eskorte und Startfracht; Kampfkolonisation bei ausgeschalteter Verteidigung und 30 % Gebäudeintegrität
> mit zwei Reaktionsfenstern. Nur der ursprüngliche Heimatplanet ist unübernehmbar.
> Die nachfolgende Stabilitäts-Invasion gilt ausschließlich für Legacy-Spielstände.

> **Ist-Regeln und Zielrichtung unterscheiden.** Dieses Dokument beschreibt den vorhandenen Spielkern.
> Die nächste Entwicklungsstufe richtet sich nach dem
> [Spieler-Sandbox-Konzept](SPIELER-SANDBOX-KONZEPT.md), insbesondere bei Spieleridentität, Rollen,
> Weltgröße, Startnachbarschaft und Forschungsdaten. Die folgende Quellenreihenfolge erklärt die aktuell
> ausgeführten Regeln; sie setzt neue Nutzeranforderungen nicht außer Kraft.

Stand: 4. Oktober 2026, Regelwerk `regeln/regelwerk.ron` in der Fassung `version: "0.1.0"`.

Diese Spezifikation beschreibt die Spielregeln von Sternenepoche so, wie der Spielkern sie umsetzt. Sie richtet sich an
alle, die Regeln ändern, Spieler anbinden, Skriptbots schreiben oder Läufe auswerten. Sie ersetzt weder das Regelwerk
noch den Regeltext der Modelle, sondern erklärt beide im Zusammenhang.

**Quellen und Vorrang.** Bei Widersprüchen gilt diese Reihenfolge:

1. `regeln/regelwerk.ron`: alle Zahlen des Spiels, kommentiert. Sein SHA-256 steht in jedem Protokoll.
2. Der Spielkern in `crates/kern/src/`: die Mechanik.
3. Der Regeltext, den die Modelle bekommen. Er wird in `crates/kern/src/regeltext.rs` aus dem Regelwerk erzeugt.
4. Das Originalkonzept `wissen/quellen/spielkonzept-original.txt`.
5. `README.md`, vor allem die Abschnitte „Was aus welchem Entwurf stammt“, „Befunde aus dem Bau“ und „Balance“.

Wo Regeltext und Kern verschieden formulieren, gilt der Kern; die gefundenen Stellen stehen in Anhang A.

**Verweise und Zahlen.** Codeverweise nennen Datei und Funktion. Dateinamen ohne Pfad liegen in `crates/kern/src/`,
etwa „wirtschaft.rs, `raten_neu`“. Zahlen stehen so wie im Regelwerk, bei Bedarf mit dem Feldnamen. Die vollständigen
Tabellen aller Gebäude-, Forschungs- und Einheitenwerte schreibt diese Spezifikation nicht ab; sie stehen in der
generierten Referenz `docs/REGELWERK.md`.

**Rechenweise.** Der Weltzustand enthält keine Gleitkommazahlen. Mengen sind Ganzzahlen in Tausendsteln einer Einheit,
Faktoren aus dem Regelwerk wirken als Festkomma mit sechs Nachkommastellen (typen.rs, `mal`, `pot`, `anteil`,
`wurzel`). Nach jeder Multiplikation oder Division wird auf Tausendstel abgeschnitten. Credits und Werteinheiten
(Abschnitt 9) werden im Verhältnis 1:1 verrechnet.

## 1. Zweck und Rahmen

### 1.1 Spielwelt und Ziel

Sternenepoche ist eine geschlossene Welt. Es gibt keinen Händler und keine Zahlquelle außerhalb der Spieler: Preise
entstehen nur aus Orders der Spieler, Marktgebühren verschwinden aus dem Spiel (README, „Was aus welchem Entwurf
stammt“). In der vorgesehenen Besetzung treten 50 Zivilisationen an. Eine Epoche dauert ein Spieljahr von 365
Spieltagen (`welt.epoche_tage: 365`). Es gewinnt, wer am Ende von Spieltag 365 die höchste Gesamtpunktzahl hat
(Abschnitt 9). Jede Zivilisation beginnt auf einer gleich ausgestatteten Heimatwelt (Abschnitt 3.4) und entwickelt
ihre Volkswirtschaft über vier Aufstiege von Stufe I bis Stufe V (Abschnitt 6.4). Die Welt entsteht aus einem
Startwert und einer Spielerzahl (welt.rs, `Welt::neu`); die Spielerzahl ist frei bis höchstens 30 je Sektor, also 60.

### 1.2 Wer spielt

Alle Spieler handeln über denselben Weg: Aktionen als JSON an `Welt::handeln` (aktion.rs). Der Kern prüft jede Aktion
vollständig, lehnt ungültige mit genauem Grund ab und protokolliert jede mit ihrem Ergebnis.

**Modelle in vier Rollen.** Ein KI-Reich (`ki: true`, welt.rs, `Spieler`) wird von vier Rollen regiert: Stratege,
Verwalter, Feldherr und Diplomat, jede gespielt von einem Sprachmodell, lokal oder über OpenRouter, auch gemischt
(README, „OpenRouter“). Zwei Orchestratoren fahren dieselbe Engine: `orchestrator/sternenepoche` (Python) und
`crates/agenten` (Rust). Für einen Modellvergleich steuert jedes Modell 12 oder 13 Reiche in allen Rollen
(`konfig/vergleich.toml`).

**Skriptbots.** Die Bots in `crates/lauf/src/bots.rs` spielen über dieselben Aktionen: Ökonom, Räuber, Igel, Händler
und der Kontrolltyp Wehrlos, ein Ökonom, der sich nie schützt. Bots sind keine KI-Reiche (`ki: false`) und handeln mit
der Sonderrolle `Rolle::Alle`, die Töpfe und Rollenrechte umgeht (typen.rs, `Rolle`), aber keine andere Spielregel.
Sie dienen der Balance und füllen Runden auf.

**Ein Mensch.** Die native Oberfläche `crates/spieler` (egui, ohne Browser) lässt einen Menschen ein Reich führen. Er
ist die ganze Regierung und handelt wie ein Bot mit `Rolle::Alle`; alle Regeln außer Töpfen und Rollenrechten gelten
(`crates/spieler/README.md`). Die anderen Reiche spielen Skriptbots. Die aktuelle Fassung von
`crates/spieler/src/lib.rs` kann zusätzlich einzelne Reiche von Modellen regieren lassen (`Session::with_models`).

### 1.3 Die vier Rollen

| Rolle | Regeltakt | frühester Wiederaufruf | Aktionen | bezahlt aus |
|---|---|---|---|---|
| Stratege | 24 h | 12 h | `doktrin`, `stufenaufstieg`, `meldung` | kein Topf (Aufstieg) |
| Verwalter | 4 h | 2 h | `bauen`, `abreissen`, `schleife_leeren`, `forschen`, `steuersatz`, `prioritaeten`, `markt_order`, `markt_storno`, `stufenaufstieg`, `fertigen` (nur zivile Schiffe und Bauteile), `flotte_senden` (nur Transport, Stationieren, Kolonisieren, Abbau, Recyceln), `flotte_zurueckrufen`, `raketen_bauen`, `meldung` | Topf wirtschaft (Bau, Markt, Treibstoff, zivile Schiffe, Bauteile), forschung (Forschung), militaer (Raketen) |
| Feldherr | 12 h | 6 h | `fertigen`, `flotte_senden` (alle Missionen), `flotte_zurueckrufen`, `verband_oeffnen`, `verband_beitreten`, `raketen_bauen`, `raketen_starten`, `meldung` | Topf militaer, bei sichtbarer feindlicher Flotte zusätzlich reserve |
| Diplomat | 24 h | 12 h | `nachricht`, `vertrag_anbieten`, `vertrag_annehmen`, `vertrag_ablehnen`, `vertrag_kuendigen`, `allianz_gruenden`, `allianz_einladen`, `allianz_beitreten`, `allianz_verlassen`, `schenken`, `meldung` | Credits direkt (Kautionen, Geschenke) |

Belege: Rechte in aktion.rs, `Aktion::zustaendig` und `erlaubte_typen`; Takt in `agenten.takt_stunden`; frühester
Wiederaufruf in regeln.rs, `AgentenRegel::frueheste_sekunden`; Töpfe in wirtschaft.rs, `topf_fuer`. Zivile Schiffe
sind kleiner und großer Transporter, Bergbauschiff, Recycler und Kolonieschiff (wirtschaft.rs, `fertigen`).

Der Stratege schreibt die Doktrin: höchstens 2.400 Zeichen Text (`doktrin_zeichen: 2400`) und die Anteile der vier
Töpfe, die zusammen 100 ergeben müssen (aktion.rs, `anwenden`). Alle Rollen sehen sie im Lagebild. Jede Rolle führt
ein Notizbuch von höchstens 6.000 Zeichen (`notiz_zeichen: 6000`), das sie bei jedem Aufruf ersetzt, und kann dem
Strategen eine Meldung von höchstens 400 Zeichen hinterlassen, die ihn weckt.

### 1.4 Ablauf eines Aufrufs

Ein Aufruf beginnt mit dem Lagebild der Rolle (sicht.rs, `sicht`; Abschnitt 10.3) und ihrem Regeltext (regeltext.rs,
`regeltext`). Die Antwort ist JSON nach einem Schema, das die Engine aus ihrer Aufzählung der Aktionen je Rolle
erzeugt (aktion.rs, `antwortschema`), mit den Feldern `begruendung` (höchstens 150 Wörter), `abfragen`, `aktionen`,
`prognose`, `notiz` und `wecker_stunden`. Abfragen sind nur lesend, höchstens 3 je Aufruf (`abfragen_je_aufruf: 3`):
`kosten`, `flugzeit`, `kampfsimulator`, `regel`, `galaxie` (sicht.rs, `werkzeug`). Höchstens 10 Aktionen gelten je
Aufruf (`aktionen_je_aufruf: 10`; `crates/lauf/src/main.rs`, Befehl `handeln`). Der Python-Orchestrator schickt nach
Abfragen eine zweite Runde mit den Ergebnissen und nach Ablehnungen genau eine Korrekturrunde
(`orchestrator/sternenepoche/lauf.py`, `_fenster`). Zum Abschluss setzt `aufruf_ende` (aktion.rs) Notizbuch und Wecker;
endgültig abgelehnte Aktionen erscheinen mit Grund im nächsten Lagebild. `prognose` dient der späteren
Kalibrierungsauswertung (README).

## 2. Zeitmodell

### 2.1 Spielzeit

Die Zeit zählt in Spielsekunden seit Beginn der Epoche (typen.rs, `SimZeit`). Ein Spieltag hat 86.400 Sekunden, die
Epoche 31.536.000 (regeln.rs, `epochenende`). Zeitangaben lauten etwa „Tag 12, 08:15“; Tag 1 beginnt bei Sekunde 0
(typen.rs, `zeittext`). Die reale Laufzeit hängt nur von Rechenzeit und Antwortzeiten der Modelle ab.

### 2.2 Ereignisse und ihre Reihenfolge

Diskrete Ereignisse liegen mit Zeitstempel in einer Prioritätswarteschlange (welt.rs, `Ereignis`, `plane`). Der
Schlüssel aus Zeit, Priorität und Laufnummer ist eindeutig; bei gleicher Zeit kommt die kleinere Priorität zuerst,
dann die kleinere Laufnummer.

| Priorität | Ereignis | ausgeführt in |
|---|---|---|
| 1 | Bau fertig | wirtschaft.rs, `bau_fertig` |
| 2 | Fertigung fertig, Raketen fertig | wirtschaft.rs, `fertigung_fertig`; erweiterung.rs, `raketen_fertig` |
| 3 | Flotte kehrt zurück | flotte.rs, `flotte_rueckkehr` |
| 4 | Orbitzeit endet (Halten, Abbau) | flotte.rs, `orbit_ende` |
| 5 | Flotte kommt an, Raketen kommen an | flotte.rs, `flotte_ankunft`; erweiterung.rs, `raketen_ankunft` |
| 6 | Marktlieferung | markt.rs, `marktlieferung` |
| 7 | Kündigungsfrist eines Vertrags endet | diplomatie.rs, `vertrag_ende` |
| 8 | Stundentick | wirtschaft.rs, `tick` |
| 9 | Tageswechsel | wirtschaft.rs, `tageswechsel` |

Stundentick und Tageswechsel sind selbst Ereignisse. Der erste Tick liegt bei Sekunde 3.600, der erste Tageswechsel
bei Sekunde 86.400; nach jeder Ausführung plant der Kern den nächsten (welt.rs, `Welt::neu`; sim.rs, `ausfuehren`).

### 2.3 Stundentick und Tageswechsel

Einmal pro Spielstunde rechnet der Kern in dieser Reihenfolge (wirtschaft.rs, `tick`): je Planet Bevölkerung
(`bevoelkerung_tick`), Stabilität (`stabilitaet_tick`), Steuern und alle Raten (`raten_neu`); je Spieler die Verteilung
des Stundeneinkommens auf die Töpfe, die Forschung (`forschung_tick`) und der Zähler für den Stufenaufstieg
(`stufen_tick`); je Planet der Start eines wartenden, jetzt bezahlbaren Bauauftrags (`bau_starten`); die
Eroberungsprüfung (flotte.rs, `eroberung_pruefen`); zuletzt die Punkte (wertung.rs, `punkte_neu`).

Einmal pro Spieltag (wirtschaft.rs, `tageswechsel`) zahlt jeder Spieler Unterhalt (Abschnitt 7.14), sein
Nachrichtenzähler wird zurückgesetzt, und Ereignisse und Meldungen, die älter als 7 Tage sind (`chronik_tage: 7`),
verlassen die Chronik. Danach laufen die Tribute (diplomatie.rs, `tribute_zahlen`), die Punkte werden neu gerechnet,
und je Spieler entsteht eine Zeile der Tageswerte (Stufe, Teilpunkte, Bevölkerung, Stabilität der Heimat, Kolonien,
Flottenwert, Produktion, Credits).

### 2.4 Ressourcen zwischen zwei Änderungen

Jeder Planet hat je Gut eine Rate pro Spielstunde, gültig seit der letzten Abrechnung. Vor jeder Änderung schreibt
`abrechnen` den Bestand fort (wirtschaft.rs): Bestand plus Rate mal vergangene Zeit. Bei positiver Rate endet die
Produktion an der Lagergrenze; der Überschuss verfällt und zählt in der Statistik als Lagerverlust. Bei negativer Rate
fällt der Bestand nie unter null. Neu berechnet werden die Raten nur bei Ticks und Ereignissen (`raten_neu`).
Lieferungen, Beute, heimkehrende Ladung, Tribute und Marktlieferungen kommen ohne Deckel ins Lager; solange ein Bestand
über der Grenze liegt, verfällt die laufende Produktion dieses Guts vollständig.

### 2.5 Entscheidungsfenster

Die Simulation rückt in Fenstern von 900 Spielsekunden vor (`welt.fenster_sekunden: 900`), also 15 Spielminuten; eine
Epoche hat 35.040 Fenster. Ein Schritt (sim.rs, `schritt`) führt alle Ereignisse bis zum Fensterende aus, rückt die
Uhr vor und bereitet das nächste Fenster vor (`fenster_vorbereiten`):

1. Die Reihenfolge aller Spieler wird neu ausgelost, aus einem eigenen Zufallsstrom je Fensternummer (welt.rs,
   `KANAL_REIHENFOLGE`, `mischen`).
2. Feindliche Flotten im Hinflug (Missionen `angriff`, `blockade`, `invasion`), deren Ankunft innerhalb der Warnzeit
   des Zielplaneten liegt, wecken sofort den Feldherrn des Ziels und jedes Verbündeten; jede Flotte nur einmal.
3. Eine leere Bauschleife weckt den Verwalter, einmal je Leerstand.
4. Die fälligen Rollen aller KI-Reiche werden in der ausgelosten Reihenfolge bestimmt.

Die Aktionen der fälligen Rollen wirken zum Zeitpunkt des Fensters, bevor die Uhr weiterläuft, in der ausgelosten
Reihenfolge der Spieler. Skriptbots ziehen ebenfalls in dieser Reihenfolge (`crates/lauf/src/main.rs`).

### 2.6 Wecken

Eine Rolle eines KI-Reichs ist fällig (sim.rs, `fenster_vorbereiten`), wenn ihr Regeltakt seit dem letzten Aufruf
vergangen ist (Stratege 24, Verwalter 4, Feldherr 12, Diplomat 24 Stunden; `agenten.takt_stunden`), wenn ihr Wecker
abgelaufen ist, oder wenn ein Auslöser vorliegt, der dringend ist oder nach Ablauf der frühesten Wiederkehr kommt.
Diese ist das Größere aus 1 Stunde (`mindestabstand_stunden: 1`) und dem halben Takt (`frueh_anteil: 0.5`): 12 Stunden
für Stratege und Diplomat, 2 für den Verwalter, 6 für den Feldherrn (regeln.rs, `frueheste_sekunden`). Einen früheren
Wecker verschiebt der Kern auf diese Zeit, mindestens auf 15 Minuten; höchstens liegt ein Wecker 30 Tage voraus
(aktion.rs, `aufruf_ende`). Beim ersten Fenster sind alle Rollen fällig (welt.rs, `Welt::neu`). Skriptbots und der
Mensch werden nicht geweckt.

| Auslöser | geweckte Rolle | dringend | Beleg |
|---|---|---|---|
| feindliche Flotte im Anflug auf eigenen oder verbündeten Planeten | Feldherr | ja | sim.rs, `fenster_vorbereiten` |
| Kampfbericht als Verteidiger | Feldherr, Stratege | nur Feldherr | flotte.rs, `gefecht` |
| Kampfbericht als Angreifer | Feldherr, Stratege | nein | flotte.rs, `gefecht` |
| Blockade oder Belagerung beginnt | Stratege | ja | flotte.rs, `kampf_am_planeten` |
| Kolonie verloren | Stratege | ja | flotte.rs, `erobern` |
| Interplanetarraketen im Anflug, Raketenschlag | Feldherr | ja | erweiterung.rs |
| Vertragsbruch durch den Partner | Diplomat, Stratege | nur Diplomat | diplomatie.rs, `brechen` |
| Bauschleife leer | Verwalter | nein | sim.rs |
| Stufenaufstieg möglich | Stratege, Verwalter | nein | wirtschaft.rs, `stufen_tick` |
| Stufe erreicht | Stratege | nein | wirtschaft.rs, `stufenaufstieg` |
| Kolonie gegründet oder erobert | Stratege, Verwalter | nein | flotte.rs, `kolonisieren`, `erobern` |
| Spionage bemerkt, Spionagebericht | Feldherr | nein | flotte.rs, `spionieren` |
| Raketen fertig | Feldherr | nein | erweiterung.rs, `raketen_fertig` |
| Nachricht, Vertragsangebot, Vertrag angenommen, gekündigt oder beendet, Geschenk, Lieferung, Allianzeinladung | Diplomat | nein | diplomatie.rs; flotte.rs |
| Meldung einer anderen Rolle | Stratege | nein | aktion.rs |
| Desertion | Stratege | nein | wirtschaft.rs, `desertion` |

### 2.7 Die Weltuhr steht während der Entscheidungen

Während die Modelle antworten, rechnet die Welt nicht. Interaktive Clients rücken mit `schritt_wenn_bereit` vor; der
Kern verweigert den Schritt, solange eine fällige Rolle ihren Aufruf nicht abgeschlossen hat (sim.rs). Latenz,
Ratenlimits und Ausfälle eines Anbieters verändern deshalb nur die Dauer eines Laufs, nie das Spiel (README,
„OpenRouter“).

### 2.8 Ende der Epoche

Das letzte Fenster endet genau bei Sekunde 31.536.000 (sim.rs, `schritt`). Dann werden die Punkte ein letztes Mal
gerechnet; danach lehnt der Kern jede Aktion ab (aktion.rs, `handeln`).

## 3. Galaxie

### 3.1 Aufbau und Koordinaten

Die Galaxie hat 2 Sektoren mit je 60 Systemen und je 12 Planetenplätzen (`sektoren: 2`, `systeme_je_sektor: 60`,
`plaetze_je_system: 12`), zusammen 1.440 Plätze. Koordinaten lauten Sektor:System:Position, etwa 1:27:7 (typen.rs,
`Koord`). Position 0 ist der Asteroidengürtel eines Systems; nur die Mission `abbau` fliegt dorthin, und sie fliegt nur
dorthin (flotte.rs, `flotte_senden`). Alle Plätze werden beim Start mit Zone und Feldzahl erzeugt; ein Planet mit
Besitzer entsteht erst durch Besiedlung (welt.rs, `Welt::neu`, `Platz`, `Planet`).

### 3.2 Zonen

| Zone | Positionen | Felder | Solar | Deuterium | Nahrung |
|---|---|---|---|---|---|
| Glut | 1 bis 3 | 80 bis 140 | ×1,4 | ×0,6 | ×0,7 |
| Leben | 4 bis 8 | 150 bis 230 | ×1,0 | ×1,0 | ×1,3 |
| Frost | 9 bis 12 | 100 bis 180 | ×0,7 | ×1,5 | ×0,6 |

Die Feldzahl eines Platzes ist gleichverteilt zwischen den Grenzen seiner Zone. Felder sind Bauplätze; jede
Gebäudestufe belegt ein Feld. Der Solarfaktor wirkt auf Solarkraftwerke, der Deuteriumfaktor auf
Deuteriumsynthesizer, der Nahrungsfaktor auf Farmen (wirtschaft.rs, `raten_neu`; flotte.rs, `kolonisieren`).

### 3.3 Systemreichtum, Nebel und Asteroidengürtel

Jedes System hat einen Reichtum an Erz und einen an Kristall, unabhängig voneinander gleichverteilt zwischen 0,7 und
1,4 (`reichtum_min`, `reichtum_max`) in Schritten von einem Tausendstel (welt.rs, `Welt::neu`). Er wirkt auf Erz- und
Kristallminen von Kolonien und auf den Abbau im Gürtel.

Nebelsysteme sind die Systeme 5, 15, 25 und so weiter (`nebel_abstand: 10`, `nebel_versatz: 5`), sechs je Sektor,
zwölf insgesamt. Nur dort darf ein Xenoextraktor stehen (wirtschaft.rs, `bauen`); Xenokristall gibt es sonst nur über
Markt und Beute. Asteroidengürtel liegen in den Systemen 3, 8, 13 und so weiter (`guertel_abstand: 5`,
`guertel_versatz: 3`), zwölf je Sektor. Kein System ist zugleich Nebel und Gürtel.

### 3.4 Heimatwelten und Startplätze

Jede Heimatwelt liegt auf Position 6 in der Lebenszone, hat 180 Felder und alle Faktoren ×1,0, auch für Erz und
Kristall; der Systemreichtum gilt für sie nicht (welt.rs, `Welt::neu`). Jeder Spieler beginnt mit 1.000 Einwohnern,
2.000 Credits, Stabilität 50, Steuersatz 10 Prozent, Grundwohnraum 1.200, einem Bestand von 2.000 Erz, 1.200 Kristall,
400 Deuterium und 800 Nahrung, je einer Stufe Erzmine, Kristallmine, Farm, Solarkraftwerk, Wohnblock und Lager, auf
Zivilisationsstufe I, mit 10 Tagen Anfängerschutz und den Topfanteilen 60 wirtschaft, 10 militaer, 15 forschung, 15
reserve (`welt.start_*`, `agenten.start_anteile`).

Bis 15 Spieler liegen alle in Sektor 1, damit sie sich begegnen; sonst werden sie abwechselnd auf die Sektoren
verteilt, bei 50 also 25 je Sektor. Ein Startsystem liegt nie in einem Nebel, und zwei Startsysteme im selben Sektor
liegen mindestens 2 Systemnummern auseinander (`start_abstand: 2`), mit mindestens einem System dazwischen (welt.rs,
`startsysteme`). Daraus folgt die Obergrenze von 30 Spielern je Sektor. Weil Nebel jedes zehnte System belegen, liegt
der nächste Nebel von jedem anderen System 1 bis 5 Systeme entfernt, die Entfernung dorthin also zwischen 2.795 und
3.175; der Unterschied zwischen zwei Spielern bleibt unter 20 Prozent. Die Völker werden per Los in möglichst gleich
große Gruppen verteilt, bei 50 Spielern 13, 13, 12 und 12. Beides prüft der Test `startplaetze_sind_fair`
(`crates/kern/tests/spiel.rs`). Die Spielernamen werden zufällig aus Silben erzeugt (welt.rs, `name_erzeugen`).

### 3.5 Entfernung und Flugzeit

Die Entfernung d (flotte.rs, `entfernung`) beträgt im gleichen System 1.000 + 5 × Positionsabstand, im gleichen Sektor
2.700 + 95 × Systemabstand und zwischen Sektoren 20.000 × Sektorabstand. Die Flugzeit in Spielsekunden ist
(flotte.rs, `flugdauer`; `zeitfaktor: 4`, `min_sekunden: 1200`):

```
T = max(1200, 4 × (10 + 350/σ × Wurzel(10 × d / v)))
```

Dabei ist v das Tempo des langsamsten Schiffs nach Antriebsforschung und σ die gewählte Geschwindigkeitsstufe von 0,1
bis 1,0; die Wurzel wird ganzzahlig gezogen. Beispiele mit einem kleinen Transporter (Tempo 10.000) ohne
Antriebsforschung:

| Strecke | d | σ | Flugzeit |
|---|---|---|---|
| Nachbarposition im selben System | 1.005 | 1,0 | 1.440 s = 24 min |
| Nachbarsystem | 2.795 | 1,0 | 2.376 s, knapp 40 min |
| Nachbarsystem | 2.795 | 0,5 | 4.716 s, gut 78 min |
| fünf Systeme weit | 3.175 | 1,0 | 2.532 s, gut 42 min |
| anderer Sektor | 20.000 | 1,0 | 6.300 s = 105 min |

Eine Spionagesonde (Tempo 100.000) braucht ins Nachbarsystem die Mindestflugzeit von 1.200 Sekunden. Der Test
`flugzeiten_wie_im_konzept` (`crates/kern/tests/spiel.rs`) prüft die Werte des Konzepts: ins Nachbarsystem rund 40
Minuten, in den anderen Sektor knapp zwei Stunden, kürzeste Flugzeit über der Fensterbreite.

### 3.6 Tempo, Treibstoff und Ladekapazität

Das Tempo eines Schiffs ist sein Grundtempo mal (1 + Bonus × Stufe seiner Antriebsforschung), mit Bonus 0,1 für
Verbrennungs-, 0,2 für Impuls- und 0,3 für Hyperraumantrieb (`flug.antrieb_bonus`; flotte.rs, `tempo`). Der Treibstoff
je Strecke in Deuterium ist 1 + Summe(Verbrauch × Anzahl) × d × σ² / 35.000, mal (1 − Ersparnis); jede Raumhafenstufe
des Startplaneten spart 4 Prozent, höchstens 40 (`treibstoff_teiler: 35000`, `raumhafen_ersparnis: 0.04`,
`raumhafen_ersparnis_max: 0.4`; flotte.rs, `flugplan`). Hin- und Rückflug werden beim Start bezahlt, bei den
Einwegmissionen `stationieren` und `kolonisieren` nur eine Strecke. Langsames Fliegen spart quadratisch. Ein kleiner
Transporter ins Nachbarsystem braucht rund 1,8 Deuterium je Strecke.

Die Ladekapazität einer Flotte ist die Summe der Ladung ihrer Schiffe mal Ladefaktor des Volks mal
(1 + 0,05 × Logistik) (`logistik_bonus: 0.05`).

### 3.7 Erkundung

Die Werte eines freien Platzes sieht ein Spieler erst nach einer Erkundung, der Mission `spionage` mit Sonden auf den
freien Platz. Sie liefert Feldzahl, Zone, Erz- und Kristallreichtum und ob das System ein Nebel ist (flotte.rs,
`spionieren`); das Ergebnis steht danach dauerhaft im Lagebild (`erkundet`).

## 4. Völker

Jedes Volk hat Faktoren auf Grundwerte; nicht genannte Faktoren sind 1,0 (regeln.rs, `VolkRegel`).

| Volk | Faktoren | wirkt in |
|---|---|---|
| Aurelianer | Ladekapazität ×1,2; Marktgebühr ×0,5; Waffen ×0,9; Steuereinnahmen ×1,1 | flotte.rs, `flugplan` und Recycling; markt.rs, `gebuehr`; kampf.rs, `Gruppe::neu`; wirtschaft.rs, `tick` |
| Krath | Waffen ×1,15; Werftbauzeit ×0,85; Forschung ×0,85; Plünderquote 0,5 statt 0,4 | kampf.rs, `Gruppe::neu`; wirtschaft.rs, `fertigungszeit` und `raten_neu`; flotte.rs, `pluendern` |
| Veyari | Bevölkerungswachstum ×1,1; Nahrung ×1,25; Panzerung ×0,85; Kolonieschiff ×0,75 | wirtschaft.rs, `bevoelkerung_tick` und `raten_neu`; kampf.rs, `Gruppe::neu`; regeln.rs, `kosten_einheit` |
| Syntheten | Forschung ×1,15; Energie ×1,1; Bevölkerungswachstum ×0,85; brauchen keine Nahrung | wirtschaft.rs, `raten_neu` und `bevoelkerung_tick` |

Der Waffenfaktor wirkt auf den Angriff von Schiffen und Verteidigung im Kampf, nicht auf Interplanetarraketen und
Bodentruppen; der Panzerungsfaktor auf die Struktur im Kampf und gegen Interplanetarraketen (erweiterung.rs,
`raketen_ankunft`). Der Kolonieschiff-Faktor der Veyari senkt Erz, Kristall, Deuterium, Legierung und Elektronik des
Kolonieschiffs, nicht Habitatmodule und Antriebskern. Der Energiefaktor der Syntheten erhöht die Stromerzeugung. Sie
verbrauchen keine Nahrung, dafür 25 Strom je 1.000 Einwohner und Stunde (`energie_je_1000_syntheten: 25.0`), und ihr
Versorgungsgrad ist der Energiefaktor des Planeten: Fehlt Strom, leidet die Bevölkerung wie bei Hunger (Test
`syntheten_hungern_ohne_strom`). Die Nahrungsbedingung der Stufe II entfällt für sie.

## 5. Wirtschaft

### 5.1 Güter und Lager

Es gibt zehn lagerbare Güter (typen.rs, `Gut`): die Rohstoffe Erz, Kristall, Deuterium und Nahrung; die verarbeiteten
Güter Legierung, Elektronik und Konsumgut; Xenokristall; die Bauteile Antriebskern und Habitatmodul. Außerhalb der
Lager stehen Credits (je Spieler, unbegrenzt), Forschungspunkte (fließen direkt in die Forschung) und Strom (nicht
lagerbar). Das Lager begrenzt jedes Gut je Planet einzeln (regeln.rs, `lagergrenze`), mit Lagerstufe L: Rohstoffe
8.000 × 1,6^L, verarbeitete Güter 1.500 × 1,6^L, Xenokristall und Bauteile 400 × 1,6^L (`lager_roh`,
`lager_verarbeitet`, `lager_selten`, `lager_faktor: 1.6`). Das Startlager der Stufe 1 fasst 12.800, 2.400 und 640.

### 5.2 Stufenformeln und Produktivität

Für eine Gebäudestufe n sind Ertrag, Arbeitskräfte-, Fachkräfte- und Strombedarf jeweils Grundwert × n × 1,1^n
(`ertragswachstum: 1.1`), die Kosten Grundkosten × Kostenfaktor^(n−1) (regeln.rs, `stufenwert`, `kosten_gebaeude`).
Die Kostenfaktoren liegen zwischen 1,4 (Wohnblock) und 2,0, bei den Großprojekten 1,0. Gebäude erreichen höchstens
Stufe 40 (`max_gebaeudestufe: 40`), Forschungen höchstens 30 (regeln.rs, `MAX_FORSCHUNG`). Kosten steigen also
exponentiell, Erträge nur leicht überlinear.

Die Produktivität eines Planeten ist 0,6 + 0,6 × Stabilität / 100 (`produktivitaet_basis`, `produktivitaet_spanne`),
also 0,6 bei Stabilität 0, 0,9 bei 50 und 1,2 bei 100. Sie wirkt auf Minen, Synthesizer, Farmen, Werke,
Xenoextraktoren und Labore, nicht auf Kraftwerke (wirtschaft.rs, `raten_neu`).

### 5.3 Strom

Strom ist nicht lagerbar (wirtschaft.rs, `raten_neu`). Solarkraftwerke liefern Ertrag × Besetzung × Solarfaktor × E.
Fusionskraftwerke liefern Ertrag × Besetzung × Deuteriumdeckung × E und verbrauchen je Stunde 6 × n × 1,1^n Deuterium
mal Besetzung (`fusion_deuterium: 6.0`); deckt der Bestand den Stundenbedarf nicht, sinken Leistung und Verbrauch
anteilig. Dabei ist E = (1 + 0,05 × Energietechnik) × Energiefaktor des Volks. Ein Versorgungsnetz erhöht die
Gesamterzeugung des Planeten um 25 Prozent. Der Verbrauch ist der Strombedarf aller Gebäude mal ihrer Besetzung, bei
den Syntheten zuzüglich der Bevölkerung. Reicht die Erzeugung nicht, laufen alle Verbraucher mit dem Faktor Erzeugung
geteilt durch Verbrauch.

### 5.4 Produktion und Verarbeitung

Die Leistung einer Anlage ist Ertrag × Besetzung × Produktivität × Energiefaktor (wirtschaft.rs, `raten_neu`). Erz- und
Kristallminen liefern Leistung × Erz- bzw. Kristallfaktor des Planeten, der Deuteriumsynthesizer Leistung ×
Deuteriumfaktor, die Farm Leistung × Nahrungsfaktor des Planeten × Nahrungsfaktor des Volks × (1 + 0,08 ×
Agrarwissenschaft), der Xenoextraktor Leistung × (1 + 0,1 × Xenomaterialkunde) ohne Planetenfaktor. Auf der Heimatwelt
sind alle Planetenfaktoren 1,0. Auf einer Kolonie sind Erz- und Kristallfaktor der Reichtum des Systems; Deuterium-,
Nahrungs- und Solarfaktor kommen aus der Zone (flotte.rs, `kolonisieren`).

Die Werke laufen in der Reihenfolge Gießerei, Elektronikwerk, Konsumgüterwerk, jedes nur so weit, wie Bestand plus
Stundenproduktion seiner Eingänge reichen. Rezepte je Einheit, jeweils plus Strom (`wirtschaft.rezepte`): Legierung aus
4 Erz, Elektronik aus 2 Kristall und 1 Erz, Konsumgut aus 1 Erz, 1 Kristall und 0,5 Deuterium. Werkstoffkunde erhöht
die Legierungsausbeute je Stufe um 5 Prozent, ohne mehr Erz zu verbrauchen. Bauteile fertigt die Orbitalwerft
(Abschnitt 7.2).

### 5.5 Arbeitskräfte, Fachkräfte und Prioritäten

60 Prozent der Einwohner arbeiten (`arbeitsquote: 0.6`). Fachkräfte gibt es 30 ohne Akademie
(`fachkraefte_basis: 30`), eine Akademie der Stufe n bildet weitere 60 × n × 1,1^n aus (`fachkraefte_je_akademie:
60.0`); nie gibt es mehr Fachkräfte als Arbeitskräfte, und Fachkräfte zählen zu den Arbeitskräften (wirtschaft.rs,
`raten_neu`).

Die Gebäude eines Planeten werden in einer Prioritätenfolge besetzt (wirtschaft.rs, `reihenfolge`): zuerst die Liste,
die der Verwalter mit `prioritaeten` für den Planeten gesetzt hat, dann Farm, Solarkraftwerk und Fusionskraftwerk, dann
alle übrigen in der festen Reihenfolge des Regelwerks. Jedes Gebäude bekommt, was nach den vorigen übrig ist; seine
Besetzung ist der kleinere Anteil aus erhaltenen Arbeits- und Fachkräften, unterbesetzte Gebäude laufen anteilig.
Automatisierung senkt den Arbeitskräftebedarf je Stufe um 4 Prozent, höchstens um 50 Prozent
(`automatisierung_bonus: 0.04`), den Fachkräftebedarf nicht. Fachkräfte brauchen Fusionskraftwerk, Elektronikwerk,
Xenoextraktor, Verwaltungszentrum, Labor, Nanofabrik, Orbitalwerft und Sensorphalanx. Bei Gebäuden, deren Wirkung
keine stündliche Produktion ist (etwa Lager, Bunker, Bauhof, Werft, Raumhafen, Akademie, Wohnblock), hängt die
Wirkung nur von der Stufe ab; ihr Arbeitsbedarf zählt dennoch in der Verteilung.

Schiffsbesatzungen und Siedler kommen aus der Bevölkerung und fehlen danach als Arbeitskräfte. Die Besatzung wird bei
der Bestellung eingezogen; mindestens 500 Einwohner müssen bleiben (wirtschaft.rs, `fertigen`). Verlorene Schiffe
kosten ihre Besatzung.

### 5.6 Bevölkerung

Die Bevölkerung P jedes Planeten wächst logistisch bis zum Wohnraum W, stündlich fortgeschrieben (wirtschaft.rs,
`bevoelkerung_tick`):

```
ΔP je Stunde = r/24 × P × (1 − P/W) × v × s,   r = 0,12 × Wachstumsfaktor des Volks
```

- Die Grundrate beträgt 12 Prozent je Tag (`wachstum_je_tag: 0.12`). Bei halb vollem Wohnraum, voller Versorgung und
  Stabilität ab 60 wächst die Bevölkerung um 6 Prozent je Tag.
- v ist die Versorgung, v = −0,5 + 1,5 × Deckung (`hunger_min: -0.5`): 1 bei voller Deckung, 0 bei einem Drittel,
  −0,5 ohne Nahrung. Bei den Syntheten ist die Deckung der Energiefaktor.
- s ist 1 ab Stabilität 60, fällt linear und ist 0 unter 40 (`wachstum_voll_ab: 60.0`, `wachstum_null_unter: 40.0`).
- Ist v negativ, schrumpft die Bevölkerung um mindestens r/24 × |v| × P je Stunde, unabhängig von W und s. Liegt P
  über W, schrumpft sie logistisch zurück, unabhängig von v und s. Ein Planet behält mindestens 100 Einwohner.

Je 1.000 Einwohner und Stunde verbraucht die Bevölkerung 40 Nahrung (`nahrung_je_1000: 40.0`; Syntheten keine) und 3
Konsumgüter (`konsum_je_1000: 3.0`). Die Deckung ist Bestand plus Stundenproduktion geteilt durch den Bedarf,
höchstens 1. Was die Produktion nicht deckt, nimmt die Bevölkerung aus dem Lager; bei Unterdeckung bleibt dort nichts
liegen, auch nicht für Habitatmodule (regeltext.rs, Abschnitt „Bevölkerung und Arbeit“). Die Konsumdeckung wirkt auf
die Stabilität und ist Bedingung für Stufe IV.

Der Wohnraum ist Grundwohnraum + 700 × n × 1,1^n für einen Wohnblock der Stufe n + 50.000 je Orbitalring
(wirtschaft.rs, `raten_neu`). Der Grundwohnraum beträgt auf der Heimatwelt 1.200 (`grundwohnraum: 1200`); auf einer
Kolonie werden die beiden Habitatmodule des Kolonieschiffs zum Grundwohnraum, 2 × 2.000 = 4.000 (`habitat_wohnraum:
2000`; flotte.rs, `kolonisieren`).

### 5.7 Stabilität

Jeder Planet hat eine Stabilität zwischen 0 und 100, die sich stündlich einem Zielwert nähert, Halbwertszeit 24
Spielstunden (`annaeherung_je_stunde: 0.02847`; wirtschaft.rs, `stabilitaet_tick`). Der Zielwert ist:

- 50 als Grundwert (`ziel_basis: 50.0`),
- plus bis zu 20 Punkte für gedeckte Konsumgüter, anteilig zur Deckung (`konsum_bonus: 20.0`),
- plus bis zu 10 Punkte für freien Wohnraum, voll ab 20 Prozent frei (`wohnraum_bonus: 10.0`, `wohnraum_frei_voll: 0.2`),
- plus 2 Punkte je Stufe Soziologie (`soziologie_je_stufe: 2.0`),
- minus 1 Punkt je Prozentpunkt Steuersatz über 15 (`steuer_frei_bis: 15`, `steuer_malus_je_punkt: 1.0`),
- minus 5 Punkte je Kolonie über der Verwaltungsgrenze (`kolonie_ueber_grenze: 5.0`),
- minus 5 Punkte je Plünderung, linear abklingend über 5 Tage, zusammen höchstens 10 (`pluenderung_malus: 5.0`,
  `pluenderung_tage: 5`, `pluenderung_malus_max: 10.0`),
- minus den Belagerungsmalus, der während einer Belagerung um 10 Punkte je Tag wächst und danach um 5 je Tag sinkt
  (`belagerung_je_tag: 10.0`, `belagerung_erholung_je_tag: 5.0`).

Der Zielwert wird auf 0 bis 100 begrenzt. Steuersatz und Kolonien wirken auf alle Planeten des Spielers. Kolonien sind
alle Planeten außer der Heimatwelt. Die Verwaltungsgrenze ist die Summe über alle eigenen Planeten von
Verwaltungszentrum-Stufe durch 2, abgerundet; ein Verwaltungszentrum trägt also eine Kolonie je zwei Stufen
(wirtschaft.rs, `verwaltungsgrenze`).

Unter 30 herrschen Unruhen (`unruhen_unter: 30.0`): Kein Bauauftrag beginnt, der laufende Bau ruht und seine
Fertigstellung verschiebt sich um die Dauer der Unruhen (sim.rs, `zeit_vorruecken`), Raketen können nicht bestellt
werden, und ein Bauauftrag, der sofort beginnen würde, wird mit Grund abgelehnt (wirtschaft.rs, `bauen`). Unter 15
kann eine belagerte Kolonie erobert werden (`uebernahme_unter: 15.0`; Abschnitt 7.10). Außerdem bestimmt die
Stabilität Wachstum und Produktivität.

### 5.8 Steuern und Credits

Der Steuersatz gilt für das ganze Reich und liegt zwischen 0 und 50 Prozent (aktion.rs, Zweig `Steuersatz`), zu
Beginn bei 10. Je Planet und Stunde bringt er Einwohner × Steuersatz (als Anteil) × 0,03 Credits mal Steuerfaktor des
Volks (`steuer_je_einwohner_stunde: 0.03`; wirtschaft.rs, `tick`); 10.000 Einwohner bei 10 Prozent bringen 30 Credits
je Stunde. Credits braucht man für Unterhalt (Abschnitt 7.14), Markt, Kautionen, Tribute und Geschenke. Gebäude,
Forschung und Einheiten kosten Güter, keine Credits. Credits zählen nicht zur Wertung.

### 5.9 Töpfe und Doktrin

Ein KI-Reich hat vier Töpfe: wirtschaft, militaer, forschung und reserve (typen.rs, `Topf`). Sie zählen Werteinheiten
(Abschnitt 9) und begrenzen, wie viel eine Rolle ausgeben darf; die Güter selbst liegen auf den Planeten und werden
dort bezahlt (wirtschaft.rs, `zahlbar`, `zahlen`).

- Das Einkommen einer Stunde ist der Wert aller positiven Nettoraten (Produktion minus Verbrauch) nach den
  Wertungsgewichten plus die Steuereinnahmen; es wird nach den Anteilen der Doktrin verteilt (wirtschaft.rs, `tick`).
- Startguthaben ist der Wert des Startbestands, 5.400 Werteinheiten, verteilt nach 60, 10, 15 und 15 Prozent, also
  3.240, 540, 810 und 810 (welt.rs, `Welt::neu`).
- Jede Ausgabe bucht den Wert der Güterkosten vom zuständigen Topf ab; reicht er nicht, wird die Aktion mit genauer
  Angabe abgelehnt. Wartende Bauaufträge prüfen den Topf erst beim Baubeginn.
- Bei einer sichtbaren feindlichen Flotte darf der Feldherr zusätzlich die Reserve nutzen; gebucht wird erst vom
  Militärtopf, der Rest von der Reserve (wirtschaft.rs, `topf_guthaben`, `topf_abbuchen`; Test
  `feldherr_zahlt_bei_warnung_auch_aus_der_reserve`).
- Eine Markt-Kauforder zieht die hinterlegten Credits zusätzlich vom Wirtschaftstopf ab; bei Storno kommen die Credits
  zurück, der Topf nicht (markt.rs, `markt_order`, `order_aufloesen`).
- Der Stufenaufstieg wird ohne Topf bezahlt. Skriptbots und der Mensch umgehen die Töpfe.

## 6. Gebäude, Forschung und Zivilisationsstufen

### 6.1 Gebäude

Es gibt 28 Gebäude (typen.rs, `Gebaeude`); Kosten, Faktoren und Bedarfe stehen in `docs/REGELWERK.md`. Nach Aufgabe,
mit der Zivilisationsstufe, ab der sie gebaut werden dürfen:

- Rohstoffe: Erzmine, Kristallmine, Deuteriumsynthesizer, Farm (I); Xenoextraktor (IV, nur in Nebelsystemen).
- Strom: Solarkraftwerk (I), Fusionskraftwerk (II).
- Verarbeitung: Gießerei, Elektronikwerk, Konsumgüterwerk (II).
- Lager und Schutz: Lager (I), Bunker (II).
- Gesellschaft und Verwaltung: Wohnblock (I), Akademie (II), Markt (III), Verwaltungszentrum (IV).
- Wissen und Bau: Labor (I), Bauhof (I), Nanofabrik (IV, braucht Bauhof 6).
- Raumfahrt: Raumhafen (III), Werft (III, braucht Raumhafen 1), Orbitalwerft (IV, braucht Werft 4).
- Militär: Sensorphalanx (III), Raketensilo (III), Kaserne (IV).
- Großprojekte: Orbitalring, Forschungsarchiv, Versorgungsnetz (V; Abschnitt 6.5).

Belegt sind die Summe aller Gebäudestufen plus die Aufträge in der Bauschleife; verfügbar sind die Felder des Planeten
plus 6 je Stufe Terraforming plus 50 je Orbitalring (wirtschaft.rs, `felder_gesamt`, `felder_belegt`).

### 6.2 Bauschleife und Abriss

Ein Bauauftrag (wirtschaft.rs, `bauen`) wird abgelehnt, wenn die Zivilisationsstufe nicht reicht, ein Xenoextraktor
außerhalb eines Nebels stehen soll, eine Voraussetzung fehlt, die Schleife schon 5 Aufträge hat, den laufenden
eingeschlossen (`warteschlange: 5`), ein Großprojekt schon steht oder bestellt ist, die Höchststufe erreicht ist oder
kein Feld frei ist. Ist die Schleife leer, prüft der Kern außerdem sofort Unruhen und Bezahlbarkeit und nennt den
Grund.

Je Planet läuft ein Bau gleichzeitig. Bezahlt wird beim Baubeginn: die Güter vom Planeten, der Wert aus dem Topf. Ist
der vorderste Auftrag nicht bezahlbar, wartet er und hält die Schleife an; der Kern versucht den Start bei jedem Tick
und nach jedem fertigen Bau erneut (`bau_starten`). Die Bauzeit wird beim Baubeginn festgelegt (`bauzeit`):

```
Bauzeit in Stunden = (Erz + Kristall + 2 × Legierung) / (2500 × (1 + Stufe Bauhof) × 2^Stufe Nanofabrik)
```

mindestens 60 Sekunden (`bauzeit_teiler: 2500`, `min_bauzeit_sekunden: 60`). Eine Erzmine der Stufe 1 (60 Erz, 15
Kristall) braucht 108 Sekunden. `schleife_leeren` entfernt alle noch nicht begonnenen Aufträge; sie waren nicht
bezahlt, es gibt nichts zu erstatten.

`abreissen` senkt ein Gebäude um eine Stufe und erstattet nichts (wirtschaft.rs, `abreissen`). Nicht möglich ist der
Abriss eines Gebäudes in der Bauschleife, eines Raketensilos unter die Zahl gelagerter und bestellter Raketen und eines
Orbitalrings, dessen Zusatzfelder belegt sind. Weil die Wertung nur Bestehendes zählt, kostet ein Abriss Punkte.

### 6.3 Forschung

Es gibt 17 Forschungen (typen.rs, `Forschung`). Geforscht wird reichsweit, ein Projekt gleichzeitig, bis zu 3 weitere
in der Schlange (`forschung_warteschlange: 3`; wirtschaft.rs, `forschen`). Ein Projekt braucht die Zivilisationsstufe
und eine Laborstufe auf dem gewählten Planeten, ohne Angabe auf der Heimatwelt, und kostet Güter von diesem Planeten
(Topf forschung) sowie Forschungspunkte. Kosten und Punkte der Stufe n sind Grundwert × Faktor^(n−1) (regeln.rs,
`kosten_forschung`, `fp_forschung`).

Forschungspunkte je Stunde sind die Summe über alle Labore aller Planeten von 10 × n × 1,1^n × Besetzung ×
Produktivität × Energiefaktor × Forschungsfaktor des Volks × (1 + 0,25 × Forschungsarchiv) (wirtschaft.rs,
`raten_neu`, `fp_rate`). Jeder Tick zieht die Punkte der Stunde ab; ist der Bedarf gedeckt, steigt die Stufe, und das
nächste Projekt der Schlange beginnt. Was dann nicht bezahlbar ist, fällt mit Meldung heraus (`forschung_tick`).

Die Wirkungen je Stufe laut Regelwerk: Energietechnik +5 Prozent Strom, Werkstoffkunde +5 Prozent Legierung,
Automatisierung −4 Prozent Arbeitskräfte (höchstens −50), Agrarwissenschaft +8 Prozent Nahrung, Soziologie +2 Punkte
Stabilitätsziel, Verbrennungs-, Impuls- und Hyperraumantrieb +10, +20 und +30 Prozent Tempo, Logistik +5 Prozent
Ladung, Computertechnik eine gleichzeitige Flotte mehr, Waffen-, Schild- und Panzertechnik je +10 Prozent,
Xenomaterialkunde +10 Prozent Xenokristall, Terraforming +6 Felder auf jedem eigenen Planeten. Astrophysik erlaubt
Kolonien (Abschnitt 7.5), Spionagetechnik verbessert Berichte und Abwehr (Abschnitt 7.11). Hyperraumantrieb kostet
Xenokristall.

### 6.4 Zivilisationsstufen

Jeder Aufstieg verlangt, dass alle Bedingungen 48 Spielstunden ohne Unterbrechung erfüllt sind
(`stufen_haltezeit_stunden: 48`). Der Kern prüft sie bei jedem Tick: erfüllt zählt eine Stunde weiter, verletzt setzt
den Zähler auf null (wirtschaft.rs, `stufen_tick`). Danach steigt das Reich mit der Aktion `stufenaufstieg` auf
(Stratege oder Verwalter); die Kosten kommen vom Bestand der Heimatwelt (`stufenaufstieg`). Gelesen werden die
Bedingungen so (`stufen_bedingungen`): Einwohner als Summe aller Planeten; Gebäudestufen als höchste Stufe auf
irgendeinem eigenen Planeten; „Nahrung und Energie im Plus“ auf allen Planeten (Nahrungsrate nicht negativ, außer bei
Syntheten, und Stromerzeugung mindestens so groß wie der Verbrauch); Stabilität und Konsumdeckung auf der Heimatwelt;
Kolonien als alle Planeten außer der Heimatwelt, auch eroberte.

| Stufe | Bedingungen | Kosten | Neu verfügbar (laut `ab_stufe` und Regelwerk) |
|---|---|---|---|
| I Gründung | Startzustand | – | Erzmine, Kristallmine, Deuteriumsynthesizer, Farm, Solarkraftwerk, Lager, Wohnblock, Labor, Bauhof; Energietechnik, Agrarwissenschaft |
| II Industrie | 2.000 Einwohner, Erzmine 5, Kristallmine 5, Nahrung und Energie im Plus | 2.000 Erz, 1.000 Kristall | Gießerei, Elektronikwerk, Konsumgüterwerk, Fusionskraftwerk, Akademie, Bunker; Raketenwerfer, Lasergeschütz; Werkstoffkunde, Automatisierung, Soziologie, Verbrennungsantrieb, Computertechnik, Waffen-, Schild- und Panzertechnik, Spionagetechnik |
| III Orbit | 8.000 Einwohner, Gießerei 3, Elektronikwerk 2, Energietechnik 3, Stabilität ab 55 | 10.000 Erz, 6.000 Kristall, 500 Legierung, 200 Elektronik | Raumhafen, Werft, Markt, Sensorphalanx, Raketensilo; Spionagesonde, kleiner und großer Transporter, leichter Jäger, Bergbauschiff; Ionengeschütz; Impulsantrieb, Astrophysik, Logistik |
| IV Sternenflug | 32.000 Einwohner, Werft 4, Impulsantrieb 3, Astrophysik 1, Stabilität ab 60, Konsumgüter zu 80 Prozent gedeckt | 40.000 Erz, 25.000 Kristall, 10.000 Deuterium, 2.000 Legierung, 1.000 Elektronik | Orbitalwerft, Kaserne, Verwaltungszentrum, Nanofabrik, Xenoextraktor; Kolonieschiff, Kreuzer, Recycler, Truppentransporter; Gaußkanone; Hyperraumantrieb, Xenomaterialkunde, Terraforming; Missionen Blockade und Invasion |
| V Imperium | 700.000 Einwohner, Hyperraumantrieb 1, 3 Kolonien | 200.000 Erz, 150.000 Kristall, 50.000 Deuterium, 10.000 Legierung, 5.000 Elektronik, 5.000 Xenokristall | Orbitalring, Forschungsarchiv, Versorgungsnetz; Schlachtschiff, Bomber, Zerstörer; Plasmawerfer, Planetenschild |

Wer auf Stufe III aufsteigt, verliert den Anfängerschutz (Abschnitt 7.15). Drei Kolonien verlangen Astrophysik 5
(Abschnitt 7.5). Xenokristall für Hyperraumantrieb und Stufe V gibt es nur aus Xenoextraktoren in Nebelsystemen, über
den Markt oder als Beute. Mit Skriptbots gemessen (24 Epochen, 50 Spieler) liegen die Medianzeiten bei Tag 13,6 bis
13,7 für Stufe II, 36,4 bis 36,5 für III, 61,4 bis 64,8 für IV und 160,8 bis 174,0 für V, in den Zielfenstern des
Konzepts (Tag 10 bis 20, 30 bis 50, 60 bis 110, 150 bis 250; README, „Stand“, „Balance“). Die Zielzeiten sind
Vorgaben für die Balance und stehen nicht im Regeltext der Modelle.

### 6.5 Großprojekte

Ab Stufe V gibt es drei Großprojekte: normale, bezahlte Bauaufträge mit Kostenfaktor 1,0, je Planet höchstens einmal
(wirtschaft.rs, `bauen`; Test `projects_are_stage_five_once_per_planet_and_ring_cannot_overflow_fields` in
erweiterung.rs). Der Orbitalring bringt +50 Felder und +50.000 Wohnraum (`orbitalring_felder: 50`,
`orbitalring_wohnraum: 50000`) und darf nicht abgerissen werden, solange seine Zusatzfelder belegt sind. Das
Forschungsarchiv erhöht die Forschungspunkte der Labore dieses Planeten um 25 Prozent (`archiv_fp_bonus: 0.25`), das
Versorgungsnetz die Stromerzeugung dieses Planeten um 25 Prozent (`versorgungsnetz_energie_bonus: 0.25`). Großprojekte
zählen wie alle Gebäude zum investierten Wert.

## 7. Militär

### 7.1 Schiffe und Verteidigung

Es gibt 12 Schiffstypen und 6 Verteidigungsanlagen (typen.rs, `Einheit`, `SCHIFFE`); alle Werte stehen in
`docs/REGELWERK.md`. Ab Stufe III gibt es Spionagesonde (Tempo 100.000, keine Besatzung, keine Ladung), kleinen und
großen Transporter (5.000 und 25.000 Ladung), leichten Jäger und Bergbauschiff. Ab Stufe IV kommen Kreuzer (Werft 4,
Schnellfeuer gegen leichte Jäger 6, Raketenwerfer 10, Sonden 5), Recycler (20.000 Ladung), Kolonieschiff (Werft 4,
braucht Orbitalwerft 1) und Truppentransporter (braucht Kaserne 1, 500 Besatzung, die als Bodentruppen zählen) hinzu,
ab Stufe V Schlachtschiff (Werft 6), Bomber (Werft 7, Schnellfeuer gegen Verteidigung) und Zerstörer (Werft 8, braucht
Orbitalwerft 3). Verteidigung: Raketenwerfer und Lasergeschütz (II), Ionengeschütz (III), Gaußkanone (IV),
Plasmawerfer und Planetenschild (V, höchstens einer je Planet, `max_anzahl: 1`). Verteidigung kostet weder Unterhalt
noch Besatzung und kann sich nicht bewegen; ihre Werte sind gegenüber dem ersten Regelwerk verdreifacht (README,
„Balance“).

### 7.2 Fertigung

Jeder Planet hat zwei Fertigungsschleifen mit je bis zu 5 Aufträgen über 1 bis 10.000 Stück (wirtschaft.rs,
`fertigen`): die Werft-Schleife für Schiffe und Verteidigung und die Orbitalwerft-Schleife für Bauteile. Schiffe
verlangen die angegebene Werftstufe auf dem Planeten; Verteidigung braucht keine Werft, nutzt aber deren Schleife.
Bezahlt wird bei der Bestellung für alle Stücke, die Besatzung wird zugleich eingezogen. Die Stücke werden einzeln
fertig (`fertigung_fertig`), jedes nach

```
Stunden je Stück = (Erz + Kristall + 2 × Legierung) / (2500 × (1 + Stufe Werft bzw. Orbitalwerft) × 2^Stufe Nanofabrik)
```

mal Werftzeitfaktor des Volks, mindestens 30 Sekunden (`fertigungszeit`). Bauteile fertigt nur ein Planet mit
Orbitalwerft: ein Antriebskern kostet 2.000 Erz, 1.000 Kristall, 200 Legierung und 100 Elektronik, ein Habitatmodul
1.500 Erz, 500 Kristall, 150 Legierung, 50 Elektronik und 100 Konsumgüter (`wirtschaft.bauteile`). Das Kolonieschiff
kostet 10.000 Erz, 20.000 Kristall, 10.000 Deuterium, 400 Legierung, 200 Elektronik, 2 Habitatmodule und 1
Antriebskern; die Bauteile werden aus dem Bestand genommen.

### 7.3 Flotten

Eine Flotte startet mit `flotte_senden` (flotte.rs). Der Start verlangt einen Raumhafen auf dem Startplaneten, die
Schiffe auf diesem Planeten, einen freien Flottenplatz (1 + Stufe Computertechnik; gezählt werden alle eigenen Flotten
im Flug und im Orbit; `flottenplaetze`) und eine Geschwindigkeit zwischen 0,1 und 1,0. Die Ladung darf die Kapazität
nicht übersteigen; der Bestand muss Ladung und Treibstoff decken, der Topf nur den Treibstoff. Schiffe und Ladung
verlassen den Planeten beim Start. Von einem blockierten Planeten startet nur ein Angriff auf die Blockadeflotte.

Rückruf (`flotte_zurueckrufen`): Im Hinflug kehrt die Flotte um und ist nach der bisher geflogenen Zeit zurück,
mindestens nach einer Minute. Im Orbit fliegt sie mit ihrer vollen Flugdauer heim; eine Abbauflotte nimmt mit, was sie
bis dahin gefördert hat. Im Rückflug ist kein Rückruf möglich. Bei der Rückkehr landen Schiffe, Ladung und Siedler auf
dem Startplaneten oder, falls dieser inzwischen einem anderen gehört, auf der Heimatwelt (`flotte_rueckkehr`).

### 7.4 Missionen

Die zehn Missionen sind `angriff`, `transport`, `stationieren`, `halten`, `spionage`, `kolonisieren`, `recyceln`,
`abbau`, `blockade` und `invasion` (typen.rs, `Mission`). Feindlich sind `angriff`, `blockade` und `invasion`; nur sie
lösen Warnungen aus. Der Verbandsangriff ist ein Zusammenschluss von Angriffen (Abschnitt 7.12).

| Mission | Rolle | Ziel und Bedingungen | Wirkung bei Ankunft |
|---|---|---|---|
| `angriff` | Feldherr | fremder Planet ohne Anfängerschutz; der eigene Planet nur bei fremder Blockade | Kampf, bei Sieg Plünderung, dann Rückflug. Hält eine fremde Flotte den Orbit, geht der Kampf gegen sie, ohne Plünderung |
| `transport` | Verwalter, Feldherr | beliebiger Planet | lädt ab und fliegt zurück; an fremde Spieler als Geschenk protokolliert; bei Blockade Umkehr mit Ladung |
| `stationieren` | Verwalter, Feldherr | anderer eigener Planet, nur eine Strecke Treibstoff | Schiffe und Ladung bleiben; ist das Ziel blockiert oder verloren, Rückflug |
| `halten` | Feldherr | Planet eines Verbündeten, 1 bis 168 Stunden (`halten_max_stunden: 168`) | bleibt im Orbit und verteidigt mit, danach Rückflug |
| `spionage` | Feldherr | nur Sonden, keine Ladung, kein eigener Planet | Bericht oder Erkundung (Abschnitt 7.11) |
| `kolonisieren` | Verwalter, Feldherr | freier Platz, Kolonieschiff, freie Kolonie, 3.000 Siedler | gründet eine Kolonie (Abschnitt 7.5) |
| `recyceln` | Verwalter, Feldherr | Koordinate mit Trümmerfeld, Recycler | sammelt bis zur Kapazität der Recycler, Rest bleibt liegen; Rückflug |
| `abbau` | Verwalter, Feldherr | Position 0 eines Gürtelsystems, Bergbauschiffe, 1 bis 48 Stunden | fördert im Orbit, Rückflug mit Ladung |
| `blockade` | Feldherr | fremder Planet, ab Stufe IV | Kampf, bei Sieg bleibt die Flotte im Orbit (Abschnitt 7.9) |
| `invasion` | Feldherr | fremde Kolonie, nie eine Heimatwelt, mit Truppentransportern, ab Stufe IV | wie Blockade, dazu Belagerung und Eroberung (Abschnitt 7.10) |

Verbündet sind zwei Spieler, wenn sie in derselben Allianz sind oder ein Verteidigungsbündnis in Kraft haben
(diplomatie.rs, `verbuendet`). Abbau fördert je Bergbauschiff und Stunde 150 Erz mal Erzreichtum und 100 Kristall mal
Kristallreichtum des Systems, begrenzt durch die freie Ladekapazität der ganzen Flotte (`abbau_erz_je_stunde: 150`,
`abbau_kristall_je_stunde: 100`; flotte.rs, `abbau_ertrag`).

### 7.5 Kolonisierung

Die Zahl erlaubter Kolonien hängt an Astrophysik: keine ohne, eine ab Stufe 1, jede zweite weitere Stufe eine mehr,
höchstens 8 (`kolonien_max: 8`; flotte.rs, `kolonien_erlaubt`). Gezählt werden bestehende Kolonien und
Kolonisierungsflotten im Hinflug. Das Kolonieschiff nimmt 3.000 Siedler aus der Bevölkerung des Startplaneten mit
(`siedler: 3000`); dort müssen mindestens 1.000 Einwohner bleiben (flotte.rs, `flotte_senden`).

Ist der Platz bei Ankunft besiedelt oder das Limit erreicht, kehrt die Flotte um (flotte.rs, `kolonisieren`). Sonst
entsteht ein Planet mit Zone und Feldern des Platzes, Faktoren aus Systemreichtum und Zone, der Ladung als Bestand, den
Siedlern als Bevölkerung, Stabilität 50 und 4.000 Grundwohnraum, aber ohne Gebäude. Das Kolonieschiff wird verbraucht,
übrige Schiffe der Flotte bleiben auf der Kolonie. Kolonisierung vervielfacht also nichts. Eine Kolonie ohne
mitgebrachte Nahrung hungert vom ersten Tag an; das ist gewollt (README, „Befunde aus dem Bau“). Jede Kolonie kostet 300
Credits Verwaltung je Tag (Abschnitt 7.14).

### 7.6 Kampfablauf

Ein Kampf (kampf.rs, `kampf`; flotte.rs, `gefecht`) stellt den Angreifern die Verteidiger gegenüber: alle Schiffe und
Verteidigungsanlagen des Zielplaneten und die Flotten von Verbündeten, die dort mit `halten` im Orbit stehen. Hält eine
fremde Flotte den Orbit, kämpft der Angreifer nur gegen sie. Jede Einheit kämpft mit Angriff × (1 + 0,1 × Waffentechnik)
× Waffenfaktor des Volks, Schild × (1 + 0,1 × Schildtechnik) und Struktur × (1 + 0,1 × Panzerung) × Panzerungsfaktor
des Volks (`tech_je_stufe: 0.1`; kampf.rs, `Gruppe::neu`).

1. Höchstens 6 Runden (`runden: 6`). Zu Beginn jeder Runde sind alle Schilde wieder voll.
2. Erst feuern alle Einheiten des Angreifers, dann alle des Verteidigers. Zerstörte Einheiten werden erst am Ende der
   Runde entfernt und feuern in dieser Runde also noch.
3. Jeder Schuss trifft ein zufälliges gegnerisches Ziel, auch ein in dieser Runde schon zerstörtes; dann bleibt er ohne
   Wirkung. Ein Schuss unter 1 Prozent des Schildwerts des Ziels verpufft (`verpuffen_anteil: 0.01`).
4. Sonst trifft der Schaden zuerst den Schild, der Rest die Struktur. Strukturschaden bleibt. Bei Struktur 0 ist die
   Einheit zerstört; unter 70 Prozent ihrer Ausgangsstruktur explodiert sie mit der Wahrscheinlichkeit 1 minus Rest
   geteilt durch Ausgangsstruktur (`explosion_unter: 0.7`).
5. Schnellfeuer r gegen einen Zieltyp: Nach dem Schuss feuert die Einheit mit Wahrscheinlichkeit (r − 1)/r erneut.
6. Der Angreifer siegt, wenn kein Verteidiger übrig ist, der Verteidiger, wenn kein Angreifer übrig ist; sonst endet
   der Kampf unentschieden, und der Angreifer kehrt heim. Ein Planet ohne Schiffe und Verteidigung ist ohne Runde
   besiegt.

Der Zufall jedes Kampfs kommt aus einem eigenen Strom je Kampfnummer (welt.rs, `KANAL_KAMPF`). Alle Beteiligten
erhalten einen Kampfbericht mit Einheiten vorher und nachher, Runden, Sieger, Beute und Trümmern (welt.rs,
`Kampfbericht`). Ein Angriff bricht Pakt und Bündnis mit dem Opfer und schließt aus einer gemeinsamen Allianz aus
(Abschnitt 8.2).

### 7.7 Trümmer und Wiederaufbau

30 Prozent von Erz und Kristall der Kosten zerstörter Schiffe beider Seiten bilden ein Trümmerfeld an der Koordinate
des Kampfs; mehrere Kämpfe addieren sich (`truemmer_anteil: 0.3`; flotte.rs, `gefecht`). Zerstörte Verteidigung
bildet keine Trümmer; 70 Prozent der zerstörten Anlagen werden nach dem Kampf sofort wiederhergestellt, gerundet
(`verteidigung_wiederaufbau: 0.7`). Recycler holen Trümmerfelder ab.

### 7.8 Plünderung und Bunker

Nach einem gewonnenen Angriff nimmt der Angreifer von jedem der acht Güter Erz bis Xenokristall (nicht Bauteile) die
Plünderquote dessen mit, was über dem Bunkerschutz liegt (flotte.rs, `pluendern`). Die Quote ist 0,4
(`kampf.pluenderquote: 0.4`), bei den Krath 0,5. Übersteigt die Beute die freie Ladekapazität der überlebenden Flotte,
wird sie anteilig gekürzt. Der Bunker schützt je Stufe 20.000 Einheiten jedes Rohstoffs und 20 Prozent davon, also
4.000, je anderem Gut (`bunker_je_stufe: 20000`, `bunker_verarbeitet_anteil: 0.2`; regeln.rs, `bunkerschutz`). Credits
sind sicher. Das Opfer verliert 5 Stabilitätspunkte, die über 5 Tage zurückkehren; mehrere Plünderungen summieren sich
höchstens auf 10.

### 7.9 Blockade

Gewinnt eine Flotte mit Mission `blockade` oder `invasion` den Kampf, bleibt sie ohne Zeitgrenze im Orbit und
blockiert den Planeten, bis sie zurückgerufen oder vernichtet wird (flotte.rs, `kampf_am_planeten`). Wirkung
(flotte.rs, `blockiert_fuer`, `flotte_ankunft`; markt.rs, `marktlieferung`):

- Transporte aller Spieler außer dem Blockierer kehren mit ihrer Ladung um, auch die des Besitzers; Stationierung und
  Halten am blockierten Planeten scheitern.
- Marktlieferungen warten und werden stündlich erneut versucht.
- Vom Planeten startet nur ein Angriff auf die Blockadeflotte (Ziel ist die eigene Koordinate).
- Angriffe Dritter auf den Planeten treffen die Blockadeflotte; geplündert wird dabei nicht.
- Produktion und Bau laufen weiter. Eine reine Blockade senkt die Stabilität nicht.

Die Blockadeflotte kostet weiter Unterhalt, belegt einen Flottenplatz und fehlt zu Hause.

### 7.10 Belagerung und Eroberung

Eine Invasion, die den Orbitkampf gewinnt, belagert die Kolonie: Jeder Tag senkt deren Stabilitätsziel um weitere 10
Punkte (Abschnitt 5.7). Bei jedem Tick prüft der Kern (flotte.rs, `eroberung_pruefen`): Ist die Stabilität unter 15 und
sind die Bodentruppen stärker als die Garnison, wechselt die Kolonie den Besitzer. Bodentruppen sind
Truppentransporter in der Flotte × 500 × (1 + 0,1 × Waffentechnik des Angreifers) (`truppen_je_transporter: 500`;
`bodentruppen`). Die Garnison ist Kasernenstufe × 300 × (1 + 0,1 × Panzerung des Verteidigers) plus 1 je 100 Einwohner
(`garnison_je_kaserne: 300`, `garnison_je_100_einwohner: 1`; `garnison`).

Bei der Eroberung (flotte.rs, `erobern`) werden offene Marktorders des Planeten aufgelöst, jede Gebäudestufe verliert 30
Prozent, abgerundet (`eroberung_stufenverlust: 0.3`), und die Bevölkerung halbiert sich, mindestens 100 bleiben
(`eroberung_bevoelkerung: 0.5`). Alle Einheiten des Planeten gehen verloren, die Schiffe der Invasionsflotte landen
dort, ihre Ladung kommt in den Bestand. Bau- und Fertigungsschleifen, Prioritäten, Mali, Raketen und Raketenaufträge
werden gelöscht, die Stabilität beginnt bei 30. Planet und Gebäudewert gehören danach dem Eroberer. Heimatwelten lassen
sich plündern und blockieren, aber nie erobern. Weil die Stabilität mit 24 Stunden Halbwertszeit fällt, dauert eine
Belagerung mehrere Tage; Verbündete können sie brechen, indem sie die Belagerungsflotte angreifen.

### 7.11 Spionage

Ein Spionageflug besteht nur aus Sonden (flotte.rs, `spionieren`). Auf einem freien Platz erkundet er (Abschnitt 3.7),
auf einem besetzten Planeten entsteht ein Bericht. Sein Umfang hängt von Info = Sondenzahl + eigene Stufe
Spionagetechnik − Stufe Spionagetechnik des Ziels ab: Den Bestand enthält jeder Bericht, ab 2 kommen die Schiffe hinzu,
ab 3 die Verteidigung, ab 5 die Gebäude, ab 7 die Forschung. Das Ziel bemerkt jeden Versuch; sein Feldherr wird
geweckt. Die Sonden werden mit einer Wahrscheinlichkeit in Promille von Schiffe am Ziel × Sonden × 2 abgeschossen,
höchstens 1.000; jede Stufe Spionagetechnik Vorsprung halbiert sie, jede Stufe Rückstand verdoppelt sie (bis 10
Stufen). Der Bericht kommt in jedem Fall an; abgeschossene Sonden sind verloren. Ein Spieler hält je Ziel den neuesten
Bericht, höchstens 30. Spionage ist kein Angriff und bricht keinen Vertrag.

Der Kampfsimulator (sicht.rs, `werkzeug`, Abfrage `kampfsimulator`) braucht einen Bericht mit Schiffen und
Verteidigung. Er rechnet 100 Kämpfe (`simulator_laeufe: 100`) mit eigenem Zufallsstrom und ohne Wirkung auf die Welt
und liefert Siegchance, erwartete Verluste und mögliche Beute; die Technik des Gegners nimmt er aus dem Bericht oder
schätzt sie wie die eigene.

### 7.12 Verbandsangriff

Mehrere Angriffsflotten können gemeinsam angreifen (flotte.rs, `verband_oeffnen`, `verband_beitreten`,
`verbandskampf`, `verband_pluendern`; Tests in `crates/kern/tests/verband.rs`):

- Öffnen: Der Feldherr öffnet eine eigene Angriffsflotte im Hinflug als Führung.
- Beitreten: Eine eigene, noch ungebundene Angriffsflotte im Hinflug zum selben Ziel tritt bei. Gehört die Führung
  einem anderen Spieler, müssen beide in derselben Allianz sein. Ein Verband umfasst höchstens 16 Flotten.
- Ankunft: Alle kommen gemeinsam zur spätesten Ankunft an; keine Flotte wird beschleunigt, keine darf mehr als 6
  Stunden gegenüber ihrer ursprünglichen Ankunft verzögert werden.
- Kampf: ein gemeinsames Gefecht, jeder Teilnehmer mit eigener Technik. Gehört das Ziel inzwischen einem Teilnehmer
  oder ist es mit einem verbündet, fliegen alle ohne Kampf zurück. Ist das Ziel blockiert, geht der Kampf gegen die
  Blockadeflotte, ohne Beute.
- Beute: ein gemeinsamer Pool, der Bunkerschutz zählt einmal. Die Quote ist der nach freiem Frachtraum gewichtete
  Mittelwert der Quoten der Teilnehmer, verteilt wird nach freiem Frachtraum. Der Stabilitätsmalus trifft das Opfer
  einmal. Eine zurückgerufene Flotte verlässt den Verband.

### 7.13 Raketensilo

Das Raketensilo (ab Stufe III) lagert zwei Raketenarten (erweiterung.rs; Werte im Block `zusatz`):

- Kapazität: 10 Plätze je Silostufe (`silo_plaetze_je_stufe: 10`), belegt von fertigen und bestellten Raketen.
- Kosten: Abfangrakete 2.000 Erz und 500 Deuterium; Interplanetarrakete 5.000 Erz, 1.000 Kristall und 2.000 Deuterium.
- Bau (`raketen_bauen`, Verwalter oder Feldherr, Topf militaer): sofort bezahlt; ein Auftrag über n Raketen ist nach
  n × 300 Sekunden auf einmal fertig (`raketen_bauzeit_sekunden: 300`), Aufträge eines Planeten laufen nacheinander.
  Bei Unruhen ist keine Bestellung möglich.
- Start (`raketen_starten`, nur Feldherr): nur Interplanetarraketen, nur auf einen Verteidigungstyp, nur im selben
  Sektor, höchstens 5 Systeme je Silostufe weit (`raketen_reichweite_je_silo: 5`). Flugzeit 1.200 Sekunden plus 60 je
  System, also immer länger als ein Fenster. Anfänger- und Schwachenschutz gelten. Der Start beendet den eigenen
  Anfängerschutz, bricht Pakt und Bündnis mit dem Ziel und warnt den Verteidiger sofort.
- Ankunft (`raketen_ankunft`): Gehört das Ziel inzwischen dem Angreifer oder steht es unter Schutz, verpufft die Salve.
  Jede Abfangrakete des Ziels vernichtet eine angreifende Rakete. Die übrigen richten je 12.000 × (1 + 0,1 ×
  Waffentechnik) Schaden an (`raketen_schaden: 12000`); zerstört werden Schaden geteilt durch die Struktur des
  gewählten Typs mal (1 + 0,1 × Panzerung) mal Panzerungsfaktor des Volks, höchstens die vorhandenen. Schilde zählen
  nicht. Es gibt keine Beute, keine Trümmer und keinen Wiederaufbau; Schiffe und Einwohner bleiben unberührt.

Fertige Raketen zählen zum Militärwert, verschossene und abgefangene als Verluste.

### 7.14 Unterhalt und Desertion

Beim Tageswechsel zahlt jeder Spieler 0,5 Prozent des Bauwerts aller eigenen Schiffe auf Planeten und in Flotten
(`unterhalt_schiffe_je_tag: 0.005`) und 300 Credits je Kolonie (`verwaltung_je_kolonie_tag: 300`; wirtschaft.rs,
`tageswechsel`). Ein Kreuzer im Wert von 36.800 Werteinheiten kostet also 184 Credits je Tag. Verteidigung kostet
nichts. Reichen die Credits nicht, fallen sie auf 0, und eine Schuld beginnt. Sobald der erste unbezahlte
Tageswechsel 3 Tage zurückliegt (`desertion_nach_tagen: 3`), desertieren bei jedem unbezahlten Tageswechsel 10 Prozent
jedes Schiffstyps (`desertion_anteil: 0.1`) auf jedem Planeten und in jeder Flotte, mindestens ein Schiff, wenn welche
da sind; das letzte Schiff einer Flotte bleibt, damit sie heimkehren kann (`desertion`). Ein bezahlter Tag beendet die
Schuld.

### 7.15 Warnzeit und Schutzregeln

Eine feindliche Flotte wird für den Besitzer des Zielplaneten und seine Verbündeten 30 Minuten vor Ankunft sichtbar;
jede Stufe Sensorphalanx auf dem Zielplaneten verlängert das um 30 Minuten (`warnzeit_basis_minuten: 30`,
`warnzeit_je_phalanx_minuten: 30`; flotte.rs, `warnzeit`, `sichtbare_angriffe`). Weil die kürzeste Flugzeit 20 Minuten
beträgt und ein Fenster 15, lässt jeder erkannte Angriff mindestens eine Reaktion zu.

Anfängerschutz gilt 10 Spieltage oder bis zum Aufstieg auf Stufe III, je nachdem, was zuerst eintritt
(`anfaengerschutz_tage: 10`, `anfaengerschutz_bis_stufe: 3`). Wer selbst eine feindliche Flotte gegen einen anderen
Spieler oder Raketen startet, verliert ihn sofort (flotte.rs, `flotte_senden`; erweiterung.rs, `raketen_starten`).
Feindliche Flotten und Raketen gegen geschützte Spieler lehnt der Kern beim Start ab. Danach sind Angriffe auf jeden
erlaubt, auch auf Schwächere. Die Stellschraube `schwachenschutz_anteil` steht auf 0.0 und ist damit aus; mit einem
Wert über 0 dürfte man Spieler unter diesem Anteil der eigenen Punkte nur angreifen, wenn sie einen selbst angegriffen
haben.

## 8. Diplomatie und Markt

### 8.1 Nachrichten

Nachrichten sind Freitext an einen oder mehrere Spieler und, mit `allianz: true`, an die eigene Allianz (diplomatie.rs,
`nachricht`), höchstens 1.200 Zeichen (`nachricht_zeichen: 1200`) und 20 je Spieler und Spieltag
(`nachrichten_je_tag: 20`). Die Zustellung ist sofort, beim Empfänger wird der Diplomat geweckt. Das Lagebild zeigt
die Nachrichten der letzten 7 Tage, an denen der Spieler beteiligt ist.

### 8.2 Verträge

Verhandelt wird in freier Sprache, Verträge aber sind Objekte der Engine mit fester Wirkung (diplomatie.rs).

| Art | Wirkung | Ende |
|---|---|---|
| `nichtangriffspakt` | ein Angriff auf den Partner ist ein Bruch | Kündigung mit 48 Stunden Frist |
| `handelsabkommen` | halbe Marktgebühr im Handel untereinander (markt.rs, `gebuehr`) | Kündigung sofort |
| `verteidigungsbuendnis` | Partner gelten als verbündet: Halten, geteilte Angriffswarnungen; ein Angriff ist ein Bruch; höchstens 3 je Spieler | Kündigung mit 48 Stunden Frist |
| `tribut` | der Anbieter zahlt täglich eine Menge Credits oder ein Gut für 1 bis 365 Tage | Ablauf; stellt der Zahler vorher ein oder kann er nicht zahlen, ist das ein Bruch |

- Angebot (`vertrag_anbieten`): Der Anbieter hinterlegt die Kaution sofort in Credits. Zwischen zwei Spielern gibt es
  je Art nur einen Vertrag in Kraft oder ein offenes Angebot, außer beim Tribut. Das Bündnislimit
  (`buendnisse_max: 3`) wird bei Angebot und Annahme geprüft.
- Annahme (`vertrag_annehmen`): Der Partner hinterlegt dieselbe Kaution; der Vertrag gilt ab sofort. Ablehnung
  (`vertrag_ablehnen`) durch eine der beiden Seiten, solange das Angebot offen ist; die Kaution geht zurück.
- Kündigung (`vertrag_kuendigen`): Pakt und Bündnis gelten bis zum Ende der Frist von 48 Stunden
  (`kuendigungsfrist_stunden: 48`) weiter, dann gehen beide Kautionen zurück (`vertrag_ende`). Ein Handelsabkommen
  endet sofort mit Rückgabe. Beim Tribut ist die Kündigung durch den Zahler ein Bruch; der Empfänger kann ihn regulär
  beenden.
- Bruch (`brechen`): Der Geschädigte erhält beide Kautionen, der Vertrag gilt als gebrochen, der Bruch steht im
  Register.

Einen Bruch löst der Kern selbst aus (`bruch_pruefen`): bei jedem Kampf durch `angriff`, `blockade` oder `invasion`,
bei einem Verbandsangriff, beim Start und bei der Ankunft von Interplanetarraketen. Gebrochen werden dann alle
Nichtangriffspakte und Verteidigungsbündnisse zwischen Angreifer und Opfer, die in Kraft oder in der Kündigungsfrist
sind. Spionage ist kein Bruch. Das Vertragsregister ist öffentlich: wer wann mit wem welchen Vertrag geschlossen,
gekündigt, beendet oder gebrochen hat, dazu Gründung, Beitritt, Austritt und Ausschluss bei Allianzen
(`registrieren`). Das Lagebild zeigt die letzten 20 Einträge.

### 8.3 Tribut

Tribute laufen beim Tageswechsel (diplomatie.rs, `tribute_zahlen`). Der Zahler überweist die vereinbarte Menge in
Credits oder als Gut von seiner Heimatwelt direkt auf die Heimatwelt des Empfängers, ohne Flug und ohne Rücksicht auf
Blockaden. Fehlt der Betrag, ist das ein Bruch. Nach der vereinbarten Zahl von Tagen endet der Tribut regulär, und
beide Kautionen gehen zurück. Gezahlte Tribute stehen in der Statistik als geschenkt und erhalten.

### 8.4 Allianzen

Eine Allianz hat höchstens 8 Mitglieder (`allianz_max: 8`; diplomatie.rs). Der Gründer wählt einen eindeutigen Namen
mit 1 bis 30 Zeichen. Jedes Mitglied darf einladen, solange Mitglieder und offene Einladungen zusammen unter 8 liegen;
beitreten kann nur, wer eingeladen ist, solange die Allianz nicht voll ist. Austreten geht jederzeit. Mitglieder haben
einen gemeinsamen Nachrichtenkanal, gelten als verbündet (Halten, geteilte Warnungen) und können gemeinsam
Verbandsangriffe fliegen. Wer ein Mitglied der eigenen Allianz angreift, wird ausgeschlossen; das steht im Register und
zählt in der Statistik als Vertragsbruch, eine Kaution gibt es dabei nicht. Jeder Spieler wird einzeln gewertet.

### 8.5 Geschenke und Lieferungen

Credits schenkt der Diplomat mit `schenken` (diplomatie.rs, `schenken`). Güter liefert ein Transport auf einen fremden
Planeten; er wird als Geschenk gezählt, und beim Empfänger wird der Diplomat geweckt (flotte.rs, `flotte_ankunft`).
Geschenke und Tribute sind erlaubt und stehen in der Statistik beider Seiten.

### 8.6 Markt

Es gibt ein Orderbuch je Gut, gehandelt wird in Credits mit Preisen in Tausendsteln (markt.rs):

- Eine Order braucht einen Markt auf dem Planeten (ab Stufe III); je Planet sind 5 offene Orders je Marktstufe erlaubt
  (`orders_je_marktstufe: 5`). Menge mindestens 1, Preis über 0.
- Eine Kauforder hinterlegt Menge × Limitpreis plus die eigene Gebühr in Credits; eine Verkaufsorder nimmt die Ware
  sofort vom Planeten.
- Ausführung sofort gegen passende Orders anderer Spieler: Ein Kauf nimmt die günstigsten Verkaufsorders bis zum
  Limit, ein Verkauf die höchsten Kauforders ab dem Limit, bei gleichem Preis die ältere. Gehandelt wird zum Preis der
  Order, die im Buch stand. Der Rest bleibt als Order im Buch. Eigene Orders treffen nie aufeinander.
- Gebühr: 2 Prozent für jede Seite (`markt.gebuehr: 0.02`) mal Marktfaktor des Volks (Aurelianer 1 Prozent), halbiert
  zwischen Partnern eines Handelsabkommens. Die Gebühr verschwindet aus dem Spiel. Der Käufer erhält die Differenz
  zwischen Hinterlegung und tatsächlichem Preis samt Gebühr zurück.
- Lieferung: Eine neutrale Handelsflotte bringt die Ware vom Planeten des Verkäufers zum Planeten des Käufers, mit der
  Flugzeit eines Schiffs mit Tempo 10.000 bei voller Fahrt (`liefertempo: 10000`). Sie kann nicht abgefangen werden
  und wartet, solange der Zielplanet blockiert ist.
- Storno (`markt_storno`): Credits oder Ware gehen zurück. Bei einer Eroberung werden alle Orders des Planeten
  aufgelöst.

Es gibt keinen Händler außerhalb der Spieler; Preise entstehen nur aus Orders.

## 9. Wertung

Gezählt wird investierter Wert, nicht Gehortetes (wertung.rs, `punkte_neu`). Grundlage ist die Werteinheit: eine
Gütermenge mal ihrem festen Gewicht (regeln.rs, `wert`). Die Gewichte sind Erz 1, Kristall 1,5, Deuterium 2, Nahrung 1,
Legierung 5, Elektronik 8, Konsumgut 4, Xenokristall 20, Antriebskern 5.300 und Habitatmodul 3.800
(`wertung.gewichte`); feste Gewichte verhindern, dass Marktpreise die Wertung verzerren. Ein Punkt entspricht 1.000
Werteinheiten (`wertung.einheit: 1000`).

| Teilwertung | was zählt | Beleg |
|---|---|---|
| Wirtschaft | für jedes Gebäude auf jedem eigenen Planeten der Wert der Kosten aller Stufen 1 bis zur aktuellen Stufe | regeln.rs, `wert_gebaeude_bis` |
| Forschung | für jede Forschung der Wert der Güterkosten aller erforschten Stufen; Forschungspunkte zählen nicht | regeln.rs, `wert_forschung_bis` |
| Militär | aktueller Wert aller eigenen Schiffe auf Planeten und in Flotten, aller Verteidigungsanlagen und aller fertigen Raketen | wirtschaft.rs, `flottenwert`; wertung.rs |
| Zivilisation | 1 Punkt je 100 Einwohner (`einwohner_je_punkt: 100`) plus der Bonus der erreichten Stufe | wertung.rs |

Wirtschaft, Forschung und Militär werden je für sich durch 1.000 geteilt und abgerundet. Der Stufenbonus ist 0, 100,
400, 1.500 oder 5.000 Punkte für Stufe I bis V (`stufenbonus: [0, 100, 400, 1500, 5000]`); gezählt wird der Bonus der
aktuellen Stufe, nicht die Summe der Boni. Die Gesamtpunktzahl ist die Summe der vier Teile.

Nicht gezählt werden gelagerte Güter, Credits, Ladung, Trümmer und offene Orders. Daher bringt Bauen und Abreißen
keine Punkte, ein Abriss kostet sie. Zerstörte Schiffe und Anlagen verlieren ihren Wert, wiederhergestellte
Verteidigung zählt wieder. Eine eroberte Kolonie zählt mit ihren verbliebenen Gebäudestufen für den Eroberer. Bauteile
im Lager zählen nicht, im Kolonieschiff zählen sie zum Militärwert, bis die Kolonie gegründet ist. Desertion und Hunger
kosten Punkte.

Die Punkte werden nach jedem Tick, jedem Tageswechsel, jedem Kampf, jeder Kolonisierung, Eroberung und jedem
Stufenaufstieg neu gerechnet; maßgeblich ist der Stand am Ende von Spieltag 365. Die Rangliste sortiert absteigend nach
Gesamtpunkten, bei Gleichstand nach Spielerkennung (wertung.rs, `punkte_neu`, `rangliste`). Sie ist öffentlich und
zeigt Rang, Name, Punkte und Stufe aller Spieler. Bündnispartner werten getrennt.

## 10. Determinismus und Fairness

### 10.1 Determinismus

Gleicher Startwert, gleiche Spielerzahl, gleiches Regelwerk und gleiches Aktionsprotokoll ergeben bitgenau denselben
Weltzustand. Dafür sorgt der Kern so:

- **Ganzzahlen statt Gleitkomma:** Mengen in Tausendsteln, Faktoren als Festkomma (typen.rs).
- **Feste Reihenfolgen:** feste Arrays mit Aufzählungen als Index und `BTreeMap` statt `HashMap`, jede Iteration in
  fester Reihenfolge (welt.rs, Kopfkommentar). Kämpfe laufen nacheinander.
- **Eigene Zufallsströme:** ChaCha8 je Zweck und laufender Nummer, abgeleitet aus dem Startwert, für Galaxie, Kampf je
  Kampfnummer, Spionage je Spionagenummer, Spielerreihenfolge je Fensternummer und Simulator (welt.rs, `strom`,
  `KANAL_*`). Ein Kampf beeinflusst nie den Zufall eines anderen.
- **Eindeutige Ereignisordnung:** Zeit, Priorität, Laufnummer (Abschnitt 2.2).
- **Hashes:** SHA-256 über den ganzen Zustand samt sortierter Ereignisschlange (sim.rs, `hash`); eine Hashkette über
  das Aktionsprotokoll (aktion.rs, `protokolliere`); ein eigener SHA-256 des Regelwerks, mit vereinheitlichten
  Zeilenenden (regeln.rs, `Regelwerk::laden`).
- **Schnappschüsse:** Der vollständige Zustand lässt sich als Bytes speichern und laden und rechnet gleich weiter
  (sim.rs, `zu_bytes`, `aus_bytes`; Test `bestaende_werden_nie_negativ_und_schnappschuss_ist_exakt`).

Nachgespielt wird aus Startwert und Aktionsprotokoll (`sternenepoche replay`, README „Schnellstart“). Laut README
ergeben zwei Läufe mit gleichem Startwert und das Nachspielen denselben Zustandshash, auch für Läufe des Orchestrators
und nach Anhalten, Fortsetzen oder einem harten Abbruch. Die Antworten der Modelle sind nicht wiederholbar, das Spiel
schon. Die Umgebung `kern::umgebung` bietet dieselbe Engine mit `reset` und `step` an; ihr Lohn ist die exakte
Punktedifferenz (umgebung.rs).

### 10.2 Fairness

- Die Weltuhr steht, während Rollen entscheiden (Abschnitt 2.7); schnellere Modelle gewinnen keine Spielzeit.
- Die kürzeste Flugzeit (1.200 Sekunden) und die kürzeste Raketenflugzeit liegen über der Fensterbreite (900
  Sekunden); sonst wird das Regelwerk nicht geladen (regeln.rs, `pruefe`).
- Gleichzeitige Aktionen laufen in einer je Fenster neu ausgelosten Reihenfolge; kein Spieler handelt systematisch
  zuerst.
- Gleiche Heimatwelten, Mindestabstand der Startsysteme, Nebelabstand innerhalb von 20 Prozent, gleich große
  Völkergruppen (Abschnitt 3.4).
- Dringende Ereignisse wecken sofort; der Anfängerschutz gibt jedem 10 Tage Aufbauzeit, sofern er nicht selbst
  angreift. Rollenrechte und Töpfe prüft der Kern bei jeder Aktion für alle KI-Reiche gleich.

### 10.3 Informationsgrenzen

Die Informationsgrenze liegt in der Engine: Ein Lagebild enthält nur die eigene Lage, öffentliche Daten und das, was
der Spieler selbst herausgefunden hat (sicht.rs, Kopfkommentar und `sicht`).

- **Eigenes:** alle eigenen Planeten mit Bestand, Raten, Lager, Energie, Arbeit, Stabilität, Schleifen, baubaren
  Gebäuden mit Kosten, Einheiten, Raketen, Garnison und Blockade; eigene Flotten; Töpfe, Forschung, nächste Stufe mit
  Bedingungen; Warnungen als reine Fakten; Ereignisse seit dem letzten Aufruf und eine Chronik der letzten 7 Tage;
  Doktrin, eigenes Notizbuch, Meldungen.
- **Öffentlich:** Rangliste, die letzten 20 Einträge des Vertragsregisters, belegte Plätze im eigenen Sektor bis 8
  Systeme um eigene Planeten mit Name, Punkten und Schutzstatus (höchstens 40), die Galaxieabfrage mit Belegung,
  Spielername, Punkten, Nebeln und Gürteln, beste Marktpreise, Trümmerfelder in den eigenen Sektoren.
- **Selbst herausgefunden:** Spionageberichte mit Alter, Erkundungen, Kampfberichte und Nachrichten mit eigener
  Beteiligung, sichtbare Angriffe auf eigene und verbündete Planeten, Verbände eigener und verbündeter Spieler,
  Raketensalven von oder gegen einen selbst.

Nicht sichtbar sind fremde Bestände, Flotten, Gebäude und Forschung ohne Bericht, die Werte unerkundeter Plätze und
fremde Raketensalven. Der Regeltext nennt weder Messung noch Modelle (regeltext.rs, Kopfkommentar); Spielernamen sind
zufällig erzeugt. Die native Oberfläche liest nur `sicht` und `werkzeug` (`crates/spieler/README.md`), die Umgebung gibt
nur die gefilterten Lagebilder heraus (umgebung.rs, `beobachten`; Test `observations_contain_only_own_planet_details`).
Im Datensatz eines Laufs enthält jede Entscheidung nur, was der Agent sah; der Weltzustand liegt getrennt in
Schnappschüssen (README, „Was aus welchem Entwurf stammt“).

## 11. Abweichungen vom Originalkonzept

Das Originalkonzept (`wissen/quellen/spielkonzept-original.txt`) ist die Grundlage für Zeitmodell, Galaxie, Völker,
Zivilisationsstufen, Rollen, Töpfe und Zahlen. Die folgenden Abweichungen sind im README begründet; die Gründe sind von
dort übernommen.

### 11.1 Befunde aus dem Bau

| Konzept | jetzt | Grund laut README |
|---|---|---|
| Bevölkerungswachstum 5 Prozent je Tag | 12 Prozent (`wachstum_je_tag: 0.12`) | logistisch gebremst erreichte ein Reich 2.000 Einwohner erst um Tag 39; mit 12 Prozent stimmen die Zielzeiten |
| mindestens zwei Systeme zwischen zwei Spielern | mindestens eines (`start_abstand: 2`) | zwei freie Systeme passen bei 25 Spielern auf 60 Systeme nicht |
| Stabilitätsverlust durch Plünderungen summiert sich | höchstens 10 Punkte (`pluenderung_malus_max: 10.0`) | ein oft geplündertes Reich fiel auf Stabilität 0 und konnte nichts mehr bauen |
| Stufe V mit 80.000 Einwohnern | 700.000 | Stufe V fiel bei ordentlicher Wirtschaft weit vor Tag 150; die Bevölkerung taktet die Stufen, Forschungsbedingungen verschieben sie kaum |
| Arbeitskräfte nach Prioritätenliste des Agenten | ohne Vorgabe zuerst Farm und Kraftwerke | ohne Liste gingen die Arbeitskräfte zuerst an die Minen; bei den Syntheten stand das Kraftwerk leer, das Reich starb |

Außerdem hält das README fest, dass Kolonien ohne mitgebrachte Nahrung vom ersten Tag an hungern, und dass das gewollt
ist.

### 11.2 Balance

Mit Skriptbots gemessen; alle Werte stehen kommentiert im Regelwerk (README, „Balance“, Tabelle „Geändert gegenüber
dem ersten Regelwerk“):

| Wert | vorher | jetzt | Grund laut README |
|---|---|---|---|
| Beutequote / Krath | 0,5 / 0,6 | 0,4 / 0,5 | Raub lohnte zu sehr |
| Unterhalt je Schiff und Tag | 0,1 Prozent | 0,5 Prozent des Bauwerts in Credits | Flotten waren eine Punktesenke ohne Kosten |
| Bunker je Stufe | 3.000 je Gut | 20.000 je Gut | Schutz, der zu den Lagergrößen passt |
| Stabilitätsverlust durch Plünderung | höchstens 20 | höchstens 10 | Mehrfachplünderung legte Reiche lahm |
| sechs Verteidigungsanlagen | Grundwerte | Struktur, Schild, Angriff ×3 | Verteidigung hielt keiner Flotte stand |
| Stufe IV | 20.000 Einwohner | 32.000 | Median lag bei Tag 53 |
| Stufe V | 80.000 Einwohner | 700.000 | mit ordentlicher Wirtschaft zu früh |
| Strombedarf der Syntheten | 30 je 1.000 Einwohner | 25 | Syntheten lagen 15 Prozent zurück |

Die Abnahme (`sternenepoche balance --abnahme`) prüft unter anderem, dass der Räuber höchstens 25 Prozent über dem
besten anderen Typ liegt, dass Raub gegen Wehrlose lohnt und Schutz wirkt, dass die Stufenzeiten in den Zielfenstern
liegen und die Völker höchstens 15 Prozent auseinander. Laut README sind alle Kriterien erfüllt. Offen ist laut README,
ob Verteidigung ohne Unterhalt voll als Punkte zählen soll; das entscheidet Karl.

### 11.3 Weckregel

Das Konzept weckt dieselbe Rolle höchstens einmal je Spielstunde, Angriffswarnungen ausgenommen. Jetzt wecken Wecker
und nicht dringende Ereignisse frühestens nach dem halben Takt (`frueh_anteil: 0.5`), dringende Ereignisse sofort.
Grund laut README („Aufrufe und Wecker“): Ohne diese Grenze stellten Modelle bei jedem Aufruf einen Wecker auf eine
Stunde, und ein Lauf kostete ein Vielfaches.

### 11.4 Übernommen aus dem zweiten Entwurf

Laut README („Was aus welchem Entwurf stammt“) kamen aus einem zweiten Entwurf hinzu: die Blockade als dritte
Konfliktform neben Plünderung und Eroberung; eine Kolonisierung, die nichts vervielfacht (Siedler aus der
Heimatbevölkerung, Habitatmodule als Wohnraum, Start nur mit der Ladung); Kautionen für Verträge, die bei Bruch an den
Geschädigten gehen; keine unbegrenzt zahlende Außenwelt (kein Händler außerhalb der Spieler, Marktgebühren
verschwinden); kein kostenloser Punktgewinn (Abriss erstattet nichts, die Wertung zählt nur Bestehendes); die Trennung
von Beobachtung und Weltzustand; das Feld `prognose` in jeder Antwort und die Messung eines Verdachts der Modelle (Feld
`verdacht` im Datensatz). Nicht übernommen wurden Wasser als weiteres Gut (mehr Buchhaltung, keine neue Entscheidung),
fünf Völker (die vier sind ausgearbeitet, die Syntheten sind der beabsichtigte Lesetest), 800 statt 1.440 Plätze,
Inventar in der Wertung (Horten soll nicht zählen) und die andere Rollenteilung.

### 11.5 Weitere Unterschiede ohne dokumentierte Begründung

Diese Unterschiede zwischen Konzept und Regelwerk zeigt der Vergleich der Quellen; das README begründet sie nicht:

- Veyari: Bevölkerungswachstum ×1,1 statt ×1,2 im Konzept. Syntheten: ×0,85 statt ×0,7.
- Stufe V: Das Konzept nennt „5.000 Xenokristall verbaut“ als Bedingung. Im Kern ist das keine Bedingung
  (wirtschaft.rs, `stufen_bedingungen`); stattdessen kostet der Aufstieg 5.000 Xenokristall. Der Kern führt die
  verbaute Menge mit (`Spieler::xeno_verbaut`), prüft sie aber nicht.
- Doktrin und Notizbuch sind in Zeichen begrenzt (2.400 und 6.000), nicht in Tokens (Konzept: 600 und 1.500).

## Anhang A: Regeltext und Kern abgeglichen

Der Regeltext wird aus dem Regelwerk erzeugt, seine Sätze sind aber von Hand geschrieben (regeltext.rs, `abschnitte`).
Beim Schreiben dieser Spezifikation und bei der Auswertung des echten Laufs fielen sechs Stellen auf, an denen
er etwas anderes sagte, als der Kern tut. Weil die
Modelle nur den Regeltext kennen, ist er jeweils an den Kern angepasst worden (4. Okt. 2026); die Mechanik blieb
unverändert, weil die Balance mit ihr gemessen ist:

1. **Verbandsangriff.** Früher „eigene oder verbündete Angriffsflotten“. Der Kern verlangt bei fremden Flotten dieselbe
   Allianz (flotte.rs, `verband_beitreten`); ein Vertrag ohne gemeinsame Allianz genügt nicht. So steht es jetzt im Text.
2. **Spionagebericht.** Früher „ab 1 die Bestände“. Der Kern liefert die Bestände in jedem Bericht (flotte.rs,
   `spionieren`); der Text sagt das jetzt.
3. **Stufenbonus.** Früher „für die Stufen II bis V einmalig …“. Der Kern zählt nur den Bonus der aktuellen Stufe
   (wertung.rs, `punkte_neu`); der Text sagt jetzt „es zählt nur die aktuelle Stufe, nichts wird addiert“.
4. **Freischaltungen.** Der Freischalttext der Stufe III nannte das Raketensilo nicht, der der Stufe V nicht die drei
   Großprojekte Orbitalring, Forschungsarchiv und Versorgungsnetz. Beides ist im Regelwerk ergänzt.
5. **Lagergrenze.** Früher „der Überschuss verfällt“. Im Kern endet an der Grenze nur die Produktion; Lieferungen, Beute,
   Marktkäufe und Tribute kommen auch darüber hinaus an. So steht es jetzt im Text.
6. **Bezahlen eines Bauauftrags.** Früher „bezahlt wird beim Baubeginn, ein nicht bezahlbarer Auftrag wartet und hält die
   Schleife an“. Das gilt im Kern nur hinter anderen Aufträgen; in eine leere Schleife beginnt ein Auftrag sofort und
   wird abgelehnt, wenn er nicht bezahlbar ist (wirtschaft.rs, `bauen`). Im echten Lauf scheiterten daran rund 340 von
   943 Bauaufträgen des Verwalters. Der Text sagt es jetzt genau.

`crates/kern/tests/regeltext.rs` hält das fest: ein Test prüft, dass jedes Gebäude und jede Anlage ab Stufe II im
Freischalttext genau ihrer Stufe steht, ein zweiter die übrigen Sätze und für den Bauauftrag auch das Verhalten des Kerns. Die vollständigen Regeltexte je Rolle stehen in
`docs/REGELTEXT.md`, erzeugt mit `sternenepoche doku`.
