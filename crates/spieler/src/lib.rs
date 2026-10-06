//! Player-session adapter: owns the authoritative core, exposes filtered observations.
//! No graphical dependency is used here; tests exercise exactly the same action path as the GUI.
//!
//! Live mode: model governments act every 15 game minutes. Inference uses an immutable
//! observation while human actions immediately change the authoritative world. Completed
//! model actions are revalidated against it; the world clock waits for the model window.
use kern::{Regelwerk, Rolle, SpielerId, Welt};
use lauf::bots::{self, Bot, Bottyp};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{mpsc, Arc},
    time::Instant,
};
use sternenepoche_agenten::{config::Config, journal::Journal};

pub const BUILTIN_RULES: &str = include_str!("../../../regeln/regelwerk.ron");
/// Reports kept in memory for the live screen; all of them are also appended to
/// `entscheidungen.jsonl` in the live folder.
const PROTOKOLL_MAX: usize = 400;

type Arbeit = mpsc::Receiver<Result<(Welt, Vec<Value>), String>>;

/// Model-governed empires of a session.
pub struct Ki {
    config: Config,
    journal: Arc<Journal>,
    ordner: PathBuf,
    reiche: Vec<SpielerId>,
    kosten: f64,
    budget: f64,
    entscheidungen: usize,
    protokoll: Vec<Value>,
    arbeit: Option<(Arbeit, Instant)>,
    /// Immutable model observation retained across retries; human actions use the live world.
    snapshot: Option<Welt>,
    fehler: Option<String>,
}

pub struct Session {
    world: Welt,
    human: SpielerId,
    bots: BTreeMap<SpielerId, Bot>,
    ki: Option<Ki>,
}

#[derive(Serialize, Deserialize)]
struct Save {
    version: u32,
    human: SpielerId,
    world: Vec<u8>,
    hash: String,
    bots: BTreeMap<SpielerId, Bot>,
}

/// Version 2 adds the live empires. The configuration is stored as JSON text (bincode cannot
/// hold `serde_json::Value`); it names the key's environment variable, never the key.
#[derive(Serialize, Deserialize)]
struct SaveV2 {
    version: u32,
    human: SpielerId,
    world: Vec<u8>,
    hash: String,
    bots: BTreeMap<SpielerId, Bot>,
    ki: Option<KiSave>,
}

#[derive(Serialize, Deserialize)]
struct KiSave {
    config: String,
    ordner: PathBuf,
    reiche: Vec<SpielerId>,
    kosten: f64,
    budget: f64,
    entscheidungen: usize,
}

/// Which empires the models govern: the next `n` identities after the human.
pub fn ki_reiche(count: usize, human: SpielerId, n: usize) -> Vec<SpielerId> {
    (1..=n)
        .map(|i| ((human as usize + i) % count) as SpielerId)
        .collect()
}

/// In interactive games every model government gets a turn every 15 game minutes.
/// The research runner retains its role-specific scheduling in the shared core.
fn menschen_fenster(world: &mut Welt) {
    if world.beendet() { return; }
    for sid in world.reihenfolge.clone() {
        if !world.spieler[sid as usize].ki { continue; }
        for rolle in [Rolle::Stratege, Rolle::Verwalter, Rolle::Feldherr, Rolle::Diplomat] {
            if !world.faellig.iter().any(|f| f.spieler == sid && f.rolle == rolle) {
                world.faellig.push(kern::welt::Faellig { spieler: sid,
                    name: world.spieler[sid as usize].name.clone(), rolle,
                    gruende: vec!["Menschenmodus: 15-Minuten-Zyklus".into()] });
            }
        }
    }
    world.faellig.sort_by_key(|f| (world.reihenfolge.iter().position(|s| *s == f.spieler).unwrap(), f.rolle.idx()));
    for f in &mut world.faellig {
        if !f.gruende.iter().any(|g| g == "Menschenmodus: 15-Minuten-Zyklus") {
            f.gruende.push("Menschenmodus: 15-Minuten-Zyklus".into());
        }
    }
}

fn manifest(config: &Config, startwert: u64, human: SpielerId, reiche: &[SpielerId]) -> Value {
    json!({
        "protocol": "spieler-live-v2-interaktiv",
        "rules_hash": sternenepoche_agenten::journal::hash(BUILTIN_RULES.as_bytes()),
        "startwert": startwert,
        "mensch": human,
        "ki_reiche": reiche,
        "config": config,
    })
}

impl Session {
    pub fn new(seed: u64, count: usize, human: SpielerId) -> Result<Self, String> {
        Self::build(seed, count, human, &[], None)
    }

    /// A game in which `config` governs `reiche` (four roles each), the human plays `human`
    /// and script bots play everyone else. Model calls are journaled in `ordner`.
    pub fn with_models(
        seed: u64,
        count: usize,
        human: SpielerId,
        reiche: &[SpielerId],
        config: Config,
        ordner: &Path,
        budget: f64,
    ) -> Result<Self, String> {
        config.validate()?;
        if reiche.is_empty() {
            return Err("Mindestens ein KI-Reich angeben".into());
        }
        if reiche
            .iter()
            .any(|r| *r == human || *r as usize >= count)
        {
            return Err("KI-Reiche müssen andere Spieler der Welt sein".into());
        }
        if !budget.is_finite() || budget < 0.0 {
            return Err("Budget muss eine Zahl ab 0 sein".into());
        }
        for name in config.rollen.values() {
            let p = &config.anbieter[name];
            if let Some(var) = p.api_key_env.as_deref().filter(|v| !v.is_empty()) {
                if std::env::var(var).map(|v| v.trim().is_empty()).unwrap_or(true) {
                    return Err(format!(
                        "Umgebungsvariable {var} fehlt: Schlüssel im Bereich Live-KI eintragen oder vor dem Start setzen"
                    ));
                }
            }
        }
        let journal = Journal::open(ordner, &manifest(&config, seed, human, reiche))?;
        let ki = Ki {
            config,
            journal: Arc::new(journal),
            ordner: ordner.to_path_buf(),
            reiche: reiche.to_vec(),
            kosten: 0.0,
            budget,
            entscheidungen: 0,
            protokoll: Vec::new(),
            arbeit: None,
            snapshot: None,
            fehler: None,
        };
        Self::build(seed, count, human, reiche, Some(ki))
    }

    fn build(
        seed: u64,
        count: usize,
        human: SpielerId,
        reiche: &[SpielerId],
        ki: Option<Ki>,
    ) -> Result<Self, String> {
        if human as usize >= count {
            return Err("Spielerkennung liegt außerhalb der Welt".into());
        }
        let mut world = Welt::neu(Regelwerk::laden(BUILTIN_RULES)?, seed, count)?;
        world.kolonisationsregeln_v2_aktivieren();
        let mut bots = BTreeMap::new();
        for sp in &mut world.spieler {
            sp.ki = reiche.contains(&sp.id);
            if sp.id != human && !sp.ki {
                bots.insert(sp.id, Bot::neu(Bottyp::ALLE[sp.id as usize % 4], sp.id));
            }
        }
        world.fenster_vorbereiten();
        menschen_fenster(&mut world);
        let mut result = Self {
            world,
            human,
            bots,
            ki,
        };
        result.bot_turn();
        Ok(result)
    }

    fn bot_turn(&mut self) {
        for sid in self.world.reihenfolge.clone() {
            if let Some(bot) = self.bots.get_mut(&sid) {
                // Interactive opponents share the same 15-minute cadence as model governments.
                // The standalone balance runner retains Bot's original two-hour cadence.
                bot.naechster = self.world.zeit;
                bots::zug(&mut self.world, sid, bot);
                bot.naechster = self.world.zeit + self.world.regeln.fenster();
            }
        }
        // The full action hash persists in core; the GUI has filtered event history.
        self.world.log_abholen();
    }
    pub fn view(&self) -> Value {
        let mut view = self.world.sicht(self.human, Rolle::Alle);
        view["kolonisationsversion"] = json!(if self.world.kolonisationsregeln_v2() { 2 } else if self.world.kolonisation.aktiv { 1 } else { 0 });
        view
    }
    pub fn query(&self, query: &Value) -> Result<Value, String> {
        self.world.werkzeug(self.human, query)
    }
    /// Roles of model empires that still have to answer in the current window.
    fn ki_offen(&self) -> bool {
        self.world.faellig.iter().any(|f| {
            self.world.spieler[f.spieler as usize].weck[f.rolle.idx()].letzter < self.world.zeit
        })
    }
    pub fn act(&mut self, action: Value) -> (bool, String) {
        if self.world.beendet() {
            return (false, "Diese Epoche ist beendet".into());
        }
        match &mut self.ki {
            None if !self.world.faellig.is_empty() => {
                (false, "Entscheidungsfenster wartet auf KI-Antworten".into())
            }
            _ => self.world.handeln(self.human, Rolle::Alle, &action),
        }
    }
    pub fn step(&mut self) -> Result<bool, String> {
        if let Some(error) = self.ki.as_ref().and_then(|k| k.fehler.as_ref()) {
            return Err(format!("KI-Fenster gescheitert: {error}"));
        }
        if self.ki.is_some() {
            if self.ki_offen() {
                self.ki_starten()?;
                return Ok(false);
            }
        } else if !self.world.faellig.is_empty() {
            return Err("Weltuhr angehalten: KI-Entscheidungen fehlen".into());
        }
        let advanced = self.world.schritt_wenn_bereit()?;
        if advanced {
            menschen_fenster(&mut self.world);
            self.bot_turn();
        }
        Ok(advanced)
    }

    /// Starts the model window on a worker thread unless it already runs.
    fn ki_starten(&mut self) -> Result<(), String> {
        let Some(ki) = self.ki.as_mut() else {
            return Ok(());
        };
        if ki.arbeit.is_some() {
            return Ok(());
        }
        if let Some(f) = &ki.fehler {
            return Err(format!("KI-Fenster gescheitert: {f}"));
        }
        if ki.kosten >= ki.budget {
            return Err(format!(
                "KI-Budget erreicht: {:.4} von {:.2} USD verbraucht. Im Bereich Live-KI erhöhen.",
                ki.kosten, ki.budget
            ));
        }
        let mut welt = ki.snapshot.get_or_insert_with(|| self.world.clone()).clone();
        welt.log_abholen();
        let (config, journal) = (ki.config.clone(), ki.journal.clone());
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let mut welt = welt;
            let result =
                sternenepoche_agenten::window(&mut welt, &config, &journal).map(|r| (welt, r));
            // Release the journal lock before reporting, so a save can be loaded right away.
            drop(journal);
            let _ = tx.send(result);
        });
        ki.arbeit = Some((rx, Instant::now()));
        Ok(())
    }

    /// Merges a finished model window without replacing live human actions.
    /// The legacy result list is empty; `None` means no window finished.
    pub fn ki_abholen(&mut self) -> Option<Vec<(Value, bool, String)>> {
        let ki = self.ki.as_mut()?;
        let result = match &ki.arbeit {
            Some((rx, _)) => match rx.try_recv() {
                Ok(r) => r,
                Err(mpsc::TryRecvError::Empty) => return None,
                Err(mpsc::TryRecvError::Disconnected) => {
                    Err("KI-Arbeitsfaden ohne Ergebnis beendet".into())
                }
            },
            None => return None,
        };
        Some(self.ki_uebernehmen(result))
    }

    /// Blocks until the running model window is finished (tests, headless use).
    pub fn ki_warten(&mut self) -> Result<Vec<(Value, bool, String)>, String> {
        let Some(ki) = self.ki.as_mut() else {
            return Ok(Vec::new());
        };
        let Some((rx, _)) = &ki.arbeit else {
            return Ok(Vec::new());
        };
        let result = rx
            .recv()
            .unwrap_or_else(|_| Err("KI-Arbeitsfaden ohne Ergebnis beendet".into()));
        let done = self.ki_uebernehmen(result);
        match &self.ki.as_ref().unwrap().fehler {
            Some(f) => Err(f.clone()),
            None => Ok(done),
        }
    }

    fn ki_uebernehmen(
        &mut self,
        result: Result<(Welt, Vec<Value>), String>,
    ) -> Vec<(Value, bool, String)> {
        let ki = self.ki.as_mut().expect("KI-Sitzung");
        ki.arbeit = None;
        let (mut welt, mut berichte) = match result {
            Ok(x) => x,
            Err(e) => {
                ki.fehler = Some(e);
                return Vec::new();
            }
        };
        // Replay model actions on the current authoritative world. Never replace human changes
        // with the worker's stale copy. Original rejects stay rejected: their correction may
        // already have been accepted on the snapshot. Conflicts are checked by the core again.
        let mut conflicts: BTreeMap<(SpielerId, String), Vec<String>> = BTreeMap::new();
        for entry in welt.log_abholen() {
            let a: Value = serde_json::from_str(&entry.aktion).expect("core action JSON");
            let key = (entry.spieler, entry.rolle.name().to_string());
            if a["typ"] == "aufruf_ende" {
                let mut hints: Vec<String> = a["hinweise"].as_array().into_iter().flatten()
                    .filter_map(|v| v.as_str().map(str::to_owned)).collect();
                hints.extend(conflicts.remove(&key).unwrap_or_default());
                self.world.aufruf_ende(entry.spieler, entry.rolle, a["notiz"].as_str(),
                    a["wecker_sekunden"].as_i64(), &hints);
            } else {
                // IDs allocated on the private snapshot can now name a different live object.
                // Require a new observation before using references created inside this window.
                let unstable_reference = ki.snapshot.as_ref().is_some_and(|base| {
                    ["flotte", "fuehrung", "versorgungsflotte"].iter().any(|field|
                        a[*field].as_u64().is_some_and(|id| !base.flotten.contains_key(&(id as u32))))
                    || a["vertrag"].as_u64().is_some_and(|id| !base.vertraege.iter().any(|v| u64::from(v.id) == id))
                    || a["order"].as_u64().is_some_and(|id| !base.orders.iter().any(|v| u64::from(v.id) == id))
                });
                let (ok, text) = if entry.ok && unstable_reference {
                    (false, "Neue Objektkennung aus paralleler Planung: im nächsten Lagebild erneut auswählen".into())
                } else if entry.ok {
                    self.world.handeln(entry.spieler, entry.rolle, &a)
                } else { (false, entry.text.clone()) };
                if entry.ok && !ok {
                    conflicts.entry(key.clone()).or_default().push(format!("Zwischenzeitlich geändert: {text}"));
                }
                if let Some(report) = berichte.iter_mut().find(|b| b["spieler"] == entry.spieler && b["rolle"] == key.1) {
                    if let Some(actions) = report["aktionen"].as_array_mut() {
                        if let Some(item) = actions.iter_mut().find(|x| x["aktion"] == a && x.get("live_geprueft").is_none()) {
                            item["snapshot_ok"] = json!(entry.ok);
                            item["ok"] = json!(ok);
                            item["text"] = json!(text);
                            item["live_geprueft"] = json!(true);
                        }
                    }
                }
            }
        }
        ki.snapshot = None;
        for b in &berichte {
            ki.kosten += b["kosten"].as_f64().unwrap_or(0.0);
        }
        ki.entscheidungen += berichte.len();
        if !berichte.is_empty() {
            let mut text = String::new();
            for b in &berichte {
                text.push_str(&b.to_string());
                text.push('\n');
            }
            let datei = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(ki.ordner.join("entscheidungen.jsonl"));
            if let Err(e) = datei.and_then(|mut f| f.write_all(text.as_bytes())) {
                ki.fehler = Some(format!("Entscheidungen nicht gespeichert: {e}"));
            }
        }
        ki.protokoll.extend(berichte);
        let zuviel = ki.protokoll.len().saturating_sub(PROTOKOLL_MAX);
        ki.protokoll.drain(..zuviel);
        self.world.log_abholen();
        Vec::new()
    }

    /// After a failed model window: retry it (`aussetzen = false`), or let the waiting roles
    /// skip this call so the game continues (`aussetzen = true`).
    pub fn ki_fehler_aufloesen(&mut self, aussetzen: bool) {
        let Some(ki) = self.ki.as_mut() else {
            return;
        };
        let Some(fehler) = ki.fehler.take() else {
            return;
        };
        if aussetzen {
            ki.snapshot = None;
            let hinweis = vec![format!("Aufruf ausgesetzt: {fehler}")];
            for f in self.world.faellig.clone() {
                if self.world.spieler[f.spieler as usize].weck[f.rolle.idx()].letzter
                    < self.world.zeit
                {
                    self.world
                        .aufruf_ende(f.spieler, f.rolle, None, None, &hinweis);
                }
            }
            self.world.log_abholen();
        }
    }

    /// Demonstration: a script bot of type `typ` plays the human's empire for `windows` windows (model
    /// windows included), so every screen can be seen in a game that has progressed.
    pub fn autopilot(&mut self, windows: usize, typ: &str) -> Result<usize, String> {
        let typ = Bottyp::aus_name(typ).ok_or_else(|| format!("Bottyp {typ} gibt es nicht: oekonom, raeuber, igel, haendler"))?;
        let mut bot = Bot::neu(typ, self.human);
        let mut advanced = 0;
        while advanced < windows && !self.ended() {
            if self.step()? {
                advanced += 1;
                bots::zug(&mut self.world, self.human, &mut bot);
                self.world.log_abholen();
            } else if self.ki_denkt() {
                self.ki_warten()?;
            } else {
                break;
            }
        }
        Ok(advanced)
    }

    /// The rules of this game. Public knowledge: everything in them is also in the rule text.
    pub fn regeln(&self) -> &Regelwerk {
        &self.world.regeln
    }
    pub fn bot_count(&self) -> usize {
        self.bots.len()
    }
    pub fn ki_aktiv(&self) -> bool {
        self.ki.is_some()
    }
    pub fn ki_denkt(&self) -> bool {
        self.ki.as_ref().is_some_and(|k| k.arbeit.is_some())
    }
    pub fn ki_budget_setzen(&mut self, budget: f64) {
        if let Some(ki) = self.ki.as_mut() {
            if budget.is_finite() && budget >= 0.0 {
                ki.budget = budget;
            }
        }
    }
    pub fn ki_protokoll(&self) -> &[Value] {
        self.ki.as_ref().map(|k| k.protokoll.as_slice()).unwrap_or(&[])
    }
    /// Status of the live empires for the GUI.
    pub fn ki_info(&self) -> Value {
        let Some(ki) = &self.ki else {
            return json!({"aktiv": false});
        };
        let reiche: Vec<Value> = ki
            .reiche
            .iter()
            .map(|id| {
                let s = &self.world.spieler[*id as usize];
                json!({"id": id, "name": s.name, "volk": s.volk.name(), "rang": s.rang,
                       "punkte": s.punkte.gesamt(), "stufe": s.stufe, "planeten": s.planeten.len()})
            })
            .collect();
        let offen: Vec<Value> = self
            .world
            .faellig
            .iter()
            .filter(|f| {
                self.world.spieler[f.spieler as usize].weck[f.rolle.idx()].letzter
                    < self.world.zeit
            })
            .map(|f| json!({"name": f.name, "rolle": f.rolle.name(), "gruende": f.gruende}))
            .collect();
        let modelle: BTreeMap<&str, &str> = ki
            .config
            .rollen
            .iter()
            .map(|(rolle, anbieter)| (rolle.as_str(), ki.config.anbieter[anbieter].modell.as_str()))
            .collect();
        json!({
            "aktiv": true,
            "denkt": ki.arbeit.is_some(),
            "denkt_sekunden": ki.arbeit.as_ref().map(|(_, t)| t.elapsed().as_secs()),
            "kosten": ki.kosten,
            "budget": ki.budget,
            "entscheidungen": ki.entscheidungen,
            "anfragen": ki.journal.request_count().unwrap_or(0),
            "fehler": ki.fehler,
            "vorgemerkt": 0,
            "ordner": ki.ordner.display().to_string(),
            "modelle": modelle,
            "reiche": reiche,
            "offen": offen,
        })
    }

    pub fn seconds(&self) -> i64 {
        self.world.zeit
    }
    pub fn window_seconds(&self) -> i64 {
        self.world.regeln.fenster()
    }
    pub fn ended(&self) -> bool {
        self.world.beendet()
    }
    pub fn hash(&self) -> String {
        self.world.hash()
    }
    /// Writes a new checkpoint; an existing file is never overwritten (see `save_replace`).
    pub fn save(&self, path: &Path) -> Result<(), String> {
        let bytes = self.save_bytes()?;
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|e| {
                format!("Speichern (vorhandene Spielstände werden nicht überschrieben): {e}")
            })?;
        file.write_all(&bytes)
            .and_then(|_| file.sync_all())
            .map_err(|e| e.to_string())
    }
    /// Replaces a checkpoint the player chose to overwrite. The new state is written completely to a
    /// sibling file first and then renamed over the old one, so a failure never leaves a half file.
    pub fn save_replace(&self, path: &Path) -> Result<(), String> {
        let bytes = self.save_bytes()?;
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut neu = path.as_os_str().to_owned();
        neu.push(".neu");
        let neu = PathBuf::from(neu);
        let mut file = std::fs::File::create(&neu).map_err(|e| e.to_string())?;
        file.write_all(&bytes)
            .and_then(|_| file.sync_all())
            .map_err(|e| e.to_string())?;
        drop(file);
        std::fs::rename(&neu, path).map_err(|e| e.to_string())
    }
    fn save_bytes(&self) -> Result<Vec<u8>, String> {
        let ki = match &self.ki {
            Some(k) if k.arbeit.is_some() || k.snapshot.is_some() => {
                return Err("KI-Reiche entscheiden gerade; nach diesem Fenster speichern".into())
            }
            Some(k) => Some(KiSave {
                config: serde_json::to_string(&k.config).map_err(|e| e.to_string())?,
                ordner: k.ordner.clone(),
                reiche: k.reiche.clone(),
                kosten: k.kosten,
                budget: k.budget,
                entscheidungen: k.entscheidungen,
            }),
            None => None,
        };
        let data = SaveV2 {
            version: 2,
            human: self.human,
            world: self.world.zu_bytes(),
            hash: self.world.hash(),
            bots: self.bots.clone(),
            ki,
        };
        bincode::serialize(&data).map_err(|e| e.to_string())
    }
    pub fn load(path: &Path) -> Result<Self, String> {
        let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
        let mut bytes = Vec::new();
        file.take(32 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() > 32 * 1024 * 1024 {
            return Err("Spielstand größer als 32 MiB".into());
        }
        let data: SaveV2 = match bincode::deserialize::<SaveV2>(&bytes) {
            Ok(d) if d.version == 2 => d,
            _ => {
                let v1: Save = bincode::deserialize(&bytes).map_err(|e| e.to_string())?;
                if v1.version != 1 {
                    return Err("Unbekannte Spielstandversion".into());
                }
                SaveV2 {
                    version: 1,
                    human: v1.human,
                    world: v1.world,
                    hash: v1.hash,
                    bots: v1.bots,
                    ki: None,
                }
            }
        };
        let world = Welt::aus_bytes(&data.world)?;
        if world.hash() != data.hash {
            return Err("Spielstand-Prüfsumme stimmt nicht".into());
        }
        let reiche = data.ki.as_ref().map(|k| k.reiche.clone()).unwrap_or_default();
        if data.human as usize >= world.spieler.len()
            || world.spieler.iter().any(|s| s.ki != reiche.contains(&s.id))
            || world.faellig.iter().any(|f| !reiche.contains(&f.spieler))
        {
            return Err("Spielstand ist keine lokale Partie aus Mensch, Skriptbots und KI-Reichen".into());
        }
        if world.regeln.fenster() <= 0 || world.regeln.epochenende() <= 0 {
            return Err("Ungültige Zeitregeln".into());
        }
        for sp in &world.spieler {
            if sp.id as usize >= world.spieler.len()
                || sp
                    .planeten
                    .iter()
                    .any(|p| *p as usize >= world.planeten.len())
            {
                return Err("Ungültige Spieler-/Planetenreferenz".into());
            }
            if sp.id != data.human && !sp.ki && !data.bots.contains_key(&sp.id) {
                return Err("Skriptbot-Zustand fehlt".into());
            }
        }
        if data.bots.contains_key(&data.human)
            || data
                .bots
                .keys()
                .any(|s| *s as usize >= world.spieler.len() || reiche.contains(s))
        {
            return Err("Ungültige Bot-Zuordnung".into());
        }
        let ki = match data.ki {
            None => None,
            Some(k) => {
                let config: Config = serde_json::from_str(&k.config).map_err(|e| e.to_string())?;
                config.validate()?;
                // A loaded game continues in a new journal folder next to the old one: an older
                // save diverges from the calls journaled after it, and the running game may
                // still hold the old folder.
                let stamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis();
                let name = k
                    .ordner
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "spieler-live".into());
                let name = name.split("-ab-").next().unwrap_or(&name).to_string();
                let ordner = k.ordner.with_file_name(format!("{name}-ab-{stamp}"));
                let journal = Journal::open(
                    &ordner,
                    &manifest(&config, world.startwert, data.human, &k.reiche),
                )?;
                Some(Ki {
                    config,
                    journal: Arc::new(journal),
                    ordner,
                    reiche: k.reiche,
                    kosten: k.kosten,
                    budget: k.budget,
                    entscheidungen: k.entscheidungen,
                    protokoll: Vec::new(),
                    arbeit: None,
                    snapshot: None,
                    fehler: None,
                })
            }
        };
        Ok(Self {
            world,
            human: data.human,
            bots: data.bots,
            ki,
        })
    }
}

/// Wall time changes only how many core windows are requested, never game formulas.
#[derive(Default)]
pub struct RealtimeClock {
    accumulator: f64,
}
impl RealtimeClock {
    pub fn reset(&mut self) {
        self.accumulator = 0.0;
    }
    pub fn update(
        &mut self,
        session: &mut Session,
        elapsed: f64,
        speed: f64,
        paused: bool,
    ) -> Result<usize, String> {
        if paused || session.ended() {
            return Ok(0);
        }
        if !elapsed.is_finite() || !speed.is_finite() || elapsed < 0.0 || speed < 0.0 {
            return Err("Ungültiger Echtzeittakt".into());
        }
        if session.ki.is_some() && session.ki_offen() && !session.ki_denkt() {
            session.ki_starten()?;
        }
        // No offline catch-up after sleep/window suspension. Work per frame is bounded.
        // While the models decide, the clock does not accumulate: the world waits for them.
        if session.ki_denkt() {
            self.reset();
            return Ok(0);
        }
        self.accumulator += elapsed.min(0.25) * speed.min(86_400.0);
        let mut steps = 0;
        while self.accumulator >= session.window_seconds() as f64 && steps < 16 {
            if !session.step()? {
                self.reset();
                break;
            }
            self.accumulator -= session.window_seconds() as f64;
            steps += 1;
        }
        Ok(steps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn human_mode_bots_and_models_share_the_fifteen_minute_cadence() {
        let mut s = Session::new(42, 6, 0).unwrap();
        assert_eq!(s.window_seconds(), 900);
        assert!(s.bots.values().all(|b| b.naechster == 900));
        assert!(s.step().unwrap());
        assert!(s.bots.values().all(|b| b.naechster == 1800));
    }
    #[test]
    fn model_actions_are_revalidated_and_report_live_conflicts() {
        let dir = temp("live-conflict");
        let mut s = Session::with_models(5, 6, 0, &[1], Config::demo(6), &dir, 1.0).unwrap();
        let mut snapshot = s.world.clone();
        snapshot.log_abholen();
        let pid = snapshot.spieler[1].heimat as usize;
        let a = json!({"typ":"bauen", "planet":snapshot.planeten[pid].koord.to_string(), "gebaeude":"erzmine"});
        assert!(snapshot.handeln(1, Rolle::Verwalter, &a).0);
        snapshot.aufruf_ende(1, Rolle::Verwalter, Some("Geplanter Ausbau"), None, &[]);
        // Resources have changed after observation; the old success must not overwrite reality.
        s.world.planeten[pid].bestand.fill(0);
        let human = s.view()["planeten"][0]["koord"].clone();
        assert!(s.act(json!({"typ":"bauen","planet":human,"gebaeude":"erzmine"})).0);
        let report = json!({"spieler":1,"rolle":"verwalter","kosten":0,"aktionen":[{"aktion":a,"ok":true}]});
        s.ki_uebernehmen(Ok((snapshot, vec![report])));
        assert!(s.world.planeten[pid].bauschleife.is_empty());
        assert_eq!(s.view()["planeten"][0]["bauschleife"].as_array().unwrap().len(),1);
        assert_eq!(s.ki_protokoll()[0]["aktionen"][0]["ok"], false);
        assert_eq!(s.ki_protokoll()[0]["aktionen"][0]["snapshot_ok"], true);
        drop(s);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn failed_model_retry_keeps_its_observation_and_human_orders() {
        let dir = temp("retry-live");
        let mut s = Session::with_models(5, 6, 0, &[1], Config::demo(6), &dir, 1.0).unwrap();
        let snapshot = s.world.clone();
        let hash = snapshot.hash();
        s.ki.as_mut().unwrap().snapshot = Some(snapshot);
        s.ki_uebernehmen(Err("Temporary test failure".into()));
        let p = s.view()["planeten"][0]["koord"].clone();
        assert!(s.act(json!({"typ":"bauen","planet":p,"gebaeude":"erzmine"})).0);
        assert_eq!(s.ki.as_ref().unwrap().snapshot.as_ref().unwrap().hash(),hash);
        s.ki_fehler_aufloesen(false);
        assert!(!s.step().unwrap());
        s.ki_warten().unwrap();
        assert_eq!(s.view()["planeten"][0]["bauschleife"].as_array().unwrap().len(),1);
        drop(s);
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn human_actions_use_core_validation_and_complete_building() {
        let mut s = Session::new(42, 8, 0).unwrap();
        let v = s.view();
        let p = v["planeten"][0]["koord"].as_str().unwrap();
        let before = v["planeten"][0]["gebaeude"]["erzmine"]
            .as_i64()
            .unwrap_or(0);
        assert!(
            !s.act(json!({"typ":"bauen","planet":"255:255:255","gebaeude":"erzmine"}))
                .0
        );
        assert!(
            s.act(json!({"typ":"bauen","planet":p,"gebaeude":"erzmine"}))
                .0
        );
        for _ in 0..96 {
            s.step().unwrap();
        }
        assert!(
            s.view()["planeten"][0]["gebaeude"]["erzmine"]
                .as_i64()
                .unwrap()
                > before
        );
    }
    #[test]
    fn filtered_observation_never_exposes_opponent_stocks() {
        let s = Session::new(42, 8, 0).unwrap();
        assert_eq!(s.view()["planeten"].as_array().unwrap().len(), 1);
        for p in s.view()["nachbarn"].as_array().unwrap() {
            assert!(p.get("bestand").is_none());
        }
    }
    #[test]
    fn real_time_pause_and_manual_windows_are_deterministic() {
        let mut a = Session::new(7, 8, 0).unwrap();
        let mut b = Session::new(7, 8, 0).unwrap();
        let mut clock = RealtimeClock::default();
        clock.update(&mut a, 0.25, 3600.0, true).unwrap();
        assert_eq!(a.seconds(), 0);
        clock.update(&mut a, 0.25, 3600.0, false).unwrap();
        b.step().unwrap();
        assert_eq!(a.hash(), b.hash());
    }
    #[test]
    fn ai_barrier_is_not_skipped() {
        let mut s = Session::new(42, 8, 0).unwrap();
        s.world.spieler[1].ki = true;
        s.world.wecke(1, Rolle::Verwalter, "test", true);
        s.world.fenster_vorbereiten();
        assert!(!s.world.faellig.is_empty());
        let t = s.seconds();
        assert!(s.step().is_err());
        assert_eq!(t, s.seconds());
    }
    #[test]
    fn checkpoint_preserves_future_and_refuses_overwrite() {
        let mut a = Session::new(11, 8, 0).unwrap();
        for _ in 0..12 {
            a.step().unwrap();
        }
        let path = std::env::temp_dir().join(format!(
            "sternenepoche-player-{}-{}.sav",
            std::process::id(),
            a.seconds()
        ));
        a.save(&path).unwrap();
        assert!(a.save(&path).is_err());
        let mut b = Session::load(&path).unwrap();
        assert_eq!(a.hash(), b.hash());
        for _ in 0..12 {
            a.step().unwrap();
            b.step().unwrap();
        }
        assert_eq!(a.hash(), b.hash());
        // Overwriting on request: the file then holds the newer state, and no temporary file stays behind.
        a.save_replace(&path).unwrap();
        assert_eq!(Session::load(&path).unwrap().hash(), a.hash());
        let mut neu = path.as_os_str().to_owned();
        neu.push(".neu");
        assert!(!PathBuf::from(neu).exists());
        std::fs::remove_file(path).unwrap();
    }

    fn temp(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!(
            "sternenepoche-live-{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&p);
        p
    }

    /// Runs `windows` windows the way the GUI does: a step either advances the clock or starts
    /// the model window, which is then merged.
    fn play(s: &mut Session, windows: usize) {
        let mut advanced = 0;
        while advanced < windows {
            if s.step().unwrap() {
                advanced += 1;
            } else {
                assert!(s.ki_denkt(), "step without model window must not stall");
                s.ki_warten().unwrap();
            }
        }
    }

    #[test]
    fn model_empires_decide_while_the_clock_stands_still() {
        let dir = temp("barriere");
        let reiche = ki_reiche(8, 0, 2);
        assert_eq!(reiche, vec![1, 2]);
        let mut s =
            Session::with_models(5, 8, 0, &reiche, Config::demo(8), &dir, 1.0).unwrap();
        assert!(s.ki_offen(), "all roles are due at the start");
        let t = s.seconds();
        assert!(!s.step().unwrap());
        assert!(s.ki_denkt());
        assert_eq!(s.seconds(), t, "the clock waits for the models");
        // A human order is applied immediately and survives model completion.
        let p = s.view()["planeten"][0]["koord"].clone();
        let (ok, text) = s.act(json!({"typ":"bauen","planet":p,"gebaeude":"erzmine"}));
        assert!(ok && !text.starts_with("Vorgemerkt"), "{text}");
        assert_eq!(s.view()["planeten"][0]["bauschleife"].as_array().unwrap().len(), 1);
        let done = s.ki_warten().unwrap();
        assert!(done.is_empty());
        assert_eq!(s.view()["planeten"][0]["bauschleife"].as_array().unwrap().len(), 1);
        assert!(!s.ki_offen());
        // Four roles of two empires answered; the offline agent builds as Verwalter.
        let reports = s.ki_protokoll();
        assert_eq!(reports.len(), 8);
        assert!(reports.iter().any(|r| r["rolle"] == "verwalter"
            && r["aktionen"].as_array().is_some_and(|a| a.iter().any(|x| x["ok"] == true))));
        let lines = std::fs::read_to_string(dir.join("entscheidungen.jsonl")).unwrap();
        assert_eq!(lines.lines().count(), 8);
        assert!(s.step().unwrap());
        assert_eq!(s.seconds(), t + s.window_seconds());
        play(&mut s, 96);
        let info = s.ki_info();
        assert_eq!(info["reiche"].as_array().unwrap().len(), 2);
        assert_eq!(info["entscheidungen"].as_u64().unwrap(), 8 * 97);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn budget_stops_the_models_and_can_be_raised() {
        let dir = temp("budget");
        let mut s = Session::with_models(5, 6, 0, &ki_reiche(6, 0, 1), Config::demo(6), &dir, 0.0)
            .unwrap();
        let t = s.seconds();
        let e = s.step().unwrap_err();
        assert!(e.contains("Budget"), "{e}");
        assert_eq!(s.seconds(), t);
        s.ki_budget_setzen(1.0);
        play(&mut s, 2);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn missing_key_is_reported_before_the_game_starts() {
        let dir = temp("schluessel");
        let mut config = Config::demo(6);
        let p = config.anbieter.get_mut("offline").unwrap();
        p.api_key_env = Some("STERNENEPOCHE_TEST_FEHLT".into());
        let e = Session::with_models(5, 6, 0, &[1], config, &dir, 1.0)
            .err()
            .unwrap();
        assert!(e.contains("STERNENEPOCHE_TEST_FEHLT"), "{e}");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn failed_window_can_be_skipped_without_losing_the_game() {
        let dir = temp("aussetzen");
        let mut s = Session::with_models(5, 6, 0, &[1], Config::demo(6), &dir, 1.0).unwrap();
        s.ki.as_mut().unwrap().fehler = Some("Test".into());
        assert!(s.step().unwrap_err().contains("Test"));
        s.ki_fehler_aufloesen(true);
        assert!(!s.ki_offen());
        assert!(s.step().unwrap());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn live_checkpoint_resumes_with_the_same_models_and_state() {
        let dir = temp("stand");
        let mut a = Session::with_models(9, 6, 0, &[1, 2], Config::demo(6), &dir, 1.0).unwrap();
        play(&mut a, 8);
        let path = dir.join("stand.sav");
        a.save(&path).unwrap();
        let hash = a.hash();
        let entscheidungen = a.ki_info()["entscheidungen"].clone();
        // The running game keeps its journal; the loaded one continues in a folder of its own.
        let mut b = Session::load(&path).unwrap();
        assert_eq!(b.hash(), hash);
        assert!(b.ki_aktiv());
        assert_eq!(b.ki_info()["entscheidungen"], entscheidungen);
        let weiter = PathBuf::from(b.ki_info()["ordner"].as_str().unwrap());
        assert_ne!(weiter, dir);
        assert!(weiter.join("manifest.json").is_file());
        play(&mut a, 8);
        play(&mut b, 8);
        assert_eq!(a.hash(), b.hash(), "offline models: same future from the checkpoint");
        drop((a, b));
        std::fs::remove_dir_all(dir).unwrap();
        std::fs::remove_dir_all(weiter).unwrap();
    }
}
