use serde_json::{json, Value};
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use sternenepoche_agenten::labor::{self, config::Config, memory::Memory, world};

fn directory(name: &str) -> PathBuf {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../laeufe/labor-tests")
        .join(format!(
            "{name}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
    fs::create_dir_all(&p).unwrap();
    p
}
#[test]
fn maps_scale_and_every_player_has_a_close_peer_without_overlapping_starts() {
    for n in [2, 3, 10, 11, 20, 50] {
        for seed in 0..16 {
            let mut c = Config::demo(n);
            c.seed = seed;
            let (w, map) = world::create(&c).unwrap();
            assert_eq!(w.belegung.len(), n);
            assert!(w.systeme.len() >= n * 2);
            assert!(w.systeme.len() <= ((n * 12).div_ceil(5) + 1).max(6));
            for p in &w.planeten {
                assert!(w.system(p.koord).is_some());
                assert!(w.planeten.iter().any(|q| q.id != p.id
                    && q.koord.sektor == p.koord.sektor
                    && q.koord.system == p.koord.system));
                assert_eq!(p.bestand, w.planeten[0].bestand);
                assert_eq!(p.rate, w.planeten[0].rate);
            }
            let times = map["travel"]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| r["reference_travel_seconds"].as_i64().unwrap())
                .collect::<Vec<_>>();
            assert!(times.iter().all(|t| *t == times[0]));
        }
    }
}
#[test]
fn private_sqlite_roundtrip_detects_wrong_owner_and_stale_writes() {
    let c = Config::demo(2);
    let m = Memory::new(0, 65536, &c.players[0]).unwrap();
    m.put(
        "task",
        "supply",
        0,
        &json!({"goal":"feed colony","source":"sensor-7"}),
    )
    .unwrap();
    assert!(m.put("task", "supply", 0, &json!("overwrite")).is_err());
    assert!(m.put("task", "../other-player", 0, &json!("bad")).is_err());
    assert_eq!(m.search("' OR 1=1 --", None, 30).unwrap().len(), 0);
    let dir = directory("memory");
    m.save(&dir.join("player.sqlite")).unwrap();
    assert!(Memory::load(&dir.join("player.sqlite"), 1, 65536).is_err());
    let loaded = Memory::load(&dir.join("player.sqlite"), 0, 65536).unwrap();
    assert_eq!(m.export_state().unwrap(), loaded.export_state().unwrap());
}
#[test]
fn planning_uses_tools_and_replay_reaches_the_real_committed_world() {
    let mut c = Config::demo(2);
    c.days = 1;
    let root = directory("real-flow");
    let result = labor::run(&c, &root, 20, false).unwrap();
    assert!(result["calls"].as_u64().unwrap() >= 4, "{result}");
    assert_eq!(
        result["hash"],
        labor::runtime::replay(&root).unwrap()["hash"]
    );
    let e: Vec<Value> =
        serde_json::from_slice(&fs::read(root.join("events/00000000.json")).unwrap()).unwrap();
    assert!(
        e.iter().any(|x| x["event_type"] == "action.committed"),
        "{e:?}"
    );
    assert!(e
        .iter()
        .any(|x| x["event_type"] == "tool.completed" && x["payload"]["name"] == "memory_write"));
    let cp: Value =
        serde_json::from_slice(&fs::read(root.join("checkpoints/00000020.json")).unwrap()).unwrap();
    let m = Memory::load(
        &root.join(cp["players"][0]["path"].as_str().unwrap()),
        0,
        c.budget.memory_bytes,
    )
    .unwrap();
    assert!(
        m.get("plan", "strategy").unwrap().unwrap()["revision"]
            .as_u64()
            .unwrap()
            >= 2
    );
    let export = root.join("export");
    let manifest = labor::audit::export(&root, &export).unwrap();
    assert_eq!(manifest["files"].as_array().unwrap().len(), 2);
    for id in 0..2 {
        for line in fs::read_to_string(export.join(format!("player-{id:03}.jsonl")))
            .unwrap()
            .lines()
        {
            let v: Value = serde_json::from_str(line).unwrap();
            assert_eq!(v["player_id"], id);
            assert_ne!(v["event_type"], "world.effect");
        }
    }
}
#[test]
fn crashes_reuse_answers_and_do_not_duplicate_actions_or_memory() {
    let mut c = Config::demo(2);
    c.days = 1;
    let baseline = directory("baseline");
    let expected = labor::run(&c, &baseline, 8, false).unwrap();
    for fault in ["after_response", "before_checkpoint"] {
        let root = directory(fault);
        assert!(labor::runtime::run_with_fault(&c, &root, 8, false, Some((0, fault))).is_err());
        let resumed = labor::run(&c, &root, 8, false).unwrap();
        assert_eq!(expected["hash"], resumed["hash"]);
        assert_eq!(expected["calls"], resumed["calls"]);
        let cpa: Value =
            serde_json::from_slice(&fs::read(baseline.join("checkpoints/00000008.json")).unwrap())
                .unwrap();
        let cpb: Value =
            serde_json::from_slice(&fs::read(root.join("checkpoints/00000008.json")).unwrap())
                .unwrap();
        assert_eq!(cpa["players"], cpb["players"]);
        assert_eq!(
            resumed["hash"],
            labor::runtime::replay(&root).unwrap()["hash"]
        );
    }
}
#[test]
fn parallel_model_delivery_does_not_change_world_or_turn_budget() {
    let c = Config::demo(4);
    let mut parallel = c.clone();
    parallel.parallel = 4;
    let a = labor::run(&c, &directory("serial"), 4, false).unwrap();
    let b = labor::run(&parallel, &directory("parallel"), 4, false).unwrap();
    assert_eq!(a["hash"], b["hash"]);
    assert_eq!(a["calls"], b["calls"]);
}
#[test]
fn manifest_and_audit_tampering_are_rejected() {
    let c = Config::demo(2);
    let root = directory("tamper");
    labor::run(&c, &root, 2, false).unwrap();
    let mut changed = c.clone();
    changed.budget.calls_per_window += 1;
    assert!(labor::run(&changed, &root, 1, false).is_err());
    let p = root.join("events/00000000.json");
    let mut events: Value = serde_json::from_slice(&fs::read(&p).unwrap()).unwrap();
    events[0]["payload"]["player"] = json!(999);
    fs::write(&p, events.to_string()).unwrap();
    assert!(labor::audit::verify(&root).is_err());
}
#[test]
fn extra_roles_do_not_multiply_player_budget() {
    let mut c = Config::demo(2);
    c.budget.calls_per_window = 2;
    for h in &mut c.players {
        let mut r = h.roles[0].clone();
        r.id = "advisor".into();
        h.roles.push(r);
    }
    let root = directory("shared-budget");
    let summary = labor::run(&c, &root, 1, false).unwrap();
    assert_eq!(summary["calls"], 4);
}

#[test]
fn matrices_rotate_all_seats_and_resume_without_extra_calls() {
    let c = Config::demo(2);
    let out = directory("matrix");
    let first = labor::experiment::matrix(&c, &out, &[12, 34], 2, false).unwrap();
    assert_eq!(first["runs"].as_array().unwrap().len(), 4);
    assert_eq!(first["runs"][0]["seat_to_initial_setup"], json!([0, 1]));
    assert_eq!(first["runs"][1]["seat_to_initial_setup"], json!([1, 0]));
    assert_eq!(
        first,
        labor::experiment::matrix(&c, &out, &[12, 34], 2, false).unwrap()
    );
}

#[test]
fn learning_packages_are_shared_and_reject_training_seed() {
    let mut c = Config::demo(2);
    c.seed = 73;
    let source = directory("learning-source");
    let run = labor::run(&c, &source, 1, false).unwrap();
    let dir = directory("learning-package");
    let concepts = dir.join("concepts.json");
    let package = dir.join("shared.json");
    fs::write(&concepts,json!([{"id":"supply","text":"Versorgung vor Ausbau prüfen; zu testende Hypothese.","evidence":[{"run_id":run["run_id"]}]}]).to_string()).unwrap();
    let result =
        labor::experiment::package(&concepts, &[source.to_string_lossy().into()], &package)
            .unwrap();
    c.mode = "shared_learning".into();
    c.learning_package = Some(package.to_string_lossy().into());
    c.learning_sha256 = Some(result["sha256"].as_str().unwrap().into());
    assert!(labor::run(&c, &directory("leaked-seed"), 1, false)
        .unwrap_err()
        .contains("Holdout"));
    c.seed = 74;
    let out = directory("shared-learning");
    labor::run(&c, &out, 1, false).unwrap();
    let cp: Value =
        serde_json::from_slice(&fs::read(out.join("checkpoints/00000001.json")).unwrap()).unwrap();
    for sid in 0..2 {
        let m = Memory::load(
            &out.join(cp["players"][sid]["path"].as_str().unwrap()),
            sid as u16,
            c.budget.memory_bytes,
        )
        .unwrap();
        assert_eq!(
            m.get("learning", "supply").unwrap().unwrap()["value"]["id"],
            "supply"
        );
    }
}

#[test]
fn confirmed_rejections_need_explicit_retry_unknown_calls_stay_blocked() {
    let c = Config::demo(2);
    let root = directory("retry");
    assert!(
        labor::runtime::run_with_fault(&c, &root, 1, false, Some((0, "after_response"))).is_err()
    );
    let path = fs::read_dir(root.join("calls"))
        .unwrap()
        .map(|x| x.unwrap().path())
        .find(|p| p.to_string_lossy().ends_with(".response.json"))
        .unwrap();
    let key = path
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .strip_suffix(".response.json")
        .unwrap();
    let mut reply: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    reply["failure"] = json!("http_429");
    reply["calls"] = json!([]);
    fs::write(&path, serde_json::to_vec(&reply).unwrap()).unwrap();
    assert!(labor::run(&c, &root, 1, false)
        .unwrap_err()
        .contains("Infrastruktur"));
    labor::runtime::retry(&root, key).unwrap();
    let resumed = labor::run(&c, &root, 1, false).unwrap();
    assert_eq!(resumed["windows"], 1);
    assert!(labor::runtime::retry(&root, "missing-call")
        .unwrap_err()
        .contains("unbekannt"));
    assert_eq!(
        labor::runtime::status(&root).unwrap()["metrics"]["events"]["call.failed"],
        1
    );
}

#[test]
#[ignore = "requires built Docker worker; real isolation, no model network"]
fn docker_runtime_restarts_from_checkpoint_with_same_world() {
    let c = Config::demo(2);
    let expected = labor::run(&c, &directory("trusted-reference"), 3, false).unwrap();
    let mut isolated = c.clone();
    isolated.sandbox = "docker".into();
    isolated.sandbox_image = Some("sternenepoche-worker:local".into());
    let root = directory("docker-runtime");
    labor::run(&isolated, &root, 1, false).unwrap();
    let actual = labor::run(&isolated, &root, 2, false).unwrap();
    assert_eq!(expected["hash"], actual["hash"]);
    assert_eq!(
        actual["hash"],
        labor::runtime::replay(&root).unwrap()["hash"]
    );
}
