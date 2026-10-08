<!-- Erzeugt mit `sternenepoche doku` aus regeln/regelwerk.ron. Nicht von Hand ändern: der Test doku_passt_zum_regelwerk vergleicht diese Datei mit dem Regelwerk. -->

# Regelwerk – Referenz

Alle Zahlen des Spiels, Version 0.1.0 (SHA-256 `39cd09b925d8baa110b6b7fbf2a2f5b0bc1370e86223953b8e7ef39e18edd7d0`). Die Begründungen der Werte stehen als Kommentare in `regeln/regelwerk.ron`, die Mechanik in `docs/SPEZIFIKATION.md`. Mengen sind Einheiten des Guts, Zeiten Sekunden, wo nichts anderes steht; Faktoren 1.0 bedeuten keine Abweichung.

## Inhalt

- [welt](#welt)
- [zonen](#zonen)
- [voelker](#voelker)
- [wirtschaft](#wirtschaft)
- [stabilitaet](#stabilitaet)
- [gebaeude](#gebaeude)
- [forschung](#forschung)
- [einheiten](#einheiten)
- [stufen](#stufen)
- [flug](#flug)
- [kampf](#kampf)
- [diplomatie](#diplomatie)
- [markt](#markt)
- [wertung](#wertung)
- [agenten](#agenten)
- [zusatz](#zusatz)
- [aktionen](#aktionen)

## welt

Größe der Galaxie, Spielerzahl, Dauer der Epoche, Zeittakt.

| Wert | Einstellung |
|---|---|
| `sektoren` | 2 |
| `systeme_je_sektor` | 60 |
| `plaetze_je_system` | 12 |
| `nebel_abstand` | 10 |
| `nebel_versatz` | 5 |
| `guertel_abstand` | 5 |
| `guertel_versatz` | 3 |
| `reichtum_min` | 0.7 |
| `reichtum_max` | 1.4 |
| `epoche_tage` | 365 |
| `fenster_sekunden` | 900 |
| `heimat_position` | 6 |
| `heimat_felder` | 180 |
| `start_abstand` | 2 |
| `start_bevoelkerung` | 1000 |
| `start_credits` | 2000 |
| `start_stabilitaet` | 50.0 |
| `start_steuersatz` | 10 |
| `grundwohnraum` | 1200 |
| `start_bestand` | erz 2000, kristall 1200, deuterium 400, nahrung 800 |
| `start_gebaeude` | erzmine 1, kristallmine 1, farm 1, solarkraftwerk 1, lager 1, wohnblock 1 |
| `max_gebaeudestufe` | 40 |

## zonen

Planetenzonen mit Feldern und Ertragsfaktoren.

| | von | bis | felder_min | felder_max | solar | deuterium | nahrung |
|---|---|---|---|---|---|---|---|
| **glut** | 1 | 3 | 80 | 140 | 1.4 | 0.6 | 0.7 |
| **leben** | 4 | 8 | 150 | 230 | 1.0 | 1.0 | 1.3 |
| **frost** | 9 | 12 | 100 | 180 | 0.7 | 1.5 | 0.6 |

## voelker

Die vier Völker mit ihren Abweichungen vom Grundwert (1.0 = keine Abweichung).

| | ladung | marktgebuehr | waffen | panzerung | steuer | werftzeit | forschung | wachstum | nahrung | energie | kolonieschiff | pluenderquote | ohne_nahrung |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| **aurelianer** | 1.2 | 0.5 | 0.9 | 1.0 | 1.1 | 1.0 | 1.0 | 1.0 | 1.0 | 1.0 | 1.0 | – | nein |
| **krath** | 1.0 | 1.0 | 1.15 | 1.0 | 1.0 | 0.85 | 0.85 | 1.0 | 1.0 | 1.0 | 1.0 | 0.5 | nein |
| **veyari** | 1.0 | 1.0 | 1.0 | 0.85 | 1.0 | 1.0 | 1.0 | 1.1 | 1.25 | 1.0 | 0.75 | – | nein |
| **syntheten** | 1.0 | 1.0 | 1.0 | 1.0 | 1.0 | 1.0 | 1.15 | 0.85 | 1.0 | 1.1 | 1.0 | – | ja |

## wirtschaft

Produktion, Lager, Bevölkerung, Steuern, Bauschleifen.

| Wert | Einstellung |
|---|---|
| `wachstum_je_tag` | 0.12 |
| `arbeitsquote` | 0.6 |
| `fachkraefte_basis` | 30 |
| `fachkraefte_je_akademie` | 60.0 |
| `nahrung_je_1000` | 40.0 |
| `konsum_je_1000` | 3.0 |
| `energie_je_1000_syntheten` | 25.0 |
| `hunger_min` | -0.5 |
| `steuer_je_einwohner_stunde` | 0.03 |
| `unterhalt_schiffe_je_tag` | 0.005 |
| `verwaltung_je_kolonie_tag` | 300 |
| `desertion_nach_tagen` | 3 |
| `desertion_anteil` | 0.1 |
| `ertragswachstum` | 1.1 |
| `lager_roh` | 8000 |
| `lager_verarbeitet` | 1500 |
| `lager_selten` | 400 |
| `lager_faktor` | 1.6 |
| `bunker_je_stufe` | 20000 |
| `bunker_verarbeitet_anteil` | 0.2 |
| `bauzeit_teiler` | 2500 |
| `min_bauzeit_sekunden` | 60 |
| `warteschlange` | 5 |
| `forschung_warteschlange` | 3 |
| `rezepte` | legierung: (erz 4.0), elektronik: (erz 1.0, kristall 2.0), konsumgut: (erz 1.0, kristall 1.0, deuterium 0.5) |
| `fusion_deuterium` | 6.0 |
| `energietechnik_bonus` | 0.05 |
| `werkstoffkunde_bonus` | 0.05 |
| `automatisierung_bonus` | 0.04 |
| `agrar_bonus` | 0.08 |
| `xeno_bonus` | 0.1 |
| `terraforming_felder` | 6 |
| `siedler` | 3000 |
| `habitat_wohnraum` | 2000 |
| `kolonien_max` | 8 |
| `bauteile` | antriebskern: (erz 2000, kristall 1000, legierung 200, elektronik 100), habitatmodul: (erz 1500, kristall 500, legierung 150, elektronik 50, konsumgut 100) |

## stabilitaet

Stabilität der Planeten: Ziele, Einflüsse, Unruhen.

| Wert | Einstellung |
|---|---|
| `ziel_basis` | 50.0 |
| `annaeherung_je_stunde` | 0.02847 |
| `konsum_bonus` | 20.0 |
| `wohnraum_bonus` | 10.0 |
| `wohnraum_frei_voll` | 0.2 |
| `soziologie_je_stufe` | 2.0 |
| `steuer_frei_bis` | 15 |
| `steuer_malus_je_punkt` | 1.0 |
| `kolonie_ueber_grenze` | 5.0 |
| `pluenderung_malus` | 5.0 |
| `pluenderung_tage` | 5 |
| `pluenderung_malus_max` | 10.0 |
| `belagerung_je_tag` | 10.0 |
| `belagerung_erholung_je_tag` | 5.0 |
| `wachstum_voll_ab` | 60.0 |
| `wachstum_null_unter` | 40.0 |
| `unruhen_unter` | 30.0 |
| `uebernahme_unter` | 15.0 |
| `produktivitaet_basis` | 0.6 |
| `produktivitaet_spanne` | 0.6 |

## gebaeude

Gebäude: Kosten der ersten Stufe, Kostenfaktor je Stufe, Bedarf, Ertrag, Freischaltung.

| | kosten | faktor | arbeiter | fachkraefte | energie | ab_stufe | ertrag | braucht | wirkung |
|---|---|---|---|---|---|---|---|---|---|
| **erzmine** | erz 60, kristall 15 | 1.5 | 25.0 | 0.0 | 10.0 | 1 | 30.0 |  | fördert Erz |
| **kristallmine** | erz 48, kristall 24 | 1.5 | 25.0 | 0.0 | 10.0 | 1 | 20.0 |  | fördert Kristall |
| **deuteriumsynthesizer** | erz 225, kristall 75 | 1.5 | 25.0 | 0.0 | 20.0 | 1 | 10.0 |  | erzeugt Deuterium |
| **farm** | erz 40, kristall 10 | 1.5 | 15.0 | 0.0 | 4.0 | 1 | 45.0 |  | erzeugt Nahrung |
| **solarkraftwerk** | erz 75, kristall 30 | 1.5 | 4.0 | 0.0 | 0.0 | 1 | 22.0 |  | erzeugt Strom |
| **fusionskraftwerk** | erz 900, kristall 360, deuterium 180 | 1.8 | 10.0 | 2.0 | 0.0 | 2 | 50.0 |  | erzeugt viel Strom, verbraucht Deuterium |
| **giesserei** | erz 600, kristall 200 | 1.6 | 40.0 | 0.0 | 20.0 | 2 | 4.0 |  | verarbeitet Erz zu Legierung |
| **elektronikwerk** | erz 500, kristall 400 | 1.6 | 30.0 | 10.0 | 25.0 | 2 | 2.5 |  | verarbeitet Kristall und Erz zu Elektronik, braucht Fachkräfte |
| **konsumgueterwerk** | erz 400, kristall 300 | 1.6 | 30.0 | 0.0 | 15.0 | 2 | 8.0 |  | stellt Konsumgüter her, die die Bevölkerung verbraucht |
| **xenoextraktor** | erz 20000, kristall 10000, legierung 500, elektronik 200 | 1.6 | 30.0 | 10.0 | 60.0 | 4 | 4.0 |  | fördert Xenokristall, nur auf Planeten in Nebelsystemen |
| **lager** | erz 500 | 2.0 | 2.0 | 0.0 | 0.0 | 1 | 0.0 |  | erhöht die Lagergrenze jedes Guts |
| **bunker** | erz 800, kristall 400 | 2.0 | 2.0 | 0.0 | 0.0 | 2 | 0.0 |  | schützt je Stufe eine feste Menge jedes Guts vor Plünderung |
| **wohnblock** | erz 120, kristall 40 | 1.4 | 3.0 | 0.0 | 3.0 | 1 | 700.0 |  | schafft Wohnraum |
| **akademie** | erz 800, kristall 600 | 1.8 | 10.0 | 0.0 | 10.0 | 2 | 0.0 |  | bildet Fachkräfte aus |
| **markt** | erz 2000, kristall 1000, legierung 50 | 1.8 | 10.0 | 0.0 | 5.0 | 3 | 0.0 |  | erlaubt Marktorders von diesem Planeten |
| **verwaltungszentrum** | erz 4000, kristall 2000, legierung 200, elektronik 100 | 2.0 | 15.0 | 5.0 | 10.0 | 4 | 0.0 |  | trägt eine Kolonie je zwei Stufen ohne Stabilitätsverlust |
| **labor** | erz 200, kristall 400, deuterium 100 | 2.0 | 3.0 | 8.0 | 12.0 | 1 | 10.0 |  | erzeugt Forschungspunkte, braucht Fachkräfte |
| **bauhof** | erz 400, kristall 120 | 2.0 | 15.0 | 0.0 | 5.0 | 1 | 0.0 |  | verkürzt Bauzeiten von Gebäuden |
| **nanofabrik** | erz 100000, kristall 50000, legierung 5000, elektronik 3000 | 2.0 | 10.0 | 20.0 | 100.0 | 4 | 0.0 | bauhof 6 | halbiert je Stufe alle Bau- und Werftzeiten |
| **raumhafen** | erz 600, kristall 300 | 2.0 | 10.0 | 0.0 | 10.0 | 1 | 0.0 |  | nötig für Flottenstarts, spart je Stufe Treibstoff |
| **werft** | erz 400, kristall 200 | 2.0 | 20.0 | 0.0 | 15.0 | 1 | 0.0 | raumhafen 1 | baut Schiffe, höhere Stufen bauen schneller und schalten Schiffstypen frei |
| **orbitalwerft** | erz 8000, kristall 5000, legierung 800, elektronik 300 | 2.0 | 20.0 | 15.0 | 40.0 | 4 | 0.0 | werft 4 | fertigt Antriebskerne und Habitatmodule, braucht Fachkräfte |
| **sensorphalanx** | erz 2000, kristall 2000, elektronik 50 | 1.8 | 3.0 | 3.0 | 20.0 | 3 | 0.0 |  | verlängert je Stufe die Vorwarnzeit vor anfliegenden Flotten |
| **kaserne** | erz 3000, kristall 1000, legierung 200 | 1.8 | 20.0 | 0.0 | 10.0 | 4 | 0.0 |  | stellt die Garnison gegen Invasionen, nötig für Truppentransporter |
| **raketensilo** | erz 12000, kristall 6000, legierung 800, elektronik 200 | 2.0 | 0.0 | 0.0 | 0.0 | 3 | 0.0 |  | lagert Abfang- und Interplanetarraketen; Kapazität und Reichweite steigen je Stufe |
| **orbitalring** | erz 300000, kristall 150000, legierung 30000, elektronik 15000, xenokristall 5000 | 1.0 | 0.0 | 0.0 | 0.0 | 5 | 0.0 |  | Großprojekt, einmal pro Planet: zusätzliche Felder und Wohnraum laut Zusatzregeln |
| **forschungsarchiv** | erz 120000, kristall 250000, elektronik 25000, xenokristall 8000 | 1.0 | 0.0 | 0.0 | 0.0 | 5 | 0.0 |  | Großprojekt, einmal pro Planet: Bonus auf lokale Forschungspunkte laut Zusatzregeln |
| **versorgungsnetz** | erz 250000, kristall 120000, legierung 25000, elektronik 15000, xenokristall 5000 | 1.0 | 0.0 | 0.0 | 0.0 | 5 | 0.0 |  | Großprojekt, einmal pro Planet: Bonus auf lokale Energieerzeugung laut Zusatzregeln |

## forschung

Forschungen: Kosten, Laborstufe, Forschungspunkte, Freischaltung.

| | kosten | faktor | fp | ab_stufe | labor | wirkung |
|---|---|---|---|---|---|---|
| **energietechnik** | kristall 400, deuterium 200 | 2.0 | 150.0 | 1 | 1 | +5 % Stromerzeugung je Stufe |
| **werkstoffkunde** | erz 800, kristall 400 | 2.0 | 300.0 | 2 | 2 | +5 % Legierungsausbeute je Stufe |
| **automatisierung** | erz 600, kristall 800 | 2.0 | 400.0 | 2 | 2 | -4 % Arbeitskräfte je Gebäude und Stufe, höchstens -50 % |
| **agrarwissenschaft** | erz 300, kristall 200 | 2.0 | 150.0 | 1 | 1 | +8 % Nahrung je Stufe |
| **soziologie** | kristall 600, deuterium 200 | 2.0 | 300.0 | 2 | 2 | +2 Punkte Stabilitätsziel je Stufe |
| **verbrennungsantrieb** | erz 200, deuterium 150 | 2.0 | 150.0 | 1 | 1 | +10 % Tempo je Stufe für Schiffe mit Verbrennungsantrieb |
| **impulsantrieb** | erz 2000, kristall 4000, deuterium 600 | 2.0 | 800.0 | 3 | 3 | +20 % Tempo je Stufe für Schiffe mit Impulsantrieb |
| **hyperraumantrieb** | erz 10000, kristall 20000, deuterium 6000, xenokristall 1000 | 2.0 | 6000.0 | 4 | 6 | +30 % Tempo je Stufe für Schiffe mit Hyperraumantrieb, braucht Xenokristall |
| **astrophysik** | erz 4000, kristall 8000, deuterium 4000 | 1.75 | 2000.0 | 3 | 3 | Stufe 1 erlaubt eine Kolonie, jede zweite weitere Stufe eine mehr |
| **logistik** | erz 1000, kristall 1000 | 2.0 | 500.0 | 3 | 2 | +5 % Ladekapazität je Stufe |
| **computertechnik** | kristall 400, deuterium 600 | 2.0 | 400.0 | 2 | 1 | eine gleichzeitige Flotte mehr je Stufe |
| **waffentechnik** | erz 800, kristall 200 | 2.0 | 400.0 | 2 | 2 | +10 % Angriff je Stufe |
| **schildtechnik** | erz 200, kristall 600 | 2.0 | 400.0 | 2 | 2 | +10 % Schild je Stufe |
| **panzerung** | erz 1000 | 2.0 | 400.0 | 2 | 2 | +10 % Struktur je Stufe |
| **spionagetechnik** | erz 100, kristall 300, deuterium 100 | 2.0 | 150.0 | 1 | 1 | genauere Spionageberichte, bessere Abwehr fremder Sonden |
| **xenomaterialkunde** | kristall 10000, xenokristall 500 | 2.0 | 4000.0 | 4 | 5 | +10 % Xenokristallförderung je Stufe |
| **terraforming** | kristall 50000, deuterium 100000, elektronik 2000 | 2.0 | 8000.0 | 4 | 6 | +6 Felder je Stufe auf jedem eigenen Planeten |

## einheiten

Schiffe und Verteidigungsanlagen: Kosten, Kampfwerte, Ladung, Tempo, Schnellfeuer.

| | kosten | struktur | schild | angriff | ladung | tempo | antrieb | verbrauch | besatzung | ab_stufe | werft | braucht | schnellfeuer | max_anzahl | wirkung |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| **spionagesonde** | kristall 300 | 1000 | 0 | 0 | 0 | 100000 | verbrennungsantrieb | 1 | 0 | 1 | 1 |  |  | 0 | Aufklärung, sehr schnell, keine Besatzung |
| **kleiner_transporter** | erz 2000, kristall 2000 | 4000 | 10 | 5 | 5000 | 10000 | verbrennungsantrieb | 10 | 5 | 2 | 1 |  |  | 0 | schneller Frachter |
| **grosser_transporter** | erz 6000, kristall 6000, legierung 50 | 12000 | 25 | 5 | 25000 | 7500 | verbrennungsantrieb | 50 | 15 | 3 | 2 |  |  | 0 | großer Frachter |
| **leichter_jaeger** | erz 3000, kristall 1000 | 4000 | 10 | 50 | 50 | 12500 | verbrennungsantrieb | 20 | 2 | 2 | 1 |  |  | 0 | billige Masse |
| **bergbauschiff** | erz 8000, kristall 4000, deuterium 2000, legierung 100 | 12000 | 20 | 5 | 10000 | 4000 | verbrennungsantrieb | 40 | 20 | 3 | 2 |  |  | 0 | baut in Asteroidengürteln Erz und Kristall ab |
| **kreuzer** | erz 20000, kristall 7000, deuterium 2000, legierung 300, elektronik 100 | 27000 | 50 | 400 | 800 | 15000 | impulsantrieb | 300 | 30 | 4 | 4 |  | spionagesonde 5, leichter_jaeger 6, raketenwerfer 10 | 0 | Schnellfeuer gegen leichte Jäger und Raketenwerfer |
| **recycler** | erz 10000, kristall 6000, deuterium 2000, legierung 100 | 16000 | 10 | 1 | 20000 | 2000 | verbrennungsantrieb | 300 | 15 | 4 | 3 |  |  | 0 | sammelt Trümmerfelder ein |
| **kolonieschiff** | erz 10000, kristall 20000, deuterium 10000, legierung 400, elektronik 200, antriebskern 1, habitatmodul 2 | 30000 | 100 | 50 | 7500 | 2500 | impulsantrieb | 1000 | 0 | 4 | 4 | orbitalwerft 1 |  | 0 | gründet eine Kolonie, nimmt Siedler aus der Bevölkerung mit |
| **truppentransporter** | erz 8000, kristall 4000, deuterium 2000, legierung 200 | 15000 | 30 | 10 | 2000 | 6000 | impulsantrieb | 200 | 500 | 4 | 3 | kaserne 1 |  | 0 | bringt Bodentruppen für Invasionen, die Truppen zählen als Besatzung |
| **schlachtschiff** | erz 45000, kristall 15000, legierung 1000, elektronik 300, xenokristall 20 | 60000 | 200 | 1000 | 1500 | 10000 | hyperraumantrieb | 500 | 100 | 5 | 6 |  | spionagesonde 5 | 0 | Kern schwerer Flotten |
| **bomber** | erz 50000, kristall 25000, deuterium 15000, legierung 1000, elektronik 500, xenokristall 50 | 75000 | 500 | 1000 | 500 | 5000 | hyperraumantrieb | 1000 | 80 | 5 | 7 |  | raketenwerfer 20, lasergeschuetz 20, ionengeschuetz 10, gausskanone 5 | 0 | Schnellfeuer gegen Verteidigungsanlagen |
| **zerstoerer** | erz 60000, kristall 50000, deuterium 15000, legierung 2000, elektronik 1000, xenokristall 200 | 110000 | 500 | 2000 | 2000 | 5000 | hyperraumantrieb | 1000 | 120 | 5 | 8 | orbitalwerft 3 | kreuzer 3, schlachtschiff 2, lasergeschuetz 10 | 0 | sehr stark, braucht viel Xenokristall |
| **raketenwerfer** | erz 2000 | 6000 | 60 | 240 | 0 | 0 | – | 0 | 0 | 2 | 0 |  |  | 0 | billige Verteidigung |
| **lasergeschuetz** | erz 1500, kristall 500 | 6000 | 75 | 300 | 0 | 0 | – | 0 | 0 | 2 | 0 |  |  | 0 | billige Verteidigung |
| **ionengeschuetz** | erz 5000, kristall 3000, elektronik 30 | 24000 | 1500 | 450 | 0 | 0 | – | 0 | 0 | 3 | 0 |  |  | 0 | starker Schild |
| **gausskanone** | erz 20000, kristall 15000, deuterium 2000, legierung 200 | 105000 | 600 | 3300 | 0 | 0 | – | 0 | 0 | 4 | 0 |  |  | 0 | schwere Verteidigung |
| **plasmawerfer** | erz 50000, kristall 50000, deuterium 30000, legierung 1000, elektronik 500 | 300000 | 900 | 9000 | 0 | 0 | – | 0 | 0 | 5 | 0 |  |  | 0 | schwerste Verteidigung |
| **planetenschild** | erz 50000, kristall 50000, legierung 2000, elektronik 1000, xenokristall 100 | 300000 | 30000 | 3 | 0 | 0 | – | 0 | 0 | 5 | 0 |  |  | 1 | sehr starker Schild, nur einer je Planet |

## stufen

Zivilisationsstufen II bis V: Bedingungen, Kosten, Freischaltungen.

| | name | einwohner | gebaeude | forschung | versorgung_plus | stabilitaet | konsum_deckung | kolonien | kosten | schaltet_frei |
|---|---|---|---|---|---|---|---|---|---|---|
| **2** | II Industrie | 2000 | erzmine 5, kristallmine 5 |  | ja | 0.0 | 0.0 | 0 | erz 2000, kristall 1000 | Gießerei, Elektronikwerk, Konsumgüterwerk, Fusionskraftwerk, Akademie, Bunker, leichte Jäger, kleine Transporter, Raketenwerfer, Lasergeschütz |
| **3** | III Orbit | 8000 | giesserei 3, elektronikwerk 2 | energietechnik 3 | nein | 55.0 | 0.0 | 0 | erz 10000, kristall 6000, legierung 500, elektronik 200 | Markt, Sensorphalanx, Raketensilo, große Transporter, Bergbauschiffe, Ionengeschütz |
| **4** | IV Sternenflug | 32000 | werft 4 | impulsantrieb 3, astrophysik 1 | nein | 60.0 | 0.8 | 0 | erz 40000, kristall 25000, deuterium 10000, legierung 2000, elektronik 1000 | Orbitalwerft, Kolonieschiff, Kreuzer, Recycler, Truppentransporter, Kaserne, Verwaltungszentrum, Nanofabrik, Xenoextraktor, Gaußkanone |
| **5** | V Imperium | 700000 |  | hyperraumantrieb 1 | nein | 0.0 | 0.0 | 3 | erz 200000, kristall 150000, deuterium 50000, legierung 10000, elektronik 5000, xenokristall 5000 | Orbitalring, Forschungsarchiv, Versorgungsnetz, Schlachtschiff, Bomber, Zerstörer, Plasmawerfer, Planetenschild |

Eine 0 oder ein leeres Feld heißt: keine Bedingung dieser Art. Haltezeit: alle Bedingungen müssen 48 Stunden ununterbrochen erfüllt sein, dann gilt der Aufstieg (Aktion `stufenaufstieg`, die Kosten werden dabei abgebucht).

## flug

Flugzeiten, Treibstoff, Haltedauern.

| Wert | Einstellung |
|---|---|
| `zeitfaktor` | 4 |
| `min_sekunden` | 1200 |
| `system_basis` | 1000 |
| `je_position` | 5 |
| `sektor_basis` | 2700 |
| `je_system` | 95 |
| `je_sektor` | 20000 |
| `treibstoff_teiler` | 35000 |
| `antrieb_bonus` | verbrennungsantrieb 0.1, impulsantrieb 0.2, hyperraumantrieb 0.3 |
| `logistik_bonus` | 0.05 |
| `raumhafen_ersparnis` | 0.04 |
| `raumhafen_ersparnis_max` | 0.4 |
| `halten_max_stunden` | 168 |
| `abbau_max_stunden` | 48 |
| `abbau_erz_je_stunde` | 150 |
| `abbau_kristall_je_stunde` | 100 |

## kampf

Kampfrunden, Beute, Trümmer, Eroberung.

| Wert | Einstellung |
|---|---|
| `runden` | 6 |
| `verpuffen_anteil` | 0.01 |
| `explosion_unter` | 0.7 |
| `truemmer_anteil` | 0.3 |
| `verteidigung_wiederaufbau` | 0.7 |
| `pluenderquote` | 0.4 |
| `tech_je_stufe` | 0.1 |
| `warnzeit_basis_minuten` | 30 |
| `warnzeit_je_phalanx_minuten` | 30 |
| `truppen_je_transporter` | 500 |
| `garnison_je_kaserne` | 300 |
| `garnison_je_100_einwohner` | 1 |
| `eroberung_stufenverlust` | 0.3 |
| `eroberung_bevoelkerung` | 0.5 |
| `simulator_laeufe` | 100 |

## diplomatie

Verträge, Kautionen, Nachrichten, Allianzen.

| Wert | Einstellung |
|---|---|
| `kuendigungsfrist_stunden` | 48 |
| `allianz_max` | 8 |
| `buendnisse_max` | 3 |
| `anfaengerschutz_tage` | 10 |
| `anfaengerschutz_bis_stufe` | 3 |
| `schwachenschutz_anteil` | 0.0 |

## markt

Marktgebühren und Lieferzeiten.

| Wert | Einstellung |
|---|---|
| `gebuehr` | 0.02 |
| `orders_je_marktstufe` | 5 |
| `liefertempo` | 10000 |

## wertung

Punkte: Werteinheiten, Gewichte, Stufenbonus.

| Wert | Einstellung |
|---|---|
| `einheit` | 1000 |
| `gewichte` | erz 1.0, kristall 1.5, deuterium 2.0, nahrung 1.0, legierung 5.0, elektronik 8.0, konsumgut 4.0, xenokristall 20.0, antriebskern 5300.0, habitatmodul 3800.0 |
| `einwohner_je_punkt` | 100 |
| `stufenbonus` | 0, 100, 400, 1500, 5000 |

## agenten

Rollen der Modelle: Takt, früheste Aufrufe, Grenzen je Aufruf.

| Wert | Einstellung |
|---|---|
| `takt_stunden` | stratege 24, verwalter 4, feldherr 12, diplomat 24 |
| `mindestabstand_stunden` | 1 |
| `frueh_anteil` | 0.5 |
| `doktrin_zeichen` | 2400 |
| `notiz_zeichen` | 6000 |
| `nachricht_zeichen` | 1200 |
| `nachrichten_je_tag` | 20 |
| `aktionen_je_aufruf` | 10 |
| `abfragen_je_aufruf` | 3 |
| `start_anteile` | wirtschaft 60, militaer 10, forschung 15, reserve 15 |
| `chronik_tage` | 7 |

## zusatz

Raketen und Großprojekte der Stufe V.

| Wert | Einstellung |
|---|---|
| `silo_plaetze_je_stufe` | 10 |
| `raketen_kosten` | erz 2000, deuterium 500, erz 5000, kristall 1000, deuterium 2000 |
| `raketen_bauzeit_sekunden` | 300 |
| `raketen_flug_min_sekunden` | 1200 |
| `raketen_flug_je_system_sekunden` | 60 |
| `raketen_reichweite_je_silo` | 5 |
| `raketen_schaden` | 12000 |
| `orbitalring_felder` | 50 |
| `orbitalring_wohnraum` | 50000 |
| `archiv_fp_bonus` | 0.25 |
| `versorgungsnetz_energie_bonus` | 0.25 |

## aktionen

Welche Rolle welche Aktion senden darf (Antwortschema der Engine, `kern::aktion`). Felder und Prüfungen jeder Aktion: `docs/AGENTEN-SCHNITTSTELLE.md`.

| Rolle | Aktionen |
|---|---|
| stratege | `doktrin`, `meldung`, `stufenaufstieg` |
| verwalter | `abreissen`, `bauen`, `fertigen`, `flotte_senden`, `flotte_versorgen`, `flotte_zurueckrufen`, `forschen`, `intern_anbieten`, `intern_kaufen`, `intern_storno`, `markt_order`, `markt_storno`, `meldung`, `prioritaeten`, `raketen_bauen`, `reparieren`, `schleife_leeren`, `steuersatz`, `stufenaufstieg` |
| feldherr | `allianz_hilfe`, `allianz_hilfe_status`, `fertigen`, `flotte_ausspaehen`, `flotte_senden`, `flotte_versorgen`, `flotte_zurueckrufen`, `meldung`, `raketen_bauen`, `raketen_starten`, `verband_beitreten`, `verband_oeffnen` |
| diplomat | `allianz_anfrage`, `allianz_ausschliessen`, `allianz_beitreten`, `allianz_einladen`, `allianz_gruenden`, `allianz_hilfe`, `allianz_hilfe_status`, `allianz_pakt_kuendigen`, `allianz_rolle`, `allianz_verlassen`, `brief_lesen`, `brief_senden`, `diplomatie_anfrage`, `diplomatie_entscheiden`, `intern_anbieten`, `intern_kaufen`, `intern_storno`, `meldung`, `nachricht`, `schenken`, `vertrag_ablehnen`, `vertrag_anbieten`, `vertrag_annehmen`, `vertrag_kuendigen` |
| alle | `abreissen`, `allianz_anfrage`, `allianz_ausschliessen`, `allianz_beitreten`, `allianz_einladen`, `allianz_gruenden`, `allianz_hilfe`, `allianz_hilfe_status`, `allianz_pakt_kuendigen`, `allianz_rolle`, `allianz_verlassen`, `bauen`, `brief_lesen`, `brief_senden`, `diplomatie_anfrage`, `diplomatie_entscheiden`, `doktrin`, `fertigen`, `flotte_ausspaehen`, `flotte_senden`, `flotte_versorgen`, `flotte_zurueckrufen`, `forschen`, `intern_anbieten`, `intern_kaufen`, `intern_storno`, `markt_order`, `markt_storno`, `meldung`, `nachricht`, `prioritaeten`, `raketen_bauen`, `raketen_starten`, `reparieren`, `schenken`, `schleife_leeren`, `steuersatz`, `stufenaufstieg`, `verband_beitreten`, `verband_oeffnen`, `vertrag_ablehnen`, `vertrag_anbieten`, `vertrag_annehmen`, `vertrag_kuendigen` |
