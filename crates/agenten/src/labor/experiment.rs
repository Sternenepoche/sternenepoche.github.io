//! Matched seed families and complete seat rotations. A matrix contains independent runs.
use super::{config::Config, Result};
use crate::journal::{hash, Journal};
use serde_json::{json, Value};
use std::{fs, path::Path};

/// Every initial model/harness assignment occupies every seat once for each seed.
/// The family, not an individual player or rotation, is the train/evaluation split unit.
pub fn matrix(
    c: &Config,
    out: &Path,
    seeds: &[u64],
    windows: usize,
    execute: bool,
) -> Result<Value> {
    c.validate()?;
    if c.previous_epoch.is_some() {
        return Err("Matrix-Fortsetzung braucht eine explizite Identitätszuordnung historischer Spieler zu rotierten Sitzen; bisher nur unabhängige cold_start/shared_learning Matrizen".into());
    }
    if seeds.is_empty() || seeds.len() > 100 || windows == 0 {
        return Err("1..100 eindeutige Seeds und positive Fensterzahl erforderlich".into());
    }
    let unique: std::collections::BTreeSet<_> = seeds.iter().collect();
    if unique.len() != seeds.len() {
        return Err("Doppelte Matrix-Seeds".into());
    }
    let contract = json!({"matrix_version":1,"config":c,"seeds":seeds,"target_windows":windows,"rotation":"all seats"});
    let journal = Journal::open(out, &contract)?;
    let family = hash(contract.to_string().as_bytes());
    let mut rows = vec![];
    for seed in seeds {
        for rotation in 0..c.players.len() {
            let mut config = c.clone();
            config.seed = *seed;
            config.players.rotate_left(rotation);
            config.primary_models.clear();
            for sid in 0..c.players.len() {
                if let Some(model) = c
                    .primary_models
                    .get(&(((sid + rotation) % c.players.len()) as u16))
                {
                    config.primary_models.insert(sid as u16, model.clone());
                }
            }
            let name = format!("seed-{seed}-rotation-{rotation:03}");
            let root = out.join(&name);
            let already = if root.join("checkpoints").exists() {
                super::runtime::checkpoint_manifests(&root)?
                    .last()
                    .map(|(i, _)| *i)
                    .unwrap_or(0)
            } else {
                0
            };
            if already > windows {
                return Err("Matrix-Lauf überschreitet den vereinbarten Zielumfang".into());
            }
            if already < windows {
                super::run(&config, &root, windows - already, execute)?;
            }
            let status = super::runtime::status(&root)?;
            let cp = super::runtime::checkpoint_manifests(&root)?;
            let index = cp.last().ok_or("Matrix-Checkpoint fehlt")?.0;
            let world = kern::Welt::aus_bytes(
                &fs::read(root.join(format!("checkpoints/{index:08}.bin")))
                    .map_err(|e| e.to_string())?,
            )?;
            let identity: Value = serde_json::from_slice(
                &fs::read(root.join("identity.json")).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
            let row = json!({"family_id":family,"seed":seed,"rotation":rotation,"run_id":identity["run_id"],"path":name,
            "seat_to_initial_setup":(0..c.players.len()).map(|seat|(seat+rotation)%c.players.len()).collect::<Vec<_>>(),
            "finished":world.beendet(),"sim_time":world.zeit,"ranking":world.rangliste(),"metrics":status["metrics"],
            "comparison_valid":status["comparison_valid"],
            "real_time_target_met":status["real_time_target_met"],
            "players":world.spieler.iter().map(|p|json!({"player_id":p.id,"initial_setup":(p.id as usize+rotation)%c.players.len(),
                "initial_harness_sha256":hash(serde_json::to_string(&config.players[p.id as usize]).unwrap().as_bytes()),
                "rank":p.rang,"points":p.punkte.gesamt(),"economy":p.punkte.wirtschaft,"research":p.punkte.forschung,
                "military":p.punkte.militaer,"civilization":p.punkte.zivilisation})).collect::<Vec<_>>(),
            "inference":if c.models.values().all(|m|m.kind=="mock"){"mock_control"}else{"configured_models"}});
            journal.put(
                &format!("{name}.result.json"),
                &serde_json::to_vec(&row).unwrap(),
            )?;
            rows.push(row);
        }
    }
    let result = json!({"family_id":family,"runs":rows,"split_unit":"entire matrix family",
        "interpretation":"Intermediate scores are not wins; use finished epochs and repeated seed families. Mock controls measure infrastructure only."});
    journal.put("results.json", &serde_json::to_vec_pretty(&result).unwrap())?;
    Ok(result)
}

/// Build a shared package from explicit concept text and a list of verified source runs.
/// Never turns raw private trajectories into knowledge automatically.
pub fn package(concepts_path: &Path, sources: &[String], out: &Path) -> Result<Value> {
    if out.exists() || sources.is_empty() {
        return Err("Neues Paket-Ziel und mindestens ein Herkunftslauf erforderlich".into());
    }
    let concepts: Vec<Value> =
        serde_json::from_slice(&fs::read(concepts_path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    if concepts.is_empty() || concepts.len() > 100 {
        return Err("1..100 ausdrücklich formulierte Konzepte erforderlich".into());
    }
    let mut runs = vec![];
    let mut rules_contract = None;
    for path in sources {
        let root = Path::new(path);
        let audit = super::audit::verify(root)?;
        let identity: Value = serde_json::from_slice(
            &fs::read(root.join("identity.json")).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        let manifest: Value = serde_json::from_slice(
            &fs::read(root.join("manifest.json")).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        if manifest["rules_sha256"] != hash(crate::RULES.as_bytes()) {
            return Err("Herkunft nutzt andere Regeln".into());
        }
        let source_config: Config =
            serde_json::from_value(manifest["config"].clone()).map_err(|e| e.to_string())?;
        let contract = source_config.rules_contract();
        if rules_contract.as_ref().is_some_and(|old| old != &contract) {
            return Err("Lernpaket darf unterschiedliche Regelversionen nicht vermischen".into());
        }
        rules_contract = Some(contract);
        let status = super::runtime::status(root)?;
        if status["comparison_valid"] != true {
            return Err("Beeinträchtigter oder unvollständig gespeicherter Lauf ist keine vergleichbare Lernquelle".into());
        }
        let cp = super::runtime::checkpoint_manifests(root)?;
        let index = cp.last().ok_or("Quellcheckpoint fehlt")?.0;
        let world = kern::Welt::aus_bytes(
            &fs::read(root.join(format!("checkpoints/{index:08}.bin")))
                .map_err(|e| e.to_string())?,
        )?;
        runs.push(json!({"run_id":identity["run_id"],"seed":manifest["config"]["seed"],"manifest_sha256":hash(manifest.to_string().as_bytes()),"audit_hash":audit["last_hash"],
            "comparison_valid":status["comparison_valid"],"real_time_target_met":status["real_time_target_met"],
            "finished_epoch":world.beendet(),"inference":if source_config.models.values().all(|m|m.kind=="mock"){"mock_control"}else{"configured_models"}}));
    }
    let mut ids = std::collections::BTreeSet::new();
    for concept in &concepts {
        let id = concept["id"].as_str().ok_or("Konzept-ID fehlt")?;
        if !super::config::identifier(id)
            || !ids.insert(id)
            || concept["text"].as_str().is_none_or(|s| s.is_empty())
            || concept.to_string().len() > 32768
        {
            return Err("Ungültiges/doppeltes Konzept oder Text >32KiB".into());
        }
        let evidence = concept["evidence"]
            .as_array()
            .filter(|a| !a.is_empty())
            .ok_or("Konzept benötigt Belege")?;
        for e in evidence {
            if !runs.iter().any(|r| e["run_id"] == r["run_id"]) {
                return Err("Konzeptbeleg gehört zu keinem verifizierten Herkunftslauf".into());
            }
        }
    }
    let result = json!({"version":2,"scope":"shared_concepts","rules_sha256":hash(crate::RULES.as_bytes()),"rules_contract":rules_contract,"source_runs":runs,
        "concepts":concepts,"review":"Explicit candidate concepts, not automatically proven causal findings. Evaluate on held-out seeds."});
    let bytes = serde_json::to_vec_pretty(&result).unwrap();
    fs::write(out, &bytes).map_err(|e| e.to_string())?;
    Ok(json!({"path":out,"sha256":hash(&bytes),"concepts":concepts.len()}))
}
