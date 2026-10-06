---
layout: default
title: "Menschenmodus und Qwen-Bildernacht · 5. Oktober 2026"
---

# Menschenmodus und Qwen-Bildernacht · 5. Oktober 2026

Karl hat in diesem Chat ausdrücklich die nächtliche GPU-Produktion über ComfyUI und Qwen Image 2.1 sowie die Menschenmodus-Integration beauftragt. Keine Modellgewichte installiert oder heruntergeladen. Alles unter D:.

## Produktion und Fortsetzung

- Katalog: 400 Motive. Ursprünglich 398; die inzwischen im Kern vorhandenen Missionen Bombardieren und Kampfkolonisieren ergänzt. Bestehende IDs und Workflows unverändert.
- Runtime: vorhandenes `D:/HRM-Models/comfyui-cu130-venv/Scripts/python.exe`, `D:/ComfyUI/main.py`, Port 8191. Compiler deaktiviert, 4 GiB VRAM für Desktop reserviert, VAE auf CPU; originale Qwen-Image-2.1-INT8-Gewichte und Qwen3-VL-Encoder. Keine anderen Anwendungen beendet. Alte externe Trellis-Reservierung bleibt unverändert; die aktuelle explizite Freigabe steht in `content/production.json`.
- Erstes Pilotporträt visuell angesehen: brauchbare Ausrichtung und Stil. Erstes finales aurelianisches Bergbauschiff visuell angesehen: vollständige Silhouette, deutliche Frachtausrüstung, richtige Farben, ohne Beschriftung. Kein Anspruch, dass damit alle 400 Bilder gesichtet sind.
- Serienworker: `tools/content/night_production.py`. Priorität Schiffe → Gebäude → Forschung → Verteidigung → übrige Kategorien. Jeweils ein Auftrag, Wiederaufnahme über gespeicherte prompt_id, kein Löschen fremder Queue und kein blinder Neuversuch bei unklarem POST.
- Wahrer Fortschritt: `content/night-status.json`; Job-/Hash-Nachweise: `content/batch-state.json` und individuelle `*.provenance.json`. Protokolle: `content/night-production.log`, `content/night-production.err`, `content/runtime/comfy.log`.
- Worker bei laufendem ComfyUI versteckt mit Python starten. Vorher `.batch.lock` und Prozessidentität prüfen. Alte Sperre nur entfernen, wenn deren PID nachweislich nicht mehr der Worker ist. Bei laufendem/unklarem Job immer vorhandene prompt_id aus der History übernehmen. Keine neue Submission derselben Arbeit.
- Ein gezielter Worker-Neustart hat die Auftragsliste von 398 auf 400 erweitert; ComfyUI und der aktive Renderauftrag wurden dabei weiterlaufen gelassen und aus der History wieder aufgenommen.
- App-Überwachung alle 30 Minuten: `sternenepoche-bildernacht-pr-fen`, still bei normalem Fortschritt. Nach Abschluss deaktivieren.

## Spieländerungen

- Neue Partien: pausiert, Tempo 1× Echtzeit, aktuelle Kolonisationsregeln v2. Alte Spielstände behalten ihre Regelversion.
- Menschliche Aktionen werden sofort vom Regelkern geprüft, auch während laufender oder gescheiterter Modellantworten. Die Weltuhr wartet auf Modellantworten; Bau, Flüge und Wirtschaft werden weiterhin in 15-Minuten-Fenstern fortgeschrieben.
- Alle vier Modellrollen und alle Skriptgegner im Menschenmodus werden alle 15 Spielminuten aufgerufen. Der autonome Forschungslauf behält seinen eigenen Zeitplan. Der Modellprompt erklärt diese Abweichung und die aktuelle Kolonisationsversion.
- Unveränderter Beobachtungsstand für Modellwiederholungen. Bei Abschluss werden Modellaktionen gegen die aktuelle Welt geprüft; kein Überschreiben durch die Arbeitskopie. Neue Objektkennungen aus der parallelen Kopie dürfen nicht versehentlich andere aktuelle Flotten/Verträge/Orders treffen; entsprechende Befehle verlangen eine neue Beobachtung.
- Native Bildzuordnung anhand Kategorie, Engine-Schlüssel und Volk. PNG-Herkunftshash, Asset-ID und Maße werden vor Anzeige geprüft. Motive an Gebäuden, Forschung, Schiffen, Verteidigung, Missionen, Zivilisationsstufen und Volksübersicht; alle anderen Motive in der vollständigen Grafikbibliothek. Fertige Bilder werden ohne Neubau des Programms nachgeladen. Fehlende Motive: „Bild folgt“.
- Reparaturansicht mit Integrität, Preis, Dauer und Knopf; Auswahl einer eigenen Orbitflotte als Transportempfänger. Neue Missionen nur bei passenden Kolonisationsregeln auswählbar. Legacy-Invasion nur in alten Partien.
- Skriptbot-Kolonisierung berücksichtigt bei aktuellen Regeln Sondenaufklärung, bewaffnete Eskorte und regelbasierte Startfracht. Historische Balancepartien behalten den bisherigen Pfad.

## Abnahme

Der gemeinsame Prüflauf `cargo test --release -p kern -p spieler -p lauf -p inhalt -p sternenepoche-agenten` besteht: Regelkern einschließlich Kolonisation v2, Aktionsschema, Flotten und Replay; Inhalt 15 Tests; native UI 20 Tests; Menschenmodus-Konfliktprüfung und Wiederholung; Bot-Aufklärung und eskortierte Kolonisierung; Agenten-/Laborintegration. Sieben Tests für externe Docker-/native Sandbox-Umgebungen bleiben wie vorgesehen ignoriert. Content-Python 10/10 und Content-JavaScript 9/9 bestehen ebenfalls. Der abschließende gezielte Spieler-Prüflauf besteht mit 13 Session- und 20 UI-Tests, einschließlich des 15-Minuten-Takts aller Skriptgegner. Das Release-Programm wurde anschließend neu gebaut.

Auch gefundene Altfehler sind behoben: fehlendes Regeltext-Beispiel für `flotte_versorgen`, veraltete generierte Regeldokumentation und eine Schema-Testwelt, die die vorausgesetzten neuen Planungsregeln noch nicht aktiviert hatte. Die Spielregeln selbst wurden dabei nicht umbalanciert.

## Nach dem Rendering

1. Alle 400 Final-Jobs vollständig; jede Katalogdatei und Provenienz auf ID, Maße, Workflow und SHA-256 prüfen. Keine Piloten als Finalbilder zählen.
2. Motive mehrerer Kategorien und aller vier Völker visuell mit view_image prüfen. Nur tatsächlich gesichtete Dateien als reviewed markieren. Fehlgeschlagene oder unpassende Bilder separat dokumentieren; bestehende Dateien nicht still überschreiben.
3. Native Galerie und Werft mit echten Bildern erneut aufnehmen, bei Bedarf neu bauen. Spielanleitung `docs/SPIELEN.md` und neuer Starter `Spielen.cmd` bleiben Einstieg.
4. Abschließenden Produktionsstand hier ergänzen und Karl kurz informieren. Bis dahin ist die Bildproduktion ausdrücklich laufend, nicht fertig.


### Sichtbare Abnahme vor Übergabe an den Nachtlauf
Native Screenshots `content/runtime/player-gallery-final.png` und `content/runtime/player-werft-final.png` wurden tatsächlich geöffnet und geprüft: erzeugte Schiffe sind in der Galerie sichtbar und in der Werft dem Spieler-Volk Syntheten zugeordnet; Werte, Sperrbedingungen und Hilfetexte bleiben lesbar. Fehlende Bilder sind sichtbar gekennzeichnet. Bei dieser Abnahme waren 21 von 400 Finalbildern vorhanden; der Audit meldete keine Fehler. Das ist eine Zwischenaufnahme, der aktuelle Stand steht weiterhin in `content/night-status.json`.

Vollständige Prüfung nach Ende: `C:/Python313/python.exe tools/content/audit_assets.py --require-complete`. Sie schreibt `content/audit-status.json` und prüft jedes Finalbild gegen Katalog, Workflow, Maße, Asset-ID und Herkunftshash.


### Offener visueller Befund (Stichprobe 05.10., 04:28 MESZ)
Astrophysik und neutrale Gausskanone wurden visuell geprueft und sind passend. `defenses.gausskanone.aurelianer` wirkt hingegen wie ein bewaffnetes Raumschiff statt einer stationaeren Verteidigung. Provenienz traegt `review_result: needs_revision`. Nach dem laufenden Erstlauf vor Abschluss separat korrigieren: Original und Nachweise archivieren, eindeutigen stationaeren Bodensockel im Prompt vorgeben und Schiffsrumpf ausschliessen; keine PNG still ueberschreiben. Weitere volksspezifische Verteidigungen daraufhin stichprobenartig ansehen. Abschluss bleibt bis zur Behebung dieses Befunds offen.

Folgestichprobe 05.10., 04:58 MESZ: Gausskanone Krath, Syntheten und Veyari vollstaendig mit view_image betrachtet. Alle drei haben erkennbare stationaere Sockel und passende Volksmerkmale; `review_result: passed`. Der offene Revisionsbefund betrifft weiterhin die aurelianische Variante.


### Fortsetzung im Chat Nachtlauf lueckenlos fortsetzen (05.10., ab 05:22 MESZ)
- Karl: ohne Luecken/Pause bis fertig weiterarbeiten; bei Speicherfehlern gezielt Platz schaffen. D: derzeit ca. 235 GB frei, keine Bereinigung notwendig.
- Alle 48 Schiffe vorhanden und Hashes korrekt. Neue Bilder wandern nach progression bzw. role_portraits; ships ist bereits fertig.
- Bestehende Heartbeat-Ueberwachung auf diesen Chat 01a10a14-b2c7-7330-80ee-b9766c2f5133 umgestellt, alle 5 statt 30 Minuten; keine zweite Automation.
- Verteidigungs-Kontaktboegen content/runtime/qa/defenses-1.jpg bis defenses-4.jpg betrachtet. Gausskanone, Ionengeschuetz und Planetenschild der Aurelianer einzeln im Vollbild geprueft: Schiffsrumpf statt stationaerer Anlage. Alle drei Original-Provenienzen needs_revision.
- Drei neue Kandidaten separat unter content/revisions/defenses-v2 vorbereitet. Versteckter Worker: C:/Python313/python.exe -u tools/content/revision_production.py defenses-v2. Zustand/Lock/Log liegen in diesem Revisionsordner. Eigener Jobzustand; keine Originale ueberschrieben. Enqueue nur bei leerer Queue oder unmittelbar hinter genau einem durch prompt_id identifizierten Hauptlauf-Job ohne weitere Wartende. ComfyUI rendert weiter strikt seriell.
- qwen_batch besitzt fuer diesen Nachlauf eine optionale queue_after_prompt-Pruefung; Standardbetrieb weiterhin nur bei leerer Queue. 12 Python-Pruefungen bestanden, einschliesslich Fremdqueue und unklarem POST ohne doppelte Submission.
- audit_assets meldet visuelle Restbefunde unter visual_issues und verweigert --require-complete bei offenen needs_revision-Befunden.
- Nach Erzeugung: alle drei Kandidaten wirklich mit view_image ansehen. Nur bestandene Kandidaten als reviewed/passed markieren. Erst nach Hauptlauf-Ende unter .batch.lock Originalbild, Provenienz, Workflow, Katalogeintrag und Job archivieren; dann neue Datei, Provenienz, Final/Pilot-Workflow, Katalog JSON/JS und Batch-Eintrag konsistent uebernehmen. Generator-Prompt-Overrides dauerhaft erhalten. Anschliessend Vollaudit und native Ansicht. Keine Kandidaten ohne Sichtpruefung veroeffentlichen.


### Erweiterte Sichtpruefung und automatische Abschlusskette (05.10., ca. 05:40 MESZ)
Alle 48 Schiffe, 140 Gebaeude und 30 Verteidigungen wurden auf Kontaktboegen wirklich angesehen (content/runtime/qa/ships-01..04.jpg, buildings-01..12.jpg, defenses-1..4.jpg). Uebersichtsabnahmen sind als overview_passed gekennzeichnet, nicht als Pixel-/Vollbildpruefung. 45 eindeutige Motivfehler muessen korrigiert werden: drei aurelianische Verteidigungen sowie 42 weitere Motive (36 Gebaeude, vier Spionagesonden, zwei Krath-Verteidigungen). Problem: die urspruenglichen langen Volkstexte dominierten haeufig den Gebaeudeauftrag; Aurelianer wurden zu Schiffen, Krath zu Laufrobotern. Die Korrekturprompts setzen das eigentliche Objekt an den Anfang, beschraenken Volksmerkmale auf Materialien/Farben und definieren klare Fundamente bzw. Sondenform. Kein ComfyUI-Enginefehler behauptet.

Revisionsordner:
- content/revisions/defenses-v2: drei Kandidaten erzeugt, alle einzeln im Vollbild betrachtet und passed. Sie zeigen nun echte Bodenfundamente.
- content/revisions/motif-corrections-v2: 42 Korrekturen in Produktion; review-findings.json listet die konkreten Befunde. Akademie, Bauhof, Deuteriumsynthesizer und Elektronikwerk Aurelianer sind bereits einzeln im Vollbild geprueft und passed. Weitere Kandidaten nach Erstellung ansehen.
- Beide Ordner haben eigene catalog.json, workflows/final, batch-state.json, status.json und production.log/.err. Keine Originals oder Hauptlauf-Workflows geaendert. Der Hauptlauf darf waehrend der seriellen Korrekturen waiting melden; das ist kein Haenger, solange die ComfyUI-Queue einen bekannten Revisionsjob abarbeitet. Gesamtfortschritt steht jetzt in content/production-progress.json.

Versteckter Abschlusswaechter: C:/Python313/python.exe -u tools/content/production_watch.py --watch; PID in .production-watch.lock, Protokolle content/production-watch.log/.err. Er aktualisiert den kombinierten Stand alle 15 Sekunden, erzeugt aber selbst keine Bilder und vergibt keine Sichtfreigaben. Sobald Hauptlauf complete und ALLE Kandidaten tatsaechlich reviewed=true UND review_result=passed sind, ruft er publish_revisions.py je Revisionsordner auf, archiviert Original-PNG, Provenienz, Katalogeintrag, Workflow und Job unter content/revision-archive, uebernimmt konsistent die geprueften Kandidaten samt JSON/JS-Katalog, Workflows und Batch-Nachweisen und fuehrt das Vollaudit aus. Erst dann meldet er ready_for_ui_review und endet. Das ist noch nicht die native UI-Abnahme.

Die finalen Prompt-/Seed-Korrekturen bleiben in content/art-overrides.json; build_catalog.py uebernimmt sie bei spaeterem Neuaufbau. Publikation ist gegen aktiven Hauptlauf gesperrt, prueft Kandidatenhash und Workflow, ueberschreibt keine ungepruefte Variante und behaelt das Originalarchiv auch bei Wiederaufnahme. 16 Content-Python-Tests bestanden, darunter Fremdqueue, verlorene POST-Antwort, Archivierung, Wiederaufnahme, Hashfehler und Nichtveroeffentlichung ungesichteter Kandidaten.

Heartbeat-Folgearbeit: Bei jeder Runde die neuen Revisionsbilder wirklich ansehen und nur passende Provenienzen reviewed=true/review_result=passed markieren. Nicht den Katalog oder batch-state des laufenden Hauptworkers parallel editieren. Waechter/Worker bei Fehler nur nach gepruefter Prozessidentitaet wiederaufnehmen. Wenn Waechter ready_for_ui_review meldet, erneutes Audit --require-complete, volle PNG-Decodierung und native CLI-Screenshots erzeugen/ansehen. Native Anwendung unter target/release/sternenepoche-spieler.exe bietet --screen NAME --screenshot ABSOLUTER_PFAD; keine echten Spielstaende laden/veraendern. Erst nach Abnahme erfolgreich abschliessen und Heartbeat deaktivieren. Bis dahin weiterarbeiten.


### Weitere Befunde und vorbeugende Promptkorrektur (05.10., ca. 05:50 MESZ)
Stand der erweiterten Abnahme: alle 267 zu diesem Zeitpunkt vorhandenen Motive tatsaechlich auf Kontaktboegen angesehen, 62 needs_revision, keine technischen Hash-/Zuordnungs-/Workflowfehler. Der Status reviewed bedeutet nur gesehen, nicht automatisch bestanden; overview_passed unterscheidet Uebersichts- von Vollbildabnahme. people-research-01..06.jpg enthaelt 49 Portraet-/Forschungs-/Stufenmotive.

Dritter Revisionsordner content/revisions/civilization-corrections-v2: 17 Kandidaten vorbereitet und eigener versteckter revision_production.py-Worker gestartet (wartet kooperativ bei fremder bzw. anderer Revisionsqueue). Davon 13 Zivilisationsstufen mit falschem dominierendem Schiff/Riesenroboter sowie Diplomat Krath und Feldherr Krath/Syntheten/Veyari mit nicht getroffener Volksidentitaet. Alle Originalbefunde in Provenienzen markiert. Gesamt nun 3+42+17=62 Korrekturen. Hauptlauf bleibt 400 Motive, Korrekturen sind keine zusaetzlichen Katalog-IDs.

26 garantiert noch nicht eingereichte Motive vorab korrigiert: Stratege/Verwalter der drei nichtmenschlichen Voelker, vier Garnisonen, zwoelf Kolonie-Landschaften und vier Hauptstaedte. Vor jedem dieser Motive beide Profile auf vorhandene Batchjobs geprueft; KEIN eingereichter Workflow veraendert. Alte Katalogeintraege unter content/revision-archive/preflight-prompts archiviert. Final/Pilot-Graph und Katalog JSON/JS konsistent aktualisiert, art-overrides.json enthaelt alle 26 Korrekturen. Damit werden bekannte Fehler vermieden, bevor GPU-Zeit verbraucht wird. 16 Python-Pruefungen danach erneut bestanden.

Der Abschlusswaechter wurde nach Erweiterung gezielt neu gestartet; ComfyUI und alle Renderworker liefen durch. Er decodiert am Ende zusaetzlich jedes PNG vollstaendig mit Pillow. audit_assets.py prueft nun auch den Finaljob gegen Provenienz (Status, ID, Profil, Pfad, Bild-/Workflowhash und prompt_id).

Wichtig fuer Abschluss: Alle DREI Revisionsordner nach neuen PNGs durchsuchen, im Vollbild oder eindeutig beschrifteten Boegen ansehen, only truly passed Kandidaten freigeben. Kandidatenstand und Freigaben stehen in content/production-progress.json; eine Warteanzeige des Hauptworkers ist waehrend der Korrekturen normal. Nicht voreilig bei 400 PNGs fertig melden.

Native Bildcache-Grenze: Bereits geladene PNGs bleiben im laufenden Player als Textur gespeichert. Neue fehlende Motive werden automatisch nachgeladen, aber bereits geladene spaeter ersetzte Motive erfordern einen Player-Neustart. Finale CLI-Screenshots starten ohnehin einen frischen Player. In diesem Chat wurde kein UI-Cache-Code geaendert.

Letzter direkter Pruefstand dieses Durchgangs: 17 Python-Tests bestanden (zusaetzlich Audit verweigert abweichenden Finaljob und meldet visuelle Regression). In motif-corrections-v2 jetzt die ersten 12 aurelianischen Gebaeudekandidaten von Akademie bis Lager einzeln angesehen und passed; in civilization-corrections-v2 Gruendung Aurelianer einzeln angesehen und passed. Die drei defenses-v2 bleiben passed. Fuer alle folgenden Motive keine Sichtfreigabe vorwegnehmen.

Heartbeat 2026-10-05T06:29:11+02:00: Alle vier aktiven Worker-PIDs/Kommandos bestaetigt; ComfyUI-Queue anhand prompt_id einem Revisionsjob zugeordnet, Sampling schreitet fort, Fehlerlogs leer. Hauptlauf bei 268/400. Neu einzeln im Vollbild geprueft und passed: Markt, Raketensilo, Sensorphalanx, Solarkraftwerk, Versorgungsnetz und Wohnblock Aurelianer (damit alle 18 aurelianischen Gebaeudekorrekturen); Gruendung Krath, Industrie Aurelianer/Krath und Orbit Aurelianer/Krath; ausserdem Hauptlauf-Stratege Aurelianer. Keine vorzeitige Publikation. Weitere neu entstandene Kandidaten bleiben fuer naechste Sichtpruefung offen.

Heartbeat 2026-10-05T07:06:42+02:00: Hauptlauf 292/400, PID 886152 und Abschlusswaechter 1101576 anhand Kommandozeilen bestaetigt. Queue ein bekannter Hauptlauf-Job, keine Wartenden; Fehlerprotokolle leer. Alle restlichen 35 Revisionskandidaten tatsaechlich auf beschrifteten Kontaktboegen heartbeat-0638 betrachtet und passend: 24 Gebaeude/Sonden/Verteidigungen und 11 Stadtstufen/Portraets. Damit alle 62 Korrekturen visuell freigegeben; Veroeffentlichung weiterhin erst nach Hauptlauf-Ende durch Waechter samt Archivierung. Zusaetzlich 23 neue Hauptlaufmotive auf main-new-1..4.jpg betrachtet und overview_passed, darunter alle korrigierten nichtmenschlichen Strategen/Verwalter. Noch kein Abschluss; Vollaudit und frische native Galerie/Werft bleiben nach Produktion offen.

Heartbeat 2026-10-05T07:09:38+02:00: Hauptlauf auf 295/400 fortgeschritten. Hauptworker und Waechter anhand PID/Kommandos bestaetigt; Queue bekannter Hauptlauf-Prompt, keine Wartenden; beide Fehlerlogs leer. Angriffswarnung, Arbeitskraefte und Archiv einzeln mit view_image angesehen und passend. Alle 62 Revisionsfreigaben bleiben vorhanden; keine doppelte Produktion oder vorzeitige Publikation.

Heartbeat 2026-10-05T07:19:02+02:00: Hauptlauf fortschreitend bei 299/400; Prozessidentitaeten und bekannte Queue bestaetigt, Logs leer. Asteroid, Emblem Aurelianer, Hauptstadt Aurelianer und Bau-fertig einzeln angesehen und passed. Neuer visueller Befund celestials.asteroidenguertel: dominierendes Raumschiff statt Himmelskoerperband, Original needs_revision. Isolierter Nachlauf celestial-corrections-v2 mit einem Kandidaten vorbereitet, klar natuerlicher Astronomie-Prompt. Damit 63 notwendige Korrekturen, bisher 62 freigegeben. Neuen Kandidaten vor Publikation wirklich ansehen.

Nachpruefung 2026-10-05T07:24:50+02:00: Asteroidenguertel-Kandidat celestial-corrections-v2 erzeugt, einzeln angesehen und passed. Alle 63 Kandidaten freigegeben. Revisionsworker normal beendet; Hauptlauf bei 307/400 weiter aktiv. Original bleibt bis zur archivierten Abschlussuebernahme erhalten.

Heartbeat 2026-10-05T07:37:48+02:00: Hauptlauf bei 319/400, Hauptworker/Waechter identifiziert, Queue bekannter Hauptjob, Logs leer. 14 neue Motive auf heartbeat-0728/main-1..3.jpg angesehen. Sechs overview_passed, acht eindeutige Motivfehler (Blockade, Botschaft, Belagerung, Desertion, Energiemangel, Eroberung, Baufelder, Bevoelkerung) needs_revision. Separater Nachlauf symbol-corrections-v2 vorbereitet: Objekt-/Raumauftrag zuerst, Schiffsdominanz ausgeschlossen bzw. Flotte klein gegenueber Planet/Stadt. Nun 71 Korrekturen insgesamt, davon 63 freigegeben. Neue acht Kandidaten nach Erzeugung wirklich ansehen, keine vorzeitige Freigabe/Publikation.

2026-10-05T08:09:09+02:00: Sieben neue Hauptmotive auf heartbeat-0743 angesehen und overview_passed. Sieben Symbolkorrekturen bestanden (Bevoelkerung einzeln, andere auf Kontaktbogen). Blockade-v2 bei Einzelpruefung mit Seeschiffen: abgelehnt; Originalkandidat/Workflow bleiben erhalten, alter Revisionskatalog unter revision-archive/rejected-symbol-v2 gesichert. Nach normal beendetem Worker nur diesen Eintrag aus v2-Katalog genommen; stattdessen blockade-v3 vorbereitet. Insgesamt weiterhin 71 Korrekturen, 70 freigegeben. Karl verlangt nun alle vorhandenen Bilder im Spiel zu verdrahten: laufende UI-Zuordnungspruefung und Hot-Reload-Ergaenzung, Produktion bleibt aktiv.

2026-10-05T08:58:38+02:00: Blockade-v3 einzeln angesehen und passed (echte Raumschiffe im Vakuum). 47 weitere Originale auf integration/new-1..6.jpg tatsaechlich angesehen. 13 neue eindeutige Motivfehler (sieben Naturplaneten mit kuenstlicher Dominanz, Nebelsystem, Nebelhintergrund, zwei Panelmaterialien, Hunger, Recyceln) markiert und als nature-material-corrections-v2 separat vorbereitet. Jetzt insgesamt 84 Korrekturen, 71 freigegeben. Hauptlauf unveraendert aktiv.

### Native Bildverdrahtung auf Karls ausdruecklichen Auftrag (2026-10-05T09:05:52+02:00)
Alle 400 Katalog-IDs haben eine erreichbare Zuordnung in der nativen UI: zusaetzliche Direktbilder fuer Ressourcen (globale Leiste, Lager, Markt), Kennzahlen, Planeten, Diplomatie, Hauptstadt, Garnison, Ereignis-/Kampf-/Spionageberichte sowie passende Hintergrund-/Berater-/Emblembilder. Kontextsammlung Bildmotive unter jedem Fachbereich erschliesst weitere Varianten; faction filtert passend zum Spieler, neutrale Assets bleiben sichtbar. Bericht content/ui-integration-status.json: 382 vorhandene Dateien, keine unverdrahtete Kategorie. 22 native Tests + 13 Sessiontests bestanden; darunter alle 16 Bildschirme und echter Dateiaustausch/Hashfehler/Erholung im Bildcache. Cache prueft Bild und Provenienz alle zwei Sekunden, laedt auch Ersatzbilder automatisch und versteckt Hashfehler. Dies ersetzt die oben dokumentierte Cache-Grenze.
Release kompiliert und gelinkt; Kopieren auf den Standardnamen scheiterte ausschliesslich am noch laufenden alten Player PID 912356 (Windows-Dateisperre). Kein Programm beendet. Fertig gelinkte Datei daher als target/release/sternenepoche-spieler-bilder.exe bereitgestellt, SHA256 48ee9eac707f251b792b0c4d3527486471d87f97613e17bdea3440e54a269769; Spielen.cmd und Live-Test.cmd verwenden diese Version. Fuer finale Screenshots DIESE EXE verwenden. Die bereits offene alte UI braucht einmal Neustart fuer die Codeaenderung, danach Bild-Hot-Reload.
Echte native Screenshots ui-bilder-kolonie.png, ui-bilder-werft.png, ui-bilder-markt.png und ui-bilder-galerie.png aufgenommen und mit view_image angesehen: Bilder geladen, Ressourcen-/Schiffszuordnung sichtbar, Texte und Bedienung lesbar. Das ist Integrationsabnahme mit laufender Bildproduktion, nicht finale Kunstabnahme. Bekannte korrigierte Varianten werden nach Hauptlauf automatisch archiviert uebernommen; aktuelle Screenshots enthalten noch Originalmotive. Finale UI-Screenshots nach Uebernahme weiterhin erforderlich.

2026-10-05T09:57:02+02:00: Hauptlauf 400/400 fertig. Alle 13 nature-material-Kandidaten auf final-production-Boegen tatsaechlich betrachtet und bestanden. Letzte 33 Originale ebenfalls angesehen; weitere 11 klare Motivfehler (drei Sterne, vier Bodenmaterialien, Sternhintergrund, Unruhen, Stufenaufstieg, Schutzende) in final-motif-corrections-v2 vorbereitet. Insgesamt jetzt 95 Revisionsmotive, 84 freigegeben. Waechter wartet weiter, kein Abschluss vor diesen Korrekturen. Karl erweitert aktiv um moderne klickbare Galaxie mit GoF-Charme, Browserbetrieb/Rust-Logik, Fog of War/Sondenberichte mit technologisch genauerer Schaetzung sowie fruehe Sonden/Jaeger und sichtbaren Technologiebaum. Diese Arbeit laeuft parallel zur letzten Bildkorrektur.
# Ergänzung: Freischaltübersicht und zwei Raketenmotive

Am 5. Oktober auf Karls ausdrücklichen Auftrag ergänzt: `Forschung` zeigt eine zusammenhängende,
aufklappbare Übersicht über die fünf Zivilisationsstufen. Alle 17 Technologien werden aus dem aktuellen
Regelwerk ihrer Stufe zugeordnet; Laborvoraussetzungen und bereits erforschte Stufen stammen aus Regeln
und eigener Beobachtung. Gebäudevoraussetzungen, Werftstufen und Antriebsbezüge sind zusätzlich einsehbar.
Es werden keine Forschungs-Vorgänger erfunden, die der Spielkern nicht kennt.

Zwei neue neutrale Motive `missiles.abfang` und `missiles.interplanetar` wurden mit dem eingebauten
Imagegen-Werkzeug erzeugt, im Vollbild gesichtet und als passende, unterscheidbare Raketen freigegeben.
Unveränderte PNGs liegen unter `content/assets/missiles/`; tatsächliche Maße 1254×1254.
Prompts und Originaldateien: `content/requests/missiles-imagegen.json`. Herkunft ist ausdrücklich
`image_gen`, kein nachträglich behaupteter ComfyUI-Job. Katalog umfasst jetzt 402 Motive.

`external-assets.json` erhält die Motive bei Python-Katalog-Neubau; beide Produktionsadapter verhindern
versehentliche ComfyUI-Neuerzeugung. Audit prüft Bild, Maße, Prompt-/Workflowmanifest und Generator.
Werft → Raketen zeigt beide Bilder auch vor Freischaltung des Silos; die Grafikbibliothek enthält sie ebenfalls.

Prüfung: 23 native Spielerprüfungen, 15 Rust-Inhaltsprüfungen, 13 Python-Inhaltsprüfungen bestanden.
Die echten Screenshots `content/runtime/qa/techtree-forschung-final.png` und `techtree-raketen.png`
wurden angesehen. Alte bereits offene Kunstkorrekturen sind damit nicht pauschal freigegeben.
`Spielen.cmd` und `Live-Test.cmd` starten die neue `sternenepoche-spieler-tech.exe`; laufende ältere
Spielerprozesse wurden nicht beendet. Für die neue Übersicht die Anwendung einmal neu starten.

