# Rust-Agentenlauf

`sternenepoche-agenten` bindet `kern` als Rust-Bibliothek direkt ein. Für den Lauf werden weder Python noch ein JSON-Engineprozess benötigt. JSON bleibt das Dateiformat für Konfiguration und das öffentliche Antwortformat der Sprachmodelle.

```powershell
$env:CARGO_HOME='D:/projekte_ki/Sternepoche/.cargo-cache'
cargo run -p sternenepoche-agenten -- demo --out laeufe/rust-demo --windows 96
cargo run -p sternenepoche-agenten -- run --config konfig/rust-agenten-demo.json --out laeufe/rust-50 --windows 96
cargo run -p sternenepoche-agenten -- export --journal laeufe/rust-demo --out laeufe/rust-demo-parquet
cargo test -p sternenepoche-agenten
```

`demo` verwendet vier Zivilisationen und einen einfachen Wirtschafts-Regelagenten. Die Beispielkonfiguration demonstriert 50 Zivilisationen mit vier Rollen. Der Regelagent ist ein reproduzierbarer Integrationstest, kein trainiertes Modell und kein ausgewogener Konkurrent. Die Anzahl `--windows` begrenzt zusätzliche 15-Minuten-Fenster; derselbe Laufordner setzt automatisch fort.

## Modelle und Entscheidungsfenster

Jede Rolle kann einen eigenen Eintrag aus `anbieter` wählen. Lokale OpenAI-kompatible Chat-Completions und OpenRouter werden unterstützt. `konfig/rust-agenten-local.json` ist eine Vorlage mit Modell-Platzhaltern. `validate --config DATEI` prüft ohne Anfrage. Nur `run ... --execute` darf Netzwerk verwenden. OpenRouter verlangt zusätzlich `remote_erlaubt: true`, den offiziellen HTTPS-Endpunkt und einen API-Key aus der benannten Umgebungsvariable. Geheimnisse werden nicht in das Journal geschrieben. Weiterleitungen, automatische HTTP-Wiederholungen und OpenRouter-Anbieterfallback sind deaktiviert.

Die Engine erstellt alle Lagebilder vor den Aktionen. Bis zu `parallel` Aufrufe laufen gleichzeitig; die Spieluhr steht. Anschließend gibt es höchstens eine lesende Werkzeugrunde mit den echten Kernwerkzeugen (`kosten`, `flugzeit`, `kampfsimulator`, `regel`, `galaxie`). Aktionen folgen der von der Engine ausgelosten Reihenfolge. Abgelehnte Aktionen erhalten eine Korrekturrunde. Rollenrechte, Eigentum, Ressourcen, Flugzeiten und sonstige Spielregeln prüft ausschließlich der Kern. Rollen und Spielerkennungen kommen niemals aus einer Modellantwort. Wecker sind relative Spielstunden wie im Kernschema.

Antworten benötigen alle sechs Schemafelder. Doppelte JSON-Schlüssel, ungültige Zahlen, falsche Rollenaktionen, zu viele Aktionen/Abfragen sowie überlange Notizen/Begründungen werden abgelehnt. Ungültige Korrekturen werden protokolliert; bereits angenommene Aktionen gelten weiter.

## Persistenz

Ein Betriebssystem-Dateilock lässt genau einen Schreiber je Laufordner zu. Konfiguration und Regelwerkhash sind unveränderlich. Vor jedem externen Aufruf wird die vollständige Anfrage synchron gespeichert; nach Erfolg die unveränderliche Antwort. Erfolgreiche parallele Antworten werden auch dann gesichert, wenn ein anderer Aufruf fehlschlägt. Ein offener Aufruf ohne Antwort sperrt die Wiederaufnahme: nach Prozessabsturz, Timeout oder HTTP-Fehler wird keine möglicherweise bezahlte Anfrage automatisch wiederholt. Solche Fälle müssen anhand des Anbieters/Journals manuell geklärt werden. Journaldateien nicht blind löschen, um eine Wiederholung zu erzwingen.

Die Engine arbeitet pro Fenster auf einer Kopie. Vollständige Fenster werden mit Binärcheckpoint und SHA-256-Marker gespeichert. Nach einer Unterbrechung wird das unvollständige Fenster vom vorherigen Checkpoint aus mit gecachten Antworten erneut berechnet. Fehlende oder verfälschte Checkpoints werden abgelehnt. Dateien werden vor ihrer Veröffentlichung mit `sync_all` synchronisiert; eine Garantie gegen jeden Hardware-/Dateisystem-Stromausfall wird nicht behauptet.

`max_anfragen` zählt dauerhaft alle reservierten Aufrufe einschließlich Abfrage- und Korrekturrunden. `max_tokens` und Zeitlimit gelten pro Aufruf. Es gibt noch keine verbindliche USD-Budgetrechnung: Preisdaten und Anbieterkosten sind nicht vereinheitlicht. Token- und Anfragelimits sind keine garantierte Geldobergrenze.

## Parquet und Informationsgrenze

Der native Rust-Exporter erstellt echte `entscheidungen.parquet`, `aktionen.parquet` und `metriken.parquet`. Zeit, Spielerkennung, Rolle, Phase und Tokenanzahl sind typisierte Spalten; variable Prompts, Antworten und Aktionsdetails bleiben Zeichenfolgen. Der Export überschreibt keinen vorhandenen Ordner und sperrt das Journal während des Lesens.

Entscheidungseingaben enthalten ausschließlich die an `Welt::sicht` gefilterten Modellnachrichten. Die separate Metriktabelle enthält ausdrücklich als `label_only` markierte Zustandskennzahlen. Vollständige Checkpoints enthalten verborgene Weltinformationen und sind kein zulässiger Modelleingang. Ein Join zukünftiger Metriken in Beobachtungen würde Informationsleckage verursachen und wird hier nicht vorgenommen.

Noch nicht portiert sind Python-spezifische Anbieter-Vergleichsmodi, komfortable Laufauswertung sowie die vollständige Tabellensammlung für Kämpfe/Markt/Diplomatie als eigene Parquet-Schemas. Die bestehende Python-Implementierung bleibt als Migrationsreferenz erhalten.
