# Labor-Review: Gateway und Spieler-Sandbox

Stand: 4. Oktober 2026. Review und Implementierung des angeforderten Sol-High-Partners. Diese Notiz trennt nachgewiesene Funktion von noch offenen Forschungs- und Ausführungsfragen.

## V4-Nachtrag

Der ausdrücklich angeforderte gpt-6.1-sol-high-Partner hat die Flotten-/Besitzwechselregeln aus der
Konzeptprüfung umgesetzt: gebundene Transporte, gezielte Orbitversorgung, einheitlichen Entsatz,
zusätzliche Flugzeit bei Rückkehrumleitung, Warteorbit, gemeinsame Reparaturbaustelle und Besetzungsfristen.
Beide Parteien werden während der Besetzung jedes Fenster und bei ihrem Abbruch benachrichtigt.
Zwei abgeschlossene Entscheidungsfenster und mindestens 1.800 Spielsekunden sind gleichzeitig nötig.
12 neue Szenarien und 7 bestehende Kolonisationstests bestanden. Details: `LABOR-ABNAHME.md`.
Die folgenden Abschnitte sind frühere V2/V3-Prüfstände; insbesondere Docker ist keine V4-Voraussetzung.

## Geprüfte Entscheidungen

- Interne Rollen besitzen frei wählbare IDs und explizite Aktionsfähigkeiten. Der Gateway authentifiziert den Spieler aus dem extern zugeordneten `Memory.owner`. Ein als `Alle` benannter interner Akteur erhält dadurch keine Engine-Rechte.
- `validate_harness_contract` prüft jede aktive Organisation gegen den unveränderlichen Spieler-Laufvertrag: Fähigkeiten bleiben eine Teilmenge der ursprünglichen Rechte. Modelle müssen aus dem gemeinsamen Pool stammen; ein Wechsel aus der ursprünglichen Modellzuordnung benötigt `allow_model_switch=true`. Diese Prüfung ist auch vor Broker-Aufrufen und bei Harness-Aktivierung in die Runtime eingebunden.
- Aktionen werden mit der zuständigen Engine-Rolle geprüft. Militärische Fertigung verwendet zusätzlich die konkreten Einheitentypen aus der Wirtschaftsregel und den Militärtopf. Der Negativtest mit leeren Töpfen belegt die Abweisung einer Aktion, die der alte `Rolle::Alle`-Pfad noch akzeptieren würde. Das Gateway verwendet diesen Pfad nicht für Spielaktionen.
- Das Gateway prüft Aktionen auf einer Kopie der eingefrorenen Welt und sammelt kanonische Absichten. Weltwirkungen entstehen erst beim gemeinsamen Commit. Die vorbereitete Aktion ist keine garantierte Zusage; Ressourcen und Rechte werden beim Commit erneut geprüft.
- Unbekannte operative Felder, mehrdeutige Fertigungsvarianten und unzulässige Mengen werden abgewiesen. Doppelte JSON-Schlüssel werden vor dem Übergang in `Value` rekursiv erkannt. Private Notizen dürfen beliebige Inhalte enthalten, verändern aber keine Systemrechte.
- Kleine Provider-Werkzeuge haben durchgehend ein Objektschema an der Wurzel. Die Vereinigung von Varianten dient dem nativen Ollama-Transport; im Gateway bleibt die genaue `anyOf`/`oneOf`-Prüfung bestehen. Damit schwächt der Transport die serverseitige Prüfung nicht ab. Die erste echte CPU-Probe zeigte den Fehler bei einem Wurzel-`anyOf`; nach der Korrektur funktionierten native `world_query`- und `memory_search`-Aufrufe.
- Ein Harness-Patch gilt erst an der nächsten Fenstergrenze. Revisionen müssen auf den aktiven Stand folgen. Ein bereits vorgemerkter, abweichender Vorschlag derselben Revision verursacht einen Konflikt; ein identischer Vorschlag ist idempotent.
- `skill_read` ruft gespeicherte deklarative Anleitung gezielt ab und markiert sie als nicht vertrauenswürdige Spielerinformation. Es führt keinen freien Code aus und erweitert keine Fähigkeiten.

## Nachgewiesene Isolation und Persistenz

Der Linux-Rust-Worker läuft mit UID/GID 65534 in Docker. Der Host setzt Netzwerk `none`, read-only Basisimage, `cap-drop=ALL`, `no-new-privileges`, 128 MiB RAM einschließlich Swapgrenze, 0,5 CPU, maximal 32 Prozesse, beschränkte Dateigröße und kleine tmpfs-Bereiche. Der Ressourcen-Test liest die wirksamen cgroup-Werte. Auf diesem Docker-Desktop-System sind es cgroups v1; v2 wird ebenfalls unterstützt.

Ein Spieler erhält ausschließlich seinen konkreten privaten Ordner auf D unter `laeufe/sandbox-workspaces/<namespace>/p<owner>`. Weder der übergeordnete Hostordner noch fremde Spielerordner oder ein Docker-Socket werden eingehängt. Die Mount- und Löschziele werden absolut kanonisiert, auf D geprüft und gegen den exakten zulässigen Pfad verglichen. Daten wurden nachweislich als nicht privilegierter Containerbenutzer auf diesem NTFS-Bind geschrieben. Die Docker-VM liegt hier auf C; deshalb werden wachsende Spieler-Datenbanken nicht in Docker-Named-Volumes gespeichert. Das vorhandene Debian-Image und das kleine Worker-Image wurden nicht verschoben.

`WorkerSession` hält eine sequenzielle NDJSON-Verbindung pro Spieler offen. Jede RPC-Antwort ist begrenzt und jeder Aufruf hat eine Frist; Timeout oder Verbindungsfehler stoppen ausschließlich den eigenen Container. Der Worker bearbeitet SQLite selbst. Der Host hält eine Sicherungskopie für Checkpoints. Wiederaufnahme importiert den autoritativen Stand transaktional und verwirft unbestätigte spätere Schreibzugriffe im Arbeitsordner.

Durch ausgeführte Tests belegt: fremde Spielernotizen fehlen; Fremdimport und Änderung der Datenbankidentität werden zurückgewiesen; Basisimage ist nicht beschreibbar; Hostpfade und Docker-Socket sind nicht vorhanden; Capability-Maske ist null; `NoNewPrivs=1`; nur Loopback ist vorhanden; Endlosschleife und Outputflut werden begrenzt; tmpfs kann nicht über seine Größe hinaus wachsen; Save/Load/Resume erhält bestätigte Revisionen und entfernt unbestätigte Notizen. Eine dauerhafte Verbindung verarbeitet zwanzig aufeinanderfolgende RPCs und bleibt nach einem fachlichen Argumentfehler verwendbar.

## Prüfstand

- `cargo test -p sternenepoche-agenten --test labor_gateway`: in der abschließenden Fassung acht Tests bestanden, einschließlich der gemeinsamen Speicherquoten; die zwei Docker-Tests sind regulär explizit als Umgebungsprüfungen markiert.
- `cargo test -p sternenepoche-agenten --test labor_gateway -- --ignored`: beide Docker-Tests tatsächlich ausgeführt und bestanden, einschließlich privatem D-Bind und dauerhaftem Worker.
- Hauptagent führte abschließend 83 reguläre Kern-/Runner-Tests und alle drei Docker-Tests erfolgreich aus, einschließlich vollständigem Docker-Match-Resume. Unter den neuen normalen Backend-Tests sind Matrix-, Lernpaket-/Holdout- und Recovery-/Retry-Prüfungen. Siehe `LABOR-ABNAHME.md` und gespeicherte Testprotokolle.
- Der Worker wurde in WSL Debian mit GCC und einer minimalen Rust-Installation ausschließlich auf D gebaut. Das Linux-Binary liegt bei etwa 1,55 MB. `tools/sandbox/build.ps1` baut mit Worker-Lockfile und verwendet das bereits lokale Debian-Image für den Docker-Build ohne Build-Netzwerk.

## Grenzen und verbleibende Arbeit

- Docker ist die nachgewiesene Prozess-/Datei-/Netzgrenze dieser Version. Ein Firecracker-/KVM-Nachweis liegt nicht vor. Trusted-Modus erlaubt nur deklarative Gateway-Werkzeuge und ist ausdrücklich keine OS-Isolation.
- Freie, vom Spieler geschriebene Skills werden noch nicht ausgeführt. Die verfügbare Skill-Nutzung besteht aus privatem Speichern und gezieltem Abruf von Anleitung. Eine spätere freie Codeausführung braucht einen zusätzlichen überprüften Vertrag für Dateien, Aufrufe und Wirkungen.
- Ein vorbereiteter Aktionsbestand ist kein atomarer Mehraktionsplan. Einzelaktionen können beim globalen Commit aufgrund vorangegangener Wirkungen abgewiesen werden; solche Ergebnisse erhalten eigene Belege. Explizite atomare oder abhängige Pakete sind noch kein bereitgestelltes Tool.
- RPC-Frames sind zusätzlich zum logischen Speicherbudget auf ungefähr 20 MiB begrenzt. Host und Worker berechnen Records identisch: UTF-8-Bytes des serialisierten Werts, des Bereichs und des Schlüssels plus 128 Bytes für Record-/Export-Overhead. Maximal 4096 Records verhindern eine extrem kleinteilige Exportaufblähung. Das logische Record-Budget liegt bei maximal 16 MiB; SQLite erhält 28 MiB physische Reserve einschließlich einer Seitenzahlgrenze, die auch beim Wiederöffnen gesetzt wird. Der Host-Import prüft den kompletten Stand einmal und importiert dann direkt, statt pro Record erneut den wachsenden gesamten Speicher zu summieren.
- Metadaten sind bei Host und Worker gleichermaßen pro Wert auf 64 KiB, auf maximal 32 Schlüssel und insgesamt 1 MiB inklusive Schlüssel begrenzt; Import prüft dieselben Grenzen. Die Spieleridentität ist unveränderlich. Eine echte NDJSON-Probe bestätigte: 32 Schlüssel angenommen, der 33. abgewiesen, danach blieb die Verbindung nutzbar. Neues Image: `sha256:d74b8d316015aeedf3a4a0d49897dbaaf408ca8ec44ff062b51af777614180ff`.
- Der Hauptagent hat die private Belegretention begrenzt: vollständige Aktionsbelege liegen im globalen Audit; das Spielerbüro erhält kompakte Belege des letzten eigenen Arbeitsfensters als Metadaten. Engine-Belege füllen dadurch nicht die Quote für eigene Pläne und Notizen.
- Die gezielten Tests belegen Schnittstellen, Rechte, Persistenz und lokale Isolation. Sie belegen weder strategische Überlegenheit eines Modells noch vollständige Forschungsabnahme auf unbekannten Karten und Gegnern. Die echte CPU-Modellprobe gehört getrennt von den technischen Mock-Läufen ausgewertet; keine GPU-Arbeit wurde für diese Implementierung ausgeführt.

## Nachtrag: native Windows-Isolation und V3-Review, 04.10.2026

Die tatsächlich ausgeführte native Alternative ist ein dauerhafter Rust-Prozess in einem Windows-LPAC mit Job Object. Sie ist keine MicroVM und virtualisiert kein Betriebssystem. `Memory::isolate_native(worker,namespace)` importiert den bestätigten privaten SQLite-Stand und verwendet dieselbe RPC-/Quotenlogik wie Docker. Inferenz, Modellgewichte, Provider-Geheimnisse und kontrollierte Engine-Tools bleiben getrennt beim Host. Config und Runtime akzeptieren `sandbox="native"` mit optionalem `sandbox_worker`; der neue Pfad wird vor Start geprüft und sein Binary-Hash bei Resume fixiert.

Die vier explizit ausgeführten Tests in `labor_native.rs` bestanden: echte LPAC-Negativproben; private SQLite mit getrennten Spielern, Save/Resume und unveränderlicher Identität; Job-Timeout und Outputflut; 50 Mockspieler mit zwei Fenstern, anschließend Resume zum dritten Fenster und identischem Replay/Audit. Der Preflight liest den normalen AppContainer-Status, null Capability-SIDs und die wirksamen Job-Grenzen. Nachgewiesen sind Fremdlesen/-schreiben und fremde Hardlinks mit Fehler 5, verweigerte Schreibrechte auf das gemeinsame Worker-Binary, fehlender Zugriff auf Parent-Prozesshandles, Kindprozessverweigerung 1816, Prozesslimit 1, 128 MiB privates Commit-Limit und CPU-Hard-Cap 5000 bei Flags 5. Die frühere private Binary-Kopie wurde entfernt: Der Host überschreibt keine potentiell vom Worker manipulierten Dateipfade mit seinen eigenen Rechten.

LPAC-spezifische Grenzen: Die Win32-Tokenabfrage Klasse 46 ist auf diesem Rechner nicht unterstützt (87). Die zusätzliche tatsächliche Zugriffsprobe auf eine existierende `ALL_APPLICATION_PACKAGES`-Canary unterscheidet daher das Verhalten von einem normalen AppContainer. Winsock scheitert bereits während der Initialisierung (10107); die Probe fängt diese native Verweigerung ab. Wenn die Initialisierung auf einer anderen Installation gelingt, verlangt der Preflight eine verweigerte Verbindung (10013). Das Startattribut `CHILD_PROCESS_POLICY` verursachte hier DLL-Initialisierungsfehler; das bereits beim Start zugeordnete Job-Prozesslimit belegt das Kindprozessverbot. Es wird kein ungeschützter Prozess als erfolgreicher Sandboxlauf ausgegeben.

Gemeinsamer 50-Spieler-Beleg nach Umstellung auf eine einzige unveränderliche EXE: `laeufe/labor-tests/native-fifty-runtime-746180-1791147376762296700/native-measurement.json`. Gemessen: Start plus Import 3097 ms, 50 kleine Meta-RPCs 14 ms, zwei Mockfenster 13224 ms, Resume plus ein Fenster 2383 ms. Aufsummierter Working Set 244076544 Bytes, privater Speicher 46665728 Bytes. Working Set zählt gemeinsame Seiten mehrfach. Diese Messung belegt die kleine Bürogrenze ohne Modellinferenz; sie belegt keine 50 parallelen Modellresidenzen oder Modellintelligenz.

Das gezielte V3-Code-Review prüfte Scheduler, Fallback und Kolonisierung. Bestätigter P1-Befund: `local_or_remote` ersetzte konfigurierte lokale Modelle durch eine neue Inventarauswahl mit `cpu_only=false`, wodurch Modellidentität, CPU-only und Kontext-/Timeoutvorgaben verloren gingen. Zusätzlich wurde jeder Inventar-/Detailfehler als fehlendes lokales Modell behandelt. Der Befund wurde an den Hauptagenten gemeldet und von ihm für eine Korrektur mit HTTP-Stubtests übernommen; deren abschließender Teststand gehört in seine Abnahme. Im gelesenen Scheduler bleibt die Welt bis zum gemeinsamen Commit eingefroren; jeder Spieler erhält pro Runde höchstens einen Aufruf, Modellgruppen rotieren über die Fenster, und bestätigte Antworten bleiben für Recovery dauerhaft gespeichert. Die gelesenen Kolonisierungsstellen schützen den ursprünglichen Heimatplaneten vor Übernahme, behalten Ausbaulevel, setzen Integrität auf höchstens 30 Prozent und prüfen Orbit, Kolonieschiff, Kolonielimit und zwei Reaktionsfenster erneut. Dabei wurde keine weitere bestätigte kritische Regression gefunden.

Die Betriebssystemgrenze folgt den Windows-Mechanismen, die Microsoft in [Launch an AppContainer](https://learn.microsoft.com/en-us/windows/win32/secauthz/implementing-an-appcontainer) und [Process creation attributes](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-updateprocthreadattribute) beschreibt. Gemeinsamer Kernel, lesbare freigegebene Windows-Systemressourcen und die konkrete Plattformabhängigkeit bleiben Grenzen dieses Adapters. Die Diagnosen sind feste Operator-RPCs, keine freien Skripte und keine Modellwerkzeuge. Für diese Implementierung führte der Reviewer keine GPU-Inferenz aus; die getrennten echten Modellbelege stammen vom Hauptagenten.

Finale native Fassung: Worker 1733120 Bytes, SHA-256 `040bd7384a8a24ef6bb88f53aa15e9f976abb0053e58565c8799551899be5040`. Die vier Umgebungsprüfungen wurden nach der letzten Härtung vollständig erneut ausgeführt und bestanden in 19,65 Sekunden. Finaler Belegordner: `laeufe/labor-tests/native-fifty-runtime-777840-1791147784434163300/`; `sandbox.json` enthält die konkreten Negativproben und den Worker-Hash, `native-measurement.json` die 50-Spieler-/Resume-/Replay-Werte. Start plus Import 3199 ms, 50 Meta-RPCs 14 ms, zwei Fenster 13789 ms, Resume plus Fenster 2412 ms; Working Set 244527104 Bytes und privater Speicher 47067136 Bytes. Native und Docker verwenden jetzt jeweils auf einen gepufferten Frame begrenzte stdin/stdout-Kanäle; dadurch kann auch eine Flut kurzer NDJSON-Zeilen keine unbegrenzte Host-RAM-Warteschlange erzeugen. Ein voller Eingabekanal führt zum Abbruch des eigenen Workers. Der echte Docker-Persistenz-/Checkpoint-/Resume-Test bestand nach dieser Kanaländerung erneut (8,70 Sekunden). Der Hauptagent meldete inzwischen die Korrektur des Fallback-P1 und 98 bestandene reguläre Tests einschließlich der neuen HTTP-Stubprüfungen.
# Ergänzung vom 5. Oktober: Forschungsabschluss

Der beauftragte Sol-6.1-High-Partner implementierte und prüfte `balance.rs` sowie `research.rs`:
18 Kartenfälle mit ausdrücklich ausgewiesenen Gebietsasymmetrien, Versorgungs-/Kampfschwellen und
gekreuzte Modell-Harness-Versuche mit identischer Capability-Union. Sämtliche Sitze werden rotiert,
Quellaudits geprüft und Unsicherheiten nur über Seedfamilien berechnet. Modellwechsel sind in diesem
kontrollierten Vergleich gesperrt; interne Rollen bleiben anpassbar.

Gemeinsame Integration: 33 gezielte Labortests bestanden, ein historischer Docker-Test übersprungen.
Zusätzlich zwölf native Mock-Kurzläufe des Forschungsbefehls und ein echter Zehn-Spieler-Qwen8-Lauf
mit 40 Calls, 14 Aktionen und 96,445 Sekunden Fensterzeit. Quellen und Grenzen stehen in
[LABOR-ABNAHME.md](LABOR-ABNAHME.md), aktuelle Übersicht in
`laeufe/labor-evidence/LATEST-V4-RESEARCH.json`. Keine Sieg- oder optimale-Balance-Behauptung aus Kurzläufen.

