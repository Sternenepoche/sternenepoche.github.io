use super::{
    config::{identifier, Harness},
    Result,
};
use rusqlite::{params, Connection, DatabaseName};
use serde_json::{json, Value};
use std::path::Path;
#[path = "memory_limits.rs"]
pub mod limits;

/// Private, real SQLite store. Runtime reconstructs it from the committed checkpoint on resume.
pub struct Memory {
    pub conn: Connection,
    pub owner: u16,
    pub quota: u64,
    remote: Option<std::cell::RefCell<RemoteMemory>>,
}
enum RemoteMemory { Docker(super::sandbox::WorkerSession), Native(super::native_sandbox::NativeSession) }
impl RemoteMemory {
    fn request(&mut self,value:&Value)->Result<Value> {match self {Self::Docker(s)=>s.request(value),Self::Native(s)=>s.request(value)}}
}
impl Memory {
    pub fn new(owner: u16, quota: u64, harness: &Harness) -> Result<Self> {
        let conn = Connection::open_in_memory().map_err(|e| e.to_string())?;
        conn.execute_batch("PRAGMA foreign_keys=ON; PRAGMA page_size=4096; PRAGMA max_page_count=7168; CREATE TABLE records(kind TEXT NOT NULL,key TEXT NOT NULL,revision INTEGER NOT NULL,value TEXT NOT NULL,PRIMARY KEY(kind,key)); CREATE TABLE meta(key TEXT PRIMARY KEY,value TEXT NOT NULL);").map_err(|e|e.to_string())?;
        conn.pragma_update(None, "max_page_count", limits::DB_MAX_PAGES)
            .map_err(|e| e.to_string())?;
        let s = Self {
            conn,
            owner,
            quota,
            remote: None,
        };
        s.meta_set("owner", &json!(owner))?;
        s.meta_set("harness", &json!(harness))?;
        s.meta_set("cursor", &json!(0))?;
        s.meta_set("day", &json!(-1))?;
        s.meta_set("calls", &json!(0))?;
        s.meta_set("reserved_micro_usd", &json!(0))?;
        Ok(s)
    }
    pub fn load(path: &Path, owner: u16, quota: u64) -> Result<Self> {
        let mut conn = Connection::open_in_memory().map_err(|e| e.to_string())?;
        conn.restore(
            DatabaseName::Main,
            path,
            None::<fn(rusqlite::backup::Progress)>,
        )
        .map_err(|e| e.to_string())?;
        conn.pragma_update(None, "max_page_count", limits::DB_MAX_PAGES)
            .map_err(|e| e.to_string())?;
        let m = Self {
            conn,
            owner,
            quota,
            remote: None,
        };
        if m.meta("owner")?.as_u64() != Some(owner as u64) {
            return Err("Fremde Spieler-Datenbank".into());
        }
        Ok(m)
    }
    pub fn save(&self, path: &Path) -> Result<()> {
        if self.remote.is_some() {
            let state = self.export_state()?;
            return Self::from_state(self.owner, self.quota, &state)?.save(path);
        }
        self.conn
            .backup(DatabaseName::Main, path, None)
            .map_err(|e| e.to_string())
    }
    pub fn isolate(mut self, image: &str, namespace: &str) -> Result<Self> {
        let state = self.export_state()?;
        let mut session =
            super::sandbox::WorkerSession::start(image, namespace, self.owner, self.quota)?;
        session.request(&json!({"op":"import","state":state}))?;
        self.remote = Some(std::cell::RefCell::new(RemoteMemory::Docker(session)));
        Ok(self)
    }
    pub fn isolate_native(mut self,worker:&Path,namespace:&str)->Result<Self> {
        let state=self.export_state()?;
        let mut session=super::native_sandbox::NativeSession::start(worker,namespace,self.owner,self.quota)?;
        session.request(&json!({"op":"import","state":state}))?;
        self.remote=Some(std::cell::RefCell::new(RemoteMemory::Native(session)));Ok(self)
    }
    fn rpc(&self, v: Value) -> Result<Value> {
        self.remote
            .as_ref()
            .ok_or("Kein Worker")?
            .borrow_mut()
            .request(&v)
    }
    pub fn export_state(&self) -> Result<Value> {
        if self.remote.is_some() {
            return self.rpc(json!({"op":"export"}));
        }
        let mut stmt = self
            .conn
            .prepare("SELECT key,value FROM meta ORDER BY key")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
            .map_err(|e| e.to_string())?;
        let mut meta = serde_json::Map::new();
        for r in rows {
            let (k, s) = r.map_err(|e| e.to_string())?;
            meta.insert(
                k,
                serde_json::from_str::<Value>(&s).map_err(|e| e.to_string())?,
            );
        }
        Ok(json!({"meta":meta,"records":self.dump()?}))
    }
    pub fn from_state(owner: u16, quota: u64, state: &Value) -> Result<Self> {
        if state["meta"]["owner"] != owner {
            return Err("Fremder Memory-Export".into());
        }
        let harness: Harness =
            serde_json::from_value(state["meta"]["harness"].clone()).map_err(|e| e.to_string())?;
        let m = Self::new(owner, quota, &harness)?;
        let records = state["records"].as_array().ok_or("Records fehlen")?;
        if records.len() as u64 > limits::MAX_RECORDS {
            return Err("Maximal 4096 private Records".into());
        }
        let mut size = 0u64;
        let mut unique = std::collections::BTreeSet::new();
        for r in records {
            let kind = r["kind"].as_str().ok_or("kind fehlt")?;
            let key = r["key"].as_str().ok_or("key fehlt")?;
            let revision = r["revision"].as_u64().ok_or("revision fehlt")?;
            let serialized = r["value"].to_string();
            if ![
                "note", "plan", "task", "skill", "belief", "receipt", "learning",
            ]
            .contains(&kind)
                || !identifier(key)
                || serialized.len() > 32_768
                || revision == 0
                || revision >= i64::MAX as u64
                || !unique.insert((kind, key))
            {
                return Err("Ungültiger oder doppelter Memory-Record".into());
            }
            size = size
                .checked_add(limits::record_bytes(kind, key, &serialized))
                .ok_or("Speichergröße übergelaufen")?;
        }
        if size > quota {
            return Err("Privates Speicherbudget ausgeschöpft".into());
        }
        m.conn
            .execute_batch("BEGIN; DELETE FROM meta;")
            .map_err(|e| e.to_string())?;
        for (k, v) in state["meta"].as_object().ok_or("Meta fehlt")? {
            m.meta_set(k, v)?;
        }
        for r in records {
            let k = r["kind"].as_str().ok_or("kind fehlt")?;
            let id = r["key"].as_str().ok_or("key fehlt")?;
            m.conn
                .execute(
                    "INSERT INTO records VALUES(?1,?2,?3,?4)",
                    params![
                        k,
                        id,
                        r["revision"].as_u64().unwrap(),
                        r["value"].to_string()
                    ],
                )
                .map_err(|e| e.to_string())?;
        }
        m.conn.execute_batch("COMMIT").map_err(|e| e.to_string())?;
        Ok(m)
    }
    pub fn meta(&self, key: &str) -> Result<Value> {
        if self.remote.is_some() {
            return self.rpc(json!({"op":"meta","key":key}));
        }
        let s: String = self
            .conn
            .query_row("SELECT value FROM meta WHERE key=?1", [key], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        serde_json::from_str(&s).map_err(|e| e.to_string())
    }
    pub fn meta_set(&self, key: &str, v: &Value) -> Result<()> {
        if self.remote.is_some() {
            self.rpc(json!({"op":"meta_set","key":key,"value":v}))?;
            return Ok(());
        }
        if key == "owner" && *v != json!(self.owner) {
            return Err("Spieleridentität unveränderlich".into());
        }
        let serialized = v.to_string();
        if key.len() > 128 || serialized.len() > limits::META_VALUE_BYTES {
            return Err("Meta außerhalb Workergrenzen".into());
        }
        let (count,total,exists):(usize,usize,bool)=self.conn.query_row("SELECT COUNT(*),COALESCE(SUM(CASE WHEN key!=?1 THEN length(CAST(key AS BLOB))+length(CAST(value AS BLOB)) ELSE 0 END),0),COALESCE(MAX(key=?1),0) FROM meta",[key],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).map_err(|e|e.to_string())?;
        if (!exists && count >= limits::META_KEYS)
            || total + key.len() + serialized.len() > limits::META_BYTES
        {
            return Err("Meta-Gesamtquote überschritten".into());
        }
        self.conn.execute("INSERT INTO meta VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",params![key,serialized]).map_err(|e|e.to_string())?;
        Ok(())
    }
    pub fn put(&self, kind: &str, key: &str, revision: u64, value: &Value) -> Result<Value> {
        if self.remote.is_some() {
            return self
                .rpc(json!({"op":"put","kind":kind,"key":key,"revision":revision,"value":value}));
        }
        let serialized = value.to_string();
        if ![
            "note", "plan", "task", "skill", "belief", "receipt", "learning",
        ]
        .contains(&kind)
            || !identifier(key)
            || serialized.len() > 32_768
        {
            return Err("Ungültiger Speicherbereich/Schlüssel oder Eintrag >32KiB".into());
        }
        let old = self.get(kind, key)?;
        let actual = old
            .as_ref()
            .and_then(|v| v["revision"].as_u64())
            .unwrap_or(0);
        if actual != revision {
            return Err(format!(
                "Versionskonflikt: erwartet {revision}, aktuell {actual}"
            ));
        }
        let (count,size):(u64,u64)=self.conn.query_row("SELECT COUNT(*),COALESCE(SUM(length(CAST(value AS BLOB))+length(CAST(kind AS BLOB))+length(CAST(key AS BLOB))+128),0) FROM records",[],|r|Ok((r.get(0)?,r.get(1)?))).map_err(|e|e.to_string())?;
        if old.is_none() && count >= limits::MAX_RECORDS {
            return Err("Maximal 4096 private Records".into());
        }
        let old_len = old
            .as_ref()
            .map(|v| limits::record_bytes(kind, key, &v["value"].to_string()))
            .unwrap_or(0);
        if size - old_len + limits::record_bytes(kind, key, &serialized) > self.quota {
            return Err("Privates Speicherbudget ausgeschöpft".into());
        }
        if revision >= i64::MAX as u64 - 1 {
            return Err("Recordrevision übergelaufen".into());
        }
        self.conn.execute("INSERT INTO records VALUES(?1,?2,?3,?4) ON CONFLICT(kind,key) DO UPDATE SET revision=excluded.revision,value=excluded.value",params![kind,key,revision+1,serialized]).map_err(|e|e.to_string())?;
        Ok(json!({"kind":kind,"key":key,"revision":revision+1,"value":value}))
    }
    pub fn get(&self, kind: &str, key: &str) -> Result<Option<Value>> {
        if self.remote.is_some() {
            let v = self.rpc(json!({"op":"get","kind":kind,"key":key}))?;
            return Ok(if v.is_null() { None } else { Some(v) });
        }
        let mut stmt = self
            .conn
            .prepare("SELECT revision,value FROM records WHERE kind=?1 AND key=?2")
            .map_err(|e| e.to_string())?;
        let mut rows = stmt.query(params![kind, key]).map_err(|e| e.to_string())?;
        if let Some(r) = rows.next().map_err(|e| e.to_string())? {
            let rev: u64 = r.get(0).map_err(|e| e.to_string())?;
            let s: String = r.get(1).map_err(|e| e.to_string())?;
            Ok(Some(
                json!({"kind":kind,"key":key,"revision":rev,"value":serde_json::from_str::<Value>(&s).map_err(|e|e.to_string())?}),
            ))
        } else {
            Ok(None)
        }
    }
    pub fn search(&self, query: &str, kind: Option<&str>, limit: usize) -> Result<Vec<Value>> {
        if self.remote.is_some() {
            return serde_json::from_value(
                self.rpc(json!({"op":"search","query":query,"kind":kind,"limit":limit}))?,
            )
            .map_err(|e| e.to_string());
        }
        // instr is literal substring search, so SQL/wildcard injection cannot broaden a query.
        let mut stmt=self.conn.prepare("SELECT kind,key,revision,value FROM records WHERE (?2 IS NULL OR kind=?2) AND (instr(lower(value),lower(?1))>0 OR instr(lower(key),lower(?1))>0) ORDER BY kind,key LIMIT ?3").map_err(|e|e.to_string())?;
        let rows = stmt
            .query_map(params![query, kind, limit.min(30)], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, u64>(2)?,
                    r.get::<_, String>(3)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        rows.map(|r| {let(k,id,rev,s)=r.map_err(|e|e.to_string())?; Ok(json!({"kind":k,"key":id,"revision":rev,"value":serde_json::from_str::<Value>(&s).map_err(|e|e.to_string())?}))}).collect()
    }
    pub fn dump(&self) -> Result<Value> {
        if self.remote.is_some() {
            return self.rpc(json!({"op":"dump"}));
        }
        let mut stmt = self
            .conn
            .prepare("SELECT kind,key,revision,value FROM records ORDER BY kind,key")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, u64>(2)?,
                    r.get::<_, String>(3)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        let mut values = Vec::new();
        for r in rows {
            let (k, id, rev, s) = r.map_err(|e| e.to_string())?;
            values.push(json!({"kind":k,"key":id,"revision":rev,"value":serde_json::from_str::<Value>(&s).map_err(|e|e.to_string())?}));
        }
        Ok(json!(values))
    }
}
