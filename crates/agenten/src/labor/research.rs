//! Crossed initial model/harness assignments, complete seat rotations, seed-level summaries.
use super::{audit, config::{Config, Harness}, experiment, runtime, Result};
use crate::journal::{hash, Journal};
use serde_json::{json, Value};
use std::{collections::{BTreeMap, BTreeSet}, fs, path::Path};

#[derive(Clone, Default)]
struct Metrics {
    counts: BTreeMap<String,u64>,
    known_micro_usd:u64,
    unknown_cost_calls:u64,
    input_tokens:u64,
    output_tokens:u64,
    elapsed_ms:u64,
}
impl Metrics {
    fn count(&mut self,key:&str) {*self.counts.entry(key.into()).or_default()+=1;}
    fn event(&mut self,e:&Value) {
        let typ=e["event_type"].as_str().unwrap_or("unknown");let p=&e["payload"];
        self.count(typ);
        match typ {
            "call.completed"=>{
                self.input_tokens+=p["usage"]["input_tokens"].as_u64().or(p["usage"]["prompt_tokens"].as_u64()).unwrap_or(0);
                self.output_tokens+=p["usage"]["output_tokens"].as_u64().or(p["usage"]["completion_tokens"].as_u64()).unwrap_or(0);
                self.elapsed_ms+=p["elapsed_ms"].as_u64().unwrap_or(0);
                if let Some(cost)=p["cost_micro_usd"].as_u64() {self.known_micro_usd+=cost;}else{self.unknown_cost_calls+=1;}
                if p["usage"]["output_truncated"]==true {self.count("call.truncated");}
            }
            "call.failed"=>{self.count("errors");self.unknown_cost_calls+=1;}
            "tool.completed"=>{
                let failed=!p["result"]["error"].is_null();if failed {self.count("tool.error");self.count("errors");}
                let name=p["name"].as_str().unwrap_or("");
                if name.starts_with("memory_") {self.count("memory.calls");if !failed {self.count(&format!("memory.success.{name}"));}}
                if name=="memory_write" && !failed {self.count("memory.writes");}
            }
            "action.rejected"|"action.skipped"=>self.count("errors"),
            _=>{}
        }
    }
    fn value(&self)->Value {json!({"event_counts":self.counts,"known_cost_micro_usd":self.known_micro_usd,
        "unknown_cost_calls":self.unknown_cost_calls,"all_completed_call_costs_known":self.unknown_cost_calls==0,
        "input_tokens":self.input_tokens,"output_tokens":self.output_tokens,"summed_call_ms":self.elapsed_ms,
        "cost_scope":"Provider-reported completed calls; electricity, hardware and failed calls are not inferred."})}
}

#[derive(Clone)]
struct Sample {seat:usize,points:i64,finished:bool,valid:bool,real_time:bool}
#[derive(Clone)]
struct Cell {model:String,harness:String}

fn uncertainty(values:&[f64])->Value {
    let n=values.len();let mean=values.iter().sum::<f64>()/n as f64;
    let variance=if n>1 {Some(values.iter().map(|x|(x-mean).powi(2)).sum::<f64>()/(n-1) as f64)}else{None};
    json!({"independent_seed_families":n,"mean_of_seed_means":mean,
        "sample_standard_deviation":variance.map(f64::sqrt),"standard_error":variance.map(|v|(v/n as f64).sqrt()),
        "uncertainty_unit":"independent seed families; seats and rotations are repeated measurements, never independent samples",
        "interpretation":if n<5 {"Descriptive intermediate result; too few seeds for a superiority claim."}else{"Descriptive seed-level uncertainty; no significance or superiority claim is made."}})
}
fn marginal(cells:&[Cell],means:&BTreeMap<usize,BTreeMap<u64,f64>>,seeds:&[u64],model:bool)->Value {
    let labels=cells.iter().map(|c|if model{&c.model}else{&c.harness}).collect::<BTreeSet<_>>();
    json!(labels.into_iter().map(|label| {
        let indices=cells.iter().enumerate().filter(|(_,c)|if model{&c.model==label}else{&c.harness==label}).map(|(i,_)|i).collect::<Vec<_>>();
        let values=seeds.iter().map(|seed|indices.iter().map(|i|means[i][seed]).sum::<f64>()/indices.len() as f64).collect::<Vec<_>>();
        json!({"level":label,"crossed_cells":indices.len(),"seed_means":seeds.iter().zip(&values).map(|(seed,mean)|json!({"seed":seed,"mean_points":mean})).collect::<Vec<_>>(),"summary":uncertainty(&values),"claim":"descriptive marginal mean; not a causal main-effect proof"})
    }).collect::<Vec<_>>())
}

/// Every selected model is crossed with every supplied initial organization.
/// Results are descriptive; short epochs and mock controls never become wins.
pub fn factorial(config:&Config,harnesses:&BTreeMap<String,Harness>,model_aliases:&[String],out:&Path,seeds:&[u64],windows:usize,execute:bool)->Result<Value> {
    if !["cold_start","shared_learning"].contains(&config.mode.as_str()) || config.previous_epoch.is_some() {return Err("Faktoriell nur unabhängiger cold_start oder gemeinsames shared_learning".into());}
    if seeds.is_empty() || seeds.len()>100 || seeds.iter().collect::<BTreeSet<_>>().len()!=seeds.len() || windows==0 {return Err("1..100 eindeutige Seeds und positive Fensterzahl erforderlich".into());}
    if model_aliases.is_empty() || model_aliases.iter().collect::<BTreeSet<_>>().len()!=model_aliases.len() || model_aliases.iter().any(|m|!config.models.contains_key(m)) {return Err("Eindeutige bekannte Modellaliase erforderlich".into());}
    let count=model_aliases.len().checked_mul(harnesses.len()).ok_or("Zu viele Faktorstufen")?;
    if !(2..=50).contains(&count) {return Err("Modell×Harness braucht 2..50 Teilnehmer".into());}
    if harnesses.keys().any(|name|!super::config::identifier(name)) {return Err("Ungültiger Harness-Variantenname".into());}
    let mut rights=None;
    for h in harnesses.values() {
        let union=h.roles.iter().flat_map(|r|r.capabilities.iter().cloned()).collect::<BTreeSet<_>>();
        if rights.as_ref().is_some_and(|old|old!=&union) {return Err("Alle Harness-Varianten benötigen dieselbe Capability-Union".into());}rights=Some(union);
    }
    let mut c=config.clone();c.version=4;c.coordinated_roles=true;c.allow_model_switch=false;c.players.clear();c.primary_models.clear();
    if c.window_seconds==0 {c.window_seconds=900;}
    if c.deadline_policy.is_empty() {c.deadline_policy="controlled".into();}
    if c.max_window_slices==0 {c.max_window_slices=4;}
    if c.provider_mode.is_empty() {c.provider_mode=if c.models.values().all(|m|m.kind=="mock"){"mock"}else if c.models.values().all(|m|m.kind=="ollama"){"local_only"}else{"mixed"}.into();}
    let mut cells=Vec::new();
    for model in model_aliases {for (name,h) in harnesses {
        let mut h=h.clone();for role in &mut h.roles {role.model=model.clone();}
        c.primary_models.insert(c.players.len() as u16,model.clone());c.players.push(h);cells.push(Cell{model:model.clone(),harness:name.clone()});
    }}
    c.validate()?;
    let contract=json!({"factorial_version":1,"config":c,"harness_variants":harnesses,"model_aliases":model_aliases,"seeds":seeds,"windows":windows,"execute":execute});
    let journal=Journal::open(out,&contract)?;let matrix_root=out.join("matrix");
    let matrix=experiment::matrix(&c,&matrix_root,seeds,windows,execute)?;
    let mut samples=BTreeMap::<(usize,u64),Vec<Sample>>::new();let mut metrics=vec![Metrics::default();count];let mut sources=Vec::new();
    let mut rotations=BTreeSet::new();
    for run in matrix["runs"].as_array().ok_or("Matrix-Läufe fehlen")? {
        let seed=run["seed"].as_u64().ok_or("Seed fehlt")?;let rotation=run["rotation"].as_u64().ok_or("Rotation fehlt")? as usize;
        if !seeds.contains(&seed) || rotation>=count || !rotations.insert((seed,rotation)) {return Err("Unerwartete/doppelte Matrix-Rotation".into());}
        let path=run["path"].as_str().ok_or("Laufpfad fehlt")?;
        if path!=format!("seed-{seed}-rotation-{rotation:03}") {return Err("Matrix-Laufpfad passt nicht zur Rotation".into());}
        let root=matrix_root.join(path);let verified=audit::verify(&root)?;
        let identity:Value=serde_json::from_slice(&fs::read(root.join("identity.json")).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
        if identity["run_id"]!=run["run_id"] {return Err("Audit-Laufidentität stimmt nicht mit Matrix überein".into());}
        let seats=run["seat_to_initial_setup"].as_array().filter(|a|a.len()==count).ok_or("Sitzzuordnung unvollständig")?;
        for (seat,initial) in seats.iter().enumerate() {if initial.as_u64()!=Some(((seat+rotation)%count) as u64){return Err("Falsche Sitzrotation".into());}}
        let cp=runtime::checkpoint_manifests(&root)?;
        for (index,_) in cp.iter().filter(|(i,_)|*i>0) {
            let events:Vec<Value>=serde_json::from_slice(&fs::read(root.join(format!("events/{:08}.json",index-1))).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
            for event in events {if let Some(sid)=event["player_id"].as_u64() {
                let initial=seats.get(sid as usize).and_then(Value::as_u64).ok_or("Audit-Spieler außerhalb der Sitzzuordnung")? as usize;
                metrics[initial].event(&event);
            }}
        }
        let players=run["players"].as_array().filter(|a|a.len()==count).ok_or("Matrix-Spielerliste unvollständig")?;let mut seen=BTreeSet::new();
        for player in players {
            let seat=player["player_id"].as_u64().ok_or("Spielersitz fehlt")? as usize;
            if seat>=count || !seen.insert(seat) {return Err("Unerwarteter/doppelter Spielersitz".into());}
            let initial=seats[seat].as_u64().ok_or("Setup-Zuordnung fehlt")? as usize;
            if player["initial_setup"].as_u64()!=Some(initial as u64){return Err("Setup-Zuordnungen widersprechen sich".into());}
            samples.entry((initial,seed)).or_default().push(Sample{seat,points:player["points"].as_i64().ok_or("Punkte fehlen")?,finished:run["finished"]==true,valid:run["comparison_valid"]==true,real_time:run["real_time_target_met"]==true});
        }
        sources.push(json!({"path":format!("matrix/{path}"),"seed":seed,"rotation":rotation,"run_id":identity["run_id"],
            "audit":verified,"manifest_sha256":hash(&fs::read(root.join("manifest.json")).map_err(|e|e.to_string())?),
            "finished":run["finished"],"comparison_valid":run["comparison_valid"],"real_time_target_met":run["real_time_target_met"]}));
    }
    if rotations.len()!=count*seeds.len(){return Err("Matrix-Sitzrotation nicht vollständig".into());}
    let mut setups=Vec::new();let mut means=BTreeMap::new();
    for (index,cell) in cells.iter().enumerate() {
        let mock=c.models[&cell.model].kind=="mock";let mut family_means=BTreeMap::new();let mut families=Vec::new();let mut eligible=!mock;
        for seed in seeds {
            let rows=samples.get(&(index,*seed)).ok_or("Seedfamilie fehlt")?;let seats=rows.iter().map(|r|r.seat).collect::<BTreeSet<_>>();
            if rows.len()!=count || seats.len()!=count {return Err("Setup durchlief nicht alle Sitze genau einmal".into());}
            let mean=rows.iter().map(|r|r.points as f64).sum::<f64>()/rows.len() as f64;family_means.insert(*seed,mean);
            let valid=rows.iter().all(|r|r.valid);let finished=rows.iter().all(|r|r.finished);let real_time=rows.iter().all(|r|r.real_time);eligible &= valid && finished;
            families.push(json!({"seed":seed,"mean_points_all_seats":mean,"seat_observations":count,"seats":seats,"all_finished":finished,"all_comparison_valid":valid,"all_real_time_targets_met":real_time}));
        }
        let values=seeds.iter().map(|seed|family_means[seed]).collect::<Vec<_>>();means.insert(index,family_means);
        setups.push(json!({"initial_setup":index,"model":cell.model,"harness":cell.harness,"inference":if mock{"mock_control"}else{"configured_model"},
            "seed_families":families,"summary":uncertainty(&values),"audit_metrics":metrics[index].value(),
            "finished_real_valid_data":eligible,"win_claim":Value::Null,"superiority_claim":Value::Null,
            "score_label":if mock{"infrastructure_control"}else if eligible{"finished_epoch_descriptive_points"}else{"intermediate_or_impaired_points"}}));
    }
    let report=json!({"format":"labor-factorial-v1","labor_version":4,"mode":c.mode,"participants":count,"models":model_aliases,
        "harness_variants":harnesses.keys().collect::<Vec<_>>(),"capability_union":rights,"model_switch_locked":true,
        "target_windows":windows,"independent_seeds":seeds,"runs":sources.len(),"setups":setups,
        "model_marginals":marginal(&cells,&means,seeds,true),"harness_marginals":marginal(&cells,&means,seeds,false),
        "source_trace":sources,"matrix_results_sha256":hash(&fs::read(matrix_root.join("results.json")).map_err(|e|e.to_string())?),
        "claims":{"wins":false,"superiority":false,"causal_main_effects":false},
        "interpretation":"All scores are descriptive. Independent uncertainty uses seed means after complete seat rotation. Mock, unfinished or invalid runs are never wins; shared seed families must remain together in learning/evaluation splits."});
    journal.put("report.json",&serde_json::to_vec_pretty(&report).map_err(|e|e.to_string())?)?;
    Ok(report)
}
