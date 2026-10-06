use super::Result;
use crate::journal::{hash, Journal};
use serde_json::{json, Value};
use std::path::Path;

#[derive(Clone)]
pub struct Audit {
    pub events: Vec<Value>,
    pub previous: String,
    pub sequence: u64,
    pub window: usize,
    pub run: String,
}
impl Audit {
    pub fn new(run: String, window: usize, sequence: u64, previous: String) -> Self {
        Self {
            events: vec![],
            previous,
            sequence,
            window,
            run,
        }
    }
    pub fn event(
        &mut self,
        kind: &str,
        time: i64,
        player: Option<u16>,
        role: Option<&str>,
        payload: Value,
    ) -> String {
        self.sequence += 1;
        let id = format!("{}:{:012}", &self.run[..16], self.sequence);
        let mut e = json!({"schema_version":1,"event_id":id,"run_id":self.run,"epoch_id":self.run,
            "sequence":self.sequence,"window_id":self.window,"sim_time":time,"event_type":kind,
            "player_id":player,"role_id":role,"visibility":if player.is_some(){"private"}else{"research"},
            "previous_hash":self.previous,"payload":payload});
        let digest = hash(e.to_string().as_bytes());
        e["hash"] = json!(digest);
        self.previous = digest;
        self.events.push(e);
        id
    }
    pub fn save(&self, j: &Journal) -> Result<()> {
        j.put(
            &format!("events/{:08}.json", self.window),
            &serde_json::to_vec(&self.events).map_err(|e| e.to_string())?,
        )
    }
}
pub fn verify(root: &Path) -> Result<Value> {
    let checkpoints = super::runtime::checkpoint_manifests(root)?;
    let identity: Value = serde_json::from_slice(
        &std::fs::read(root.join("identity.json")).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let manifest: Value = serde_json::from_slice(
        &std::fs::read(root.join("manifest.json")).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    if identity["contract_sha256"] != hash(manifest.to_string().as_bytes())
        || checkpoints.is_empty()
    {
        return Err("Vertrag/Laufidentität oder Anfangscheckpoint ungültig".into());
    }
    let mut checked = std::collections::BTreeMap::<String, String>::new();
    for (index, cp) in &checkpoints {
        if cp["index"] != *index {
            return Err("Falscher Checkpoint-Index".into());
        }
        let world = std::fs::read(root.join(format!("checkpoints/{index:08}.bin")))
            .map_err(|e| e.to_string())?;
        if cp["world_sha256"] != hash(&world) {
            return Err("Weltartefakt beschädigt".into());
        }
        for p in cp["players"].as_array().ok_or("DB-Belege fehlen")? {
            let path = p["path"].as_str().ok_or("DB-Pfad fehlt")?;
            if !path.starts_with("memory/")
                || path.contains("..")
                || path.contains('\\')
                || path.contains(':')
            {
                return Err("Ungültiger Artefaktpfad".into());
            }
            let actual = if let Some(h) = checked.get(path) {
                h.clone()
            } else {
                let h = hash(&std::fs::read(root.join(path)).map_err(|e| e.to_string())?);
                checked.insert(path.to_string(), h.clone());
                h
            };
            if p["sha256"] != actual {
                return Err("DB-Artefakt beschädigt".into());
            }
        }
    }
    let mut previous = String::new();
    let mut seq = 0u64;
    let mut count = 0;
    let mut seen_ids = std::collections::BTreeSet::new();
    for (index, cp) in checkpoints.iter().filter(|(i, _)| *i > 0) {
        let bytes = std::fs::read(root.join(format!("events/{:08}.json", index - 1)))
            .map_err(|e| e.to_string())?;
        if cp["events_sha256"] != hash(&bytes) {
            return Err("Audit-Dateihash stimmt nicht".into());
        }
        let events: Vec<Value> = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        for mut e in events {
            let actual = e
                .as_object_mut()
                .ok_or("Ereignis kein Objekt")?
                .remove("hash")
                .ok_or("Ereignishash fehlt")?;
            seq += 1;
            if e["run_id"] != identity["run_id"]
                || e["window_id"] != index - 1
                || e["previous_hash"] != previous
                || e["sequence"] != seq
                || actual != hash(e.to_string().as_bytes())
            {
                return Err(format!("Audit-Kette beschädigt bei {seq}"));
            }
            if let Some(parents) = e["payload"]["parent_ids"].as_array() {
                if parents
                    .iter()
                    .any(|id| id.as_str().is_none_or(|id| !seen_ids.contains(id)))
                {
                    return Err(
                        "Fachereignis verweist auf eine fehlende oder zukünftige Ursache".into(),
                    );
                }
            }
            let event_id = e["event_id"]
                .as_str()
                .ok_or("Ereignis-ID fehlt")?
                .to_string();
            if !seen_ids.insert(event_id) {
                return Err("Doppelte Ereignis-ID".into());
            }
            if matches!(
                e["event_type"].as_str(),
                Some("scheduler.finished" | "scheduler.overrun")
            ) && e["payload"].get("completion_path").is_some()
            {
                let expected = format!("deadlines/{:08}.finished.json", index - 1);
                if e["payload"]["completion_path"] != expected
                    || e["payload"]["completion_sha256"]
                        != hash(&std::fs::read(root.join(expected)).map_err(|e| e.to_string())?)
                {
                    return Err("Fensterabschluss nachträglich verändert".into());
                }
            }
            if e["event_type"] == "scheduler.window" || e["event_type"] == "scheduler.deferred" {
                let deferred = e["event_type"] == "scheduler.deferred";
                let default = format!("deadlines/{:08}.json", index - 1);
                let field = if deferred {
                    "closure_path"
                } else {
                    "deadline_path"
                };
                let digest = if deferred {
                    "closure_sha256"
                } else {
                    "deadline_sha256"
                };
                let path = e["payload"][field].as_str().unwrap_or(&default);
                let prefix = format!(
                    "deadlines/{:08}.{}-",
                    index - 1,
                    if deferred { "closed" } else { "slice" }
                );
                let valid = (!deferred && path == default)
                    || path
                        .strip_prefix(&prefix)
                        .and_then(|s| s.strip_suffix(".json"))
                        .is_some_and(|s| s.len() == 2 && s.bytes().all(|b| b.is_ascii_digit()));
                if !valid
                    || e["payload"][digest]
                        != hash(&std::fs::read(root.join(path)).map_err(|e| e.to_string())?)
                {
                    return Err("Fensterfrist oder Arbeitsabschnitt nachträglich verändert".into());
                }
            }
            if e["event_type"] == "call.completed"
                || e["event_type"] == "call.failed"
                || e["event_type"] == "call.skipped"
            {
                let key = e["payload"]["call_id"].as_str().ok_or("Call-ID fehlt")?;
                if !super::config::identifier(key) {
                    return Err("Ungültige Call-ID".into());
                }
                let bytes = std::fs::read(root.join(format!("calls/{key}.response.json")))
                    .map_err(|e| e.to_string())?;
                if e["payload"]["response_sha256"] != hash(&bytes) {
                    return Err("Modellantwort nachträglich verändert".into());
                }
                if e["event_type"] == "call.failed"
                    && e["payload"]["retry_sha256"]
                        != hash(
                            &std::fs::read(root.join(format!("calls/{key}.retry.json")))
                                .map_err(|e| e.to_string())?,
                        )
                {
                    return Err("Retry-Beleg nachträglich verändert".into());
                }
            }
            if e["event_type"] == "call.queued" {
                let key = e["payload"]["call_id"].as_str().ok_or("Call-ID fehlt")?;
                if !super::config::identifier(key) {
                    return Err("Ungültige Call-ID".into());
                }
                let bytes = std::fs::read(root.join(format!("calls/{key}.request.json")))
                    .map_err(|e| e.to_string())?;
                if e["payload"]["input_sha256"] != hash(&bytes) {
                    return Err("Modelleingabe nachträglich verändert".into());
                }
            }
            previous = actual.as_str().ok_or("Hash kein Text")?.into();
            count += 1;
        }
        if cp["audit_hash"] != previous || cp["audit_sequence"] != seq {
            return Err("Audit-Checkpoint passt nicht".into());
        }
    }
    Ok(json!({"events":count,"last_hash":previous,"checkpoints":checkpoints.len(),"verified":true}))
}

/// Player trajectories contain only observations and outputs available to that player, never
/// world.effect (global state). Future results remain separate from decision input.
pub fn export(root: &Path, out: &Path) -> Result<Value> {
    let integrity = verify(root)?;
    if out.exists() {
        return Err("Exportziel muss neu sein".into());
    }
    std::fs::create_dir_all(out).map_err(|e| e.to_string())?;
    let cp = super::runtime::checkpoint_manifests(root)?;
    let mut players = std::collections::BTreeMap::<u16, Vec<Value>>::new();
    for (index, _) in cp.iter().filter(|(i, _)| *i > 0) {
        let events: Vec<Value> = serde_json::from_slice(
            &std::fs::read(root.join(format!("events/{:08}.json", index - 1)))
                .map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        for e in events {
            if let Some(player) = e["player_id"].as_u64() {
                players.entry(player as u16).or_default().push(e);
            }
        }
    }
    let mut files = Vec::new();
    for (id, events) in players {
        let name = format!("player-{id:03}.jsonl");
        let text = events
            .into_iter()
            .map(|e| e.to_string() + "\n")
            .collect::<String>();
        std::fs::write(out.join(&name), &text).map_err(|e| e.to_string())?;
        files.push(json!({"path":name,"sha256":hash(text.as_bytes()),"visibility":"player","player_id":id}));
    }
    let source = std::fs::read(root.join("manifest.json")).map_err(|e| e.to_string())?;
    let identity: Value = serde_json::from_slice(
        &std::fs::read(root.join("identity.json")).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let source_manifest: Value = serde_json::from_slice(&source).map_err(|e| e.to_string())?;
    let config: super::config::Config =
        serde_json::from_value(source_manifest["config"].clone()).map_err(|e| e.to_string())?;
    let quality = super::runtime::status(root)?;
    let index = cp.last().ok_or("Quellcheckpoint fehlt")?.0;
    let world = kern::Welt::aus_bytes(
        &std::fs::read(root.join(format!("checkpoints/{index:08}.bin")))
            .map_err(|e| e.to_string())?,
    )?;
    let manifest = json!({"dataset_version":2,"run_id":identity["run_id"],"source_run_sha256":hash(&source),"split_unit":"whole_run_and_seed_family",
        "rules_contract":config.rules_contract(),"comparison_valid":quality["comparison_valid"],"real_time_target_met":quality["real_time_target_met"],
        "finished_epoch":world.beendet(),"inference":if config.models.values().all(|m|m.kind=="mock"){"mock_control"}else{"configured_models"},
        "evaluation_inputs":"player observations only; subsequent labels must not be joined as prior knowledge",
        "files":files,"audit":integrity});
    std::fs::write(
        out.join("dataset.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .map_err(|e| e.to_string())?;
    Ok(manifest)
}
