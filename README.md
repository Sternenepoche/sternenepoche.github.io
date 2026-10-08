# Sternenepoche

**8. Oktober 2026 – gemeinsame Onlinewelt:** 30 serverseitige Skriptbots und 20 mögliche
Spielerplätze, davon zunächst drei freigegeben; weitere Anmeldungen kommen auf die Warteliste.
Menschen, lokale Ollama-Agenten, OpenRouter-Agenten und Mischbetrieb spielen dieselbe Rust-Welt.
Vor dem Start werden Volk und Spielweise gewählt; die Vor- und Nachteile kommen aus dem aktiven Regelprofil.

[Online anmelden und spielen](https://desktop-3dei636.taila4f584.ts.net/) ·
[Website](https://sternenepoche.github.io/) · [Spielanleitung](docs/SPIELEN.md) ·
[Architektur und Regeln](docs/ONLINE-KONZEPT.md) · [Betrieb und VPS-Umzug](docs/SERVER-BETRIEB.md).

GitHub Pages liefert Website und Browserclient. Der autoritative Rust-Dienst läuft zunächst auf Karls PC;
Tailscale Funnel liefert den öffentlichen TLS-Zugang ohne Tailscale-Pflicht für Mitspieler.
`Server-starten.cmd` startet Spiel (`http://127.0.0.1:8890`) und getrenntes lokales Dashboard
(`http://127.0.0.1:8891`). Das Dashboard wird nicht öffentlich ausgeliefert.

Der Menschenclient verbindet bebilderte Bau-/Forschungskacheln mit servergestützten Fortschrittsanzeigen
und einem räumlichen Sternenatlas: Galaxie → Sektor → Sonnensystem → Planet.
Systeme erscheinen als Sternwolken, Planeten laufen visuell um die zentrale Sonne.
Typ, Bewohner und Ressourcen bleiben bis zum jeweiligen Sondenbericht unbekannt.
Pause, Niederlage und Epochenende sperren Befehle einheitlich; Ansichten und Berichte bleiben lesbar.
Der Online-Regelsatz `regeln/online-v1.ron` umfasst Kolonisation, Aufklärung, Saven und dauerhaftes Ausscheiden.
Das geschlossene Forschungslabor und der native Einzelspielermodus bleiben eigene Betriebsarten.

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
- `regeln/online-v1.ron` – aktuelles Onlineprofil; `regeln/regelwerk.ron` – Basisprofil für Labor und Altstände (SHA-256 im Regelwerk-Text)
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
