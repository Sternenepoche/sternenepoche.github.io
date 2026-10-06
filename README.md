# Sternenepoche

**Geschlossene Spielwelt: 50 Zivilisationen, ein Spieljahr (365 Tage), höchste Punktzahl.**
Jede Zivilisation wird von vier Sprachmodellen in vier Rollen regiert — Stratege, Verwalter,
Feldherr, Diplomat. Diese Seite ist die öffentliche Auslieferung des Projekts über GitHub Pages.

## Live

🌐 **https://sternenepoche.github.io/**

| Einstieg | Inhalt |
|---|---|
| [Spielen](docs/SPIELEN.html) | Anleitung und Steuerung des Spiels |
| [Spezifikation](docs/SPEZIFIKATION.html) | Die vollständige Mechanik, Abschnitt für Abschnitt |
| [Regelwerk-Referenz](docs/REGELWERK.html) | Alle Zahlen, Tabellen erzeugt aus `regeln/regelwerk.ron` |
| [Projekt-README](docs/PROJEKT-README.html) | Stand, Balance, Befunde aus dem Bau |
| [Bestandsaufnahme](Bestandsaufnahme.html) | Interaktive Übersicht über die Dateien |

## Im Repository

- `docs/` – gesamte Dokumentation inkl. Bilder
- `regeln/regelwerk.ron` – die eine Quelle aller Spielzahlen (SHA-256 im Regelwerk-Text)
- `crates/` – Rust-Workspace: `kern` (Engine), `lauf` (Fenster-Schleife), `agenten`, `spieler`, `inhalt`, `wissen`
- `orchestrator/` – Python-Laufleitung (OpenRouter/lokal)
- `betrachter/` – 3D-Ansicht der Spielwelt im Browser
- `konfig/`, `wissen/`, `COORDINATION.md`, `Cargo.toml`

Nicht enthalten (lokale Laufdaten, mehrere Gigabyte): `target/`, `content/`,
`wissen/snapshots/`, `tools/sandbox/`, `laeufe/`, `saves/`.

## Lokal bauen (Engine)

```sh
cargo test --workspace
cargo run -p sternenepoche-lauf -- --hilfe
```

## Lizenz und Stand

Regelwerk Version 0.1.0. Alle Begründungen der Zahlen stehen als Kommentare in
`regeln/regelwerk.ron`, die Mechanik in `docs/SPEZIFIKATION.md`.
