# Native Rust-Trainingsumgebung

`kern::umgebung::Umgebung` bietet `reset(seed)` und `step(GemeinsameAktion)` ohne Python, Grafik-, Netzwerk- oder Modellabhängigkeiten. Dies sind Gymnasium-ähnliche Übergangssemantiken, keine registrierte Python-Gymnasium-Umgebung.

Die Konfiguration legt Spielerzahl, kontrollierte Spieler und maximales Schrittlimit fest. Jeder gemeinsame Schritt enthält exakt alle kontrollierten Spieler als `BTreeMap<SpielerId, Vec<RollenAktionen>>`. Eine leere Liste ist ein ausdrücklicher No-op. Nicht kontrollierte Spieler bleiben ausdrücklich untätig; auch ihre fälligen Rollenfenster werden geschlossen.

Die API prüft Teilnehmer, eindeutige Rollen, normale Aktionslimits, Objektform und Rollenfälligkeit vor jeder Zustandsänderung. Bei einem solchen Eingabefehler bleiben Spielzeit, Güter, Zufallszustand und Aktionsprotokoll unverändert. Fachlich ungültige Spielaktionen werden anschließend regulär vom Spielkern zurückgewiesen und im Ergebnis gemeldet; sie rollen andere gültige Aktionen desselben Fensters nicht zurück. `Rolle::Alle` ist als ausführende Rolle verboten. Bereits fällige Rollen werden in der deterministisch ausgelosten Reihenfolge des Spielkerns ausgeführt, dann vergeht ein reguläres Entscheidungsfenster.

Rückgabe: gefilterte Spielerbeobachtungen, exakte ganzzahlige Punktedifferenzen als Belohnung, `terminated` für das normale Epochenende, `truncated` für ein vorher erreichtes Trainingslimit sowie eigene Aktionsresultate und nächste fällige Rollen. Nach einem Ende wird `step` bis zum nächsten `reset` abgewiesen. `reset` mit gleichem Startwert und gleiche gemeinsame Aktionen reproduzieren dieselben Zustands-Hashes. Die API gibt keinen globalen Weltzustand oder verborgene Gegnerbestände zurück.

```rust
use kern::{Regelwerk, umgebung::Umgebung};
use std::collections::BTreeSet;

let rules = Regelwerk::laden(&std::fs::read_to_string("regeln/regelwerk.ron")?)?;
let mut env = Umgebung::neu(rules, 42, 50, (0..50).collect::<BTreeSet<_>>(), 2000)?;
let initial = env.reset(42)?;
let next = env.step(env.no_op())?;
assert_eq!(next.info.schritt, 1);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Die Trainingsumgebung erstellt keine Referenzaktionen, Bot-Gegner oder Train/Test-Aufteilung von selbst. Solche Politiken müssen explizit über die gemeinsamen Aktionen geliefert werden.
