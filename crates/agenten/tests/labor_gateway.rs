use kern::{Rolle, Welt};
use serde_json::{json, Value};
use sternenepoche_agenten::labor::{
    broker::ToolCall,
    config::{Config, Role},
    gateway,
    memory::Memory,
    sandbox::{DockerSandbox, Limits},
};

fn setup() -> (Config, Welt, Memory, Role) {
    let c = Config::demo(2);
    let (w, _) = sternenepoche_agenten::labor::world::create(&c).unwrap();
    let r = c.players[0].roles[0].clone();
    let m = Memory::new(0, c.budget.memory_bytes, &c.players[0]).unwrap();
    (c, w, m, r)
}
fn call(name: &str, args: Value) -> ToolCall {
    ToolCall {
        id: "test".into(),
        name: name.into(),
        arguments: args,
    }
}

#[test]
fn actions_are_prepared_against_frozen_world_and_use_real_role() {
    let (c, w, m, r) = setup();
    let hash = w.hash();
    let p = w.planeten[w.spieler[0].heimat as usize].koord.to_string();
    let result = gateway::execute(
        &w,
        &m,
        &c,
        &r,
        &call(
            "game_bauen",
            json!({"planet":p,"gebaeude":"solarkraftwerk"}),
        ),
    )
    .unwrap();
    assert_eq!(w.hash(), hash);
    assert_eq!(
        gateway::engine_role(result.intent.as_ref().unwrap()).unwrap(),
        Rolle::Verwalter
    );
    assert_eq!(result.result["status"], "prepared");
}
#[test]
fn variable_role_names_do_not_grant_engine_capabilities() {
    let (mut c, w, _, mut r) = setup();
    r.id = "Alle".into();
    r.capabilities = vec!["meldung".into()];
    c.players[0].roles = vec![r.clone()];
    let m = Memory::new(0, c.budget.memory_bytes, &c.players[0]).unwrap();
    assert!(gateway::execute(
        &w,
        &m,
        &c,
        &r,
        &call("game_steuersatz", json!({"prozent":10}))
    )
    .is_err());
    let mut forged = r.clone();
    forged.capabilities.push("steuersatz".into());
    assert!(gateway::execute(
        &w,
        &m,
        &c,
        &forged,
        &call("game_steuersatz", json!({"prozent":10}))
    )
    .is_err());
    assert!(gateway::execute(
        &w,
        &m,
        &c,
        &r,
        &call(
            "memory_write",
            json!({"kind":"receipt","key":"forged","revision":0,"value":{}})
        )
    )
    .is_err());
    let mut modified = c.players[0].clone();
    modified.roles[0].capabilities.push("steuersatz".into());
    m.meta_set("harness", &json!(modified)).unwrap();
    assert!(gateway::execute(
        &w,
        &m,
        &c,
        &modified.roles[0],
        &call("game_steuersatz", json!({"prozent":10}))
    )
    .is_err());
}

#[test]
fn prepared_actions_cannot_bypass_empty_engine_pots() {
    let (c, mut w, m, r) = setup();
    w.spieler[0].toepfe = [0; kern::TOEPFE];
    let p = w.planeten[w.spieler[0].heimat as usize].koord.to_string();
    let action = json!({"planet":p,"gebaeude":"solarkraftwerk"});
    assert!(gateway::execute(&w, &m, &c, &r, &call("game_bauen", action.clone())).is_err());
    // Establish that this fixture really detects the legacy bypass the gateway forbids.
    let mut unchecked = w.clone();
    let mut legacy = action;
    legacy["typ"] = json!("bauen");
    assert!(unchecked.handeln(0, Rolle::Alle, &legacy).0);
}

#[test]
fn storage_quotas_include_record_overhead_and_cap_metadata_and_record_count() {
    let (c, _, mut m, _) = setup();
    let charge = sternenepoche_agenten::labor::memory::limits::record_bytes(
        "note",
        "n",
        &json!("ö").to_string(),
    );
    m.quota = charge - 1;
    assert!(m.put("note", "n", 0, &json!("ö")).is_err());
    m.quota = charge;
    assert!(m.put("note", "n", 0, &json!("ö")).is_ok());
    m.put("note", "n", 1, &Value::Null).unwrap();
    assert!(m.meta_set("owner", &json!(1)).is_err());
    assert!(m.meta_set("oversized", &json!("x".repeat(65_535))).is_err());
    for i in 0..15 {
        m.meta_set(&format!("large-{i}"), &json!("x".repeat(65_534)))
            .unwrap();
    }
    assert!(m.meta_set("large-16", &json!("x".repeat(65_534))).is_err());
    m.meta_set("large-0", &Value::Null).unwrap();
    m.meta_set("replacement", &json!("x".repeat(65_534)))
        .unwrap();
    let (_, _, baseline, _) = setup();
    let mut state = baseline.export_state().unwrap();
    state["records"] = json!((0..4096)
        .map(|n| json!({"kind":"note","key":format!("n-{n}"),"revision":1,"value":null}))
        .collect::<Vec<_>>());
    let full = Memory::from_state(0, c.budget.memory_bytes, &state).unwrap();
    assert!(full
        .put("note", "over-count", 0, &Value::Null)
        .unwrap_err()
        .contains("4096"));
    assert!(full.put("note", "n-0", 1, &json!("updated")).is_ok());
    let old_count = state["meta"].as_object().unwrap().len();
    for n in 0..32 - old_count {
        full.meta_set(&format!("m-{n}"), &json!(0)).unwrap();
    }
    assert!(full.meta_set("overflow-key", &json!(0)).is_err());
    let pages: u64 = full
        .conn
        .query_row("PRAGMA max_page_count", [], |r| r.get(0))
        .unwrap();
    assert_eq!(pages, 7168);
}
#[test]
fn unknown_and_ambiguous_operational_fields_are_rejected() {
    let (c, w, m, r) = setup();
    for tool in gateway::tools(&r) {
        assert_eq!(tool["function"]["parameters"]["type"], "object");
        assert!(tool["function"]["parameters"]["properties"].is_object());
    }
    assert!(gateway::execute(
        &w,
        &m,
        &c,
        &r,
        &call("game_steuersatz", json!({"prozent":10,"spieler":1}))
    )
    .is_err());
    assert!(gateway::execute(
        &w,
        &m,
        &c,
        &r,
        &call(
            "memory_write",
            json!({"kind":"note","key":"n","revision":0,"value":"ok","owner":1})
        )
    )
    .is_err());
    assert!(
        sternenepoche_agenten::protocol::strict_json("{\"prozent\":10,\"prozent\":20}").is_err()
    );
    assert!(gateway::execute(&w,&m,&c,&r,&call("game_fertigen",json!({"planet":"1:1:5","einheit":"kleiner_transporter","bauteil":"antriebskern","anzahl":1}))).is_err());
    assert!(gateway::execute(
        &w,
        &m,
        &c,
        &r,
        &call(
            "game_fertigen",
            json!({"planet":"1:1:5","einheit":"kleiner_transporter","anzahl":0})
        )
    )
    .is_err());
}
#[test]
fn fabrication_routes_military_and_civilian_pots() {
    assert_eq!(
        gateway::engine_role(
            &json!({"typ":"fertigen","planet":"1:1:5","einheit":"spionagesonde","anzahl":1})
        )
        .unwrap(),
        Rolle::Feldherr
    );
    assert_eq!(
        gateway::engine_role(
            &json!({"typ":"fertigen","planet":"1:1:5","einheit":"bergbauschiff","anzahl":1})
        )
        .unwrap(),
        Rolle::Verwalter
    );
    assert_eq!(gateway::engine_role(&json!({"typ":"flotte_senden","start":"1:1:5","ziel":"1:1:6","mission":"angriff","schiffe":{}})).unwrap(),Rolle::Feldherr);
}
#[test]
fn memory_and_harness_changes_are_private_and_versioned() {
    let (c, w, m, r) = setup();
    let other = Memory::new(1, c.budget.memory_bytes, &c.players[1]).unwrap();
    gateway::execute(
        &w,
        &m,
        &c,
        &r,
        &call(
            "memory_write",
            json!({"kind":"plan","key":"private","revision":0,"value":{"text":"mine"}}),
        ),
    )
    .unwrap();
    assert!(other.get("plan", "private").unwrap().is_none());
    assert!(gateway::execute(
        &w,
        &m,
        &c,
        &r,
        &call(
            "memory_write",
            json!({"kind":"plan","key":"private","revision":0,"value":"stale"})
        )
    )
    .is_err());
    let mut next = c.players[0].clone();
    next.revision = 1;
    next.roles[0].id = "expansion".into();
    gateway::execute(
        &w,
        &m,
        &c,
        &r,
        &call("harness_patch", json!({"harness":next})),
    )
    .unwrap();
    assert_eq!(m.meta("harness").unwrap()["revision"], 0);
    assert_eq!(m.meta("pending_harness").unwrap()["revision"], 1);
    let mut conflicting = next.clone();
    conflicting.roles[0].id = "conflicting".into();
    assert!(gateway::execute(
        &w,
        &m,
        &c,
        &r,
        &call("harness_patch", json!({"harness":conflicting}))
    )
    .is_err());
    assert!(gateway::execute(
        &w,
        &m,
        &c,
        &r,
        &call(
            "harness_patch",
            json!({"harness":{"revision":1,"roles":[],"identity":1}})
        )
    )
    .is_err());
}

#[test]
fn model_switch_requires_run_permission_and_skills_are_declarative() {
    let (mut c, w, m, r) = setup();
    c.models
        .insert("alternative".into(), c.models["demo"].clone());
    let mut next = c.players[0].clone();
    next.revision = 1;
    next.roles[0].model = "alternative".into();
    assert!(gateway::execute(
        &w,
        &m,
        &c,
        &r,
        &call("harness_patch", json!({"harness":next}))
    )
    .is_err());
    c.allow_model_switch = true;
    gateway::execute(
        &w,
        &m,
        &c,
        &r,
        &call("harness_patch", json!({"harness":next})),
    )
    .unwrap();
    gateway::execute(&w,&m,&c,&r,&call("memory_write",json!({"kind":"skill","key":"supply","revision":0,"value":{"instructions":"Check reserves with world_query."}}))).unwrap();
    let out =
        gateway::execute(&w, &m, &c, &r, &call("skill_read", json!({"key":"supply"}))).unwrap();
    assert_eq!(out.result["authority"], "untrusted_guidance");
    assert!(out.intent.is_none());
}

#[test]
#[ignore = "requires existing Linux Docker engine and built sternenepoche-worker:local image"]
fn docker_private_sqlite_and_negative_probes() {
    let image = "sternenepoche-worker:local";
    let namespace = format!("test-{}", std::process::id());
    let s = DockerSandbox::with_workspace(image, Limits::default(), &namespace).unwrap();
    let c = Config::demo(2);
    let _ = s.destroy(0);
    let _ = s.destroy(1);
    for owner in [0, 1] {
        s.memory(
            owner,
            c.budget.memory_bytes,
            &json!({"op":"new","harness":c.players[owner as usize]}),
        )
        .unwrap();
    }
    s.memory(
        0,
        c.budget.memory_bytes,
        &json!({"op":"put","kind":"note","key":"secret","revision":0,"value":"player-zero"}),
    )
    .unwrap();
    assert_eq!(
        s.memory(
            1,
            c.budget.memory_bytes,
            &json!({"op":"get","kind":"note","key":"secret"})
        )
        .unwrap(),
        Value::Null
    );
    assert!(s
        .memory(
            0,
            c.budget.memory_bytes,
            &json!({"op":"meta_set","key":"owner","value":1})
        )
        .is_err());
    let state = s
        .memory(0, c.budget.memory_bytes, &json!({"op":"export"}))
        .unwrap();
    assert!(s
        .memory(
            1,
            c.budget.memory_bytes,
            &json!({"op":"import","state":state})
        )
        .is_err());
    assert!(s
        .memory(
            0,
            c.budget.memory_bytes,
            &json!({"op":"get","kind":"note","key":"secret","owner":1})
        )
        .is_err());
    let probe = sternenepoche_agenten::labor::sandbox::preflight(image).unwrap();
    assert_eq!(probe["isolation_probe"], "passed");
    let e=s.execute(0,&["/bin/sh".into(),"-c".into(),"test ! -e /workspace/memory.sqlite && test ! -e /var/run/docker.sock && test ! -e /host && test \"$(id -u)\" = 65534 && ! touch /etc/forbidden && printf private".into()],b"").unwrap();
    assert_eq!(e.exit_code, 0);
    assert_eq!(e.stdout, "private");
    let limits=s.execute(0,&["/bin/sh".into(),"-c".into(),"if test -e /sys/fs/cgroup/memory.max; then cat /sys/fs/cgroup/memory.max /sys/fs/cgroup/pids.max /sys/fs/cgroup/cpu.max; else cat /sys/fs/cgroup/memory/memory.limit_in_bytes /sys/fs/cgroup/pids/pids.max; printf '%s %s\\n' \"$(cat /sys/fs/cgroup/cpu/cpu.cfs_quota_us)\" \"$(cat /sys/fs/cgroup/cpu/cpu.cfs_period_us)\"; fi".into()],b"").unwrap();
    assert_eq!(limits.exit_code, 0, "{}", limits.stderr);
    assert_eq!(limits.stdout, "134217728\n32\n50000 100000\n");
    let disk = s
        .execute(
            0,
            &[
                "/bin/sh".into(),
                "-c".into(),
                "dd if=/dev/zero of=/workspace/too-big bs=1048576 count=20 2>/dev/null".into(),
            ],
            b"",
        )
        .unwrap();
    assert_ne!(disk.exit_code, 0);
    let tiny = DockerSandbox::new(
        image,
        Limits {
            timeout_seconds: 2,
            output_bytes: 128,
            ..Limits::default()
        },
    )
    .unwrap();
    assert!(tiny
        .execute(
            0,
            &["/bin/sh".into(), "-c".into(), "while :; do :; done".into()],
            b""
        )
        .unwrap_err()
        .contains("Laufzeit"));
    assert!(tiny
        .execute(0, &["/bin/sh".into(), "-c".into(), "yes flood".into()], b"")
        .unwrap_err()
        .contains("Ausgabe"));
    s.destroy(0).unwrap();
    s.destroy(1).unwrap();
}

#[test]
#[ignore = "requires built sternenepoche-worker:local Docker image"]
fn docker_memory_checkpoint_resume_and_persistent_server() {
    let (c, _, m, _) = setup();
    let image = "sternenepoche-worker:local";
    let namespace = format!("resume-{}", std::process::id());
    let sandbox = DockerSandbox::with_workspace(image, Limits::default(), &namespace).unwrap();
    let _ = sandbox.destroy(0);
    let checkpoint = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../target/worker-checkpoint-{}.sqlite",
        std::process::id()
    ));
    let m = m.isolate(image, &namespace).unwrap();
    assert!(m.dump().unwrap().as_array().unwrap().is_empty());
    m.put("plan", "survives", 0, &json!({"next":"resume"}))
        .unwrap();
    m.save(&checkpoint).unwrap();
    m.put("note", "uncommitted", 0, &json!("roll back on resume"))
        .unwrap();
    drop(m);
    let resumed = Memory::load(&checkpoint, 0, c.budget.memory_bytes)
        .unwrap()
        .isolate(image, &namespace)
        .unwrap();
    assert_eq!(
        resumed.get("plan", "survives").unwrap().unwrap()["revision"],
        1
    );
    assert!(resumed.get("note", "uncommitted").unwrap().is_none());
    assert_eq!(resumed.meta("calls").unwrap(), 0);
    let state = resumed.export_state().unwrap();
    drop(resumed);
    let mut server = sternenepoche_agenten::labor::sandbox::WorkerSession::start(
        image,
        &namespace,
        0,
        c.budget.memory_bytes,
    )
    .unwrap();
    server
        .request(&json!({"op":"import","state":state}))
        .unwrap();
    for _ in 0..20 {
        assert_eq!(
            server.request(&json!({"op":"meta","key":"owner"})).unwrap(),
            0
        );
    }
    assert!(server
        .request(&json!({"op":"get","kind":"plan","key":"survives","unknown":true}))
        .is_err());
    assert_eq!(
        server
            .request(&json!({"op":"get","kind":"plan","key":"survives"}))
            .unwrap()["revision"],
        1
    );
    let meta_count = state["meta"].as_object().unwrap().len();
    for n in 0..32 - meta_count {
        server
            .request(&json!({"op":"meta_set","key":format!("quota-{n}"),"value":0}))
            .unwrap();
    }
    assert!(server
        .request(&json!({"op":"meta_set","key":"too-many-meta","value":0}))
        .is_err());
    assert_eq!(
        server.request(&json!({"op":"meta","key":"owner"})).unwrap(),
        0
    );
    let mut full_state = state.clone();
    full_state["records"] = json!((0..4096)
        .map(|n| json!({"kind":"note","key":format!("n-{n}"),"revision":1,"value":null}))
        .collect::<Vec<_>>());
    server
        .request(&json!({"op":"import","state":full_state}))
        .unwrap();
    assert!(server
        .request(
            &json!({"op":"put","kind":"note","key":"too-many-records","revision":0,"value":null})
        )
        .unwrap_err()
        .contains("4096"));
    drop(server);
    sandbox.destroy(0).unwrap();
    std::fs::remove_file(checkpoint).unwrap();
    let small = Memory::new(0, 65_536, &c.players[0]).unwrap();
    let exact_half = json!("x".repeat(32_633));
    assert_eq!(
        sternenepoche_agenten::labor::memory::limits::record_bytes(
            "note",
            "a",
            &exact_half.to_string()
        ),
        32_768
    );
    small.put("note", "a", 0, &exact_half).unwrap();
    small.put("note", "b", 0, &exact_half).unwrap();
    assert!(small.put("note", "c", 0, &Value::Null).is_err());
    let small = small.isolate(image, &namespace).unwrap();
    assert!(small.put("note", "c", 0, &Value::Null).is_err());
    small.put("note", "a", 1, &Value::Null).unwrap();
    small.put("note", "c", 0, &Value::Null).unwrap();
    drop(small);
    sandbox.destroy(0).unwrap();
}
