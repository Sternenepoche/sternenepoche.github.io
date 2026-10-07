# Sternenepoche

**7. Oktober 2026 – gemeinsame Onlinewelt:** Rust-Dienst mit 30 Bots, 20 möglichen
Plätzen (zunächst drei freigegeben, weitere Anmeldungen auf Warteliste), Browser für Mensch/Agent/Mischbetrieb und getrenntem lokalem Verwaltungsdashboard.
Zunächst auf Karls PC, später mit derselben Datenbank auf einem VPS. Start über
`Server-starten.cmd`; Spiel `http://127.0.0.1:8890`, Dashboard `http://127.0.0.1:8891`.
System- und Planetensonden, Geheimdienst/Überwachung, Flottenspionage und Saven sind im
separaten Online-Regelsatz umgesetzt. GitHub Pages liefert den Browser; die Rust-Welt
benötigt den laufenden PC/VPS. Öffentlicher TLS-Zugang über Tailscale Funnel ist eingerichtet: [Online anmelden und spielen](https://desktop-3dei636.taila4f584.ts.net/). Mitspieler benötigen kein Tailscale.
[Inventur](docs/INVENTUR-2026-10-07.md) · [Konzept](docs/ONLINE-KONZEPT.md) ·
[Start, Verwaltung und VPS-Umzug](docs/SERVER-BETRIEB.md).


**Gemeinsame Spielwelt: 30 Skriptbots und 20 Plätze für Menschen, Agenten oder Mischbetrieb.**
Die vier Agentrollen sind Stratege, Verwalter, Feldherr und Diplomat; Anbieter und Modell
können je Rolle ausgewählt werden. Das bisherige geschlossene Labor mit 50 Modellreichen
bleibt als eigener Modus erhalten. GitHub Pages liefert Website und Browser aus.

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
