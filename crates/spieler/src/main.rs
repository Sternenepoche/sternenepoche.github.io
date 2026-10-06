#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod ansicht;

use ansicht::stil::{self, GOLD, GUT, LEISE, SCHLECHT};
use ansicht::{namen, Bildschirm};
use eframe::egui::{self, Color32, RichText};
use kern::{Gut, Regelwerk, SCHIFFE};
use serde_json::{json, Value};
use spieler::{RealtimeClock, Session};
use std::collections::BTreeMap;
use std::rc::Rc;
use sternenepoche_agenten::config::Config;
use std::{path::Path, time::Instant};

/// Names of all screens in navigation order, for `--screen` and the layout tests.
const SCREENS: [Bildschirm; 16] = Bildschirm::ALLE;

/// Project root, for configurations and the live folders under `laeufe/`.
fn root() -> std::path::PathBuf {
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    crate_dir
        .parent()
        .and_then(Path::parent)
        .unwrap_or(crate_dir)
        .to_path_buf()
}

/// A question before an action that cannot be undone (demolition, attack, leaving an alliance ...).
struct Rueckfrage {
    frage: String,
    was: Nachgefragt,
}

/// Was nach einem „Ja“ geschieht: ein Befehl an den Kern oder das Überschreiben eines Spielstands.
enum Nachgefragt {
    Befehl(Value),
    Ueberschreiben(String),
}

struct Player {
    session: Session,
    /// Rules of the running session (they never change within one), shared by all screens.
    regeln: Rc<Regelwerk>,
    clock: RealtimeClock,
    previous: Instant,
    paused: bool,
    speed: f64,
    /// Fast-forward target in simulation seconds (+1 hour, +1 day); pauses when reached.
    ziel_zeit: Option<i64>,
    screen: Bildschirm,
    colony: usize,
    status: String,
    status_ok: bool,
    /// The player's own recent orders with the core's answer, newest last.
    verlauf: Vec<(String, bool, String)>,
    rueckfrage: Option<Rueckfrage>,
    willkommen_aus: bool,
    ende_gesehen: bool,
    save_path: String,
    ereignisse_gelesen: i64,
    ereignisse_neuestes: i64,
    ereignisse_angesehen: bool,
    vorher: Bildschirm,
    neu_startwert: u64,
    target: String,
    mission: String,
    versorgungsziel: Option<u32>,
    ships: [i64; SCHIFFE],
    cargo: [i64; 10],
    flight_speed: f64,
    hold_hours: i64,
    flotten_vorschau: Option<(String, Result<Value, String>)>,
    flotten_simulation: Option<Result<Value, String>>,
    werft_reiter: usize,
    werft_anzahl: BTreeMap<String, i64>,
    raketen_anzahl: i64,
    rocket_type: String,
    rocket_target: String,
    good: String,
    order_side: String,
    price: f64,
    market_quantity: f64,
    diplo: ansicht::diplomatie::DiploForm,
    berichte_reiter: usize,
    prioritaeten_entwurf: Option<(String, Vec<String>, Vec<String>)>,
    steuer_entwurf: Option<(i64, i64)>,
    anleitung_suche: String,
    sector: u8,
    galaxie_cache: BTreeMap<u8, (i64, Vec<Value>)>,
    galaxie_auswahl: Option<String>,
    result: Value,
    command: String,
    template: usize,
    abfrage: String,
    abfrage_vorlage: usize,
    texture: Option<egui::TextureHandle>,
    texture_key: String,
    overlay: String,
    catalog: Option<inhalt::catalog::Catalog>,
    art_filter: String,
    art_textures: BTreeMap<String, ansicht::bilder::ArtCacheEntry>,
    screenshot_path: Option<std::path::PathBuf>,
    maximieren: u8,
    screenshot_frames: u32,
    /// Start form and filter of the live screen.
    live: LiveForm,
    /// A manual step hit the model window; it is taken as soon as the models have answered.
    step_after_models: bool,
}

struct LiveForm {
    config: String,
    empires: usize,
    players: usize,
    seed: u64,
    budget: f64,
    key: String,
    filter: String,
    only_errors: bool,
}

impl Default for LiveForm {
    fn default() -> Self {
        Self {
            config: "konfig/spieler-live.json".into(),
            empires: 3,
            players: 50,
            seed: 42,
            budget: 1.0,
            key: String::new(),
            filter: String::new(),
            only_errors: false,
        }
    }
}

/// A new folder under laeufe/ for a live game, never an existing one: two games must not share a journal
/// (milliseconds, and a counter for two starts within the same millisecond).
fn live_folder(offline: bool) -> std::path::PathBuf {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let base = root().join("laeufe");
    let name = format!("spieler-live-{}{stamp}", if offline { "attrappe-" } else { "" });
    let mut folder = base.join(&name);
    let mut n = 1;
    while folder.exists() {
        n += 1;
        folder = base.join(format!("{name}-{n}"));
    }
    folder
}

/// "Tag 3, 14:15" from simulation seconds.
fn game_time(seconds: i64) -> String {
    stil::spielzeit(seconds)
}

fn session_status(session: &Session) -> String {
    if session.ki_aktiv() {
        let n = session.ki_info()["reiche"].as_array().map_or(0, Vec::len);
        format!(
            "Live-Partie: {n} KI-Reiche über Modelle, {} Skriptbots. Modellaufrufe kosten Geld; Stand und Budget im Bereich Live-KI.",
            session.bot_count()
        )
    } else {
        format!(
            "Lokale Partie: ein Mensch und {} deterministische Skriptbots. Keine Modellaufrufe; KI-Reiche startet der Bereich Live-KI.",
            session.bot_count()
        )
    }
}

fn label(key: &str) -> String {
    namen::name(key)
}
fn text(v: &Value) -> String {
    if let Some(s) = v.as_str() {
        s.to_string()
    } else if v.is_null() {
        "—".into()
    } else {
        v.to_string()
    }
}
fn data(ui: &mut egui::Ui, value: &Value) {
    match value {
        Value::Object(map) => {
            for (k, v) in map {
                if v.is_object() || v.is_array() {
                    egui::CollapsingHeader::new(label(k))
                        .id_salt(k)
                        .default_open(true)
                        .show(ui, |ui| data(ui, v));
                } else {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new(format!("{}:", label(k))).color(LEISE));
                        ui.label(text(v));
                    });
                }
            }
        }
        Value::Array(a) => {
            if a.is_empty() {
                ui.weak("Keine Einträge");
            }
            for (n, v) in a.iter().enumerate() {
                ui.push_id(n, |ui| {
                    if v.is_object() {
                        ui.group(|ui| data(ui, v));
                    } else {
                        data(ui, v);
                    }
                });
            }
        }
        _ => {
            ui.label(text(value));
        }
    }
}

impl Player {
    fn new(ctx: &egui::Context, session: Session) -> Self {
        stil::thema(ctx);
        let sektor = heimat_sektor(&session);
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            status: session_status(&session),
            regeln: Rc::new(session.regeln().clone()),
            session,
            clock: RealtimeClock::default(),
            previous: Instant::now(),
            paused: true,
            speed: 1.0,
            ziel_zeit: None,
            screen: Bildschirm::Uebersicht,
            colony: 0,
            status_ok: true,
            verlauf: Vec::new(),
            rueckfrage: None,
            willkommen_aus: false,
            ende_gesehen: false,
            save_path: neuer_spielstand(),
            ereignisse_gelesen: -1,
            ereignisse_neuestes: -1,
            ereignisse_angesehen: false,
            vorher: Bildschirm::Uebersicht,
            neu_startwert: stamp % 1000,
            target: String::new(),
            mission: "transport".into(),
            ships: [0; SCHIFFE],
            cargo: [0; 10],
            flight_speed: 1.0,
            hold_hours: 1,
            flotten_vorschau: None,
            flotten_simulation: None,
            werft_reiter: 0,
            werft_anzahl: BTreeMap::new(),
            raketen_anzahl: 1,
            rocket_type: "abfang".into(),
            rocket_target: "raketenwerfer".into(),
            good: "erz".into(),
            order_side: "verkauf".into(),
            price: 1.0,
            market_quantity: 100.0,
            diplo: Default::default(),
            berichte_reiter: 0,
            prioritaeten_entwurf: None,
            steuer_entwurf: None,
            anleitung_suche: String::new(),
            sector: sektor,
            galaxie_cache: BTreeMap::new(),
            galaxie_auswahl: None,
            result: Value::Null,
            command: String::new(),
            template: 0,
            abfrage: String::new(),
            abfrage_vorlage: 0,
            texture: None,
            texture_key: String::new(),
            overlay: "keine".into(),
            catalog: inhalt::catalog::Catalog::load(root().join("content")).ok(),
            art_filter: String::new(),
            art_textures: BTreeMap::new(),
            versorgungsziel: None,
            screenshot_path: None,
            maximieren: 0,
            screenshot_frames: 0,
            live: LiveForm::default(),
            step_after_models: false,
        }
    }

    /// Replaces the running game (new live game, loaded save) and resets everything tied to it.
    fn neue_session(&mut self, session: Session) {
        self.regeln = Rc::new(session.regeln().clone());
        self.session = session;
        self.sector = heimat_sektor(&self.session);
        self.paused = true;
        self.ziel_zeit = None;
        self.clock.reset();
        self.colony = 0;
        self.texture_key.clear();
        self.galaxie_cache.clear();
        self.galaxie_auswahl = None;
        self.result = Value::Null;
        self.step_after_models = false;
        self.ende_gesehen = false;
        self.flotten_vorschau = None;
        self.versorgungsziel = None;
        self.prioritaeten_entwurf = None;
        self.steuer_entwurf = None;
        self.ereignisse_gelesen = -1;
        self.ereignisse_neuestes = -1;
        self.ereignisse_angesehen = false;
    }

    /// Sends an order through the core; its answer goes to the status line and the order history.
    fn act(&mut self, value: Value) {
        let typ = value["typ"].as_str().unwrap_or("?").to_string();
        let (ok, message) = self.session.act(value);
        let message = namen::woerter(&message);
        self.status_ok = ok;
        self.status = message.clone();
        self.verlauf.push((format!("{} · {}", game_time(self.session.seconds()), label(&typ)), ok, message));
        let zuviel = self.verlauf.len().saturating_sub(30);
        self.verlauf.drain(..zuviel);
        self.galaxie_cache.clear();
    }

    /// Speichert den Spielstand; `ersetzen` nur nach Rückfrage, wenn die Datei schon existiert.
    fn speichern(&mut self, pfad: &str, ersetzen: bool) {
        let ergebnis = if ersetzen { self.session.save_replace(Path::new(pfad)) } else { self.session.save(Path::new(pfad)) };
        match ergebnis {
            Ok(()) => {
                self.status_ok = true;
                self.status = format!("Gespeichert: {pfad}");
            }
            Err(e) => {
                self.status_ok = false;
                self.status = e;
            }
        }
    }

    /// Asks before an order that cannot be undone.
    fn bestaetigen(&mut self, frage: &str, aktion: Value) {
        self.rueckfrage = Some(Rueckfrage { frage: frage.to_string(), was: Nachgefragt::Befehl(aktion) });
    }
    fn gallery(&mut self, ui: &mut egui::Ui) {
        ansicht::hilfe::kopf(ui, Bildschirm::Galerie, &self.regeln.clone());
        ui.label(RichText::new("Diese Seite zeigt nur an, was schon gezeichnet ist und was noch fehlt; erzeugt wird hier nichts.").color(LEISE));
        ui.horizontal(|ui| {
            ui.label("Suche nach Motiv, Volk oder Kategorie");
            ui.text_edit_singleline(&mut self.art_filter);
        });
        let Some(mut catalog) = self.catalog.clone() else {
            ui.weak("Kein Inhaltskatalog am Projektpfad gefunden");
            return;
        };
        ui.label(format!(
            "{} Motive · Stil {}",
            catalog.assets.len(),
            catalog.style
        ));
        let filter = self.art_filter.to_lowercase();
        catalog.assets.sort_by_cached_key(|a| !root().join("content").join(&a.file).is_file());
        for asset in catalog.assets.iter().filter(|a| {
            format!("{} {} {}", a.id, a.label, a.category)
                .to_lowercase()
                .contains(&filter)
        }) {
            ui.push_id(&asset.id, |ui| {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        let generated = self.motiv_datei(ui, asset, egui::vec2(192.0, 128.0));
                        ui.vertical(|ui| {
                            ui.strong(&asset.label);
                            ui.weak(format!(
                                "{} · {}",
                                asset.category,
                                asset.faction.as_deref().unwrap_or("gemeinsam")
                            ));
                            ui.label(if generated {
                                "Qwen-Bild vorhanden · Herkunft geprüft"
                            } else {
                                "Qwen-Bild ausstehend"
                            });
                        });
                    });
                    egui::CollapsingHeader::new("Motiv und Herkunft").show(ui, |ui| {
                        ui.label(&asset.prompt);
                        ui.monospace(&asset.file);
                        ui.label(format!(
                            "Seed {} · {} × {}",
                            asset.seed, asset.width, asset.height
                        ));
                    });
                });
            });
        }
    }
    fn step(&mut self) {
        match self.session.step() {
            Ok(true) => {}
            Ok(false) if self.session.ki_denkt() => {
                self.step_after_models = true;
                self.status_ok = true;
                self.status = "KI-Reiche entscheiden; die Weltuhr wartet auf ihre Antworten".into();
            }
            Ok(false) => {
                self.paused = true;
                self.status = "Epoche beendet".into();
            }
            Err(e) => {
                self.paused = true;
                self.status_ok = false;
                self.status = e;
            }
        }
    }
    fn command(&mut self, ui: &mut egui::Ui, p: &Value) {
        ansicht::hilfe::kopf(ui, Bildschirm::Befehle, &self.regeln.clone());
        let templates = command_templates(p["koord"].as_str().unwrap_or("1:1:6"));
        stil::titel(ui, "⌨ Befehl: ändert das Spiel wie ein Klick");
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("templates")
                .selected_text(templates[self.template].0)
                .height(420.0)
                .show_ui(ui, |ui| {
                    for (i, (titel, _)) in templates.iter().enumerate() {
                        ui.selectable_value(&mut self.template, i, *titel);
                    }
                });
            if ui.button("Vorlage übernehmen").clicked() || self.command.is_empty() {
                self.command = serde_json::to_string_pretty(&templates[self.template].1).unwrap();
            }
        });
        ui.add(
            egui::TextEdit::multiline(&mut self.command)
                .code_editor()
                .desired_rows(8)
                .desired_width(f32::INFINITY),
        );
        if ui.button("Befehl ausführen").clicked() {
            match serde_json::from_str(&self.command) {
                Ok(v) => self.act(v),
                Err(e) => {
                    self.status_ok = false;
                    self.status = e.to_string();
                }
            }
        }

        stil::titel(ui, "🔍 Abfrage: liest nur, ändert nichts");
        let vorlagen = query_templates(p["koord"].as_str().unwrap_or("1:1:6"));
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("abfragen")
                .selected_text(vorlagen[self.abfrage_vorlage].0)
                .show_ui(ui, |ui| {
                    for (n, (titel, _)) in vorlagen.iter().enumerate() {
                        ui.selectable_value(&mut self.abfrage_vorlage, n, *titel);
                    }
                });
            if ui.button("Vorlage übernehmen").clicked() || self.abfrage.is_empty() {
                self.abfrage = serde_json::to_string_pretty(&vorlagen[self.abfrage_vorlage].1).unwrap();
            }
        });
        ui.add(egui::TextEdit::multiline(&mut self.abfrage).code_editor().desired_rows(6).desired_width(f32::INFINITY));
        if ui.button("Abfragen").clicked() {
            self.result = match serde_json::from_str::<Value>(&self.abfrage) {
                Ok(q) => self.session.query(&q).unwrap_or_else(|e| json!({"fehler": e})),
                Err(e) => json!({"fehler": e.to_string()}),
            };
        }
        if !self.result.is_null() {
            stil::karte(ui, |ui| data(ui, &self.result));
        }
    }
}

impl Player {
    /// Merges a finished model window and reports the human orders kept meanwhile.
    fn collect_models(&mut self) {
        let Some(done) = self.session.ki_abholen() else {
            return;
        };
        if let Some(e) = self.session.ki_info()["fehler"].as_str() {
            self.paused = true;
            self.step_after_models = false;
            self.status_ok = false;
            self.status = format!("KI-Fenster gescheitert: {e} – Bereich Live-KI");
            return;
        }
        if !done.is_empty() {
            let accepted = done.iter().filter(|d| d.1).count();
            let rejected: Vec<&str> = done.iter().filter(|d| !d.1).map(|d| d.2.as_str()).collect();
            self.status_ok = rejected.is_empty();
            self.status = format!(
                "Vorgemerkte Befehle ausgeführt: {accepted} von {} angenommen{}",
                done.len(),
                if rejected.is_empty() {
                    String::new()
                } else {
                    format!(" · abgelehnt: {}", rejected.join("; "))
                }
            );
        } else if self.status.starts_with("KI-Reiche entscheiden") {
            self.status_ok = true;
            self.status = "KI-Reiche haben entschieden".into();
        }
        if std::mem::take(&mut self.step_after_models) {
            self.step();
        }
    }

    fn start_live(&mut self, offline: bool) {
        let form = &self.live;
        let config = if offline {
            Ok(Config::demo(form.players))
        } else {
            let path = root().join(&form.config);
            std::fs::read_to_string(&path)
                .map_err(|e| format!("{}: {e}", path.display()))
                .and_then(|t| {
                    serde_json::from_str::<Config>(&t).map_err(|e| format!("{}: {e}", form.config))
                })
        };
        let config = match config {
            Ok(c) => c,
            Err(e) => {
                self.status_ok = false;
                self.status = e;
                return;
            }
        };
        // The key lives only in this process: never in a file, a save or the journal.
        let key = std::mem::take(&mut self.live.key);
        if !key.trim().is_empty() {
            for p in config.anbieter.values() {
                if let Some(var) = p.api_key_env.as_deref().filter(|v| !v.is_empty()) {
                    std::env::set_var(var, key.trim());
                }
            }
        }
        let folder = live_folder(offline);
        let form = &self.live;
        let empires = spieler::ki_reiche(form.players, 0, form.empires);
        match Session::with_models(
            form.seed,
            form.players,
            0,
            &empires,
            config,
            &folder,
            form.budget,
        ) {
            Ok(session) => {
                self.neue_session(session);
                self.save_path = neuer_spielstand();
                self.status_ok = true;
                self.status = format!(
                    "{} Pausiert: mit „Fortsetzen“ beginnt die Zeit, die KI-Reiche entscheiden im ersten Fenster.",
                    session_status(&self.session)
                );
            }
            Err(e) => {
                self.status_ok = false;
                self.status = e;
            }
        }
    }

    fn live_screen(&mut self, ui: &mut egui::Ui) {
        ansicht::hilfe::kopf(ui, Bildschirm::LiveKi, &self.regeln.clone());
        let info = self.session.ki_info();
        if info["aktiv"] == true {
            ui.horizontal_wrapped(|ui| {
                if info["denkt"] == true {
                    ui.spinner();
                    ui.strong(format!(
                        "Die Modelle entscheiden seit {} s, die Weltuhr wartet.",
                        info["denkt_sekunden"].as_u64().unwrap_or(0)
                    ));
                } else if info["fehler"].is_string() {
                    ui.colored_label(Color32::from_rgb(255, 147, 130), "Das letzte KI-Fenster ist gescheitert.");
                } else {
                    ui.strong("Bereit. Die KI-Reiche entscheiden in jedem Fenster, in dem eine ihrer Rollen dran ist.");
                }
            });
            if let Some(e) = info["fehler"].as_str() {
                ui.colored_label(Color32::from_rgb(255, 147, 130), e);
                ui.horizontal_wrapped(|ui| {
                    if ui.button("Erneut versuchen").clicked() {
                        self.session.ki_fehler_aufloesen(false);
                        self.status_ok = true;
                        self.status = "Neuer Versuch beim nächsten Fenster".into();
                    }
                    if ui
                        .button("Diese Aufrufe aussetzen und weiterspielen")
                        .clicked()
                    {
                        self.session.ki_fehler_aufloesen(true);
                        self.status_ok = true;
                        self.status = "Aufrufe ausgesetzt; die Rollen kommen beim nächsten Anlass wieder dran".into();
                    }
                });
                ui.weak("Ein offener Aufruf mit unbekanntem Ausgang wird nie automatisch wiederholt, weil der Anbieter ihn berechnet haben kann. Aussetzen ist dann der sichere Weg.");
            }
            let kosten = info["kosten"].as_f64().unwrap_or(0.0);
            let tage = self.session.seconds() as f64 / 86_400.0;
            egui::Grid::new("live-status")
                .num_columns(2)
                .striped(true)
                .show(ui, |ui| {
                    if let Some(m) = info["modelle"].as_object() {
                        for (rolle, modell) in m {
                            ui.label(label(rolle));
                            ui.monospace(text(modell));
                            ui.end_row();
                        }
                    }
                    ui.label("Kosten");
                    ui.horizontal(|ui| {
                        ui.label(format!("{kosten:.4} USD"));
                        if tage >= 0.5 {
                            ui.weak(format!("· {:.4} USD je Spieltag", kosten / tage));
                        }
                    });
                    ui.end_row();
                    ui.label("Budget");
                    ui.horizontal(|ui| {
                        let mut budget = info["budget"].as_f64().unwrap_or(0.0);
                        if ui
                            .add(
                                egui::DragValue::new(&mut budget)
                                    .range(0.0..=100.0)
                                    .speed(0.05)
                                    .suffix(" USD"),
                            )
                            .changed()
                        {
                            self.session.ki_budget_setzen(budget);
                        }
                        ui.weak("Ist es erreicht, hält die Partie vor dem nächsten KI-Fenster an.");
                    });
                    ui.end_row();
                    ui.label("Entscheidungen");
                    ui.label(format!(
                        "{} · {} Modellanfragen",
                        text(&info["entscheidungen"]),
                        text(&info["anfragen"])
                    ));
                    ui.end_row();
                    ui.label("Protokoll");
                    ui.monospace(text(&info["ordner"]));
                    ui.end_row();
                });
            if info["vorgemerkt"].as_u64().unwrap_or(0) > 0 {
                ui.label(format!(
                    "{} eigene Befehle vorgemerkt, sie folgen nach den KI-Entscheidungen.",
                    text(&info["vorgemerkt"])
                ));
            }
            if let Some(offen) = info["offen"].as_array().filter(|o| !o.is_empty()) {
                egui::CollapsingHeader::new(format!("{} Rollen sind in diesem Fenster dran", offen.len()))
                    .show(ui, |ui| {
                        for o in offen {
                            ui.label(format!(
                                "{} · {} · {}",
                                text(&o["name"]),
                                label(o["rolle"].as_str().unwrap_or("")),
                                o["gruende"]
                                    .as_array()
                                    .map(|g| g.iter().map(text).collect::<Vec<_>>().join(", "))
                                    .unwrap_or_default()
                            ));
                        }
                    });
            }
            ui.separator();
            ui.strong("KI-Reiche");
            egui::Grid::new("live-reiche").striped(true).show(ui, |ui| {
                for h in ["Reich", "Volk", "Rang", "Punkte", "Stufe", "Planeten"] {
                    ui.strong(h);
                }
                ui.end_row();
                for r in info["reiche"].as_array().into_iter().flatten() {
                    ui.label(text(&r["name"]));
                    ui.label(label(r["volk"].as_str().unwrap_or("")));
                    for k in ["rang", "punkte", "stufe", "planeten"] {
                        ui.label(text(&r[k]));
                    }
                    ui.end_row();
                }
            });
            ui.separator();
            ui.horizontal_wrapped(|ui| {
                ui.strong("Entscheidungen der Modelle");
                let names: Vec<String> = info["reiche"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|r| r["name"].as_str().map(String::from))
                    .collect();
                egui::ComboBox::from_id_salt("live-filter")
                    .selected_text(if self.live.filter.is_empty() {
                        "Alle Reiche".to_string()
                    } else {
                        self.live.filter.clone()
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.live.filter, String::new(), "Alle Reiche");
                        for n in names {
                            ui.selectable_value(&mut self.live.filter, n.clone(), n);
                        }
                    });
                ui.checkbox(&mut self.live.only_errors, "nur mit Ablehnungen oder Fehlern");
            });
            let reports: Vec<Value> = self
                .session
                .ki_protokoll()
                .iter()
                .rev()
                .filter(|r| self.live.filter.is_empty() || r["name"] == self.live.filter.as_str())
                .filter(|r| {
                    !self.live.only_errors
                        || r["fehler"].as_array().is_some_and(|f| !f.is_empty())
                })
                .take(60)
                .cloned()
                .collect();
            if reports.is_empty() {
                ui.weak("Noch keine Entscheidungen. Mit „Fortsetzen“ oder „+ 15 Spielminuten“ beginnt das erste Fenster.");
            }
            for (i, r) in reports.iter().enumerate() {
                ui.push_id(i, |ui| live_report(ui, r));
            }
        } else {
            ui.label("Diese Partie läuft nur mit Skriptbots. Hier startet eine neue Partie, in der Sprachmodelle einige Reiche regieren: je Reich vier Rollen (Stratege, Verwalter, Feldherr, Diplomat), dieselben Aktionen und Regeln wie in den Orchestrator-Läufen.");
        }
        ui.separator();
        ui.heading("Neue Partie");
        ui.label("Du spielst das erste Reich, die nächsten regieren Sprachmodelle, alle übrigen Skriptbots. Die laufende Partie wird dabei verworfen; vorher speichern, wenn sie bleiben soll.");
        egui::Grid::new("live-form").num_columns(2).show(ui, |ui| {
            ui.label("Modellkonfiguration");
            ui.text_edit_singleline(&mut self.live.config);
            ui.end_row();
            ui.label("KI-Reiche");
            ui.add(egui::DragValue::new(&mut self.live.empires).range(1..=12));
            ui.end_row();
            ui.label("Reiche insgesamt");
            ui.add(egui::DragValue::new(&mut self.live.players).range(2..=50));
            ui.end_row();
            ui.label("Startwert");
            ui.add(egui::DragValue::new(&mut self.live.seed));
            ui.end_row();
            ui.label("Budget");
            ui.add(
                egui::DragValue::new(&mut self.live.budget)
                    .range(0.0..=100.0)
                    .speed(0.05)
                    .suffix(" USD"),
            );
            ui.end_row();
            ui.label("OpenRouter-Schlüssel");
            ui.vertical(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.live.key)
                        .password(true)
                        .hint_text(
                            if std::env::var("OPENROUTER_API_KEY").is_ok_and(|k| !k.trim().is_empty()) {
                                "OPENROUTER_API_KEY ist gesetzt"
                            } else {
                                "sk-or-…"
                            },
                        ),
                );
                ui.weak("Nur nötig, wenn OPENROUTER_API_KEY nicht gesetzt ist. Der Schlüssel bleibt im Speicher dieses Programms, nie in einer Datei.");
            });
            ui.end_row();
        });
        self.live.empires = self.live.empires.min(self.live.players.saturating_sub(1)).max(1);
        ui.horizontal_wrapped(|ui| {
            if ui.button("Partie mit KI-Reichen starten").clicked() {
                self.start_live(false);
            }
            if ui.button("Attrappe ohne Modell und Kosten").clicked() {
                self.start_live(true);
            }
        });
        ui.weak("Gemessen am 4. Okt. 2026 mit den Modellen aus konfig/spieler-live.json: rund 0,001 USD je Modellaufruf und 30 bis 40 Aufrufe je KI-Reich und Spieltag in der Aufbauphase. Jedes Fenster, in dem eine KI-Rolle dran ist, wartet 5 bis 30 Sekunden auf die Antworten.");
    }
}

/// One model decision: who, why, what was ordered and how the core answered.
fn live_report(ui: &mut egui::Ui, r: &Value) {
    ui.group(|ui| {
        ui.horizontal_wrapped(|ui| {
            ui.strong(format!(
                "{} · {} · {}",
                game_time(r["zeit"].as_i64().unwrap_or(0)),
                text(&r["name"]),
                label(r["rolle"].as_str().unwrap_or(""))
            ));
            if let Some(g) = r["gruende"].as_array() {
                ui.weak(g.iter().map(text).collect::<Vec<_>>().join(", "));
            }
            ui.weak(format!("{:.4} USD", r["kosten"].as_f64().unwrap_or(0.0)));
        });
        if let Some(b) = r["begruendung"].as_str() {
            ui.label(b);
        }
        for a in r["aktionen"].as_array().into_iter().flatten() {
            let ok = a["ok"] == true;
            let mut befehl = a["aktion"].to_string();
            if befehl.chars().count() > 180 {
                befehl = befehl.chars().take(180).collect::<String>() + "…";
            }
            ui.horizontal_wrapped(|ui| {
                ui.colored_label(
                    if ok {
                        Color32::from_rgb(152, 207, 175)
                    } else {
                        Color32::from_rgb(255, 147, 130)
                    },
                    if ok { "✔" } else { "✖" },
                );
                ui.monospace(befehl);
                if !ok || a["korrektur"] == true {
                    ui.weak(text(&a["text"]));
                }
            });
        }
        for f in r["fehler"].as_array().into_iter().flatten() {
            ui.colored_label(Color32::from_rgb(255, 147, 130), text(f));
        }
        egui::CollapsingHeader::new("Notiz und Prognose").show(ui, |ui| {
            ui.label(text(&r["notiz"]));
            ui.weak(text(&r["prognose"]));
            if let Some(h) = r["wecker_stunden"].as_f64() {
                ui.weak(format!("Wecker in {h} Spielstunden"));
            }
        });
    });
}

impl Player {
    /// Advances the clock: in real time while running, or window by window towards a fast-forward target.
    fn uhr(&mut self, ctx: &egui::Context) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.previous).as_secs_f64();
        self.previous = now;
        self.collect_models();
        if self.session.ki_denkt() {
            ctx.request_repaint_after(std::time::Duration::from_millis(250));
        }
        if let Some(ziel) = self.ziel_zeit {
            // Precise: step until the target, at most 16 windows per frame, waiting for model windows.
            let mut n = 0;
            while self.session.seconds() < ziel && n < 16 && !self.session.ki_denkt() && !self.session.ended() {
                match self.session.step() {
                    Ok(true) => n += 1,
                    Ok(false) => break,
                    Err(e) => {
                        self.status_ok = false;
                        self.status = e;
                        self.ziel_zeit = None;
                        break;
                    }
                }
            }
            if self.session.seconds() >= ziel || self.session.ended() {
                self.ziel_zeit = None;
                self.status_ok = true;
                self.status = format!("Angehalten bei {}.", game_time(self.session.seconds()));
            }
            ctx.request_repaint_after(std::time::Duration::from_millis(16));
            return;
        }
        if let Err(e) = self.clock.update(&mut self.session, elapsed, self.speed, self.paused) {
            self.paused = true;
            self.status_ok = false;
            self.status = e;
        }
        if !self.paused {
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        }
    }

    fn vorspulen(&mut self, sekunden: i64) {
        self.paused = true;
        self.clock.reset();
        self.ziel_zeit = Some(self.session.seconds() + sekunden);
    }

    fn kopfleiste(&mut self, ui: &mut egui::Ui, v: &Value) {
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("STERNENEPOCHE").size(24.0).strong().color(GOLD));
            ui.add_space(8.0);
            let volk = v["volk"].as_str().unwrap_or("");
            ui.label(RichText::new(text(&v["name"])).size(18.0).strong())
                .on_hover_text("Name deines Reichs");
            ui.label(RichText::new(format!("({})", label(volk))).color(LEISE)).on_hover_text(namen::volk_text(volk));
            ui.separator();
            let tag = stil::i(&v["tag"]);
            let tage = stil::i(&v["epoche_tage"]).max(1);
            ui.label(RichText::new(text(&v["zeit"])).size(17.0)).on_hover_text(format!("Spieltag {tag} von {tage}"));
            ui.add(egui::ProgressBar::new(tag as f32 / tage as f32).desired_width(90.0).desired_height(8.0).fill(GOLD.linear_multiply(0.6)))
                .on_hover_text(format!("Epoche: Tag {tag} von {tage}"));
            ui.separator();
            ui.label(RichText::new(format!("👑 Stufe {}", stil::roemisch(stil::i(&v["stufe"]))))).on_hover_text("Zivilisationsstufe");
            ui.label(format!("🏆 Rang {} von {}", stil::z(&v["rang"]), stil::z(&v["spielerzahl"])));
            ui.label(RichText::new(format!("{} Punkte", stil::z(&v["punkte"]["gesamt"]))).color(GOLD));
            ui.label(format!("💰 {} Credits", stil::z(&v["credits"])));
        });
        ui.horizontal_wrapped(|ui| {
            let laeuft = !self.paused || self.ziel_zeit.is_some();
            let knopf = if laeuft { "⏸ Anhalten" } else { "▶ Weiter" };
            if ui.add(egui::Button::new(RichText::new(knopf).strong()).fill(if laeuft { Color32::from_rgb(60, 92, 120) } else { Color32::from_rgb(46, 110, 70) })).clicked() {
                if self.ziel_zeit.is_some() {
                    self.ziel_zeit = None;
                    self.paused = true;
                } else {
                    self.paused = !self.paused;
                }
                self.clock.reset();
                self.status_ok = true;
                self.status = if self.paused {
                    format!("Angehalten bei {}.", game_time(self.session.seconds()))
                } else {
                    format!("Die Zeit läuft ({}).", tempo_name(self.speed))
                };
            }
            if ui.button("+15 min").on_hover_text("Ein Entscheidungsfenster weiter, dann anhalten").clicked() {
                self.paused = true;
                self.clock.reset();
                self.step();
            }
            if ui.button("+1 Stunde").clicked() {
                self.vorspulen(3600);
            }
            if ui.button("+1 Tag").clicked() {
                self.vorspulen(86_400);
            }
            ui.label(RichText::new("Tempo").color(LEISE));
            egui::ComboBox::from_id_salt("tempo").selected_text(tempo_name(self.speed)).show_ui(ui, |ui| {
                for s in [1.0, 60.0, 900.0, 3600.0, 86_400.0] {
                    ui.selectable_value(&mut self.speed, s, tempo_name(s));
                }
            });
            if let Some(z) = self.ziel_zeit {
                ui.spinner();
                ui.label(RichText::new(format!("spule vor bis {}", game_time(z))).color(GOLD));
            }
            let info = self.session.ki_info();
            if info["aktiv"] == true {
                ui.separator();
                ui.label(format!("💻 {} KI-Reiche · {:.3} USD", info["reiche"].as_array().map_or(0, Vec::len), info["kosten"].as_f64().unwrap_or(0.0)));
                if info["denkt"] == true {
                    ui.spinner();
                    ui.label(RichText::new(format!("KI entscheidet seit {} s – die Weltuhr wartet", info["denkt_sekunden"].as_u64().unwrap_or(0))).color(GOLD));
                }
            } else {
                ui.label(RichText::new(format!("· {} Skriptbots", self.session.bot_count())).color(LEISE));
            }
        });
        ui.add_space(2.0);
    }

    /// Resources of the active colony: stock, change per hour and how full the storage is.
    fn rohstoffleiste(&mut self, ui: &mut egui::Ui, p: &Value) {
        if p.is_null() {
            return;
        }
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 14.0;
            for g in Gut::ALLE {
                let k = g.name();
                let (b, r, l) = (stil::i(&p["bestand"][k]), stil::i(&p["rate"][k]), stil::i(&p["lager"][k]).max(1));
                if b == 0 && r == 0 && matches!(g, Gut::Xenokristall | Gut::Antriebskern | Gut::Habitatmodul | Gut::Legierung | Gut::Elektronik | Gut::Konsumgut) {
                    continue;
                }
                let anteil = b as f32 / l as f32;
                let gruppe = ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 4.0;
                        if !self.motiv_groesse(ui, "resources", k, p, egui::vec2(24.0, 22.0)) {
                            ui.label(RichText::new(namen::gut_zeichen(k)).size(16.0));
                        }
                        ui.label(RichText::new(stil::zahl(b)).strong());
                        ui.label(RichText::new(stil::rate(r)).small().color(if r > 0 { GUT } else if r < 0 { SCHLECHT } else { LEISE }));
                    });
                    ui.add(egui::ProgressBar::new(anteil.clamp(0.0, 1.0)).desired_width(92.0).desired_height(4.0).fill(stil::fuellfarbe(anteil)));
                });
                stil::ueber(ui, gruppe.response).on_hover_text(format!(
                    "{}: {} von {} Lagerplatz, {} je Stunde{}",
                    label(k),
                    stil::zahl(b),
                    stil::zahl(l),
                    stil::rate(r),
                    match p["voll_in_stunden"][k].as_i64() {
                        Some(0) => " · Lager voll, Produktion steht".to_string(),
                        Some(h) => format!(" · voll {}", stil::in_zeit(h * 60)),
                        None => String::new(),
                    }
                ));
            }
            ui.separator();
            let (e, ver) = (stil::i(&p["energie"]["erzeugung"]), stil::i(&p["energie"]["verbrauch"]));
            ui.label(RichText::new(format!("⚡ {} / {}", stil::zahl(e), stil::zahl(ver))).color(if ver > e { SCHLECHT } else { GUT }))
                .on_hover_text("Strom: erzeugt / gebraucht. Fehlt Strom, arbeiten alle Anlagen nur anteilig.");
            ui.label(format!("👥 {}", stil::z(&p["bevoelkerung"]))).on_hover_text(format!("Einwohner, Wohnraum {}", stil::z(&p["wohnraum"])));
        });
    }

    fn navigation(&mut self, ui: &mut egui::Ui, v: &Value, planets: &[Value]) {
        let neue = v["nachrichten"].as_array().map_or(0, |n| n.iter().filter(|x| x["neu"] == true).count())
            + v["vertraege"].as_array().map_or(0, |n| n.iter().filter(|x| x["status"] == "angebot an dich").count());
        let angriffe = v["angriffe"].as_array().map_or(0, Vec::len);
        let ereignisse = ansicht::berichte::ungelesen(v, self.ereignisse_gelesen);
        let mut gruppe = "";
        egui::ScrollArea::vertical().id_salt("nav").show(ui, |ui| {
            for b in SCREENS {
                if b.gruppe() != gruppe {
                    gruppe = b.gruppe();
                    ui.add_space(6.0);
                    ui.label(RichText::new(gruppe.to_uppercase()).small().color(LEISE));
                }
                let zusatz = match b {
                    Bildschirm::Diplomatie if neue > 0 => format!("  ({neue})"),
                    Bildschirm::Flotten if angriffe > 0 => format!("  ⚔{angriffe}"),
                    Bildschirm::Berichte if ereignisse > 0 => format!("  ({ereignisse})"),
                    _ => String::new(),
                };
                let text = RichText::new(format!("{}  {}{zusatz}", b.zeichen(), b.name())).size(16.0);
                if ui.selectable_label(self.screen == b, text).clicked() {
                    self.screen = b;
                    self.result = Value::Null;
                }
            }
            ui.separator();
            ui.label(RichText::new("AKTIVE KOLONIE").small().color(LEISE));
            egui::ComboBox::from_id_salt("colony")
                .selected_text(planets.get(self.colony).map(|p| text(&p["koord"])).unwrap_or_default())
                .show_ui(ui, |ui| {
                    for (n, p) in planets.iter().enumerate() {
                        let art = if p["heimat"] == true { "Heimatwelt" } else { "Kolonie" };
                        ui.selectable_value(&mut self.colony, n, format!("{} ({art})", text(&p["koord"])));
                    }
                });
            ui.separator();
            ui.label(RichText::new("SPIELSTAND").small().color(LEISE));
            ui.add(egui::TextEdit::singleline(&mut self.save_path).desired_width(170.0));
            // Vorhandene Spielstände zur Auswahl, neueste zuerst; gelesen nur, solange die Liste offen ist.
            egui::ComboBox::from_id_salt("spielstaende").width(170.0).selected_text("Spielstand wählen …").show_ui(ui, |ui| {
                let mut staende: Vec<(std::time::SystemTime, String)> = std::fs::read_dir("saves")
                    .into_iter()
                    .flatten()
                    .flatten()
                    .filter(|e| e.path().extension().is_some_and(|x| x == "sav"))
                    .map(|e| (e.metadata().and_then(|m| m.modified()).unwrap_or(std::time::UNIX_EPOCH), e.path().to_string_lossy().replace('\\', "/")))
                    .collect();
                staende.sort_by(|a, b| b.0.cmp(&a.0));
                if staende.is_empty() {
                    ui.label(RichText::new("Noch nichts gespeichert.").color(LEISE));
                }
                for (_, pfad) in staende.into_iter().take(25) {
                    if ui.selectable_label(self.save_path == pfad, &pfad).clicked() {
                        self.save_path = pfad;
                    }
                }
            });
            ui.horizontal(|ui| {
                if ui.button("💾 Speichern").clicked() {
                    let pfad = self.save_path.clone();
                    if Path::new(&pfad).exists() {
                        self.rueckfrage = Some(Rueckfrage {
                            frage: format!("Den Spielstand {pfad} gibt es schon. Mit dem jetzigen Stand überschreiben?"),
                            was: Nachgefragt::Ueberschreiben(pfad),
                        });
                    } else {
                        self.speichern(&pfad, false);
                    }
                }
                if ui.button("📂 Laden").clicked() {
                    match Session::load(Path::new(&self.save_path)) {
                        Ok(s) => {
                            self.neue_session(s);
                            self.status_ok = true;
                            self.status = format!("Spielstand geladen; pausiert. {}", session_status(&self.session));
                        }
                        Err(e) => {
                            self.status_ok = false;
                            self.status = e;
                        }
                    }
                }
            });
            egui::CollapsingHeader::new("🆕 Neue Partie").id_salt("neue-partie").show(ui, |ui| {
                ui.label(RichText::new("Du gegen 49 Skriptbots. Jeder Startwert ergibt eine andere Galaxie und ein anderes Volk. Die laufende Partie geht verloren, wenn du sie nicht speicherst.").small().color(LEISE));
                ui.horizontal(|ui| {
                    ui.label("Startwert");
                    ui.add(egui::DragValue::new(&mut self.neu_startwert).range(0..=999_999));
                });
                if ui.button("Partie beginnen").clicked() {
                    match Session::new(self.neu_startwert, 50, 0) {
                        Ok(s) => {
                            self.neue_session(s);
                            self.save_path = neuer_spielstand();
                            self.willkommen_aus = false;
                            self.screen = Bildschirm::Uebersicht;
                            let v = self.session.view();
                            self.status_ok = true;
                            self.status = format!("Neue Partie, Startwert {}: Du regierst {} ({}). Pausiert, mit ▶ läuft die Zeit.",
                                self.neu_startwert, ansicht::hilfe::text(&v["name"]), namen::name(v["volk"].as_str().unwrap_or("")));
                        }
                        Err(e) => {
                            self.status_ok = false;
                            self.status = e;
                        }
                    }
                }
            });
            egui::CollapsingHeader::new("⚙ Ansicht").id_salt("ansicht-einstellungen").show(ui, |ui| {
                let zoom = ui.ctx().zoom_factor();
                ui.horizontal(|ui| {
                    ui.label("Schriftgröße");
                    if ui.add_enabled(zoom > 0.75, egui::Button::new("A−")).clicked() {
                        ui.ctx().set_zoom_factor((zoom - 0.1).max(0.7));
                    }
                    ui.label(format!("{:.0} %", zoom * 100.0));
                    if ui.add_enabled(zoom < 1.95, egui::Button::new("A+")).clicked() {
                        ui.ctx().set_zoom_factor((zoom + 0.1).min(2.0));
                    }
                    if ui.button("100 %").clicked() {
                        ui.ctx().set_zoom_factor(1.0);
                    }
                });
                ui.label(RichText::new("Auch mit Strg + Plus / Minus, Strg + 0 setzt zurück. Mit Tab, Leertaste und Enter lässt sich alles ohne Maus bedienen. Das Spiel hat keinen Ton und keine Animationen außer der Ladeanzeige; jeder Zustand steht auch als Text da, nicht nur als Farbe.").small().color(LEISE));
            });
        });
    }

    fn render(&mut self, ctx: &egui::Context) {
        self.uhr(ctx);
        // Wer die Ereignisse angesehen hat und den Bereich verlässt, hat sie gelesen.
        if self.screen != self.vorher {
            if self.vorher == Bildschirm::Berichte && self.ereignisse_angesehen {
                self.ereignisse_gelesen = self.ereignisse_gelesen.max(self.ereignisse_neuestes);
                self.ereignisse_angesehen = false;
            }
            self.vorher = self.screen;
        }
        let v = self.session.view();
        let planets = v["planeten"].as_array().cloned().unwrap_or_default();
        self.colony = self.colony.min(planets.len().saturating_sub(1));
        let p = planets.get(self.colony).cloned().unwrap_or(Value::Null);
        egui::TopBottomPanel::top("top").frame(egui::Frame::new().fill(Color32::from_rgb(10, 17, 26)).inner_margin(egui::Margin::symmetric(12, 6))).show(ctx, |ui| {
            self.kopfleiste(ui, &v);
        });
        egui::TopBottomPanel::top("rohstoffe").frame(egui::Frame::new().fill(Color32::from_rgb(15, 26, 38)).inner_margin(egui::Margin::symmetric(12, 6))).show(ctx, |ui| {
            self.rohstoffleiste(ui, &p);
        });
        egui::SidePanel::left("navigation").resizable(false).exact_width(205.0).show(ctx, |ui| {
            self.navigation(ui, &v, &planets);
        });
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(if self.status_ok { "✔" } else { "✖" }).color(if self.status_ok { GUT } else { SCHLECHT }));
                ui.label(RichText::new(&self.status).color(if self.status_ok { Color32::from_rgb(170, 214, 186) } else { SCHLECHT }));
                if !self.verlauf.is_empty() {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.menu_button(format!("Deine letzten Befehle ({})", self.verlauf.len()), |ui| {
                            for (wann, ok, antwort) in self.verlauf.iter().rev() {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(if *ok { "✔" } else { "✖" }).color(if *ok { GUT } else { SCHLECHT }));
                                    ui.label(RichText::new(wann).color(LEISE));
                                    ui.label(antwort);
                                });
                            }
                        });
                    });
                }
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().id_salt(("inhalt", self.screen as usize)).show(ui, |ui| {
                // Platz für die Bildlaufleiste lassen, damit rechts nichts abgeschnitten wird; auf breiten
                // Bildschirmen steht der Inhalt mittig statt am linken Rand.
                let verfuegbar = ui.available_width() - 18.0;
                let breite = verfuegbar.min(1100.0);
                let rand = ((verfuegbar - breite) / 2.0).floor().max(0.0);
                ui.horizontal_top(|ui| {
                ui.add_space(rand);
                ui.vertical(|ui| {
                ui.set_width(breite);
                // Urgent problems are visible from every screen, with a way to the overview.
                if self.screen != Bildschirm::Uebersicht {
                    let dringend = ansicht::hilfe::hinweise(&v, &self.regeln).into_iter().filter(|h| h.farbe == SCHLECHT).count();
                    if dringend > 0 {
                        stil::hinweis_karte(ui, SCHLECHT, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(format!("⚠ {dringend} dringende Hinweise"));
                                if ui.link("zur Übersicht ›").clicked() {
                                    self.screen = Bildschirm::Uebersicht;
                                }
                            });
                        });
                        ui.add_space(4.0);
                    }
                }
                self.bildkopf(ui, &v);
                match self.screen {
                    Bildschirm::Uebersicht => self.uebersicht(ui, &v),
                    Bildschirm::Kolonie => self.kolonie(ui, &v, &p),
                    Bildschirm::Gebaeude => self.gebaeude(ui, &v, &p),
                    Bildschirm::Forschung => self.forschung(ui, &v),
                    Bildschirm::Werft => self.werft(ui, &v, &p),
                    Bildschirm::Flotten => self.flotten(ui, &v, &p),
                    Bildschirm::Galaxie => self.galaxie(ui, &v),
                    Bildschirm::Markt => self.markt(ui, &v, &p),
                    Bildschirm::Diplomatie => self.diplomatie(ui, &v),
                    Bildschirm::Regierung => self.regierung(ui, &v),
                    Bildschirm::Berichte => self.berichte(ui, &v),
                    Bildschirm::Rangliste => self.rangliste(ui, &v),
                    Bildschirm::Anleitung => self.anleitung(ui, &v),
                    Bildschirm::LiveKi => self.live_screen(ui),
                    Bildschirm::Befehle => self.command(ui, &p),
                    Bildschirm::Galerie => self.gallery(ui),
                }
                self.bildsammlung(ui, &v);
                });
                });
            });
        });
        self.dialoge(ctx, &v);
    }

    /// Confirmation for irreversible orders and the end of the epoch.
    fn dialoge(&mut self, ctx: &egui::Context, v: &Value) {
        if let Some(r) = &self.rueckfrage {
            let mut antwort = None;
            egui::Window::new("Bist du sicher?")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.set_max_width(460.0);
                    ui.label(RichText::new(&r.frage).size(17.0));
                    ui.add_space(8.0);
                    let ja = match r.was {
                        Nachgefragt::Befehl(_) => "Ja, ausführen",
                        Nachgefragt::Ueberschreiben(_) => "Ja, überschreiben",
                    };
                    ui.horizontal(|ui| {
                        if stil::hauptknopf(ui, ja, true) {
                            antwort = Some(true);
                        }
                        if ui.button("Abbrechen").clicked() {
                            antwort = Some(false);
                        }
                    });
                });
            match antwort {
                Some(true) => match self.rueckfrage.take().unwrap().was {
                    Nachgefragt::Befehl(a) => self.act(a),
                    Nachgefragt::Ueberschreiben(pfad) => self.speichern(&pfad, true),
                },
                Some(false) => self.rueckfrage = None,
                None => {}
            }
        }
        if self.session.ended() && !self.ende_gesehen {
            let mut zu = false;
            egui::Window::new("Die Epoche ist vorbei")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(RichText::new(format!("{} beendet die Epoche auf Rang {} von {} mit {} Punkten.", text(&v["name"]), stil::z(&v["rang"]), stil::z(&v["spielerzahl"]), stil::z(&v["punkte"]["gesamt"]))).size(18.0).color(GOLD));
                    ui.add_space(6.0);
                    for e in v["rangliste"].as_array().into_iter().flatten().take(10) {
                        let eigen = e[1] == v["name"];
                        ui.label(RichText::new(format!("{}. {} – {} Punkte, Stufe {}", stil::z(&e[0]), text(&e[1]), stil::z(&e[2]), stil::roemisch(stil::i(&e[3])))).color(if eigen { GOLD } else { stil::TEXT }));
                    }
                    ui.add_space(6.0);
                    if ui.button("Schließen").clicked() {
                        zu = true;
                    }
                });
            if zu {
                self.ende_gesehen = true;
                self.screen = Bildschirm::Rangliste;
            }
        }
    }
}

impl eframe::App for Player {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Windows übergeht „maximiert“, solange das Fenster noch unsichtbar ist (eframe zeigt es erst nach dem
        // ersten Bild); deshalb in den ersten Bildern ein paar Mal nachreichen.
        if self.maximieren > 0 {
            self.maximieren -= 1;
            if self.maximieren < 8 {
                ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(true));
            }
            ctx.request_repaint();
        }
        self.render(ctx);
        if let Some(path) = &self.screenshot_path {
            let captured = ctx.input(|i| {
                i.events.iter().find_map(|e| {
                    if let egui::Event::Screenshot { image, .. } = e {
                        Some(image.clone())
                    } else {
                        None
                    }
                })
            });
            if let Some(image) = captured {
                let pixels: Vec<u8> = image.pixels.iter().flat_map(|p| p.to_array()).collect();
                match image::save_buffer(
                    path,
                    &pixels,
                    image.width() as u32,
                    image.height() as u32,
                    image::ColorType::Rgba8,
                ) {
                    Ok(()) => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
                    Err(e) => {
                        self.status_ok = false;
                        self.status = format!("Screenshot: {e}");
                    }
                }
                self.screenshot_path = None;
            } else {
                self.screenshot_frames += 1;
                if self.screenshot_frames == 3 {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(
                        egui::UserData::default(),
                    ));
                }
                ctx.request_repaint_after(std::time::Duration::from_millis(50));
            }
        }
    }
}

/// Dateiname für den Spielstand einer neuen Partie, damit sie keinen älteren Stand überschreibt.
fn neuer_spielstand() -> String {
    let jetzt = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
    format!("saves/spieler-{}.sav", jetzt.as_secs())
}

/// Anzeigename einer Geschwindigkeit der Weltuhr.
fn tempo_name(s: f64) -> &'static str {
    match s as i64 {
        1 => "1× Echtzeit",
        60 => "60× (1 min je s)",
        900 => "900× (15 min je s)",
        3600 => "3600× (1 h je s)",
        _ => "86400× (1 Tag je s)",
    }
}

/// Sektor der Heimatwelt, damit die Galaxiekarte dort öffnet.
fn heimat_sektor(session: &Session) -> u8 {
    let v = session.view();
    let heimat = v["planeten"].as_array().into_iter().flatten().find(|p| p["heimat"] == true).map(|p| ansicht::hilfe::text(&p["koord"])).unwrap_or_default();
    heimat.split(':').next().and_then(|s| s.parse().ok()).unwrap_or(1)
}

/// Vorlagen der Befehlszentrale: jeder Befehl, den ein Mensch schicken darf, gebaut wie in den Dialogen.
fn command_templates(p: &str) -> Vec<(&'static str, Value)> {
    use ansicht::befehl as b;
    let mut schiffe = serde_json::Map::new();
    schiffe.insert("kleiner_transporter".into(), json!(1));
    let mut ladung = serde_json::Map::new();
    ladung.insert("erz".into(), json!(1000));
    vec![
        ("Gebäude bauen", b::bauen(p, "erzmine")),
        ("Gebäude reparieren", b::reparieren(p, "erzmine")),
        ("Eigene Orbitflotte versorgen", b::flotte_versorgen(p, "1:1:6", &json!(1), &schiffe, 1.0, &ladung)),
        ("Gebäude abreißen", b::abreissen(p, "erzmine")),
        ("Bauschleife leeren", b::schleife_leeren(p)),
        ("Prioritäten der Arbeitskräfte", b::prioritaeten(p, &["solarkraftwerk".into(), "farm".into(), "erzmine".into(), "kristallmine".into()])),
        ("Forschen", b::forschen("energietechnik")),
        ("Schiffe oder Verteidigung fertigen", b::fertigen_einheit(p, "leichter_jaeger", 10)),
        ("Bauteile fertigen", b::fertigen_bauteil(p, "habitatmodul", 1)),
        ("Raketen bauen", b::raketen_bauen(p, "abfang", 5)),
        ("Raketen starten", b::raketen_starten(p, "1:1:6", 2, "raketenwerfer")),
        ("Flotte senden", b::flotte_senden(p, "1:1:6", "transport", &schiffe, 1.0, &ladung, 0)),
        ("Flotte zurückrufen", b::flotte_zurueckrufen(&json!(1))),
        ("Verband öffnen", b::verband_oeffnen(&json!(1))),
        ("Verband beitreten", b::verband_beitreten(&json!(1), &json!(2))),
        ("Marktorder", b::markt_order(p, "erz", "verkauf", 1000.0, 0.06)),
        ("Marktorder stornieren", b::markt_storno(&json!(1))),
        ("Steuersatz", b::steuersatz(15)),
        ("Zivilisationsstufe aufsteigen", b::stufenaufstieg()),
        ("Nachricht", b::nachricht(&["NAME".into()], false, "Frieden?")),
        ("Vertrag anbieten", b::vertrag_anbieten("NAME", "nichtangriffspakt", 500.0, None)),
        ("Tributvertrag anbieten", b::vertrag_anbieten("NAME", "tribut", 0.0, Some((Some("erz"), 100.0, 7)))),
        ("Vertrag annehmen", b::vertrag_annehmen(&json!(1))),
        ("Vertrag ablehnen", b::vertrag_ablehnen(&json!(1))),
        ("Vertrag kündigen", b::vertrag_kuendigen(&json!(1))),
        ("Allianz gründen", b::allianz_gruenden("Sternenbund")),
        ("In die Allianz einladen", b::allianz_einladen("NAME")),
        ("Allianz beitreten", b::allianz_beitreten("Sternenbund")),
        ("Allianz verlassen", b::allianz_verlassen()),
        ("Credits schenken", b::schenken("NAME", 100.0)),
    ]
}

/// Lesende Abfragen der Engine, dieselben, die die Modellreiche als Werkzeug haben.
fn query_templates(p: &str) -> Vec<(&'static str, Value)> {
    vec![
        ("Kosten eines Gebäudes", json!({"typ": "kosten", "planet": p, "gebaeude": "erzmine"})),
        ("Kosten einer Forschung", json!({"typ": "kosten", "forschung": "energietechnik"})),
        ("Kosten einer Einheit", json!({"typ": "kosten", "planet": p, "einheit": "leichter_jaeger"})),
        ("Kosten von Raketen", json!({"typ": "kosten", "planet": p, "rakete": "abfang", "anzahl": 5})),
        ("Flugzeit und Treibstoff", json!({"typ": "flugzeit", "start": p, "ziel": "1:1:6", "schiffe": {"kleiner_transporter": 1}, "geschwindigkeit": 1.0})),
        ("Kampfsimulator (braucht Spionagebericht)", json!({"typ": "kampfsimulator", "ziel": "1:1:6", "schiffe": {"leichter_jaeger": 50}})),
        ("Regel nachschlagen", json!({"typ": "regel", "stichwort": "kolonie"})),
        ("Galaxie lesen", json!({"typ": "galaxie", "sektor": 1, "von": 1, "bis": 10})),
    ]
}

const HELP: &str = "Sternenepoche native player
  --seed N              deterministic world seed (default 42)
  --player N            human identity (default 0)
  --load FILE           resume a native player checkpoint
  --smoke-test          test a full day with 50 players without opening a window
  --screenshot FILE.png render one real native frame, save PNG and close
  --screen NAME         open this screen first, e.g. Live-KI
  --werft-reiter N       shipyard tab: 0 ships, 1 defense, 2 components, 3 missiles

Live test with model empires (the same can be started in the window, screen Live-KI):
  --ki CONFIG           model configuration (JSON like konfig/spieler-live.json); key from OPENROUTER_API_KEY
  --attrappe            offline stand-in models instead of --ki, no network, no cost
  --ki-reiche N         empires governed by models, the next N after the human (default 3)
  --reiche N            empires in the world (default 50)
  --budget USD          stop before the next model window once reached (default 1.0)
  --smoke-test-ki       play --fenster N windows (default 96, one day) with the models, without a window
  --vorlauf N           play N windows before the window opens (paused afterwards), e.g. for a demonstration
  --autopilot TYP       during --vorlauf a script bot (oekonom, raeuber, igel, haendler) plays your empire";

fn option<T: std::str::FromStr>(args: &[String], name: &str, default: T) -> Result<T, String>
where
    T::Err: std::fmt::Display,
{
    match args.windows(2).find(|w| w[0] == name) {
        Some(w) => w[1].parse::<T>().map_err(|e| format!("{name}: {e}")),
        None => Ok(default),
    }
}

/// A live session from the command line, or `None` without `--ki`/`--attrappe`.
fn live_from_args(args: &[String], seed: u64, human: u16) -> Result<Option<Session>, String> {
    let offline = args.iter().any(|a| a == "--attrappe");
    let path = args.windows(2).find(|w| w[0] == "--ki").map(|w| w[1].clone());
    if !offline && path.is_none() {
        return Ok(None);
    }
    let players: usize = option(args, "--reiche", 50)?;
    let empires: usize = option(args, "--ki-reiche", 3)?;
    let budget: f64 = option(args, "--budget", 1.0)?;
    let config = match path {
        Some(p) if !offline => {
            let p = if Path::new(&p).is_file() {
                std::path::PathBuf::from(p)
            } else {
                root().join(p)
            };
            let t = std::fs::read_to_string(&p).map_err(|e| format!("{}: {e}", p.display()))?;
            serde_json::from_str::<Config>(&t).map_err(|e| format!("{}: {e}", p.display()))?
        }
        _ => Config::demo(players),
    };
    let folder = live_folder(offline);
    let empires = spieler::ki_reiche(players, human, empires);
    Session::with_models(seed, players, human, &empires, config, &folder, budget).map(Some)
}

/// Plays windows the way the GUI does: a step either advances the clock or waits for the models.
fn play(session: &mut Session, windows: usize) -> Result<usize, String> {
    let mut advanced = 0;
    while advanced < windows && !session.ended() {
        if session.step()? {
            advanced += 1;
        } else if session.ki_denkt() {
            session.ki_warten()?;
        } else {
            break;
        }
    }
    Ok(advanced)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("{HELP}");
        return Ok(());
    }
    if args.iter().any(|a| a == "--smoke-test") {
        let mut session = Session::new(42, 50, 0)?;
        let v = session.view();
        let p = v["planeten"][0]["koord"].clone();
        let before = v["planeten"][0]["gebaeude"]["erzmine"]
            .as_u64()
            .unwrap_or(0);
        let result = session.act(json!({"typ":"bauen","planet":p,"gebaeude":"erzmine"}));
        if !result.0 {
            return Err(result.1.into());
        }
        for _ in 0..96 {
            session.step()?;
        }
        let after = session.view()["planeten"][0]["gebaeude"]["erzmine"]
            .as_u64()
            .unwrap_or(0);
        if after <= before {
            return Err("Bauauftrag wurde nicht abgeschlossen".into());
        }
        println!(
            "{}",
            json!({"ok":true,"players":50,"human":0,"bots":49,"seconds":session.seconds(),"erzmine_before":before,"erzmine_after":after,"hash":session.hash()})
        );
        return Ok(());
    }
    let seed: u64 = option(&args, "--seed", 42)?;
    let human: u16 = option(&args, "--player", 0)?;
    if args.iter().any(|a| a == "--smoke-test-ki") {
        let mut session = live_from_args(&args, seed, human)?
            .ok_or("--smoke-test-ki braucht --ki KONFIG oder --attrappe")?;
        let windows: usize = option(&args, "--fenster", 96)?;
        let started = Instant::now();
        let advanced = play(&mut session, windows)?;
        let info = session.ki_info();
        let accepted = session
            .ki_protokoll()
            .iter()
            .flat_map(|r| r["aktionen"].as_array().cloned().unwrap_or_default())
            .filter(|a| a["ok"] == true)
            .count();
        println!(
            "{}",
            json!({"ok": true, "fenster": advanced, "zeit": game_time(session.seconds()),
                   "sekunden_echtzeit": started.elapsed().as_secs(),
                   "entscheidungen": info["entscheidungen"], "aktionen_angenommen": accepted,
                   "anfragen": info["anfragen"], "kosten_usd": info["kosten"],
                   "reiche": info["reiche"], "ordner": info["ordner"], "hash": session.hash()})
        );
        return Ok(());
    }
    let mut session = if let Some(a) = args.windows(2).find(|w| w[0] == "--load") {
        Session::load(Path::new(&a[1]))?
    } else if let Some(s) = live_from_args(&args, seed, human)? {
        s
    } else {
        Session::new(seed, 50, human)?
    };
    match args.windows(2).find(|w| w[0] == "--autopilot") {
        Some(w) => {
            session.autopilot(option(&args, "--vorlauf", 0)?, &w[1])?;
        }
        None => {
            play(&mut session, option(&args, "--vorlauf", 0)?)?;
        }
    }
    let screen = args
        .windows(2)
        .find(|w| w[0] == "--screen")
        .and_then(|w| Bildschirm::aus_name(&w[1]));
    let screenshot = args
        .windows(2)
        .find(|w| w[0] == "--screenshot")
        .map(|w| std::path::PathBuf::from(&w[1]));
    let yard_tab: usize = option(&args, "--werft-reiter", 0)?;
    if yard_tab > 3 { return Err("--werft-reiter muss 0..3 sein".into()); }
    // Zum Spielen gleich bildschirmfüllend; Bilder für die Doku behalten die feste Größe.
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1300.0, 900.0])
            .with_min_inner_size([900.0, 650.0])
            .with_title("Sternenepoche"),
        ..Default::default()
    };
    // Mit --load geöffnet, speichert „Speichern“ (nach Rückfrage) in dieselbe Datei.
    let geladen = args.windows(2).find(|w| w[0] == "--load").map(|w| w[1].replace('\\', "/"));
    eframe::run_native(
        "Sternenepoche · Player",
        options,
        Box::new(move |cc| {
            let mut app = Player::new(&cc.egui_ctx, session);
            if let Some(pfad) = geladen {
                app.save_path = pfad;
            }
            app.maximieren = if screenshot.is_none() { 10 } else { 0 };
            app.screenshot_path = screenshot;
            app.werft_reiter = yard_tab;
            if let Some(b) = screen {
                app.screen = b;
            }
            Ok(Box::new(app))
        }),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_native_screens_layout_without_a_window_or_gpu() {
        let ctx = egui::Context::default();
        let mut app = Player::new(&ctx, Session::new(42, 50, 0).unwrap());
        for screen in SCREENS {
            app.screen = screen;
            let output = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1300.0, 900.0),
                    )),
                    ..Default::default()
                },
                |ctx| app.render(ctx),
            );
            assert!(
                output.shapes.len() > 10,
                "screen {} has no layout",
                screen.name()
            );
            assert_eq!(
                app.session.seconds(),
                0,
                "opening a screen advanced the simulation"
            );
        }
    }
    fn frame(ctx: &egui::Context, app: &mut Player) -> egui::FullOutput {
        ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1300.0, 900.0),
                )),
                ..Default::default()
            },
            |ctx| app.render(ctx),
        )
    }

    /// The live screen with offline stand-in models: a manual step waits for the models,
    /// the window is merged on a later frame and the step is then taken.
    #[test]
    fn live_screen_runs_model_empires_through_the_gui_path() {
        let dir = std::env::temp_dir().join(format!(
            "sternenepoche-gui-live-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let session =
            Session::with_models(3, 10, 0, &spieler::ki_reiche(10, 0, 2), Config::demo(10), &dir, 1.0)
                .unwrap();
        let ctx = egui::Context::default();
        let mut app = Player::new(&ctx, session);
        assert!(app.status.starts_with("Live-Partie: 2 KI-Reiche"), "{}", app.status);
        app.screen = Bildschirm::LiveKi;
        assert!(frame(&ctx, &mut app).shapes.len() > 10);
        app.step();
        assert!(app.session.ki_denkt());
        assert_eq!(app.session.seconds(), 0, "the clock waits for the models");
        let started = Instant::now();
        while app.session.seconds() == 0 {
            assert!(started.elapsed().as_secs() < 30, "model window never merged");
            frame(&ctx, &mut app);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert_eq!(app.session.seconds(), app.session.window_seconds());
        assert_eq!(app.session.ki_protokoll().len(), 8);
        let output = frame(&ctx, &mut app);
        assert!(output.shapes.len() > 40, "decisions are listed");
        drop(app);
        std::fs::remove_dir_all(dir).unwrap();
    }

    /// The start form behind the two buttons: offline stand-in, a missing key, and a key typed into the
    /// field, which must reach the process environment and no file.
    #[test]
    fn start_form_handles_offline_missing_key_and_typed_key() {
        let ctx = egui::Context::default();
        let mut app = Player::new(&ctx, Session::new(42, 10, 0).unwrap());
        app.live.players = 10;
        app.live.empires = 2;
        let mut folders = Vec::new();

        app.start_live(true);
        assert!(app.status_ok, "{}", app.status);
        assert!(app.status.starts_with("Live-Partie: 2 KI-Reiche"), "{}", app.status);
        folders.push(std::path::PathBuf::from(app.session.ki_info()["ordner"].as_str().unwrap()));
        assert!(folders[0].file_name().unwrap().to_string_lossy().starts_with("spieler-live-attrappe-"));
        app.step();
        assert!(app.session.ki_denkt());
        app.session.ki_warten().unwrap();

        let dir = std::env::temp_dir().join(format!("sternenepoche-form-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut config: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(root().join("konfig/spieler-live.json")).unwrap()).unwrap();
        for p in config["anbieter"].as_object_mut().unwrap().values_mut() {
            p["api_key_env"] = json!("STERNENEPOCHE_TEST_FORMULAR");
        }
        let path = dir.join("live.json");
        std::fs::write(&path, config.to_string()).unwrap();
        app.live.config = path.to_string_lossy().into_owned();
        let before = app.session.hash();
        app.start_live(false);
        assert!(!app.status_ok && app.status.contains("STERNENEPOCHE_TEST_FORMULAR"), "{}", app.status);
        assert_eq!(app.session.hash(), before, "a failed start keeps the running game");

        let key = "sk-or-test-nie-in-dateien";
        app.live.key = key.into();
        app.start_live(false);
        assert!(app.status_ok, "{}", app.status);
        assert!(app.live.key.is_empty(), "the field is cleared");
        assert_eq!(std::env::var("STERNENEPOCHE_TEST_FORMULAR").unwrap(), key);
        let folder = std::path::PathBuf::from(app.session.ki_info()["ordner"].as_str().unwrap());
        assert_ne!(folder, folders[0]);
        folders.push(folder.clone());
        let save = dir.join("stand.sav");
        app.session.save(&save).unwrap();
        for f in [folder.join("manifest.json"), save.clone()] {
            let bytes = std::fs::read(&f).unwrap();
            assert!(!String::from_utf8_lossy(&bytes).contains(key), "{} contains the key", f.display());
        }
        drop(app);
        for f in folders {
            std::fs::remove_dir_all(f).unwrap();
        }
        std::fs::remove_dir_all(dir).unwrap();
    }

    /// Every screen in a game that has progressed (stage III and IV, colonies, fleets, reports, market), so that
    /// no screen fails on data that only appears later.
    #[test]
    fn alle_bildschirme_im_fortgeschrittenen_spiel() {
        let mut session = Session::new(7, 20, 0).unwrap();
        session.autopilot(7000, "raeuber").unwrap();
        let ctx = egui::Context::default();
        let mut app = Player::new(&ctx, session);
        let v = app.session.view();
        assert!(stil::i(&v["stufe"]) >= 3, "the autopilot reached stage {}", v["stufe"]);
        for b in SCREENS {
            app.screen = b;
            for reiter in 0..6 {
                app.werft_reiter = reiter.min(3);
                app.diplo.reiter = reiter;
                app.berichte_reiter = reiter.min(2);
                let output = frame(&ctx, &mut app);
                assert!(output.shapes.len() > 10, "{} empty", b.name());
            }
        }
        // A planet chosen on the map becomes the fleet target.
        app.screen = Bildschirm::Galaxie;
        app.galaxie_auswahl = Some("1:3:6".into());
        frame(&ctx, &mut app);
        assert!(app.galaxie_cache.contains_key(&app.sector), "the sector was read");
        assert_eq!(app.sector, heimat_sektor(&app.session), "the map opens where the home planet is");
    }

    /// "+1 Tag" stops exactly one day later.
    #[test]
    fn vorspulen_haelt_genau_an() {
        let ctx = egui::Context::default();
        let mut app = Player::new(&ctx, Session::new(3, 8, 0).unwrap());
        app.vorspulen(86_400);
        let mut n = 0;
        while app.ziel_zeit.is_some() {
            frame(&ctx, &mut app);
            n += 1;
            assert!(n < 100, "fast-forward never ends");
        }
        assert_eq!(app.session.seconds(), 86_400);
        assert!(app.paused);
    }

    /// Ein Bild mit Mauseingaben.
    fn frame_mit(ctx: &egui::Context, app: &mut Player, events: Vec<egui::Event>) -> egui::FullOutput {
        ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1300.0, 900.0))),
                events,
                ..Default::default()
            },
            |ctx| app.render(ctx),
        )
    }

    /// Mitte des ersten gezeichneten Texts, der `text` enthält und zwischen `von_x` und `bis_x` beginnt.
    fn text_position(output: &egui::FullOutput, text: &str, von_x: f32, bis_x: f32) -> Option<egui::Pos2> {
        output.shapes.iter().find_map(|s| match &s.shape {
            egui::Shape::Text(t) if t.galley.text().contains(text) && t.pos.x >= von_x && t.pos.x < bis_x => {
                Some(t.pos + t.galley.rect.center().to_vec2())
            }
            _ => None,
        })
    }

    /// Ein echter Mausklick (bewegen, drücken, loslassen) an einer Stelle.
    fn klick(ctx: &egui::Context, app: &mut Player, pos: egui::Pos2) {
        let knopf = |pressed| egui::Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed, modifiers: egui::Modifiers::NONE };
        frame_mit(ctx, app, vec![egui::Event::PointerMoved(pos)]);
        frame_mit(ctx, app, vec![knopf(true)]);
        frame_mit(ctx, app, vec![knopf(false)]);
        frame(ctx, app);
    }

    /// Klicks wie ein Mensch: Navigation, Bau eines Gebäudes, Sprung aus „Was jetzt ansteht“. Gefunden wird jeder
    /// Knopf über seinen gezeichneten Text, geklickt mit Drücken und Loslassen.
    #[test]
    fn klicks_wie_ein_mensch() {
        let ctx = egui::Context::default();
        let mut app = Player::new(&ctx, Session::new(11, 50, 0).unwrap());
        app.willkommen_aus = true;
        let bild = frame(&ctx, &mut app);
        let ziel = text_position(&bild, "Gebäude", 0.0, 200.0).expect("Gebäude in der Navigation");
        klick(&ctx, &mut app, ziel);
        assert_eq!(app.screen, Bildschirm::Gebaeude, "the navigation click switches the screen");

        let bild = frame(&ctx, &mut app);
        let knopf = text_position(&bild, "Auf Stufe 2 ausbauen", 200.0, 1300.0).expect("a build button");
        let vorher = app.session.view()["planeten"][0]["bauschleife"].as_array().map_or(0, Vec::len);
        klick(&ctx, &mut app, knopf);
        let nachher = app.session.view()["planeten"][0]["bauschleife"].as_array().map_or(0, Vec::len);
        assert_eq!(nachher, vorher + 1, "the click queued a building: {}", app.status);

        app.screen = Bildschirm::Uebersicht;
        let bild = frame(&ctx, &mut app);
        // Rechts der Navigation (die endet vor x = 200) steht als Erstes der Knopf des Hinweises „Es läuft keine Forschung“.
        let hinweis = text_position(&bild, "🔬 Forschung", 200.0, 1300.0).expect("a hint button to research");
        klick(&ctx, &mut app, hinweis);
        assert_eq!(app.screen, Bildschirm::Forschung, "the hint button leads to its screen");
    }

    /// Ereignisse gelten als gelesen, sobald man sie angesehen und den Bereich verlassen hat; neue kommen hinzu.
    #[test]
    fn ereignisse_werden_gelesen() {
        let ctx = egui::Context::default();
        let mut app = Player::new(&ctx, Session::new(3, 8, 0).unwrap());
        app.session.autopilot(400, "oekonom").unwrap();
        let v = app.session.view();
        let vorher = ansicht::berichte::ungelesen(&v, app.ereignisse_gelesen);
        assert!(vorher > 0, "the autopilot produced events");
        app.screen = Bildschirm::Berichte;
        frame(&ctx, &mut app);
        assert_eq!(ansicht::berichte::ungelesen(&v, app.ereignisse_gelesen), vorher, "still unread while looking");
        app.screen = Bildschirm::Uebersicht;
        frame(&ctx, &mut app);
        assert_eq!(ansicht::berichte::ungelesen(&app.session.view(), app.ereignisse_gelesen), 0);
        app.session.autopilot(200, "oekonom").unwrap();
        assert!(ansicht::berichte::ungelesen(&app.session.view(), app.ereignisse_gelesen) > 0, "later events are new again");
    }

    /// Every character the interface shows exists in egui's built-in fonts. A missing glyph shows as an empty
    /// box, as two check marks and an info sign did; the test reads all interface sources (without comments)
    /// and checks every non-ASCII character.
    #[test]
    fn alle_symbole_gibt_es_in_der_schrift() {
        let quellen = [
            include_str!("main.rs"),
            include_str!("ansicht/mod.rs"),
            include_str!("ansicht/stil.rs"),
            include_str!("ansicht/namen.rs"),
            include_str!("ansicht/hilfe.rs"),
            include_str!("ansicht/uebersicht.rs"),
            include_str!("ansicht/kolonie.rs"),
            include_str!("ansicht/gebaeude.rs"),
            include_str!("ansicht/forschung.rs"),
            include_str!("ansicht/werft.rs"),
            include_str!("ansicht/flotten.rs"),
            include_str!("ansicht/galaxie.rs"),
            include_str!("ansicht/markt.rs"),
            include_str!("ansicht/diplomatie.rs"),
            include_str!("ansicht/regierung.rs"),
            include_str!("ansicht/berichte.rs"),
            include_str!("ansicht/befehl.rs"),
        ];
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |_| {});
        let mut fehlt = std::collections::BTreeSet::new();
        ctx.fonts(|f| {
            for q in quellen {
                for zeile in q.lines().filter(|z| !z.trim_start().starts_with("//")) {
                    for c in zeile.chars().filter(|c| !c.is_ascii()) {
                        if !f.has_glyph(&egui::FontId::proportional(16.0), c) {
                            fehlt.insert(format!("{c} U+{:04X}", c as u32));
                        }
                    }
                }
            }
        });
        assert!(fehlt.is_empty(), "Zeichen ohne Glyphe: {fehlt:?}");
    }

    #[test]
    fn advanced_templates_parse_as_real_core_actions() {
        let erlaubt = kern::aktion::erlaubte_typen(kern::Rolle::Alle);
        for (titel, value) in command_templates("1:1:6") {
            assert!(erlaubt.contains(&value["typ"].as_str().unwrap()), "{titel}");
            serde_json::from_value::<kern::aktion::Aktion>(value).unwrap();
        }
    }

    /// Jede Abfragevorlage beantwortet der Kern; nur der Kampfsimulator verlangt erst einen Spionagebericht.
    #[test]
    fn abfragevorlagen_beantwortet_der_kern() {
        let session = Session::new(42, 50, 0).unwrap();
        let heimat = session.view()["planeten"][0]["koord"].as_str().unwrap().to_string();
        for (titel, abfrage) in query_templates(&heimat) {
            match session.query(&abfrage) {
                Ok(_) => assert_ne!(abfrage["typ"], "kampfsimulator", "{titel}"),
                Err(e) => assert!(abfrage["typ"] == "kampfsimulator" && e.contains("Spionagebericht"), "{titel}: {e}"),
            }
        }
    }
}
