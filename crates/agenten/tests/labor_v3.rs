use serde_json::{json, Value};
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use sternenepoche_agenten::labor::{self, config::Config, memory::Memory};
fn dir(label: &str) -> PathBuf {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../laeufe/labor-tests")
        .join(format!(
            "v3-{label}-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
    fs::create_dir_all(&p).unwrap();
    p
}
#[test]
fn v3_coordinates_roles_replays_and_resumes() {
    let mut c = Config::v3(2);
    c.days = 1;
    c.budget.calls_per_window = 2;
    for h in &mut c.players {
        let mut r = h.roles[0].clone();
        r.id = "versorgung".into();
        h.roles.push(r);
    }
    assert_eq!(c.execution_roles(0, &c.players[0]).len(), 1);
    let p = dir("resume");
    let result = labor::run(&c, &p, 2, false).unwrap();
    assert_eq!(result["calls"], 4);
    let b = fs::read(p.join("checkpoints/00000002.bin")).unwrap();
    assert!(kern::Welt::aus_bytes(&b).unwrap().kolonisation.aktiv);
    labor::runtime::replay(&p).unwrap();
    labor::run(&c, &p, 1, false).unwrap();
    labor::audit::verify(&p).unwrap();
}
#[test]
fn provider_modes_and_cloud_weights_are_strict() {
    let mut c = Config::v3(2);
    c.provider_mode = "local_only".into();
    assert!(c.validate().is_err());
    let m = c.models.get_mut("demo").unwrap();
    m.kind = "ollama".into();
    m.url = Some("http://127.0.0.1:11434".into());
    c.validate().unwrap();
    let mut remote = c.models["demo"].clone();
    remote.kind = "openrouter".into();
    remote.url = Some("https://openrouter.ai/api/v1".into());
    remote.api_key_env = Some("TEST_KEY".into());
    remote.reserve_micro_usd = 100;
    c.models.insert("remote".into(), remote);
    assert!(c.validate().is_err());
    c.provider_mode = "mixed".into();
    c.validate().unwrap();
    let local = &c.models["demo"];
    let remote = &c.models["remote"];
    assert_eq!(
        labor::broker::serial_local_width(&[remote, local, remote, local], 8),
        3
    );
    assert_eq!(
        labor::broker::serial_local_width(&[local, local, remote], 8),
        1
    );
    assert!(!labor::broker::local_weights(
        &json!({"name":"large:cloud","size":9000000}),
        &json!({"model_info":{"a.context_length":8192}})
    ));
    assert!(!labor::broker::local_weights(
        &json!({"name":"alias","size":9000000}),
        &json!({"remote_host":"https://cloud","model_info":{"a.context_length":8192}})
    ));
    assert!(labor::broker::local_weights(
        &json!({"name":"local","size":9000000}),
        &json!({"model_info":{"a.context_length":8192}})
    ));
}
#[test]
fn tool_loading_is_compact_and_cannot_expand_rights_or_reveal_enemy_planets() {
    use labor::{broker::ToolCall, gateway};
    let c = Config::v3(2);
    let (w, _) = labor::world::create(&c).unwrap();
    let m = Memory::new(0, c.budget.memory_bytes, &c.players[0]).unwrap();
    let role = c.execution_roles(0, &c.players[0])[0].clone();
    let small = gateway::tools_for(&c, &role, &gateway::initial_tools());
    assert!(small.len() <= 8);
    assert!(serde_json::to_vec(&small).unwrap().len() < 8000);
    let call = |name: &str, a: Value| ToolCall {
        id: "t".into(),
        name: name.into(),
        arguments: a,
    };
    assert!(gateway::execute(
        &w,
        &m,
        &c,
        &role,
        &call(
            "tool_select",
            json!({"names":["game_flotte_senden","own_state"]})
        )
    )
    .is_ok());
    assert!(gateway::execute(
        &w,
        &m,
        &c,
        &role,
        &call("tool_select", json!({"names":["execute_shell"]}))
    )
    .is_err());
    gateway::execute(
        &w,
        &m,
        &c,
        &role,
        &call(
            "memory_write",
            json!({"kind":"plan","key":"strategy","value":"feed colonies"}),
        ),
    )
    .unwrap();
    gateway::execute(
        &w,
        &m,
        &c,
        &role,
        &call(
            "memory_write",
            json!({"kind":"plan","key":"strategy","value":"escort colony"}),
        ),
    )
    .unwrap();
    assert_eq!(m.get("plan", "strategy").unwrap().unwrap()["revision"], 2);
    assert!(gateway::execute(
        &w,
        &m,
        &c,
        &role,
        &call(
            "memory_write",
            json!({"kind":"plan","key":"strategy","revision":0,"value":"stale explicit revision"})
        )
    )
    .is_err());
    let enemy = w.planeten[w.spieler[1].heimat as usize].koord.to_string();
    assert!(gateway::execute(
        &w,
        &m,
        &c,
        &role,
        &call("own_state", json!({"planet":enemy}))
    )
    .is_err());
    let own = w.planeten[w.spieler[0].heimat as usize].koord.to_string();
    assert!(gateway::execute(&w, &m, &c, &role, &call("own_state", json!({"planet":own}))).is_ok());
}
#[test]
fn expired_window_is_recorded_without_dispatch_and_cannot_be_extended_on_resume() {
    let c = Config::v3(2);
    let p = dir("deadline");
    fs::create_dir_all(p.join("deadlines")).unwrap();
    fs::write(
        p.join("deadlines/00000000.json"),
        json!({"window":0,"deadline_unix_ms":1,"budget_seconds":900}).to_string(),
    )
    .unwrap();
    labor::run(&c, &p, 1, false).unwrap();
    assert!(!fs::read_dir(p.join("calls")).unwrap().any(|e| e
        .unwrap()
        .file_name()
        .to_string_lossy()
        .ends_with("sent.json")));
    let events: Vec<Value> =
        serde_json::from_slice(&fs::read(p.join("events/00000000.json")).unwrap()).unwrap();
    assert_eq!(
        events
            .iter()
            .filter(|e| e["event_type"] == "call.skipped")
            .count(),
        2
    );
    labor::runtime::replay(&p).unwrap();
    fs::write(
        p.join("deadlines/00000000.json"),
        json!({"deadline_unix_ms":99999999999999u64}).to_string(),
    )
    .unwrap();
    assert!(labor::audit::verify(&p).is_err());
}
#[test]
fn epoch_continuation_only_transfers_own_memory_to_new_seed() {
    let mut c = Config::v3(2);
    c.days = 1;
    c.budget.calls_per_window = 2;
    let p = dir("epoch-a");
    assert_eq!(labor::run(&c, &p, 96, false).unwrap()["finished"], true);
    let last = labor::runtime::checkpoint_manifests(&p)
        .unwrap()
        .last()
        .unwrap()
        .1
        .clone();
    let old = Memory::load(
        &p.join(last["players"][0]["path"].as_str().unwrap()),
        0,
        c.budget.memory_bytes,
    )
    .unwrap();
    assert!(!old.search("", Some("plan"), 30).unwrap().is_empty());
    let mut next = c.clone();
    next.mode = "continuation".into();
    next.previous_epoch = Some(p.to_string_lossy().into());
    assert!(labor::run(&next, &dir("bad-seed"), 1, false).is_err());
    next.seed += 1;
    let p2 = dir("epoch-b");
    labor::run(&next, &p2, 1, false).unwrap();
    let cp = labor::runtime::checkpoint_manifests(&p2).unwrap()[0]
        .1
        .clone();
    let new = Memory::load(
        &p2.join(cp["players"][0]["path"].as_str().unwrap()),
        0,
        c.budget.memory_bytes,
    )
    .unwrap();
    assert_eq!(
        new.search("", Some("plan"), 30).unwrap().len(),
        old.search("", Some("plan"), 30).unwrap().len()
    );
    assert!(!new.meta("previous_epoch").unwrap().is_null());
    labor::runtime::replay(&p2).unwrap();
}

fn ollama_stub(responses: Vec<(&'static str, Value)>) -> (String, std::thread::JoinHandle<()>) {
    use std::io::{Read, Write};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let handle = std::thread::spawn(move || {
        for (status, body) in responses {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(3)))
                .unwrap();
            let mut b = [0; 16384];
            let _ = stream.read(&mut b).unwrap();
            let body = body.to_string();
            write!(stream,"HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).unwrap();
        }
    });
    (url, handle)
}
fn fallback_config(url: &str) -> Config {
    let mut c = Config::v3(2);
    c.provider_mode = "local_or_remote".into();
    let m = c.models.get_mut("demo").unwrap();
    m.kind = "ollama".into();
    m.model = "fixture:7b".into();
    m.url = Some(url.into());
    m.cpu_only = true;
    m.context = 16384;
    m.timeout_seconds = 17;
    m.resident_bytes = Some(1234567);
    let mut remote = m.clone();
    remote.kind = "openrouter".into();
    remote.model = "fixture/remote".into();
    remote.url = Some("https://openrouter.ai/api/v1".into());
    remote.api_key_env = Some("TEST_KEY".into());
    remote.reserve_micro_usd = 100;
    c.models.insert("remote".into(), remote);
    c
}
#[test]
fn fallback_preserves_explicit_local_identity_cpu_and_resource_settings() {
    let (url, server) = ollama_stub(vec![
        (
            "200 OK",
            json!({"models":[{"name":"fixture:7b","size":9000000,"digest":"pinned"}]}),
        ),
        ("200 OK", json!({"models":[]})),
        (
            "200 OK",
            json!({"capabilities":["tools"],"model_info":{"fixture.context_length":32768}}),
        ),
    ]);
    let c = fallback_config(&url);
    let result = labor::broker::resolve_policy(&c).unwrap();
    server.join().unwrap();
    assert_eq!(result.models["demo"], c.models["demo"]);
    assert!(!result.models.contains_key("remote"));
    assert_eq!(result.players[0].roles[0].model, "demo");
}
#[test]
fn fallback_rejects_inventary_errors_and_only_uses_remote_after_empty_inventory() {
    let (url, server) = ollama_stub(vec![(
        "500 Internal Server Error",
        json!({"error":"inventory failed"}),
    )]);
    assert!(labor::broker::resolve_policy(&fallback_config(&url))
        .unwrap_err()
        .contains("kein Remote-Fallback"));
    server.join().unwrap();
    let (url, server) = ollama_stub(vec![
        ("200 OK", json!({"models":[]})),
        ("200 OK", json!({"models":[]})),
        ("200 OK", json!({"models":[]})),
        ("200 OK", json!({"models":[]})),
    ]);
    let result = labor::broker::resolve_policy(&fallback_config(&url)).unwrap();
    server.join().unwrap();
    assert_eq!(result.models.len(), 1);
    assert!(result.models.contains_key("remote"));
}
#[test]
fn matrix_rotates_primary_models_with_harnesses() {
    let mut c = Config::v3(2);
    let mut b = c.models["demo"].clone();
    b.model = "second".into();
    c.models.insert("second".into(), b);
    c.primary_models.insert(0, "demo".into());
    c.primary_models.insert(1, "second".into());
    let p = dir("primary-rotation");
    labor::experiment::matrix(&c, &p, &[1], 1, false).unwrap();
    let m: Value =
        serde_json::from_slice(&fs::read(p.join("seed-1-rotation-001/manifest.json")).unwrap())
            .unwrap();
    assert_eq!(m["config"]["primary_models"]["0"], "second");
    assert_eq!(m["config"]["primary_models"]["1"], "demo");
}
