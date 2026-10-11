# Singleplayer und lokale Sitzung

Im Cockpit führt **Hier auch Singleplayer offline spielen** zur Seite **Lokale Sitzung**.
Ein Spielername und ein Volk genügen für Singleplayer. Die Spielwelt läuft als eingebettete Rust-Laufzeit
im Cockpit; ein separates Serverprogramm, Internetzugang und ein Sprachmodell sind nicht erforderlich.
Alle vorhandenen Spielregeln gelten auch hier, einschließlich Forschung, Flotten, Handel und Allianzen.

Standard sind 30 Bots. Die vier vorhandenen Bot-Typen betreiben Wirtschaft, Handel, Verteidigung oder
Raubzüge. Sie beantworten Nachrichten mit vorgefertigten, thematisch passenden deutschen Texten und
treffen verbindliche Entscheidungen über die normalen Spielaktionen. Eine Nachricht verspricht keine
automatische Warenlieferung oder militärische Hilfe. Vier lokal erzeugte deutsche Funksprüche lassen
sich in den lokalen Einstellungen und im Briefkasten abspielen.

| Botzahl | Sektoren | Sonnensysteme je Sektor |
| --- | ---: | ---: |
| 0 bis 50 | 2 | 80 |
| 51 bis 100 | 4 | 80 |
| 101 bis 200 | 4 | 120 |
| 201 bis 400 | 8, die vollständige lokale Galaxie | 120 |

Je Botzahl gibt es einen eigenen Spielstand. Ein vorhandener Spielstand wird fortgesetzt;
geänderte Größen greifen bei einem neuen Spielstand. Jede neue Welt bekommt eine zufällige Galaxie.
**Neue Partie beginnen …** verschiebt den bisherigen Spielstand dieser Botzahl nach `archiv/`
(es wird nichts gelöscht); der nächste Start erzeugt eine neue Galaxie. Menschen und eigene Agenten teilen freie Plätze.
Die Welt reserviert konservativ 20 Prozent der Systeme für Nebel und andere Spielbedingungen:
`Sektoren × Systeme × 80 % − Bots` ergibt die obere Zahl zusätzlicher Spielerplätze.
Zu Beginn sind höchstens 32 davon freigegeben; der Host kann die Freigabe in den lokalen
Einstellungen bis zur Weltgrenze erhöhen. Die Hardwareanzeige hilft, eine kleinere, flüssige Runde zu wählen.
Jeder Befehl wird sofort dauerhaft gespeichert, zusammen mit den Antworten der Bots darauf.
Reine Zeitschritte schreiben den vollständigen Spielstand höchstens alle 30 Sekunden; beim
Verlassen wird er immer geschrieben. Nach einem Absturz fehlen also höchstens die letzten
Sekunden Spielzeit, nie ein Befehl; das Archiv wird beim Laden passend zum letzten Checkpoint bereinigt.
Bei Speicherfehlern hält die Welt an.
Die aktive Partie zeigt **Pause / Fortsetzen** und **Speichern** direkt in der Kopfzeile. Während einer Änderung wird der Knopf gesperrt, bis der Spielkern bestätigt; eine alte Statusabfrage überschreibt die Bestätigung nicht. Die Offline-Startwerbung und Anmeldeformulare verschwinden während der Partie.
**Speichern** schreibt sofort einen Checkpoint, ohne die Partie zu verlassen. Beim regulären Schließen wird erneut gespeichert; beim nächsten Cockpit-Start öffnet sich die letzte Partie automatisch. Pause, Spielzeit, ausgewählter Planet, Spielbereich und die Ebene der Galaxiekarte werden wieder aufgenommen. **Sitzung verlassen** beendet diese automatische Wiederaufnahme bewusst.
Während das Cockpit geschlossen ist, vergeht keine Spielzeit. Der Host kann pausieren und
1×, 10×, 30× oder 60× Geschwindigkeit wählen; 1× ist Echtzeit (die Welt rechnet in Fenstern
von 30 Spielsekunden, der Rest wird mitgeführt).

Damit lange Runden auch auf schwacher Hardware flüssig bleiben, hält die lokale Welt nur die
jüngere Geschichte im Arbeitsspeicher: Kommunikationsereignisse des aktuellen und des
vorigen Spieltags (alle anderen stehen im Archiv der Datenbank), Kampfberichte und Tageswerte
der letzten 30 Spieltage. Das Aktions- und Fachereignisprotokoll auf der Platte umfasst die
letzten 14 Spieltage; es dient nur der Auswertung und wird von keiner Spielfunktion gelesen.
Briefe bleiben vollständig lesbar.

Für gemeinsames Spielen aktiviert der Ersteller **Lokalen Host-Server aktivieren**, trägt ein
Masterpasswort mit 12 bis 128 Zeichen ein und startet die Sitzung. Das Passwort liegt im
Windows-Anmeldeinformationsspeicher. Mitspieler öffnen **Lokale Sitzung**, suchen Hosts im Netzwerk,
wählen einen Host und melden sich mit Name und Masterpasswort an. Der Name behält seinen Spielplatz
und gehört dem Gerät, mit dem er zuerst beigetreten ist (ein zufälliges Gerätegeheimnis im
Anmeldeinformationsspeicher bzw. im Browser). Wählt ein zweiter Spieler denselben Namen, bekommt
er eine klare Ablehnung statt des fremden Reichs. Namen des Host-Rechners (Host und seine
Agentenplätze) lassen sich nur dort verwenden. Wechselt ein Mitspieler das Gerät, gibt der Host
den Namen unter **Sitzung aktiv** frei; der nächste Beitritt mit diesem Namen übernimmt ihn.

Der Host sieht unter **Sitzung aktiv** die LAN-Adressen seines Rechners und je einen QR-Code.
Er enthält nur die Adresse (nie das Passwort) und öffnet das Spiel im Browser eines Geräts im
selben Netz; Name und Masterpasswort geben Mitspieler selbst ein. Im Browser wählen sie danach
Volk und Spielweise im üblichen Platzdialog.

Hängende Verbindungen (etwa ein zugeklappter Laptop mitten in einer Anfrage) belegen nur ihre
eigene Verbindung höchstens 10 Sekunden; die Welt und alle anderen Spieler laufen weiter.

Die automatische Suche nutzt lokalen UDP-Multicast auf Port 38991, der Host standardmäßig
TCP-Port 8892. Bei WLAN-Isolation oder blockiertem Multicast lässt sich die lokale IP des Hosts
direkt eintragen, beispielsweise `http://192.168.1.20:8892`. Windows kann beim Hosting eine
Netzwerkfreigabe anfordern. Der Host muss für die gemeinsame Sitzung geöffnet bleiben.
Beim Beenden bleibt der Spielstand beim Host gespeichert.

Der lokale Modus verwendet eine leichte Sternenkarte mit Übergängen von Galaxie zu Sektor und
Sonnensystem. Bis zu zwölf dekorative Schiffe fliegen im Hintergrund; ihre Animation lässt sich
pausieren und berücksichtigt die Einstellung für reduzierte Bewegung. Mit **Schiffsverkehr aktivieren**
lässt sich die Bewegung ausdrücklich einschalten. Die Systemliste bleibt erreichbar.
Der einschaltbare Sparmodus zeigt Ratsfiguren als Bilder. Die Spielsimulation und Botentscheidungen
laufen in Rust; die vorhandene WebView stellt die Oberfläche dar. Für die Standardbots werden keine Modelle geladen.

## Eigene Agentenspieler und Hardwareanzeige

Sechs vorbereitete Plätze führen jeweils ein eigenes Reich. Weitere Plätze lassen sich bis zur
Weltgrenze hinzufügen. Aktivierung, Name, Modellquelle, Profil, Takt und Kontext sind je Platz einstellbar.
Sparsam verwendet ein kleines installiertes Modell, Mittel ein mittleres und Max das größte
geschätzt passende Modell. Custom erlaubt die ausdrückliche Wahl eines vorhandenen Modells.
Ollama lokal, Ollama nur CPU und der eigene API-Endpunkt aus der Werkstatt stehen zur Auswahl.
Es werden keine Modelle automatisch heruntergeladen. Ohne installiertes Modell bleibt dieser Platz aus.

Jeder Agent arbeitet nacheinander als Verwalter, Diplomat, Feldherr und Stratege. Er erhält nur
seine zulässige Spielsicht. Befehle laufen mit eigener Anmeldung und zeitlich begrenzter Freigabe
durch dieselben Regeln wie andere Spielzüge. Die gemeinsame Inferenzwarteschlange lässt nur
eine Modellanfrage gleichzeitig zu, auch beim Stoppen und erneuten Starten.

Das Cockpit ermittelt CPU-Threads, freien Arbeitsspeicher, GPU/VRAM und vorhandene Modelle im
Hintergrund. Während einer lokalen Sitzung werden RAM und GPU jede Minute neu geprüft;
eine laufende Hardwareabfrage wird nicht mehrfach gestartet. Bei einem Mix unterschiedlicher
lokaler Modelle, Geräte oder Kontextgrößen wird das jeweils verwendete Modell nach der Antwort
entladen. Dadurch bleiben nicht mehrere Modelle dieser Warteschlange zugleich im Speicher.
Der Balken schätzt den Modellbedarf mit 40 Prozent Zeitreserve; nach ersten Zügen
fließen gemessene Modellzeiten ein. Bei Überlast wächst der Abstand zwischen Modellzügen.
Eine neue Platzempfehlung aus Messungen setzt erfolgreiche Züge in allen vier Rollen je Agent voraus.
Für Welt und Menschenplätze zeigt das Dashboard zusätzlich das 95. Perzentil der letzten
60 Welt-Ticks sowie eine konservative Hochrechnung mit CPU- und RAM-Reserve. Die Zahlen sind
Orientierung, keine Zusage: fremde Programme, Modellwechsel und fortgeschrittene Welten verändern die Last.
Bei steigenden Tickzeiten können Tempo, Freigabe, Modellgröße und aktive Plätze reduziert werden.

Prüfung: echte HTTP-Anmeldung, zwei Spieler, falsches Passwort, Wiederaufnahme, doppelte Befehle,
Bot-Nachrichten, Allianzbeitritt, interner Handel, Pause, einmalige Browserübergabe, Multicast-Suche
und alle Weltgrößengrenzen einschließlich 400 Bots werden durch Rust-Tests geprüft.

## Testpiloten auf diesem PC

Unter **Lokale Sitzung** (vor dem Start) und **Werkstatt → Konto** sind die fünf vorhandenen Testpiloten sichtbar. Die private lokale Datei `testpiloten-zugaenge.txt` im Harness-Arbeitsordner ermöglicht die direkte Anmeldung. Passwörter bleiben im Rust-Kern und fehlen im Download-Paket und auf der Webseite. Ohne diese private Datei sind die Auswahlknöpfe sichtbar, aber gesperrt. Testpiloten benutzen ihre bestehenden Serverkonten; sie ersetzen keine Offline-Spielstände.

## Credits und Darstellung

Auch kleine Ressourcen-, Energie- und Credits-Symbole sind KI-generiert. Ihre Herkunft steht gemeinsam in den Credits; kleine Symbole tragen keine überlagernde Einzelmarkierung. Große Spielbilder bleiben gekennzeichnet. Ratsfiguren stehen fest auf ihrem Platz und können in der 3D-Ansicht sanft gedreht werden; im Sparmodus bleiben sie als ruhiges Bild sichtbar.
