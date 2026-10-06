use kern::{typen::*, welt::Erkundung};
use serde_json::{json, Value};
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use sternenepoche_agenten::labor::{self, config::Config};

fn dir(label: &str) -> PathBuf {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../laeufe/labor-tests")
        .join(format!(
            "v4-{label}-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
    fs::create_dir_all(&p).unwrap();
    p
}

#[test]
fn compact_maps_keep_equal_homes_and_competing_expansion() {
    for n in [2, 10, 50] {
        let (w, map) = labor::world::create(&Config::v4(n)).unwrap();
        assert_eq!(w.systeme.len(), n / 2);
        assert_eq!(w.plaetze.len() - n, 5 * n);
        assert_eq!(map["generator"], "paired-contested-v2");
        assert!(w.kolonisationsregeln_v2());
        let first = &w.planeten[w.spieler[0].heimat as usize];
        for p in &w.planeten {
            assert_eq!(p.bestand, first.bestand);
            assert_eq!(p.gebaeude, first.gebaeude);
            assert_eq!(p.faktor, first.faktor);
        }
        for s in &w.systeme {
            assert_eq!(
                w.planeten
                    .iter()
                    .filter(|p| p.koord.sektor == s.sektor && p.koord.system == s.nummer)
                    .count(),
                2
            );
            assert!(s.nebel); // Every pair can access the late-game resource chain.
        }
    }
}

#[test]
fn colony_plan_is_private_read_only_and_separates_founding_from_supply() {
    let (mut w, _) = labor::world::create(&Config::v4(2)).unwrap();
    let pid = w.spieler[0].heimat as usize;
    let start = w.planeten[pid].koord;
    let target = Koord::neu(start.sektor, start.system, 6);
    let sp = &mut w.spieler[0];
    sp.stufe = 4;
    sp.forschung[Forschung::Astrophysik.idx()] = 3;
    sp.forschung[Forschung::Computertechnik.idx()] = 3;
    sp.toepfe = [1_000_000 * M; TOEPFE];
    let p = &mut w.planeten[pid];
    p.bevoelkerung = 10000 * M;
    p.bestand = [100000 * M; GUETER];
    p.gebaeude[Gebaeude::Raumhafen.idx()] = 1;
    p.einheiten[Einheit::Kolonieschiff.idx()] = 1;
    p.einheiten[Einheit::Kreuzer.idx()] = 1;
    p.einheiten[Einheit::GrosserTransporter.idx()] = 20;
    let mut cargo = w.koloniefracht(0);
    cargo[Gut::Erz.idx()] += 2000 * M;
    cargo[Gut::Kristall.idx()] += 2000 * M;
    let cargo: std::collections::BTreeMap<_, _> = Gut::ALLE
        .iter()
        .filter(|g| cargo[g.idx()] > 0)
        .map(|g| (g.name(), cargo[g.idx()] as f64 / M as f64))
        .collect();
    let mut q = json!({"typ":"kolonieplan","start":start.to_string(),"ziel":target.to_string(),
        "schiffe":{"kolonieschiff":1,"kreuzer":1,"grosser_transporter":20},"ladung":cargo,"aufbau":[]});
    // Names come from the engine rather than aliases in a fixture.
    q["schiffe"] = json!(std::collections::BTreeMap::from([
        (Einheit::Kolonieschiff.name(), 1),
        (Einheit::Kreuzer.name(), 1),
        (Einheit::GrosserTransporter.name(), 20)
    ]));
    assert!(w.werkzeug(0, &q).unwrap_err().contains("Spionagesonde"));
    w.spieler[0].erkundet.insert(
        target,
        Erkundung {
            zeit: 0,
            felder: 230,
            zone: Zone::Leben,
            reich_erz: 1000,
            reich_kristall: 1000,
            nebel: true,
        },
    );
    let before = w.hash();
    let empty = w.werkzeug(0, &q).unwrap();
    assert_eq!(empty["start_moeglich"], true, "{empty}");
    assert_eq!(empty["versorgung_im_modell_gedeckt"], false);
    q["aufbau"] = json!(["solarkraftwerk", "farm", "farm"]);
    let planned = w.werkzeug(0, &q).unwrap();
    assert_eq!(planned["aufbau_vollstaendig"], true, "{planned}");
    assert_eq!(
        planned["grundversorgung_im_modell_gedeckt"], true,
        "{planned}"
    );
    assert_eq!(
        planned["versorgung_im_modell_gedeckt"], false,
        "Konsumgüter müssen separat gedeckt werden: {planned}"
    );
    assert!(planned["versorgungsabrisse"]
        .as_array()
        .unwrap()
        .iter()
        .any(|s| s["gut"] == "konsumgut"));
    assert_eq!(w.hash(), before);
    assert!(!w.belegung.contains_key(&target));
    assert!(w.werkzeug(1, &q).is_err());
}

#[test]
fn two_mock_windows_commit_replay_and_resume_without_inference() {
    let mut c = Config::v4(2);
    c.budget.calls_per_window = 2;
    let p = dir("pipeline");
    let out = labor::run(&c, &p, 2, false).unwrap();
    assert_eq!(out["calls"], 4);
    assert_eq!(labor::runtime::replay(&p).unwrap()["verified"], true);
    assert_eq!(
        labor::runtime::status(&p).unwrap()["comparison_valid"],
        true
    );
    labor::run(&c, &p, 1, false).unwrap();
    assert_eq!(labor::runtime::replay(&p).unwrap()["verified"], true);
    let events: Vec<Value> =
        serde_json::from_slice(&fs::read(p.join("events/00000000.json")).unwrap()).unwrap();
    assert!(events.iter().any(|e| e["event_type"] == "action.committed"));
}

#[test]
fn deadline_never_turns_missing_service_into_a_clean_loss() {
    for policy in ["controlled", "throughput"] {
        let mut c = Config::v4(2);
        c.deadline_policy = policy.into();
        c.max_window_slices = 1;
        let p = dir(policy);
        fs::create_dir_all(p.join("deadlines")).unwrap();
        fs::write(
            p.join("deadlines/00000000.json"),
            json!({"window":0,"deadline_unix_ms":1,"budget_seconds":900}).to_string(),
        )
        .unwrap();
        let result = labor::run(&c, &p, 1, false);
        if policy == "controlled" {
            assert!(result.unwrap_err().contains("Kapazitätsgrenze"));
            assert!(!p.join("checkpoints/00000001.bin").exists());
        } else {
            result.unwrap();
            assert_eq!(
                labor::runtime::status(&p).unwrap()["comparison_valid"],
                false
            );
        }
        assert!(!fs::read_dir(p.join("calls")).unwrap().any(|e| e
            .unwrap()
            .file_name()
            .to_string_lossy()
            .ends_with("sent.json")));
    }
}

#[test]
fn catch_up_reuses_completed_work_without_new_player_budgets() {
    let mut c = Config::v4(2);
    c.budget.calls_per_window = 2;
    c.parallel = 1;
    let p = dir("catch-up");
    // Crash after first durable reply represents a partially served, still frozen window.
    assert!(labor::runtime::run_with_fault(&c, &p, 1, false, Some((0, "after_response"))).is_err());
    let first = fs::read_dir(p.join("calls"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| p.to_string_lossy().ends_with("response.json"))
        .unwrap();
    let first_bytes = fs::read(&first).unwrap();
    // Deterministically model expiry without waiting 15 minutes.
    fs::write(
        p.join("deadlines/00000000.json"),
        json!({"window":0,"deadline_unix_ms":1,"budget_seconds":900}).to_string(),
    )
    .unwrap();
    let result = labor::run(&c, &p, 1, false).unwrap();
    assert_eq!(result["calls"], 4);
    assert_eq!(fs::read(&first).unwrap(), first_bytes);
    assert!(p.join("deadlines/00000000.slice-01.json").exists());
    let status = labor::runtime::status(&p).unwrap();
    assert_eq!(status["comparison_valid"], true);
    assert_eq!(status["real_time_target_met"], false);
    assert_eq!(labor::runtime::replay(&p).unwrap()["verified"], true);
    let events: Vec<Value> =
        serde_json::from_slice(&fs::read(p.join("events/00000000.json")).unwrap()).unwrap();
    for sid in 0..2 {
        assert_eq!(
            events
                .iter()
                .filter(|e| e["event_type"] == "call.completed" && e["player_id"] == sid)
                .count(),
            2
        );
        assert_eq!(
            events
                .iter()
                .filter(|e| e["event_type"] == "action.committed" && e["player_id"] == sid)
                .count(),
            1
        );
    }
    assert!(events
        .iter()
        .any(|e| e["event_type"] == "scheduler.deferred"));
}

#[test]
fn durable_window_completion_recovers_after_event_write_without_new_calls() {
    let mut c = Config::v4(2);
    c.budget.calls_per_window = 2;
    let p = dir("durable-completion");
    assert!(
        labor::runtime::run_with_fault(&c, &p, 1, false, Some((0, "after_events")))
            .unwrap_err()
            .contains("after durable events")
    );
    let events = fs::read(p.join("events/00000000.json")).unwrap();
    let completed = fs::read(p.join("deadlines/00000000.finished.json")).unwrap();
    assert_eq!(
        labor::runtime::status(&p).unwrap()["comparison_valid"],
        false
    );
    let result = labor::run(&c, &p, 1, false).unwrap();
    assert_eq!(result["protocol"], "persistent-players-v4");
    assert_eq!(result["calls"], 4);
    assert_eq!(result["comparison_valid"], true);
    assert_eq!(fs::read(p.join("events/00000000.json")).unwrap(), events);
    assert_eq!(
        fs::read(p.join("deadlines/00000000.finished.json")).unwrap(),
        completed
    );
    assert_eq!(labor::runtime::replay(&p).unwrap()["verified"], true);
    fs::write(p.join("deadlines/00000000.finished.json"), b"{}").unwrap();
    assert!(labor::audit::verify(&p)
        .unwrap_err()
        .contains("Fensterabschluss"));
}

#[test]
fn learning_versions_and_all_private_records_reach_the_actual_briefing() {
    let mut c = Config::v4(2);
    c.budget.calls_per_window = 1;
    c.window_seconds = 60;
    let source = dir("learning-source");
    let run = labor::run(&c, &source, 1, false).unwrap();
    assert_eq!(
        labor::runtime::capacity(&source).unwrap()["window_seconds"],
        60
    );
    let d = dir("learning-files");
    let concepts = d.join("concepts.json");
    let package = d.join("package.json");
    let mut entries=(0..35).map(|n|json!({"id":format!("a{n:03}"),"text":"Geprüfte Hypothese, kein strategischer Nachweis.","evidence":[{"run_id":run["run_id"]}]})).collect::<Vec<_>>();
    entries.push(json!({"id":"z-urgent","text":"Diese ältere dringende Erinnerung darf nicht hinter alphabetisch früheren Einträgen verschwinden.","priority":100,"due_at":0,"evidence":[{"run_id":run["run_id"]}]}));
    fs::write(&concepts, serde_json::to_vec(&entries).unwrap()).unwrap();
    let proof = labor::experiment::package(&concepts, &[source.to_string_lossy().into()], &package)
        .unwrap();
    let pack: Value = serde_json::from_slice(&fs::read(&package).unwrap()).unwrap();
    assert_eq!(pack["version"], 2);
    assert_eq!(pack["rules_contract"]["labor_version"], 4);
    assert_eq!(pack["source_runs"][0]["inference"], "mock_control");
    assert_eq!(pack["source_runs"][0]["finished_epoch"], false);
    c.seed += 1;
    c.mode = "shared_learning".into();
    c.learning_package = Some(package.to_string_lossy().into());
    c.learning_sha256 = Some(proof["sha256"].as_str().unwrap().into());
    let target = dir("learning-target");
    labor::run(&c, &target, 1, false).unwrap();
    let events: Vec<Value> =
        serde_json::from_slice(&fs::read(target.join("events/00000000.json")).unwrap()).unwrap();
    for event in events
        .iter()
        .filter(|e| e["event_type"] == "observation.delivered")
    {
        assert!(event["payload"]["memory"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["key"] == "z-urgent"));
    }
    let exported = labor::audit::export(&target, &d.join("export")).unwrap();
    assert_eq!(exported["rules_contract"]["labor_version"], 4);
    assert_eq!(exported["comparison_valid"], true);
    let mut old = Config::v3(2);
    old.mode = c.mode;
    old.seed = c.seed;
    old.learning_package = c.learning_package;
    old.learning_sha256 = c.learning_sha256;
    assert!(labor::run(&old, &dir("wrong-rules"), 1, false)
        .unwrap_err()
        .contains("Regelversion"));
}

#[test]
fn impaired_throughput_is_labeled_in_exports_and_rejected_as_learning_evidence() {
    let mut c = Config::v4(2);
    c.deadline_policy = "throughput".into();
    let p = dir("impaired-data");
    fs::create_dir_all(p.join("deadlines")).unwrap();
    fs::write(
        p.join("deadlines/00000000.json"),
        json!({"window":0,"deadline_unix_ms":1,"budget_seconds":900}).to_string(),
    )
    .unwrap();
    let result = labor::run(&c, &p, 1, false).unwrap();
    assert_eq!(result["comparison_valid"], false);
    let d = dir("impaired-export");
    let exported = labor::audit::export(&p, &d.join("data")).unwrap();
    assert_eq!(exported["comparison_valid"], false);
    assert_eq!(exported["real_time_target_met"], false);
    let concepts = d.join("concepts.json");
    fs::write(&concepts,json!([{"id":"wrong-conclusion","text":"Infrastrukturfehler dürfen keine Strategiehypothese bestätigen.","evidence":[{"run_id":result["run_id"]}]}]).to_string()).unwrap();
    assert!(labor::experiment::package(
        &concepts,
        &[p.to_string_lossy().into()],
        &d.join("rejected.json")
    )
    .unwrap_err()
    .contains("Beeinträchtigter"));
}

#[test]
fn bound_market_deliveries_are_visible_only_to_the_buyer_through_the_gateway() {
    use labor::{broker::ToolCall, gateway, memory::Memory};
    let c = Config::v4(2);
    let (mut w, _) = labor::world::create(&c).unwrap();
    let destination = w.spieler[1].heimat;
    w.plane(
        900,
        kern::welt::EreignisArt::MarktlieferungGebunden {
            planet: destination,
            empfaenger: 0,
            gut: Gut::Nahrung,
            menge: 50 * M,
        },
    );
    for sid in 0..2 {
        let m = Memory::new(sid, c.budget.memory_bytes, &c.players[sid as usize]).unwrap();
        let role = c.execution_roles(sid, &c.players[sid as usize])[0].clone();
        let query = ToolCall {
            id: "market-view".into(),
            name: "own_state".into(),
            arguments: json!({"section":"marktlieferungen"}),
        };
        let outcome = gateway::execute(&w, &m, &c, &role, &query).unwrap();
        assert_eq!(outcome.result["total"], if sid == 0 { 1 } else { 0 });
        let tools = gateway::tools_for(&c, &role, &["own_state".into()]);
        assert!(tools
            .iter()
            .find(|t| t["function"]["name"] == "own_state")
            .unwrap()["function"]["parameters"]["properties"]["section"]["enum"]
            .as_array()
            .unwrap()
            .contains(&json!("marktlieferungen")));
    }
}

#[test]
fn oversized_initial_briefing_cannot_silently_remove_a_reaction_opportunity() {
    for policy in ["controlled", "throughput"] {
        let mut c = Config::v4(2);
        c.deadline_policy = policy.into();
        c.budget.max_input_bytes = 4096;
        let root = dir("initial-context");
        let result = labor::run(&c, &root, 1, false);
        if policy == "controlled" {
            assert!(result.unwrap_err().contains("Anfangslagebild"));
            assert!(!root.join("checkpoints/00000001.json").exists());
            assert_eq!(
                labor::runtime::status(&root).unwrap()["comparison_valid"],
                false
            );
        } else {
            assert_eq!(result.unwrap()["comparison_valid"], false);
        }
    }
}
