---
layout: default
title: "Dokumentation der Sternenepoche"
---

# Dokumentation der Sternenepoche

**Aktuelle Entwicklungspriorität:** autonome Spieler, persistente Sandboxes und strategische Harnesses;
zuerst Backend und Logik, später UI. Einstieg in die neue Zielarchitektur:
[LABOR-BACKEND.md](LABOR-BACKEND.md) beschreibt die implementierte Rust-Laufzeit;
[LABOR-ABNAHME.md](LABOR-ABNAHME.md) ihre Prüfungen. Das umfassendere
[SPIELER-SANDBOX-KONZEPT.md](SPIELER-SANDBOX-KONZEPT.md) beschreibt den umgesetzten B0–B7-Vertrag
und weitergehende Forschungsziele. Kapazitätsplanung, Balancebericht und Modell×Harness-Vergleiche sind
als CLI-Befehle verfügbar; die Abnahme trennt technische Fertigstellung von empirischen Leistungsbehauptungen.

Eine geschlossene Welt, in der 50 Zivilisationen ein Spieljahr lang um die höchste Punktzahl konkurrieren; jede
wird von vier Sprachmodellen in vier Rollen regiert, oder von einem Skriptbot, oder von dir. Der Einstieg mit
Schnellstart und Stand steht im `README.md` im Projektordner. Hier liegt alles Weitere.

| Dokument | Inhalt | Für wen |
|---|---|---|
| [LABOR-BACKEND.md](LABOR-BACKEND.md) | V4: native Rust-Büros ohne Docker, faire Entscheidungsfenster, Hauptmodelle, Ollama-only, Kolonieplanung, Besitzwechsel und versioniertes Lernen | Backend, Betrieb, Forschung |
| [LABOR-ABNAHME.md](LABOR-ABNAHME.md) | Ausgeführte Tests, echte Modellproben und offene Abnahmen | Prüfung, Forschung |
| [LABOR-REVIEW-SOL-HIGH.md](LABOR-REVIEW-SOL-HIGH.md) | Angeforderte Gegenprüfung mit gpt-6.1-sol high | Entwicklung |
| [SPIELER-SANDBOX-KONZEPT.md](SPIELER-SANDBOX-KONZEPT.md) | Neue Zielarchitektur: dauerhafte Spieler, anpassbare Rollen/Skills, Tool-Schnittstelle, Ollama/OpenRouter-Broker, Paarstarts, Audit, Lernen und Backend-Abnahme | Backend, Spieldesign, Forschung |
| [SPIELEN.md](SPIELEN.md) | Selbst spielen: Start, Bildschirm, Zeit, ein Fahrplan für die ersten Tage, jeder Bereich mit Bild, Stufen, Völker, Punkte, Speichern, Hilfe bei Problemen | alle, die spielen wollen |
| [LIVE-TEST.md](LIVE-TEST.md) | Mit echten Modellen live: selbst gegen KI-Reiche spielen (Spieler-Oberfläche) oder einem Lauf im Browser zuschauen; Checkliste | alle, die es ausprobieren wollen |
| [SPEZIFIKATION.md](SPEZIFIKATION.md) | Die Spielregeln: Zeit, Galaxie, Völker, Wirtschaft, Gebäude, Forschung, Stufen, Militär, Diplomatie, Markt, Wertung, Determinismus, Abweichungen vom Konzept | Spieldesign, Prüfung |
| [REGELWERK.md](REGELWERK.md) | Alle Zahlen als Tabellen, erzeugt aus `regeln/regelwerk.ron` | Nachschlagen |
| [REGELTEXT.md](REGELTEXT.md) | Der Regeltext, den die Modelle je Rolle bekommen, erzeugt aus dem Regelwerk | Prompt-Arbeit, Prüfung |
| [AGENTEN-SCHNITTSTELLE.md](AGENTEN-SCHNITTSTELLE.md) | Was ein Modell sieht, darf und antworten muss: Rollen, Antwortschema, alle Aktionen und Abfragen, Ablauf eines Aufrufs, Wecker, Informationsgrenze, Fehlerverhalten | Modellanbindung |
| [ARCHITEKTUR.md](ARCHITEKTUR.md) | Aufbau: Crates, Orchestratoren, Oberflächen, Zeitmodell, Determinismus, Brücke, Datenformate, Erweitern | Entwicklung |
| [BETRIEB.md](BETRIEB.md) | Bauen, Testen, alle Befehle, Konfigurationen, OpenRouter und Kosten, Fortsetzen, lange Läufe, Auswertung, Wissensdatenbank, Fehlerbehebung | Betrieb |
| [BALANCE.md](BALANCE.md) | Messung mit Skriptbots, Abnahmekriterien, Ergebnis, Änderungen, wie man Schieflagen findet | Balance |
| [ECHTER-LAUF.md](ECHTER-LAUF.md) | Der Lauf mit echten Modellen über OpenRouter: Modellwahl, Verlauf, Ergebnis gegen die Bots, Kosten, Befunde | Auswertung |

## Lesereihenfolge

- **Nächste Backend-Ausbaustufe:** SPIELER-SANDBOX-KONZEPT, danach ARCHITEKTUR und AGENTEN-SCHNITTSTELLE als Bestandsreferenz.
- **Selbst spielen:** SPIELEN (Doppelklick auf `Spielen.cmd`), gegen Sprachmodelle dann LIVE-TEST.
- **Modelle beobachten:** LIVE-TEST, dann SPEZIFIKATION in Auszügen.
- **Ein Modell anbinden oder Prompts ändern:** AGENTEN-SCHNITTSTELLE, REGELTEXT, dann BETRIEB (Konfiguration).
- **Am Code arbeiten:** ARCHITEKTUR, dann SPEZIFIKATION; vor dem Abschluss die Prüfungen aus ARCHITEKTUR 8.4.
- **Regeln oder Balance ändern:** SPEZIFIKATION, REGELWERK, BALANCE.

## Was erzeugt ist und was geschrieben

`REGELWERK.md` und `REGELTEXT.md` erzeugt die Engine:

```bash
./target/release/sternenepoche.exe doku
```

Ein Test (`doku_passt_zum_regelwerk` in `crates/lauf`) schlägt an, sobald sich das Regelwerk ändert und die beiden
Dateien nicht neu erzeugt wurden. Alle übrigen Dokumente sind geschrieben und gegen den Code geprüft (Stand
4. Okt. 2026); sie nennen zu jeder Mechanik die Datei und meist die Funktion, damit sich jede Aussage nachprüfen
lässt.

## Wissensdatenbank

Alle Dokumente, Quelltexte, das Regelwerk, der Inhaltskatalog und die Einträge aus `wissen/knowledge.json`
(Anforderungen, Entscheidungen, Befunde, offene Punkte) stehen außerdem durchsuchbar in einer DuckDB-Datenbank:

```bash
./target/release/sternenepoche-wissen.exe build
```

```bash
./target/release/sternenepoche-wissen.exe query wissen/current.json search "{\"q\":\"Weckregel\"}"
```

Lesend als HTTP-API (`serve`, Port 8197, nur 127.0.0.1) oder als MCP-Server (`mcp`). Einzelheiten in
BETRIEB, Abschnitt 9.
