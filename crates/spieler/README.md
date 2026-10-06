# Native Rust player mode

`sternenepoche-spieler` opens a native egui desktop interface. It uses `kern::Welt`
directly, the same deterministic baseline bots as `lauf`, and the CPU planet renderer
from `inhalt`. It has no JavaScript or Python runtime and does not contact a model
provider or ComfyUI.

From the repository root in PowerShell:

```powershell
$env:CARGO_HOME = 'D:\projekte_ki\Sternepoche\.cargo-cache'
cargo run -p spieler --bin sternenepoche-spieler -- --seed 42 --player 0
cargo run -p spieler --bin sternenepoche-spieler -- --smoke-test
cargo test -p spieler
```

The new local game has one human and 49 baseline script players. `--player 0` selects
the human civilization at creation; the selected identity remains fixed in the save.
These are **not LLM opponents**. The Live-KI screen can add model opponents through the existing Rust provider bridge. All four roles are called every 15 game minutes; due windows must finish before time advances. Humans keep issuing immediate commands while models think. Model actions are revalidated on the live world instead of replacing it with a stale snapshot.

New sessions use colonization v2, including scouting, escorts, cargo, bombardment, occupation and repairs. Legacy saves retain their original colonization version.

Start paused at 1× real time. Resume to advance with the wall-time accumulator, select a simulation
speed, or advance one 15-minute decision window. Sleep/suspension does not trigger
unbounded offline catch-up. Planet economy, research, queues, fleet missions, combat,
trade, contracts, civilization requirements and event time are calculated by the core.

Views show only `Welt::sicht(human, Rolle::Alle)` and authorized `Welt::werkzeug`
results. No opponent stock, research, unscouted richness or hidden fleet is read by
the interface. Human commands use `Rolle::Alle`: the human is the entire government,
as with a script player, so government role budgets are not enforced. Physical costs,
ownership and all other action rules remain authoritative in `Welt::handeln`.

The 12 screens provide colonies, all buildings, research, ships/defense/components,
all ten fleet missions, galaxy, market, contracts/alliance/message controls,
civilization progression, reports and the native graphics catalog. The catalog marks
pending Qwen art honestly and can load completed PNG artwork without a browser.
Advanced JSON command input exposes additional
core actions before specialized forms exist. Every result and rejection is displayed.

Save files include the world, human identity, baseline bot memory, version and full
world checksum. Saving uses exclusive creation: choose a new name for each checkpoint.
Load pauses immediately. `--load path.sav` also loads at startup. Only saves from this
local player mode are accepted; existing headless runner checkpoints use another
format. Saves are local trusted files, not a network multiplayer protocol.

Art production is active under Karl’s 5 October authorization. Every completed PNG is loaded by engine key and faction after provenance and dimensions are checked. The gallery covers all 400 motifs; missing art is marked “Bild folgt”. Remaining presentation limitations:
research prerequisites are shown as core facts rather than invented visual dependency
arrows; several advanced diplomatic and new engine actions use the JSON command panel.
The planet silhouette is decorative, generated from its visible coordinate and climate;
the displayed production factors and quantities always come from the engine.
