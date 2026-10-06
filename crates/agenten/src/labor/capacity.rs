//! Reusable admission planning from verified calls; no synthetic throughput promises.
use super::{
    broker::Reply,
    config::{Config, Model},
    Result,
};
use crate::journal::hash;
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

fn identity(m: &Model) -> String {
    json!({"kind":m.kind,"model":m.model,"url":m.url,"digest":m.digest,
        "context":m.context,"cpu_only":m.cpu_only,"thinking":m.thinking,"provider":m.provider})
    .to_string()
}
#[derive(Default)]
struct Evidence {
    samples: Vec<Sample>,
    runs: BTreeSet<String>,
    valid_tools: usize,
    tool_errors: usize,
    memory_writes: usize,
    prepared_actions: usize,
}
struct Sample {
    input: u64,
    output: u64,
    prompt_ms: f64,
    generation_ms: f64,
    cold_ms: u64,
    tools: usize,
    truncated: bool,
}

/// An estimate uses the entire reserved context and output budget, rather than extrapolating a tiny answer.
/// Serial inference is the conservative common denominator for local, mixed and remote configurations.
pub fn plan(config: &Config, sources: &[String]) -> Result<Value> {
    config.validate()?;
    if config.version < 4 || sources.is_empty() {
        return Err(
            "Kapazitätsplanung benötigt V4 und mindestens einen verifizierten Herkunftslauf".into(),
        );
    }
    let mut evidence = BTreeMap::<String, Evidence>::new();
    let mut source_refs = vec![];
    let mut seen = BTreeSet::new();
    for source in sources {
        let root = Path::new(source);
        let status = super::runtime::status(root)?;
        let manifest = fs::read(root.join("manifest.json")).map_err(|e| e.to_string())?;
        let run: Value = serde_json::from_slice(
            &fs::read(root.join("identity.json")).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        let id = run["run_id"].as_str().ok_or("Lauf-ID fehlt")?.to_string();
        if !seen.insert(id.clone()) {
            continue;
        }
        let usable = status["comparison_valid"] == true;
        source_refs.push(json!({"path":source,"run_id":id,"audit_hash":status["audit"]["last_hash"],
            "manifest_sha256":hash(&manifest),"usable":usable,
            "reason":if usable{"verified committed calls"}else{"pending or impaired comparison excluded"}}));
        if !usable {
            continue;
        }
        // Only calls linked from committed, verified events can contribute evidence.
        for (index, _) in super::runtime::checkpoint_manifests(root)?
            .into_iter()
            .filter(|(i, _)| *i > 0)
        {
            let events: Vec<Value> = serde_json::from_slice(
                &fs::read(root.join(format!("events/{:08}.json", index - 1)))
                    .map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
            for event in events
                .iter()
                .filter(|e| e["event_type"] == "call.completed")
            {
                let key = event["payload"]["call_id"]
                    .as_str()
                    .ok_or("Call-ID fehlt")?;
                let request: Value = serde_json::from_slice(
                    &fs::read(root.join(format!("calls/{key}.request.json")))
                        .map_err(|e| e.to_string())?,
                )
                .map_err(|e| e.to_string())?;
                let model: Model =
                    serde_json::from_value(request["model"].clone()).map_err(|e| e.to_string())?;
                let reply: Reply = serde_json::from_slice(
                    &fs::read(root.join(format!("calls/{key}.response.json")))
                        .map_err(|e| e.to_string())?,
                )
                .map_err(|e| e.to_string())?;
                if !["ollama", "openrouter"].contains(&model.kind.as_str())
                    || reply.failure.is_some()
                    || (model.kind == "ollama" && reply.usage["local_inference"] != true)
                {
                    continue;
                }
                let input = reply.usage["input_tokens"]
                    .as_u64()
                    .or_else(|| reply.usage["prompt_tokens"].as_u64())
                    .unwrap_or(0);
                let output = reply.usage["output_tokens"]
                    .as_u64()
                    .or_else(|| reply.usage["completion_tokens"].as_u64())
                    .unwrap_or(0);
                // Remote APIs have no separate timings: charge full observed wall time to
                // both reserved token components, conservatively including network latency.
                let generation_ms = if model.kind == "openrouter" {
                    reply.elapsed_ms as f64
                } else {
                    reply.usage["eval_duration_ns"].as_u64().unwrap_or(0) as f64 / 1_000_000.
                };
                if input == 0 || output == 0 || generation_ms <= 0. || reply.elapsed_ms == 0 {
                    continue;
                }
                let cold_ms = reply.usage["load_duration_ns"].as_u64().unwrap_or(0) / 1_000_000;
                // Old runs lack prompt duration; remaining wall time is a conservative surrogate.
                let prompt_ms = if model.kind == "openrouter" {
                    reply.elapsed_ms as f64
                } else {
                    reply.usage["prompt_eval_duration_ns"]
                        .as_u64()
                        .map(|n| n as f64 / 1_000_000.)
                        .unwrap_or(
                            (reply.elapsed_ms.saturating_sub(cold_ms) as f64 - generation_ms)
                                .max(1.),
                        )
                };
                let e = evidence.entry(identity(&model)).or_default();
                e.runs.insert(id.clone());
                for tool in events.iter().filter(|t| {
                    t["event_type"] == "tool.completed" && t["payload"]["call_id"] == key
                }) {
                    let p = &tool["payload"];
                    if p["result"].get("error").is_some() {
                        e.tool_errors += 1;
                    } else {
                        e.valid_tools += 1;
                        if p["name"] == "memory_write" {
                            e.memory_writes += 1;
                        }
                        if p["result"]["status"] == "prepared" {
                            e.prepared_actions += 1;
                        }
                    }
                }
                e.samples.push(Sample {
                    input,
                    output,
                    prompt_ms,
                    generation_ms,
                    cold_ms,
                    tools: reply.calls.len(),
                    truncated: reply.usage["output_truncated"] == true,
                });
            }
        }
    }
    let available_ms = config.window_seconds.saturating_mul(800); // 20% for variation and office work.
    let mut profiles = vec![];
    let mut per_player = vec![];
    let mut used = BTreeSet::new();
    let mut all_measured = true;
    for (sid, h) in config.players.iter().enumerate() {
        let roles =
            if config.coordinated_roles && !config.primary_models.contains_key(&(sid as u16)) {
                h.roles.clone()
            } else {
                config.execution_roles(sid as u16, h)
            };
        let mut worst = 0u64;
        for r in roles {
            let model = &config.models[&r.model];
            if model.context <= config.budget.output_tokens {
                return Err(
                    "Ausgabebudget lässt keinen Eingabekontext für den Spieler übrig".into(),
                );
            }
            let e = evidence.get(&identity(model));
            let adequate = e.is_some_and(|e| e.samples.len() >= 4);
            all_measured &= adequate;
            let input_reserved = model.context.saturating_sub(config.budget.output_tokens);
            let ms = e
                .map(|e| {
                    e.samples
                        .iter()
                        .map(|s| {
                            let prompt = s.prompt_ms / s.input as f64 * input_reserved as f64;
                            let generation = s.generation_ms / s.output as f64
                                * config.budget.output_tokens as f64;
                            (prompt + generation).ceil().max(1.) as u64
                        })
                        .max()
                        .unwrap_or(0)
                })
                .unwrap_or(0);
            worst = worst.max(ms);
            if used.insert(r.model.clone()) {
                profiles.push(json!({"alias":r.model,"model":model.model,"digest":model.digest,"context":model.context,
                    "cpu_only":model.cpu_only,"thinking":model.thinking,"provider":model.kind,
                    "samples":e.map(|e|e.samples.len()).unwrap_or(0),"source_runs":e.map(|e|&e.runs),
                    "adequately_sampled":adequate,"reserved_input_tokens":input_reserved,"reserved_output_tokens":config.budget.output_tokens,
                    "conservative_warm_call_ms":if adequate{Some(ms)}else{None},
                    "cold_load_ms":e.map(|e|e.samples.iter().map(|s|s.cold_ms).max().unwrap_or(0)),
                    "tool_calls":e.map(|e|e.samples.iter().map(|s|s.tools).sum::<usize>()).unwrap_or(0),
                    "valid_tools":e.map(|e|e.valid_tools).unwrap_or(0),
                    "tool_errors":e.map(|e|e.tool_errors).unwrap_or(0),
                    "memory_writes":e.map(|e|e.memory_writes).unwrap_or(0),
                    "prepared_actions":e.map(|e|e.prepared_actions).unwrap_or(0),
                    "truncated_samples":e.map(|e|e.samples.iter().filter(|s|s.truncated).count()).unwrap_or(0)}));
            }
        }
        per_player.push(worst);
    }
    // Adaptive role/model changes cannot silently inherit evidence for a cheaper model.
    if config.allow_model_switch {
        all_measured = false;
    }
    let cold_per_round = profiles
        .iter()
        .map(|p| p["cold_load_ms"].as_u64().unwrap_or(0))
        .sum::<u64>();
    let one_round = per_player
        .iter()
        .copied()
        .fold(cold_per_round, u64::saturating_add);
    let requested_ms = one_round.saturating_mul(config.budget.calls_per_window as u64);
    let recommended_calls = if all_measured && one_round > 0 {
        (available_ms / one_round).min(config.budget.calls_per_window as u64)
    } else {
        0
    };
    let feasible = all_measured && requested_ms <= available_ms;
    Ok(
        json!({"plan_version":1,"config_sha256":hash(serde_json::to_string(config).unwrap().as_bytes()),
        "rules_contract":config.rules_contract(),"players":config.players.len(),"window_seconds":config.window_seconds,
        "reserved_time_ms":config.window_seconds.saturating_mul(1000).saturating_sub(available_ms),
        "requested_calls_per_player":config.budget.calls_per_window,"profiles":profiles,"sources":source_refs,
        "coverage_complete":all_measured,"estimated_serial_ms":if all_measured{Some(requested_ms)}else{None},
        "estimated_feasible":feasible,"largest_equal_calls_per_player":recommended_calls,
        "status":if !all_measured{"insufficient_evidence"}else if feasible{"fits_estimate"}else{"reduce_equal_budget_or_players"},
        "assumptions":["Full reserved input context and output tokens; worst observed per-token time, not tiny-answer extrapolation.",
            "Serial calls; full cold-load cost per model per round, even if weights remain resident.",
            "At least four real calls per exact model/digest/context/CPU/thinking/provider identity; mocks never measure throughput. Remote calls charge full wall time to both token components.",
            "Hardware contention, context-dependent slowdown and unseen workloads can exceed this estimate. Runtime fairness barrier remains authoritative.",
            "Every model needs its own evidence; local_only never uses remote fallback. Timing estimates and strategic quality are separate."],
        "strategic_quality_proven":false}),
    )
}

/// Change one shared call budget before a NEW match, never during an ongoing comparison.
pub fn fit(config: &Config, sources: &[String], out: &Path) -> Result<Value> {
    if out.exists() {
        return Err("Zielkonfiguration existiert bereits".into());
    }
    let report = plan(config, sources)?;
    let calls = report["largest_equal_calls_per_player"]
        .as_u64()
        .unwrap_or(0);
    if calls == 0 {
        return Err(format!(
            "Kein belegbares gemeinsames Budget für diese Spielerzahl: {}",
            report["status"]
        ));
    }
    let mut fitted = config.clone();
    fitted.budget.calls_per_window = calls as u32;
    fitted.validate()?;
    fs::write(out, serde_json::to_vec_pretty(&fitted).unwrap()).map_err(|e| e.to_string())?;
    Ok(json!({"config":out,"calls_per_player":calls,"plan":report,"new_match_required":true}))
}
