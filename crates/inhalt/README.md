# Rust-Inhalte

`inhalt` ersetzt die Laufzeit der früheren Python-Bildpipeline und des JavaScript-Planetenrenderers. Der bestehende JSON-Katalog bleibt Inhaltsdaten. Der deterministische Spielkern benötigt diese Bibliothek nicht.

```powershell
$env:CARGO_HOME='D:/projekte_ki/Sternepoche/.cargo-cache'
cargo run -p inhalt --bin sternenepoche-inhalt -- validate
cargo run -p inhalt --bin sternenepoche-inhalt -- sync-engine
cargo run -p inhalt --bin sternenepoche-inhalt -- prepare
cargo run -p inhalt --bin sternenepoche-inhalt -- export-planets
```

Der Renderer liefert RGBA-Puffer und PNGs: Albedo, Höhe, Rauheit, Wolken und beleuchtete Orbitalansicht. Kugelrauschen sorgt für identische Texturränder; Pole werden explizit zusammengeführt. Rohstoffmarkierungen sind kosmetische Bänder des vom Spielkern beobachteten Faktors, keine simulierten geografischen Lagerstätten. Ohne `profile_known` und einen endlichen positiven Faktor wird kein Rohstoff angezeigt. In Materialkarten werden niemals Rohstoffdaten eingebrannt.

`prepare` schreibt Pilot- und Produktionsgraphen aus der vorhandenen geprüften Qwen-2.1-Baseline. Kein Netzwerk, Modellstart oder Download. GPU-Freigabe bleibt in `content/production.json` ausdrücklich `false`, solange Karls Trellis-Reservierung gilt.

`sync-engine` ergänzt die vier neu entworfenen Gebäudemotive Raketensilo, Orbitalring, Forschungsarchiv und Versorgungsnetz jeweils neutral und für alle vier Völker, sobald sie im Regelwerk vorhanden sind. Es liest Voraussetzungen direkt aus dem Rust-Regelwerk, aktualisiert die Regelherkunft und erhält bestehende Prompts sowie Liefermetadaten. Weitere unbekannte Spielinhalte werden bei `validate` als fehlend gemeldet, niemals durch generische Platzhalter verdeckt.

Nach einer tatsächlichen Freigabe kann `step --execute --asset ID --profile pilot` genau einen Auftrag voranbringen. Bei belegter Queue wird nichts eingereiht. Ein laufender Auftrag wird über die gespeicherte Prompt-ID abgefragt. Vor POST wird ein dauerhaftes `submitting`-Journal geschrieben; bei unklarer Antwort wird niemals automatisch nochmals gesendet. Das gemeinsame `.batch.lock` verhindert Parallelbetrieb mit dem alten Python-Skript. Bestehende Journale mit alten Fingerprints werden zur ausdrücklichen Abstimmung angehalten, nicht dupliziert. Fertige Ergebnisse werden mit SHA-256 und Workflow-Herkunft gespeichert, bleiben bis Sichtprüfung `reviewed: false`.

Nur HTTP-Loopback ist erlaubt, kein Proxy, kein Neustart von ComfyUI, keine fremden Jobs abbrechen. Die GPU-Freigabe wird vor Netzwerkzugriff und unmittelbar vor jedem POST geprüft. Queueprüfung und POST sind zwei ComfyUI-Anfragen; eine simultane fremde Einreichung zwischen beiden ist serverseitig nicht atomar ausschließbar. Deshalb ersetzt eine leere Queue niemals die explizite GPU-Freigabe.
