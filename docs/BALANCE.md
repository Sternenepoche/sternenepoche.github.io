---
layout: default
title: "Balance"
---

# Balance

Wie das Gleichgewicht des Spiels gemessen wird, nach welchen Kriterien es abgenommen ist, was dafür geändert
wurde und wie man eine neue Schieflage findet. Stand 4. Okt. 2026; Zuständigkeit laut `COORDINATION.md`:
Claude (Balance und Stufe V).

## Messen mit Skriptbots

Sprachmodelle sind zu teuer und zu langsam, um ein Spieljahr mit 50 Reichen dutzendfach zu wiederholen. Die
Balance wird deshalb mit Skriptbots gemessen (`crates/lauf/src/bots.rs`). Sie handeln über dieselben Aktionen wie
die Modelle und wissen über andere Spieler nur, was öffentlich ist oder was ihre eigenen Sonden berichtet haben.

| Bottyp | Spielweise |
|---|---|
| Ökonom | baut Wirtschaft, Bevölkerung und Forschung aus, kolonisiert, schützt sich nach der ersten Plünderung, nimmt Nichtangriffspakte an |
| Räuber | baut Flotte und forscht Spionage, Waffen und Panzerung, plündert schwächere Nachbarn, lehnt Pakte ab, schützt sich nach der ersten Plünderung |
| Igel | Wirtschaft mit Sensorphalanx, Bunker und Verteidigung (höchstens die Hälfte seines Gebäudewerts) |
| Händler | Wirtschaft mit Markt: verkauft Rohstoffe und Fertigwaren ab 70 Prozent der Lagergrenze, kauft unter 10 Prozent zu; bietet den drei nächsten Nachbarn Nichtangriffspakte an |
| Wehrlos | Kontrolltyp: ein Ökonom, der sich auch nach Plünderungen nie schützt |

Die Standardmischung sind die vier Typen des Konzepts, reihum auf die Spieler verteilt. Jede Epoche ist
deterministisch: gleicher Startwert, gleicher Verlauf (`epoche --pruefen` belegt es mit Hashvergleich und
Nachspielen).

## Abnahme

```bash
./target/release/sternenepoche.exe balance --abnahme
```

Die Abnahme fährt drei Aufstellungen über je 12 Startwerte mit 50 Spielern und 365 Tagen (Mischung aller vier
Typen, Räuber gegen Wehrlose, Räuber gegen Igel) und prüft die mit Codex vereinbarten Kriterien
(`COORDINATION.md`, 3. Okt. 2026, 23:38) samt dem Völkerkriterium vom 4. Okt.:

| Kriterium | Bedingung |
|---|---|
| Keine Dominanz | Der Räuber liegt höchstens 25 Prozent über dem besten anderen Typ. |
| Raub lohnt sich | Gegen wehrlose Ökonomen gewinnt der Räuber über 80 Prozent der Kämpfe und liegt vorn. |
| Schutz wirkt | Der Igel verliert mindestens 80 Prozent weniger an Plünderung als der wehrlose Ökonom. |
| Entwicklung | Stufe II bis IV erreichen mindestens 90 Prozent, Median in Tag 10–20, 30–50 und 60–110; Stufe V erreichen bei Räuber und Igel mindestens 80 Prozent, Median in Tag 150–250. |
| Völker ausgeglichen | Das beste Volk liegt höchstens 15 Prozent über dem schwächsten. |
| Keine Plateaus | Alle Typen wachsen in den letzten 60 Tagen. |

Der Befehl endet mit Exitcode 1, wenn ein Kriterium fehlt. Mit `--laeufe 24` rechnet er 24 statt 12 Startwerte
je Aufstellung; eine Epoche braucht hier rund 28 Sekunden, die Abnahme mit 12 Startwerten gut 4 Minuten.

## Ergebnis

`sternenepoche balance --abnahme --laeufe 24`, 4. Okt. 2026, alle Kriterien erfüllt. Punkte im Mittel,
Stufenzeiten als Median:

| Bottyp | Punkte | Stufe II | Stufe III | Stufe IV | Stufe V (erreicht) |
|---|---|---|---|---|---|
| Ökonom | 156.233 | Tag 13,6 | Tag 36,4 | Tag 61,4 | Tag 160,8 (100 %) |
| Räuber | 171.472 | Tag 13,6 | Tag 36,4 | Tag 61,6 | Tag 174,0 (99 %) |
| Igel | 164.668 | Tag 13,6 | Tag 36,5 | Tag 64,8 | Tag 167,3 (100 %) |
| Händler | 156.321 | Tag 13,7 | Tag 36,5 | Tag 64,2 | Tag 164,8 (99 %) |

| Volk | Punkte | Stufe IV | Stufe V |
|---|---|---|---|
| Aurelianer | 162.437 | Tag 64,2 | Tag 163,5 |
| Krath | 164.918 | Tag 61,4 | Tag 167,4 |
| Veyari | 160.717 | Tag 56,5 | Tag 152,9 |
| Syntheten | 160.651 | Tag 71,4 | Tag 219,9 |

- Räuber-Faktor 1,04 (höchstens 1,25 erlaubt).
- Raub lohnt sich: Räuber 188.582 gegen Wehrlose 121.827 Punkte.
- Schutz wirkt: Der Igel verliert 1 Punkt an Plünderung, der Wehrlose 110.546.
- Völker: 160.651 bis 164.918 Punkte, Faktor 1,03 (höchstens 1,15 erlaubt).

## Was geändert wurde

### Regelwerk

Alle Werte stehen kommentiert in `regeln/regelwerk.ron`, die vollständige Referenz in `docs/REGELWERK.md`.

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

Verworfen nach Messung: Schiffshülle gleich Struktur/10 (kurze Kämpfe gewinnt die größere Seite, der Igel
wird schwächer), Stufe IV mit 30.000 Einwohnern (Räuber-Faktor 2,6), Stabilitätsmalus nur bei Beute über null
(keine messbare Wirkung, deshalb keine Kernänderung).

### Bots

Die Hauptursache der alten Schieflage waren die Bots, nicht die Regeln. Ab Tag 120 stand die Wirtschaft der
friedlichen Bots still, während der Räuber mit Schiffen eine unbegrenzte Punktesenke hatte. Behoben in
`crates/lauf/src/bots.rs`:

- Kolonien schickten Erz über 60 Prozent ihrer Lagergrenze heim und konnten ihre eigenen Kraftwerke nie
  bezahlen; jetzt wird nichts im Kreis geschickt, Kolonien versorgen sich mit Lager und Strom.
- Lager zuerst, wenn ein Bau mehr kostet, als das Lager fasst.
- Kraftwerke nach Kosten je Stromeinheit statt einer festen Fusionsgrenze.
- Forschung läuft immer weiter: billigste verfügbare Stufe, Terraforming bei voller Heimat, unerreichbare
  Planpunkte blockieren nicht. Forschung, Labor und Gebäude der nächsten Stufe lesen die Bots aus dem Regelwerk.
- Bei Stufe IV zuerst Orbitalwerft und Habitatmodule, dann Wachstum: Habitatmodule kosten Konsumgüter, und eine
  große Bevölkerung verbraucht sie vollständig. Wer erst wächst, kommt nie an Kolonien und nie an Stufe V.
- Ein Feld der Heimat bleibt für den Orbitalring frei; Großprojekte ab Stufe V.
- Rücklage für Ausbauziel und Aufstieg vor Schiff- und Verteidigungskauf; Räuber schützen sich nach der ersten
  Plünderung und fliegen langsamer statt ohne Treibstoff.

Dieselbe Falle wie die Bots erwischt auch Modelle. Deshalb sagt der Regeltext im Abschnitt Bevölkerung: „Was die
Produktion nicht deckt, nimmt die Bevölkerung aus dem Lager: bei Unterdeckung bleibt dort nichts liegen, auch
nicht für Habitatmodule.“

## Eine Schieflage finden

Die Lehre aus dieser Arbeit: Scheinbare Regelschieflagen waren meist blockierte Bots. Nach jeder Bot-Korrektur
kippte das Bild (Räuber-Faktor von 1,3 auf 0,75 und zurück auf 1,04). Regelwerte gegen blockierte Bots zu
justieren verschiebt nur Artefakte. Die Reihenfolge ist deshalb:

1. Woran scheitern Bots an Stufe V? Je Bottyp: wer sie erreicht, wann, und welche Bedingung den übrigen fehlt.

```bash
./target/release/sternenepoche.exe stufe5
```

2. Was tut ein einzelner Spieler? Alle 15 Tage sein Zustand, je Planet der nächste Bau und das Gut, das dafür
   fehlt.

```bash
./target/release/sternenepoche.exe epoche --startwert 1 --spur 7
```

3. Bots reparieren, danach Regelwerte in Varianten messen, mit eigenem `CARGO_TARGET_DIR` und je Variante
   einer kopierten Binärdatei, damit Läufe parallel rechnen.
4. Abnahme mit `--laeufe 24` und Determinismus mit `epoche --pruefen`.

Die Bevölkerung taktet die Stufen; Forschungsbedingungen verschieben sie kaum.

## Offen

- **Verteidigung zählt ohne Unterhalt voll als Punkte.** Der Igel steckt bis zur Hälfte seines Gebäudewerts
  hinein und liegt trotzdem nicht vorn; nötig ist keine Änderung. Ob das so gewollt ist, entscheidet Karl.
- Modelle spielen anders als Bots. Der echte Lauf (`docs/ECHTER-LAUF.md`) zeigt, wie ein Modellreich gegen die
  Bots abschneidet; für eine Balance unter Modellen braucht es mehrere Modellreiche und mehrere Startwerte.
