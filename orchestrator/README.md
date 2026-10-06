# Modellanbindung für Sternenepoche

Python 3.11 oder neuer, ausschließlich Standardbibliothek. Unterstützt lokale Chat-Completions-Endpunkte und OpenRouter; pro Rolle frei konfigurierbar. Keine Installation oder Modelldownloads notwendig.

Vom Projektordner aus:

```powershell
python -m unittest discover -s orchestrator -p "test_*.py" -v
python -m orchestrator --config orchestrator/models.example.toml --window orchestrator/window.example.json
```

Der zweite Befehl prüft ausschließlich die Konfiguration. Die Beispielmodellnamen sind Platzhalter und müssen vor echter Inferenz ersetzt werden.

Für einen echten Lauf eine eigene TOML-Konfiguration anlegen. Bei OpenRouter den gewünschten Rollenblock gemäß Kommentar in `models.example.toml` ersetzen und `OPENROUTER_API_KEY` in der Prozessumgebung setzen. Keine Schlüssel in Projektdateien oder Chats speichern. Die genaue Modell-ID muss vom gewählten Endpunkt bereitgestellt werden und JSON-Ausgaben unterstützen.

Ein Entscheidungsfenster explizit ausführen:

```powershell
python -m orchestrator --config orchestrator/models.toml --window orchestrator/window.example.json --execute --output D:/projekte_ki/Sternepoche/window-results.jsonl
```

Die Ausgabedatei muss neu sein. Der Aufruf archiviert Eingaben, rohe Antworttexte, geparste Entscheidungen, tatsächlich gemeldetes Modell, Usage, Latenz und Fehler. API-Schlüssel und HTTP-Header werden nicht archiviert. Jede abgeschlossene Antwort wird sofort geschrieben und geflusht; die Zeilenreihenfolge spiegelt deshalb die Ankunft wider und ist keine Aktionspriorität. Der abschließende Datensatz `window_complete` unterscheidet vollständig abgeschlossene Fenster von abgebrochenen Läufen. Dies ist ein Entscheidungsarchiv, noch kein Engine-Replay. Noch laufende HTTP-Anfragen können bei Prozessabbruch verloren gehen; ein fehlender Abschlussdatensatz darf nicht automatisch erneut kostenpflichtig ausgeführt werden.

## Übergabe an die Engine

`WindowRunner.run(list[DecisionRequest])` liefert `list[DecisionResult]` erst nach Abschluss aller Anfragen. Ein Fehler bleibt ein explizites Ergebnis mit `decision=None`; die Engine entscheidet anhand ihrer festgelegten Laufpolitik über Pause, Korrektur oder ausgelassenen Zug. Der Adapter ändert weder Spielzeit noch Weltzustand. Es gibt keine stillschweigende Ersatzstrategie.

Die Reihenfolge der zurückgegebenen Ergebnisse dient stabiler Archivierung. Die Engine muss ihre eigene, vom Seed abhängige Reihenfolge für gleichzeitige Aktionen verwenden.

Beobachtungen müssen bereits in der Engine nach Spielerrechten gefiltert sein. Notizbücher werden je Spieler und Rolle übergeben; der Adapter besitzt keine gemeinsame Gesprächshistorie. Ein Fachvalidator muss Aktionsrechte, Kosten, Reservierungen, konkrete Aktionsschemas und zukünftige Wecker prüfen. Die Zeichenbegrenzung des Notizbuchs ersetzt noch keine modellabhängige Tokenzählung.

Der Standard nutzt `response_format=json_object` und prüft den Antwortumschlag lokal. Für ein vollständiges Aktionsschema kann `ChatClient.complete(messages, schema=...)` striktes JSON Schema anfordern; ein vollständiger lokaler JSON-Schema-Validator ist noch nicht enthalten. Schemaunterstützung hängt vom gewählten Modell/Provider ab.

OpenRouter verwendet den offiziellen HTTPS-Endpunkt, `require_parameters=true` und deaktivierte Provider-Fallbacks. Diese Anfrageparameter sind nach der [OpenRouter-Schnellstartanleitung](https://openrouter.ai/docs/quickstart) und der [Structured-Outputs-Dokumentation](https://openrouter.ai/docs/guides/features/structured-outputs) umgesetzt (geprüft am 03.10.2026).

`max_requests` begrenzt Aufrufe je Clientinstanz, `max_tokens` die angeforderte Antwortlänge. Es gibt keine automatischen HTTP-Retries und keinen Wechsel von lokal zu Cloud. Diese Limits sind kein persistentes Dollarbudget: Vor einer langen bezahlten Epoche sind ein Anbieter-Ausgabenlimit, persistente Kostenabrechnung und ein freigegebenes Runbudget erforderlich. Bisherige Tests verwenden einen lokalen HTTP-Server beziehungsweise einen Mock; ein echter OpenRouter-End-to-End-Test steht noch aus.

Die vollständige Spielengine, Rollenwecker, Regeltexte, Budgettöpfe, Datensatzexporte und der Replaybetrachter werden separat integriert. Der aktuelle Stand darf nicht als fertiges Spiel oder bestätigte Spielbalance bezeichnet werden.

## Wiederaufnahme und persistente Budgets

Für längere Läufe zusätzlich `--journal D:/projekte_ki/Sternepoche/laeufe/epoche.sqlite3 --run-id epoche-001` übergeben. Das Zielverzeichnis muss existieren. Dieselbe Epochenkennung und dieselbe Datenbank nach Neustarts weiterverwenden; für den JSONL-Export jeweils eine neue `--output`-Datei verwenden.

Das SQLite-Journal reserviert jeden Aufruf vor dem Netzwerkzugriff atomar. Es speichert das vollständige Fenster, den tatsächlichen Prompt, Antworten, Nutzungsdaten und Fehler. Erfolgreich gespeicherte Antworten werden unverändert wiederverwendet. Ein Prozessabbruch zwischen Reservierung und Antwort lässt einen `pending`-Eintrag zurück: Der Lauf stoppt mit einer klaren Meldung, ohne einen möglicherweise bereits berechneten Aufruf zu wiederholen. Auch gespeicherte Fehler werden nicht heimlich erneut angefragt.

Regeln, Beobachtungen, Notizbücher, Rollenbesetzung und Modellkonfiguration werden gegen die gespeicherten Werte geprüft. Änderungen unter derselben Fensteridentität werden zurückgewiesen. Die Aufrufgrenze gilt mit Journal über Client- und Prozessneustarts hinweg je Rolle und Epoche. Mehrere Prozesse können dieselbe Anfrage nicht doppelt reservieren.

Dies garantiert keine einmalige Anwendung in der Engine: Sie muss importierte Fenster anhand ihrer eigenen Kennung und ihres Checkpoints gegen doppelte Anwendung schützen. Eine automatische Korrekturrunde und manuelle Auflösung unklarer `pending`-Einträge sind noch nicht implementiert.

Journal ohne Modellaufrufe für die spätere Auswertung exportieren:

```powershell
python -m orchestrator.audit --journal D:/projekte_ki/Sternepoche/laeufe/epoche.sqlite3 --run-id epoche-001 --output D:/projekte_ki/Sternepoche/laeufe/entscheidungen.jsonl
```

Der Export öffnet die Datenbank nur lesend und überschreibt keine Ausgabedateien. Er enthält die tatsächlich verwendeten lokalen Beobachtungen und Prompts, keine nachträglich ergänzten Weltzustände. Fehler und unvollständige Aufrufe bleiben erhalten. Noch fehlen Engine-Ereignisse, spätere Folgen und Trainingslabels; der Export ist kein fertig beschrifteter Trainingsdatensatz.
