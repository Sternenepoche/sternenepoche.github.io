#![cfg(windows)]
use serde_json::json;
use std::{path::PathBuf, time::Duration};
use sternenepoche_agenten::labor::{config::Config, memory::Memory, native_sandbox::NativeSession};

fn worker() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/debug/sternenepoche-player-worker.exe")
}
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../laeufe/sandbox-workspaces")
}

#[test]
#[ignore = "build native worker using tools/sandbox/build-native.ps1"]
fn native_lpac_denies_foreign_files_network_children_and_parent_handles() {
    let evidence = sternenepoche_agenten::labor::native_sandbox::preflight(&worker()).unwrap();
    println!("{}", evidence);
    let proof = &evidence["proof"];
    assert_eq!(proof["app_container"], true);
    assert_eq!(proof["low_privilege_app_container"], true);
    assert_eq!(proof["all_packages_read_error"], 5);
    assert_eq!(proof["job_cpu_rate"], 5000);
    assert_eq!(proof["foreign_hardlink_allowed"], false);
    assert_eq!(proof["worker_binary_read_allowed"], true);
    assert_eq!(proof["worker_binary_write_error"], 5);
    assert!(!proof["environment"]
        .as_array()
        .unwrap()
        .iter()
        .any(|v| v.as_str().unwrap().contains("API_KEY")));
}

#[test]
#[ignore = "build native worker using tools/sandbox/build-native.ps1"]
fn native_private_sqlite_checkpoint_resume_and_foreign_player() {
    let c = Config::demo(2);
    let namespace = format!("native-memory-{}", std::process::id());
    let m0 = Memory::new(0, c.budget.memory_bytes, &c.players[0])
        .unwrap()
        .isolate_native(&worker(), &namespace)
        .unwrap();
    let m1 = Memory::new(1, c.budget.memory_bytes, &c.players[1])
        .unwrap()
        .isolate_native(&worker(), &namespace)
        .unwrap();
    m0.put("note", "private", 0, &json!({"strategy":"mine"}))
        .unwrap();
    assert!(m1.get("note", "private").unwrap().is_none());
    let cp = root().join(format!("{namespace}-checkpoint.sqlite"));
    m0.save(&cp).unwrap();
    m0.put("note", "uncommitted", 0, &json!("discard")).unwrap();
    drop(m0);
    let restored = Memory::load(&cp, 0, c.budget.memory_bytes)
        .unwrap()
        .isolate_native(&worker(), &namespace)
        .unwrap();
    assert_eq!(
        restored.get("note", "private").unwrap().unwrap()["revision"],
        1
    );
    assert!(restored.get("note", "uncommitted").unwrap().is_none());
    assert!(restored.meta_set("owner", &json!(1)).is_err());
    drop(restored);
    drop(m1);
    std::fs::remove_file(cp).unwrap();
}

#[test]
#[ignore = "build native worker using tools/sandbox/build-native.ps1"]
fn native_timeout_and_output_flood_stop_the_job() {
    let namespace = format!("native-resource-{}", std::process::id());
    let mut spinning = NativeSession::start_with_timeout(
        &worker(),
        &namespace,
        0,
        16_777_216,
        Duration::from_millis(500),
    )
    .unwrap();
    assert!(spinning
        .request(&json!({"op":"spin_probe"}))
        .unwrap_err()
        .contains("Timeout"));
    assert!(spinning.request(&json!({"op":"probe"})).is_err());
    drop(spinning);
    let mut flood = NativeSession::start(&worker(), &namespace, 0, 16_777_216).unwrap();
    assert!(flood
        .request(&json!({"op":"flood_probe"}))
        .unwrap_err()
        .contains("Ausgabegrenze"));
}

#[test]
#[ignore = "build native worker using tools/sandbox/build-native.ps1"]
fn native_fifty_players_two_windows_resume_replay_and_memory_measurement() {
    use std::time::{Instant, SystemTime, UNIX_EPOCH};
    use sternenepoche_agenten::labor::{audit, runtime};
    let namespace = format!("native-fifty-{}", std::process::id());
    let mut c = Config::v3(50);
    c.sandbox = "native".into();
    c.sandbox_worker = Some(worker().to_string_lossy().to_string());
    let started = Instant::now();
    let mut sessions = vec![];
    let mut usage = vec![];
    for sid in 0..50 {
        let mut session =
            NativeSession::start(&worker(), &namespace, sid, c.budget.memory_bytes).unwrap();
        let state = Memory::new(sid, c.budget.memory_bytes, &c.players[sid as usize])
            .unwrap()
            .export_state()
            .unwrap();
        session
            .request(&json!({"op":"import","state":state}))
            .unwrap();
        usage.push(session.memory_usage().unwrap());
        sessions.push(session);
    }
    let startup_ms = started.elapsed().as_millis();
    assert!(NativeSession::start(&worker(), &namespace, 0, c.budget.memory_bytes).is_err());
    let rpc = Instant::now();
    for session in &mut sessions {
        session
            .request(&json!({"op":"meta","key":"owner"}))
            .unwrap();
    }
    let fifty_rpc_ms = rpc.elapsed().as_millis();
    drop(sessions);
    let out = root().parent().unwrap().join("labor-tests").join(format!(
        "native-fifty-runtime-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let elapsed = Instant::now();
    let first = runtime::run(&c, &out, 2, false).unwrap();
    let two_windows_ms = elapsed.elapsed().as_millis();
    assert_eq!(first["players"], 50);
    assert_eq!(first["windows"], 2);
    let elapsed = Instant::now();
    let resumed = runtime::run(&c, &out, 1, false).unwrap();
    let resume_ms = elapsed.elapsed().as_millis();
    assert_eq!(resumed["windows"], 3);
    let replay = runtime::replay(&out).unwrap();
    assert_eq!(replay["hash"], resumed["hash"]);
    assert_eq!(replay["verified"], true);
    audit::verify(&out).unwrap();
    let evidence = json!({"players":50,"startup_import_ms":startup_ms,"fifty_meta_rpc_ms":fifty_rpc_ms,"two_windows_ms":two_windows_ms,"resume_one_window_ms":resume_ms,
        "working_set_total_bytes":usage.iter().map(|v|v["working_set_bytes"].as_u64().unwrap()).sum::<u64>(),"private_total_bytes":usage.iter().map(|v|v["private_bytes"].as_u64().unwrap()).sum::<u64>(),
        "working_set_max_bytes":usage.iter().map(|v|v["working_set_bytes"].as_u64().unwrap()).max(),"per_worker":usage,"first":first,"resumed":resumed,"replay":replay});
    std::fs::write(
        out.join("native-measurement.json"),
        serde_json::to_vec_pretty(&evidence).unwrap(),
    )
    .unwrap();
    println!("native50 evidence={} startup_import={}ms fifty_meta={}ms windows2={}ms resume={}ms working_set_total={} private_total={}",out.display(),startup_ms,fifty_rpc_ms,two_windows_ms,resume_ms,evidence["working_set_total_bytes"],evidence["private_total_bytes"]);
}
