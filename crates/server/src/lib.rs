//! One authoritative Rust world, transactional accounts/commands, no model proxy.
mod monitoring;
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use kern::{aktion::Aktion, regeln::Regelwerk, typen::*, Welt};
use lauf::bots::{Bot, Bottyp};
use rand::{rngs::OsRng, RngCore};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub const API_VERSION: u32 = 1;
pub const BOT_COUNT: u16 = 30;
pub const SEATS: u16 = 20;
pub fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
pub fn token() -> String {
    let mut b = [0u8; 32];
    OsRng.fill_bytes(&mut b);
    hex(&b)
}
fn hex(b: &[u8]) -> String {
    b.iter().map(|n| format!("{n:02x}")).collect()
}
pub fn digest(s: &str) -> String {
    hex(&Sha256::digest(s.as_bytes()))
}
pub fn password_hash(s: &str) -> Result<String, String> {
    if s.len() < 12 || s.len() > 128 {
        return Err("Passwort braucht 12 bis 128 Zeichen".into());
    }
    Argon2::default()
        .hash_password(s.as_bytes(), &SaltString::generate(&mut OsRng))
        .map(|h| h.to_string())
        .map_err(|_| "Passwort konnte nicht verarbeitet werden".into())
}
pub fn password_matches(password: &str, hash: &str) -> bool {
    PasswordHash::new(hash).ok().is_some_and(|h| {
        Argon2::default()
            .verify_password(password.as_bytes(), &h)
            .is_ok()
    })
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Lease {
    account: i64,
    hash: String,
    until: i64,
    roles: Vec<Rolle>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Runtime {
    pub world_id: String,
    pub tempo: u32,
    pub paused: bool,
    pub bots: BTreeMap<SpielerId, Bot>,
    pub bots_enabled: bool,
    pub revision: u64,
    leases: BTreeMap<SpielerId, Lease>,
    #[serde(default = "default_bot_period")]
    pub bot_period_secs: i64,
    agent_status: BTreeMap<SpielerId, Value>,
    pub bot_actions: u64,
    pub bot_rejected: u64,
    #[serde(default)]
    bot_trace: BTreeMap<SpielerId, Vec<Value>>,
    #[serde(default)]
    public_history: Vec<Value>,
    #[serde(default)]
    pub maintenance: String,
}
#[derive(Clone, Debug)]
pub struct Account {
    pub id: i64,
    pub name: String,
    pub password: String,
    pub sid: Option<SpielerId>,
    pub mode: String,
    pub banned: bool,
}
fn default_bot_period() -> i64 {
    7200
}
pub struct Game {
    pub world: Welt,
    pub runtime: Runtime,
    pub db: Connection,
    pub data_root: PathBuf,
    pub storage_error: Option<String>,
    started: std::time::Instant,
    audit_context: Option<Value>,
}
pub type Reply = Result<Value, (u16, String)>;
fn err(code: u16, text: &str) -> (u16, String) {
    (code, text.into())
}
fn str_field<'a>(v: &'a Value, k: &str) -> Result<&'a str, (u16, String)> {
    v[k].as_str()
        .ok_or_else(|| err(400, &format!("Feld {k} fehlt")))
}
fn account_from(row: &rusqlite::Row) -> rusqlite::Result<Account> {
    Ok(Account {
        id: row.get(0)?,
        name: row.get(1)?,
        password: row.get(2)?,
        sid: row.get(3)?,
        mode: row.get(4)?,
        banned: row.get(5)?,
    })
}
fn fresh(seed: u64, rules: &str) -> Result<(Welt, Runtime), String> {
    let mut r = Regelwerk::laden(rules)?;
    r.welt.fenster_sekunden = 1;
    let mut world = Welt::neu(r, seed, (BOT_COUNT + SEATS) as usize)?;
    let mut bots = BTreeMap::new();
    for sid in 0..BOT_COUNT {
        world.spieler[sid as usize].ki = false;
        bots.insert(sid, Bot::neu(Bottyp::ALLE[sid as usize % 4], sid));
    }
    for sid in BOT_COUNT..BOT_COUNT + SEATS {
        world.spieler[sid as usize].name = format!("Freier Startplatz {}", sid - BOT_COUNT + 1);
        world.startplatz_reservieren(sid)?;
    }
    Ok((
        world,
        Runtime {
            world_id: token(),
            tempo: 1,
            paused: false,
            bots,
            bots_enabled: true,
            bot_period_secs: 7200,
            revision: 0,
            leases: BTreeMap::new(),
            agent_status: BTreeMap::new(),
            bot_actions: 0,
            bot_rejected: 0,
            bot_trace: BTreeMap::new(),
            public_history: Vec::new(),
            maintenance: String::new(),
        },
    ))
}
impl Game {
    pub fn open(root: &Path, seed: u64) -> Result<Self, String> {
        std::fs::create_dir_all(root).map_err(|e| e.to_string())?;
        let db = Connection::open(root.join("spiel.sqlite3")).map_err(|e| e.to_string())?;
        db.busy_timeout(std::time::Duration::from_secs(5))
            .map_err(|e| e.to_string())?;
        db.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL; PRAGMA foreign_keys=ON;
            CREATE TABLE IF NOT EXISTS checkpoint(id INTEGER PRIMARY KEY CHECK(id=1),version INTEGER NOT NULL,world BLOB NOT NULL,runtime BLOB NOT NULL);
            CREATE TABLE IF NOT EXISTS accounts(id INTEGER PRIMARY KEY,name TEXT NOT NULL COLLATE NOCASE UNIQUE,password TEXT NOT NULL,sid INTEGER UNIQUE,mode TEXT NOT NULL,banned INTEGER NOT NULL DEFAULT 0);
            CREATE TABLE IF NOT EXISTS sessions(hash TEXT PRIMARY KEY,account INTEGER NOT NULL REFERENCES accounts(id),expires INTEGER NOT NULL);
            CREATE TABLE IF NOT EXISTS commands(account INTEGER NOT NULL REFERENCES accounts(id),world_id TEXT NOT NULL,request_id TEXT NOT NULL,body_hash TEXT NOT NULL,result TEXT NOT NULL,created INTEGER NOT NULL,PRIMARY KEY(account,world_id,request_id));
            CREATE TABLE IF NOT EXISTS audit(id INTEGER PRIMARY KEY,time INTEGER NOT NULL,account INTEGER,kind TEXT NOT NULL,result TEXT NOT NULL);").map_err(|e|e.to_string())?;
        let saved: Option<(u32, Vec<u8>, Vec<u8>)> = db
            .query_row(
                "SELECT version,world,runtime FROM checkpoint WHERE id=1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        let (world, runtime) = if let Some((version, w, r)) = saved {
            if version != API_VERSION {
                return Err("Unbekannte Server-Datenversion; Originaldaten bleiben erhalten".into());
            }
            (
                Welt::aus_bytes(&w)?,
                serde_json::from_slice(&r).map_err(|e| e.to_string())?,
            )
        } else {
            fresh(seed, include_str!("../../../regeln/online-v1.ron"))?
        };
        let mut g = Self {
            world,
            runtime,
            db,
            data_root: root.into(),
            storage_error: None,
            started: std::time::Instant::now(),
            audit_context: None,
        };
        // No downtime catch-up and no model command survives a process restart.
        g.runtime.leases.clear();
        g.save().map_err(|e| e.1)?;
        Ok(g)
    }
    fn save(&mut self) -> Result<(), (u16, String)> {
        let w = self.world.zu_bytes();
        let r = serde_json::to_vec(&self.runtime)
            .map_err(|_| err(500, "Checkpoint nicht serialisierbar"))?;
        let result=self.db.execute("INSERT INTO checkpoint VALUES(1,?1,?2,?3) ON CONFLICT(id) DO UPDATE SET version=excluded.version,world=excluded.world,runtime=excluded.runtime",params![API_VERSION,w,r]);
        match result {
            Ok(_) => Ok(()),
            Err(_) => {
                self.storage_error = Some("Speicherung fehlgeschlagen; Welt angehalten".into());
                self.runtime.paused = true;
                Err(err(503, "Speicherung fehlgeschlagen; Welt angehalten"))
            }
        }
    }
    fn transaction<F>(&mut self, f: F) -> Reply
    where
        F: FnOnce(&mut Self) -> Reply,
    {
        if self.storage_error.is_some() {
            return Err(err(503, "Speicherung gestört; zuerst Datenbank prüfen"));
        }
        let w = self.world.clone();
        let r = self.runtime.clone();
        self.db
            .execute_batch("BEGIN IMMEDIATE")
            .map_err(|_| err(503, "Datenbank belegt"))?;
        let result=f(self).and_then(|v| {
            if let Some(context)=&self.audit_context {
                self.db.execute("INSERT INTO audit(time,account,kind,result) VALUES(?1,NULL,'admin',?2)",params![now(),json!({"context":context,"revision":self.runtime.revision+1,"world_id":self.runtime.world_id}).to_string()]).map_err(|_|err(503,"Admin-Audit konnte nicht gespeichert werden"))?;
            }
            self.runtime.revision+=1; self.world.log_abholen(); self.save()?; Ok(v)
        });
        if result.is_ok() && self.db.execute_batch("COMMIT").is_ok() {
            return result;
        }
        let _ = self.db.execute_batch("ROLLBACK");
        self.world = w;
        self.runtime = r;
        if self.storage_error.is_some() {
            self.runtime.paused = true;
        }
        if result.is_ok() {
            self.storage_error = Some("Commit fehlgeschlagen".into());
            self.runtime.paused = true;
            return Err(err(503, "Speicherung fehlgeschlagen"));
        }
        result
    }
    pub fn advance(&mut self, seconds: u32) -> Reply {
        if self.runtime.paused || self.world.beendet() {
            return Ok(json!({"paused":true}));
        }
        self.transaction(|g| {
            for _ in 0..seconds.min(3600) {
                if !g.world.schritt() {
                    break;
                }
                if g.runtime.bots_enabled {
                    let sids:Vec<_>=g.runtime.bots.keys().copied().collect();
                    for sid in sids {
                        let bot=g.runtime.bots.get_mut(&sid).unwrap();
                        let due = bot.naechster <= g.world.zeit;
                        let prev = g.world.spieler[sid as usize].statistik.clone();
                        lauf::bots::zug(&mut g.world, sid, bot);
                        if due {
                            bot.naechster = g.world.zeit + g.runtime.bot_period_secs;
                        }
                        let next = &g.world.spieler[sid as usize].statistik;
                        g.runtime.bot_actions += (next.aktionen - prev.aktionen) as u64;
                        g.runtime.bot_rejected += (next.abgelehnt - prev.abgelehnt) as u64;
                        if due {g.record_bot_turn(sid);}
                    }
                }
                g.world.log_abholen();
                if g.world.zeit % STUNDE == 0 {g.record_public_history();}
            }
            g.runtime.leases.retain(|_, l| l.until > now());
            Ok(json!({"sekunden":g.world.zeit}))
        })
    }
    pub fn account_by_name(&self, name: &str) -> Option<Account> {
        self.db
            .query_row(
                "SELECT id,name,password,sid,mode,banned FROM accounts WHERE name=?1",
                [name],
                account_from,
            )
            .optional()
            .ok()
            .flatten()
    }
    pub fn authenticate(&self, raw: &str) -> Result<Account, (u16, String)> {
        if raw.len() != 64 {
            return Err(err(401, "Anmeldung erforderlich"));
        }
        let a=self.db.query_row("SELECT a.id,a.name,a.password,a.sid,a.mode,a.banned FROM accounts a JOIN sessions s ON s.account=a.id WHERE s.hash=?1 AND s.expires>?2",params![digest(raw),now()],account_from).optional().map_err(|_|err(503,"Datenbank nicht lesbar"))?.ok_or_else(||err(401,"Sitzung abgelaufen"))?;
        if a.banned {
            return Err(err(403, "Konto gesperrt"));
        }
        Ok(a)
    }
    fn session(&mut self, a: &Account) -> Reply {
        let raw = token();
        let expires = now() + 7 * 86400;
        self.db
            .execute("DELETE FROM sessions WHERE expires<=?1", [now()])
            .map_err(|_| err(503, "Datenbankfehler"))?;
        self.db
            .execute(
                "INSERT INTO sessions VALUES(?1,?2,?3)",
                params![digest(&raw), a.id, expires],
            )
            .map_err(|_| err(503, "Sitzung nicht gespeichert"))?;
        Ok(
            json!({"token":raw,"name":a.name,"mode":a.mode,"spieler":a.sid,"expires":expires,"world_id":self.runtime.world_id,"api_version":API_VERSION}),
        )
    }
    pub fn register(&mut self, v: &Value, hashed: String) -> Reply {
        let name = str_field(v, "name")?.trim();
        if name.chars().count() < 3
            || name.chars().count() > 24
            || !name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return Err(err(400, "Name: 3 bis 24 Buchstaben, Ziffern, - oder _"));
        }
        if self
            .world
            .spieler
            .iter()
            .any(|s| s.name.eq_ignore_ascii_case(name))
            || self.account_by_name(name).is_some()
        {
            return Err(err(409, "Name bereits vergeben"));
        }
        self.transaction(|g| {
            g.db.execute(
                "INSERT INTO accounts(name,password,mode) VALUES(?1,?2,'zuschauer')",
                params![name, hashed],
            )
            .map_err(|_| err(409, "Name bereits vergeben"))?;
            let a = g
                .account_by_name(name)
                .ok_or_else(|| err(503, "Konto konnte nicht gelesen werden"))?;
            g.session(&a)
        })
    }
    pub fn login_verified(&mut self, id: i64) -> Reply {
        let a = self
            .db
            .query_row(
                "SELECT id,name,password,sid,mode,banned FROM accounts WHERE id=?1",
                [id],
                account_from,
            )
            .map_err(|_| err(401, "Anmeldung fehlgeschlagen"))?;
        if a.banned {
            return Err(err(403, "Konto gesperrt"));
        }
        self.transaction(|g| g.session(&a))
    }
    pub fn lobby(&self) -> Value {
        json!({"api_version":API_VERSION,"world_id":self.runtime.world_id,"revision":self.runtime.revision,"sekunden":self.world.zeit,
          "paused":self.runtime.paused,"tempo":self.runtime.tempo,"beendet":self.world.beendet(),"bots":BOT_COUNT,"freie_plaetze":self.world.aufklaerung.inaktive_spieler.len(),"plaetze":SEATS,
            "rangliste":self.world.rangliste(),"regeln":self.world.regeln.version,"maintenance":self.runtime.maintenance,
            "epoche_tage":self.world.regeln.welt.epoche_tage,"verlauf":self.runtime.public_history.iter().rev().take(48).cloned().collect::<Vec<_>>()})
    }
    pub fn claim(&mut self, a: &Account, v: &Value) -> Reply {
        let mode = str_field(v, "mode")?;
        if !["mensch", "agent", "gemischt"].contains(&mode) {
            return Err(err(400, "Modus mensch, agent oder gemischt wählen"));
        }
        let volk =
            Volk::aus_name(str_field(v, "volk")?).ok_or_else(|| err(400, "Unbekanntes Volk"))?;
        self.transaction(|g| {
            let current = g
                .account_by_name(&a.name)
                .ok_or_else(|| err(401, "Konto fehlt"))?;
            let sid = if let Some(sid) = current.sid {
                sid
            } else {
                let sid = g
                    .world
                    .aufklaerung
                    .inaktive_spieler
                    .iter()
                    .next()
                    .copied()
                    .ok_or_else(|| err(409, "Alle 20 Teilnehmerplätze sind belegt"))?;
                if g.world.beendet() {
                    return Err(err(409, "Epoche beendet"));
                }
                g.world
                    .startplatz_aktivieren(sid, a.name.clone(), volk)
                    .map_err(|e| err(409, &e))?;
                sid
            };
            g.db.execute(
                "UPDATE accounts SET sid=?1,mode=?2 WHERE id=?3",
                params![sid, mode, a.id],
            )
            .map_err(|_| err(503, "Platz nicht gespeichert"))?;
            Ok(json!({"spieler":sid,"mode":mode,"world_id":g.runtime.world_id}))
        })
    }
    fn sid(a: &Account) -> Result<SpielerId, (u16, String)> {
        a.sid
            .ok_or_else(|| err(403, "Zuschauer haben keinen Spielplatz"))
    }
    pub fn view(&self, a: &Account, role: Rolle) -> Reply {
        let sid = Self::sid(a)?;
        let mut v = self.world.sicht(sid, role);
        v["world_id"] = json!(self.runtime.world_id);
        v["revision"] = json!(self.runtime.revision);
        v["paused"] = json!(self.runtime.paused);
        v["mode"] = json!(a.mode);
        v["agent"] = json!(self
            .runtime
            .leases
            .get(&sid)
            .filter(|l| l.until > now())
            .map(|l| json!({"roles":l.roles,"until":l.until})));
        Ok(v)
    }
    pub fn tool(&self, a: &Account, v: &Value) -> Reply {
        let sid = Self::sid(a)?;
        self.world.werkzeug(sid, v).map_err(|e| err(400, &e))
    }
    fn world_matches(&self, v: &Value) -> Result<(), (u16, String)> {
        if v["world_id"].as_str() != Some(&self.runtime.world_id) {
            return Err(err(409, "Welt wurde ersetzt; Ansicht neu laden"));
        }
        Ok(())
    }
    pub fn lease(&mut self, a: &Account, v: &Value) -> Reply {
        let sid = Self::sid(a)?;
        self.world_matches(v)?;
        self.transaction(|g| {
            let action = str_field(v, "action")?;
            if action == "stop" {
                if let Some(raw) = v["lease"].as_str() {
                    if g.runtime
                        .leases
                        .get(&sid)
                        .is_some_and(|l| l.hash != digest(raw))
                    {
                        return Err(err(409, "Diese Agentfreigabe wurde schon ersetzt"));
                    }
                }
                g.runtime.leases.remove(&sid);
                return Ok(json!({"stopped":true}));
            }
            if action == "heartbeat" {
                let l = g
                    .runtime
                    .leases
                    .get_mut(&sid)
                    .filter(|l| {
                        l.account == a.id
                            && l.until > now()
                            && Some(l.hash.as_str()) == v["lease"].as_str().map(digest).as_deref()
                    })
                    .ok_or_else(|| err(409, "Agentfreigabe abgelaufen"))?;
                l.until = now() + 90;
                return Ok(json!({"until":l.until}));
            }
            if action != "start" {
                return Err(err(400, "Unbekannter Agentbefehl"));
            }
            if a.mode == "mensch" || a.mode == "zuschauer" {
                return Err(err(403, "Zuerst Agent- oder Mischmodus wählen"));
            }
            if g.runtime.paused || g.world.beendet() {
                return Err(err(409, "Welt angehalten oder beendet"));
            }
            if g.runtime.leases.get(&sid).is_some_and(|l| l.until > now()) {
                return Err(err(409, "Ein Agent steuert dieses Reich bereits"));
            }
            let roles: Vec<Rolle> = serde_json::from_value(v["roles"].clone())
                .map_err(|_| err(400, "Rollenliste fehlt"))?;
            if roles.is_empty() || roles.len() > 4 || roles.contains(&Rolle::Alle) {
                return Err(err(400, "Eine bis vier Regierungsrollen wählen"));
            }
            let raw = token();
            let until = now() + 90;
            g.runtime.leases.insert(
                sid,
                Lease {
                    account: a.id,
                    hash: digest(&raw),
                    roles,
                    until,
                },
            );
            Ok(json!({"lease":raw,"until":until,"world_id":g.runtime.world_id}))
        })
    }
    pub fn command(&mut self, a: &Account, v: &Value) -> Reply {
        let sid = Self::sid(a)?;
        self.world_matches(v)?;
        let request = str_field(v, "request_id")?;
        if request.is_empty() || request.len() > 80 {
            return Err(err(400, "Ungültige Befehlskennung"));
        }
        let body_hash = digest(&v.to_string());
        let existing:Option<(String,String)>=self.db.query_row("SELECT body_hash,result FROM commands WHERE account=?1 AND world_id=?2 AND request_id=?3",params![a.id,self.runtime.world_id,request],|r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(|_|err(503,"Datenbankfehler"))?;
        if let Some((h, result)) = existing {
            if h != body_hash {
                return Err(err(
                    409,
                    "Befehlskennung schon für anderen Inhalt verwendet",
                ));
            }
            return serde_json::from_str(&result)
                .map_err(|_| err(503, "Gespeicherte Antwort defekt"));
        }
        if self.runtime.paused || self.world.beendet() {
            return Err(err(409, "Welt angehalten oder beendet"));
        }
        let actions = v["aktionen"]
            .as_array()
            .filter(|a| a.len() <= 8 && (!a.is_empty() || v.get("lease").is_some()))
            .ok_or_else(|| err(400, "Höchstens acht Aktionen erlaubt"))?;
        let role = if v.get("lease").is_some() {
            let l = self
                .runtime
                .leases
                .get(&sid)
                .filter(|l| {
                    l.account == a.id
                        && l.until > now()
                        && Some(l.hash.as_str()) == v["lease"].as_str().map(digest).as_deref()
                })
                .ok_or_else(|| err(409, "Agent gestoppt oder Freigabe abgelaufen"))?;
            let role = Rolle::aus_name(str_field(v, "rolle")?)
                .filter(|r| l.roles.contains(r))
                .ok_or_else(|| err(403, "Rolle nicht für Agent freigegeben"))?;
            role
        } else {
            if let Some(l) = self.runtime.leases.get(&sid).filter(|l| l.until > now()) {
                for action in actions {
                    let action: Aktion = serde_json::from_value(action.clone())
                        .map_err(|_| err(400, "Aktion nicht lesbar"))?;
                    if action.zustaendig().iter().all(|r| l.roles.contains(r)) {
                        return Err(err(
                            409,
                            "Diese Rolle steuert gerade ein Agent; zuerst stoppen",
                        ));
                    }
                }
            }
            Rolle::Alle
        };
        self.transaction(|g| {
            let result:Vec<Value>=actions.iter().map(|action| {let (ok,text)=g.world.handeln(sid,role,action);json!({"ok":ok,"text":text})}).collect();
            if role!=Rolle::Alle {
                let notes=v["notiz"].as_str();
                let wake=v["wecker_stunden"].as_f64().filter(|n|n.is_finite()).map(|n|(n.clamp(0.0,720.0)*3600.0) as i64);
                let hints=result.iter().filter(|v|v["ok"]==false).filter_map(|v|v["text"].as_str().map(str::to_string)).collect::<Vec<_>>();
                g.world.aufruf_ende(sid,role,notes,wake,&hints);
            }
            let response=json!({"ergebnisse":result,"world_id":g.runtime.world_id,"revision":g.runtime.revision+1});
            g.db.execute("INSERT INTO commands VALUES(?1,?2,?3,?4,?5,?6)",params![a.id,g.runtime.world_id,request,body_hash,response.to_string(),now()]).map_err(|_|err(503,"Befehl konnte nicht gespeichert werden"))?;
            g.db.execute("INSERT INTO audit(time,account,kind,result) VALUES(?1,?2,'spiel',?3)",params![now(),a.id,response.to_string()]).map_err(|_|err(503,"Protokoll nicht gespeichert"))?;
            Ok(response)
        })
    }
    pub fn agent_report(&mut self, a: &Account, v: &Value) -> Reply {
        let sid = Self::sid(a)?;
        self.world_matches(v)?;
        let l = self
            .runtime
            .leases
            .get(&sid)
            .filter(|l| l.until > now() && l.hash == digest(v["lease"].as_str().unwrap_or("")))
            .ok_or_else(|| err(409, "Agentfreigabe fehlt"))?;
        let _ = l;
        self.transaction(|g| {
            let clean=json!({"at":now(),"status":v["status"].as_str().unwrap_or("unbekannt").chars().take(80).collect::<String>(),"role":v["role"].as_str().unwrap_or(""),
              "calls":v["calls"].as_u64().unwrap_or(0),"errors":v["errors"].as_u64().unwrap_or(0),"tokens":v["tokens"].as_u64().unwrap_or(0),"latency_ms":v["latency_ms"].as_u64().unwrap_or(0),"accepted":v["accepted"].as_u64().unwrap_or(0),"rejected":v["rejected"].as_u64().unwrap_or(0),
              "provider":v["provider"].as_str().unwrap_or("").chars().take(32).collect::<String>(),"model":v["model"].as_str().unwrap_or("").chars().take(100).collect::<String>(),
              "decision":v["decision"].as_str().unwrap_or("").chars().take(1600).collect::<String>(),"cost":v["cost"].as_f64().filter(|c|c.is_finite()&&*c>=0.0)});
            g.runtime.agent_status.insert(sid,clean);Ok(json!({"ok":true}))
        })
    }
    pub fn logout(&mut self, a: &Account, raw: &str) -> Reply {
        self.transaction(|g| {
            g.db.execute("DELETE FROM sessions WHERE hash=?1", [digest(raw)])
                .map_err(|_| err(503, "Abmeldung fehlgeschlagen"))?;
            if let Some(sid) = a.sid {
                g.runtime.leases.remove(&sid);
            }
            Ok(json!({"ok":true}))
        })
    }
    pub fn admin_status(&self) -> Value {
        let accounts=self.db.prepare("SELECT id,name,sid,mode,banned FROM accounts ORDER BY id").and_then(|mut s|s.query_map([],|r|Ok(json!({"id":r.get::<_,i64>(0)?,"name":r.get::<_,String>(1)?,"spieler":r.get::<_,Option<u16>>(2)?,"mode":r.get::<_,String>(3)?,"banned":r.get::<_,bool>(4)?})))?.collect::<Result<Vec<_>,_>>()).unwrap_or_default();
        let bots=self.runtime.bots.iter().map(|(sid,b)|{let s=&self.world.spieler[*sid as usize];json!({"spieler":sid,"name":s.name,"typ":b.typ,"naechster":b.naechster,"aktionen":s.statistik.aktionen,"abgelehnt":s.statistik.abgelehnt,"stufe":s.stufe,"punkte":s.punkte.gesamt()})}).collect::<Vec<_>>();
        let db_bytes = ["spiel.sqlite3", "spiel.sqlite3-wal", "spiel.sqlite3-shm"]
            .iter()
            .map(|p| {
                std::fs::metadata(self.data_root.join(p))
                    .map(|m| m.len())
                    .unwrap_or(0)
            })
            .sum::<u64>();
        let invariant_errors = self
            .world
            .planeten
            .iter()
            .filter(|p| p.bestand.iter().any(|n| *n < 0) || p.bevoelkerung < 0)
            .map(|p| format!("Negativer Bestand/Bevölkerung auf {}", p.koord))
            .collect::<Vec<_>>();
        json!({"lobby":self.lobby(),"monitor":self.monitoring_status(),"storage_error":self.storage_error,"world_hash":self.world.hash(),"uptime_s":self.started.elapsed().as_secs(),"db_bytes":db_bytes,"invariant_errors":invariant_errors,"bots_enabled":self.runtime.bots_enabled,"bot_period_secs":self.runtime.bot_period_secs,"epoche_tage":self.world.regeln.welt.epoche_tage,"bots":bots,"accounts":accounts,
            "agents":self.runtime.agent_status.keys().chain(self.runtime.leases.keys()).copied().collect::<std::collections::BTreeSet<_>>().iter().map(|sid|json!({"spieler":sid,"active":self.runtime.leases.get(sid).is_some_and(|l|l.until>now()),"roles":self.runtime.leases.get(sid).map(|l|&l.roles),"stats":self.runtime.agent_status.get(sid).cloned().unwrap_or_else(||json!({"status":"Wartet auf erste Modellantwort","calls":0,"errors":0,"tokens":0,"latency_ms":0}))})).collect::<Vec<_>>(),
            "flotten":self.world.flotten.len(),"ereignisse":self.world.ereignisse.len(),"bot_actions":self.runtime.bot_actions,"bot_rejected":self.runtime.bot_rejected})
    }
    pub fn backup(&self) -> Result<PathBuf, (u16, String)> {
        let dir = self.data_root.join("backups");
        std::fs::create_dir_all(&dir).map_err(|_| err(503, "Backupverzeichnis fehlt"))?;
        let path = dir.join(format!("spiel-{}-{}.sqlite3", now(), &token()[..8]));
        self.db
            .backup(rusqlite::DatabaseName::Main, &path, None)
            .map_err(|_| err(503, "Backup fehlgeschlagen"))?;
        Ok(path)
    }
    pub fn admin(&mut self, v: &Value) -> Reply {
        let fields = [
            "action",
            "spieler",
            "id",
            "banned",
            "tempo",
            "paused",
            "bots_enabled",
            "bot_period_secs",
            "epoche_tage",
            "credits",
            "bestand",
            "seed",
            "typ",
            "maintenance",
        ];
        let context = Value::Object(
            fields
                .iter()
                .filter_map(|k| v.get(*k).map(|val| ((*k).into(), val.clone())))
                .collect(),
        );
        self.audit_context = Some(context.clone());
        let result = self.admin_inner(v);
        if result.is_ok() && matches!(v["action"].as_str(), Some("backup" | "inspect" | "map" | "events" | "audit" | "rules_profile" | "rules_validate" | "reset_preview")) {
            if self
                .db
                .execute(
                    "INSERT INTO audit(time,account,kind,result) VALUES(?1,NULL,'admin',?2)",
                    params![
                        now(),
                        json!({"context":context,"world_id":self.runtime.world_id}).to_string()
                    ],
                )
                .is_err()
            {
                self.audit_context = None;
                return Err(err(503, "Admin-Audit konnte nicht gespeichert werden"));
            }
        }
        self.audit_context = None;
        result
    }
    fn admin_inner(&mut self, v: &Value) -> Reply {
        self.world_matches(v)?;
        match str_field(v,"action")? {
            "map"|"events"|"audit"|"rules_profile"|"rules_validate"|"reset_preview"=>self.admin_read(v),
            "backup"=>Ok(json!({"backup":self.backup()?.file_name().unwrap().to_string_lossy()})),
            "settings"=>self.transaction(|g| {
                if let Some(n)=v["tempo"].as_u64(){if !(1..=3600).contains(&n){return Err(err(400,"Tempo zwischen 1 und 3600"));}g.runtime.tempo=n as u32;}
                if let Some(p)=v["paused"].as_bool(){g.runtime.paused=p;}
                if let Some(p)=v["bots_enabled"].as_bool(){g.runtime.bots_enabled=p;}
                if let Some(m)=v["maintenance"].as_str(){g.runtime.maintenance=m.chars().take(400).collect();}
                if let Some(n)=v["bot_period_secs"].as_i64(){if !(900..=86400).contains(&n){return Err(err(400,"Botabstand zwischen 900 und 86400 Spielsekunden"));}g.runtime.bot_period_secs=n;}
                if let Some(n)=v["epoche_tage"].as_i64(){if n<=g.world.zeit/TAG || n>10000{return Err(err(400,"Epochenende muss nach der aktuellen Spielzeit liegen, maximal 10000 Tage"));}
                    let mut r=(*g.world.regeln).clone();r.welt.epoche_tage=n;g.world.regeln=std::sync::Arc::new(r);}
                Ok(g.lobby())}),
            "inspect"=>{
                let sid=v["spieler"].as_u64().filter(|n|*n<(BOT_COUNT+SEATS) as u64).ok_or_else(||err(400,"Spielernummer fehlt"))? as u16;
                Ok(self.world.sicht(sid,Rolle::Alle))
            },
            "account"=>self.transaction(|g| {
                let id=v["id"].as_i64().ok_or_else(||err(400,"Kontonummer fehlt"))?;
                let banned=v["banned"].as_bool().ok_or_else(||err(400,"Sperrstatus fehlt"))?;
                if g.db.execute("UPDATE accounts SET banned=?1 WHERE id=?2",params![banned,id]).map_err(|_|err(503,"Konto nicht gespeichert"))?!=1{return Err(err(404,"Konto fehlt"));}
                if banned {g.db.execute("DELETE FROM sessions WHERE account=?1",[id]).map_err(|_|err(503,"Sitzungen nicht gesperrt"))?;g.runtime.leases.retain(|_,l|l.account!=id);}
                Ok(json!({"ok":true}))}),
            "bot"=>self.transaction(|g| {
                let sid=v["spieler"].as_u64().filter(|n|*n<BOT_COUNT as u64).ok_or_else(||err(400,"Botnummer ungültig"))? as u16;
                let typ=Bottyp::aus_name(str_field(v,"typ")?).ok_or_else(||err(400,"Unbekannter Bottyp"))?;
                let bot=g.runtime.bots.get_mut(&sid).unwrap();bot.typ=typ;bot.naechster=g.world.zeit;Ok(json!({"ok":true}))}),
            "stop_agent"=>self.transaction(|g| {let sid=v["spieler"].as_u64().filter(|n|*n<(BOT_COUNT+SEATS) as u64).ok_or_else(||err(400,"Spielernummer fehlt"))? as u16;g.runtime.leases.remove(&sid);Ok(json!({"ok":true}))}),
            "player_edit"=>{
                self.backup()?;
                self.transaction(|g| {
                    let sid=v["spieler"].as_u64().filter(|n|*n<(BOT_COUNT+SEATS) as u64).ok_or_else(||err(400,"Spielernummer fehlt"))? as u16;
                    if !g.world.spieler_aktiv(sid){return Err(err(400,"Freier Platz kann nicht bearbeitet werden"));}
                    let pid=g.world.spieler[sid as usize].heimat as usize;
                    if let Some(n)=v["credits"].as_i64(){if !(0..=1_000_000_000).contains(&n){return Err(err(400,"Credits außerhalb Grenze"));}g.world.spieler[sid as usize].credits=n*M;}
                    if let Some(m)=v["bestand"].as_object(){g.world.abrechnen(pid);for (k,n) in m {let gut=Gut::aus_name(k).ok_or_else(||err(400,"Unbekanntes Gut"))?;let n=n.as_i64().filter(|n|(0..=1_000_000_000).contains(n)).ok_or_else(||err(400,"Ungültiger Bestand"))?;g.world.planeten[pid].bestand[gut.idx()]=n*M;}}
                    g.world.raten_neu(pid);g.world.punkte_neu();Ok(json!({"ok":true}))
                })
            },
            "reset"=>{
                if v["confirm"].as_str()!=Some(&format!("RESET {}",self.runtime.world_id)){return Err(err(400,"Resetbestätigung passt nicht zur aktuellen Welt"));}
                let seed=v["seed"].as_u64().unwrap_or_else(||OsRng.next_u64());
                let rules=self.validated_profile(v)?;
                let (world,runtime)=fresh(seed,&rules).map_err(|e|err(400,&e))?;
                let backup=self.backup()?;
                self.transaction(move |g| {g.world=world;g.runtime=runtime;
                    g.db.execute_batch("UPDATE accounts SET sid=NULL,mode='zuschauer'; DELETE FROM sessions; DELETE FROM commands;").map_err(|_|err(503,"Reset nicht gespeichert"))?;
                    Ok(json!({"world_id":g.runtime.world_id,"backup":backup.file_name().unwrap().to_string_lossy()}))})
            },
            _=>Err(err(400,"Unbekannter Verwaltungsbefehl"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn game() -> Game {
        let p = std::env::temp_dir().join(format!("sternenepoche-test-{}", token()));
        Game::open(&p, 83).unwrap()
    }
    fn user(g: &mut Game, name: &str) -> (Account, String) {
        let v = g
            .register(
                &json!({"name":name}),
                password_hash("lange-test-passphrase").unwrap(),
            )
            .unwrap();
        (
            g.authenticate(v["token"].as_str().unwrap()).unwrap(),
            v["token"].as_str().unwrap().into(),
        )
    }
    #[test]
    fn seats_freeze_atomic_claim_and_restart() {
        let mut g = game();
        g.runtime.bots_enabled = false;
        assert_eq!(g.lobby()["freie_plaetze"], 20);
        let old = g.world.planeten[g.world.spieler[30].heimat as usize].clone();
        let credits = g.world.spieler[30].credits;
        for _ in 0..24 {
            g.advance(3600).unwrap();
        }
        let p = &g.world.planeten[g.world.spieler[30].heimat as usize];
        assert_eq!(old.bestand, p.bestand);
        assert_eq!(old.bevoelkerung, p.bevoelkerung);
        assert_eq!(credits, g.world.spieler[30].credits);
        for i in 0..20 {
            let (a, _) = user(&mut g, &format!("Spieler{i}"));
            g.claim(&a, &json!({"mode":"mensch","volk":"veyari"}))
                .unwrap();
        }
        let (a, _) = user(&mut g, "Einundzwanzig");
        assert_eq!(
            g.claim(&a, &json!({"mode":"mensch","volk":"krath"}))
                .unwrap_err()
                .0,
            409
        );
        assert_eq!(
            g.world.spieler[30].schutz_bis,
            86400 + g.world.regeln.diplomatie.anfaengerschutz_tage * TAG
        );
        let hash = g.world.hash();
        let root = g.data_root.clone();
        drop(g);
        let g = Game::open(&root, 1).unwrap();
        assert_eq!(hash, g.world.hash());
        assert_eq!(g.lobby()["freie_plaetze"], 0);
    }
    #[test]
    fn spectator_duplicate_private_lease_stop_reset() {
        let mut g = game();
        let (watch, raw) = user(&mut g, "Zuschauer");
        assert_eq!(g.view(&watch, Rolle::Alle).unwrap_err().0, 403);
        assert!(g.lobby().get("planeten").is_none());
        assert!(g.admin_status().to_string().find(&watch.password).is_none());
        g.claim(&watch, &json!({"mode":"gemischt","volk":"krath"}))
            .unwrap();
        let a = g.authenticate(&raw).unwrap();
        let start = g.world.planeten[g.world.spieler[a.sid.unwrap() as usize].heimat as usize]
            .koord
            .to_string();
        let cmd = json!({"world_id":g.runtime.world_id,"request_id":"befehl-1","aktionen":[{"typ":"bauen","planet":start,"gebaeude":"erzmine"}]});
        let result = g.command(&a, &cmd).unwrap();
        let count = g.world.spieler[30].statistik.aktionen;
        assert_eq!(result, g.command(&a, &cmd).unwrap());
        assert_eq!(count, g.world.spieler[30].statistik.aktionen);
        let mut different = cmd.clone();
        different["aktionen"][0]["gebaeude"] = json!("farm");
        assert_eq!(g.command(&a, &different).unwrap_err().0, 409);
        let lease = g
            .lease(
                &a,
                &json!({"world_id":g.runtime.world_id,"action":"start","roles":["verwalter"]}),
            )
            .unwrap();
        g.agent_report(&a,&json!({"world_id":g.runtime.world_id,"lease":lease["lease"],"status":"aktiv","calls":1})).unwrap();
        let mut remembered = g.runtime.bots[&0].clone();
        remembered.gespaeht.insert(Koord::neu(1, 2, 3), 42);
        g.runtime.bots.insert(0, remembered);
        let runtime_bytes = serde_json::to_vec(&g.runtime).unwrap();
        let restored: Runtime = serde_json::from_slice(&runtime_bytes).unwrap();
        assert_eq!(restored.agent_status[&30]["calls"], 1);
        assert_eq!(restored.bots[&0].gespaeht[&Koord::neu(1, 2, 3)], 42);
        let late = json!({"world_id":g.runtime.world_id,"request_id":"late","lease":lease["lease"],"rolle":"verwalter","aktionen":[{"typ":"bauen","planet":start,"gebaeude":"farm"}]});
        g.lease(&a, &json!({"world_id":g.runtime.world_id,"action":"stop"}))
            .unwrap();
        assert_eq!(g.command(&a, &late).unwrap_err().0, 409);
        let wid = g.runtime.world_id.clone();
        g.admin(&json!({"world_id":wid,"action":"reset","confirm":format!("RESET {wid}")}))
            .unwrap();
        assert_eq!(g.command(&a, &cmd).unwrap_err().0, 409);
        assert_eq!(g.authenticate(&raw).unwrap_err().0, 401);
        assert_eq!(g.lobby()["freie_plaetze"], 20);
    }
    #[test]
    fn rollback_storage_and_passwords() {
        let mut g = game();
        let hash = password_hash("richtiges Passwort").unwrap();
        assert!(password_matches("richtiges Passwort", &hash));
        assert!(!password_matches("falsch", &hash));
        let old = g.world.hash();
        let rev = g.runtime.revision;
        let wid = g.runtime.world_id.clone();
        assert!(g
            .admin(&json!({"world_id":wid,"action":"settings","tempo":123,"paused":true}))
            .is_ok());
        assert!(g.admin(&json!({"world_id":wid,"action":"player_edit","spieler":0,"credits":500,"bestand":{"erz":100,"xxx":5}})).is_err());
        assert_eq!(old, g.world.hash());
        assert_eq!(rev + 1, g.runtime.revision);
        let audit_count: i64 =
            g.db.query_row("SELECT count(*) FROM audit WHERE kind='admin'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(audit_count, 1);
        g.admin(&json!({"world_id":wid,"action":"inspect","spieler":0}))
            .unwrap();
        let audit_count: i64 =
            g.db.query_row("SELECT count(*) FROM audit WHERE kind='admin'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(audit_count, 2);
    }
    #[test]
    fn thirty_bots_one_game_day_and_backup_resume() {
        let mut g = game();
        for _ in 0..24 {
            g.advance(3600).unwrap();
        }
        assert_eq!(g.lobby()["freie_plaetze"], 20);
        assert!(g.world.spieler[..30]
            .iter()
            .all(|s| s.statistik.aktionen > 0));
        assert!(g.admin_status()["invariant_errors"]
            .as_array()
            .unwrap()
            .is_empty());
        let root = g.data_root.join("restore-test");
        std::fs::create_dir(&root).unwrap();
        std::fs::copy(g.backup().unwrap(), root.join("spiel.sqlite3")).unwrap();
        let mut restored = Game::open(&root, 0).unwrap();
        assert_eq!(g.world.hash(), restored.world.hash());
        g.advance(3600).unwrap();
        restored.advance(3600).unwrap();
        assert_eq!(g.world.hash(), restored.world.hash());
        assert_eq!(g.runtime.bot_actions, restored.runtime.bot_actions);
    }
    #[test]
    fn monitoring_privacy_profile_preview_and_legacy_runtime() {
        let mut g=game();g.advance(3600).unwrap();
        let status=g.admin_status();assert_eq!(status["monitor"]["bots"].as_array().unwrap().len(),30);
        assert!(status["monitor"]["bots"].as_array().unwrap().iter().any(|b|!b["verlauf"].as_array().unwrap().is_empty()));
        let history=g.lobby()["verlauf"].as_array().unwrap().clone();assert_eq!(history.len(),1);
        assert_eq!(history[0].as_object().unwrap().keys().cloned().collect::<std::collections::BTreeSet<_>>(),["kolonien","punkte","reiche","sekunden","stufen"].into_iter().map(str::to_owned).collect());
        let wid=g.runtime.world_id.clone();let hash=g.world.hash();
        let profile=g.admin(&json!({"action":"rules_profile","world_id":wid})).unwrap()["text"].as_str().unwrap().to_owned();
        let preview=g.admin(&json!({"action":"reset_preview","world_id":wid,"rules":profile})).unwrap();
        assert_eq!(hash,g.world.hash());assert_eq!(wid,g.runtime.world_id);
        for invalid in [profile.replace("sektoren: 2","sektoren: 0"),profile.replace("treibstoff_teiler: 35000","treibstoff_teiler: 0"),profile.replace("reichtum_min: 0.7","reichtum_min: NaN")] {
            if invalid!=profile {assert!(g.admin(&json!({"action":"reset_preview","world_id":wid,"rules":invalid})).is_err());assert_eq!(hash,g.world.hash());}
        }
        assert!(g.admin(&json!({"action":"reset","world_id":wid,"confirm":format!("RESET {wid}"),"rules":"invalid"})).is_err());
        assert!(!g.data_root.join("backups").exists());assert_eq!(hash,g.world.hash());
        let mut old=serde_json::to_value(&g.runtime).unwrap();for k in ["bot_trace","public_history","maintenance"]{old.as_object_mut().unwrap().remove(k);}
        let restored:Runtime=serde_json::from_value(old).unwrap();assert!(restored.bot_trace.is_empty());assert!(restored.public_history.is_empty());
        g.admin(&json!({"action":"reset","world_id":wid,"confirm":format!("RESET {wid}"),"rules":profile})).unwrap();
        assert_eq!(g.world.regeln.hash,preview["profile_hash"].as_str().unwrap());assert_eq!(g.lobby()["freie_plaetze"],20);
    }
    #[test]
    fn thirty_bots_thirty_days_health_and_bounded_history() {
        let mut g=game();
        for _ in 0..(30*24){g.advance(3600).unwrap();}
        assert_eq!(g.lobby()["freie_plaetze"],20);
        assert_eq!(g.runtime.public_history.len(),168);
        assert!(g.runtime.bot_trace.values().all(|t|t.len()<=20));
        assert!(g.admin_status()["invariant_errors"].as_array().unwrap().is_empty());
        assert!(g.world.spieler[..30].iter().all(|s|s.statistik.aktionen>30));
        let summary=json!({"spieltage":30,"aktionen":g.runtime.bot_actions,"abgelehnt":g.runtime.bot_rejected,
            "stufen":g.world.spieler[..30].iter().map(|s|s.stufe).collect::<Vec<_>>(),"kolonien":g.world.spieler[..30].iter().map(|s|s.planeten.len()).collect::<Vec<_>>()});
        println!("30-Tage-Botprüfung: {summary}");
        assert!(g.runtime.bot_trace.values().flatten().all(|v|v["text"].as_str().is_none_or(|text|!text.contains("benötigt einen eigenen Geheimdienst")&&!text.contains("Freier Startplatz"))));
    }
}
