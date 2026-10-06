use kern::{Rolle, Welt};
use serde_json::json;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
use sternenepoche_agenten::{
    config::Config,
    journal::Journal,
    protocol::{messages, Decision},
    run, RULES,
};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/agenten-tests")
            .join(format!(
                "{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::SeqCst)
            ));
        fs::create_dir_all(&p).unwrap();
        Self(p)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn world() -> Welt {
    Welt::neu(kern::Regelwerk::laden(RULES).unwrap(), 42, 4).unwrap()
}

#[test]
fn resume_matches_uninterrupted_world() {
    let a = Temp::new();
    let b = Temp::new();
    let config = Config::demo(4);
    let first = run(&config, &a.0, 8, false).unwrap();
    let resumed = run(&config, &a.0, 16, false).unwrap();
    let uninterrupted = run(&config, &b.0, 24, false).unwrap();
    assert_eq!(resumed["hash"], uninterrupted["hash"]);
    assert_eq!(resumed["anfragen"], uninterrupted["anfragen"]);
    assert!(first["anfragen"].as_u64().unwrap() > 0);
    assert_eq!(resumed["zeit"], 24 * 900);
}

#[test]
fn interrupted_window_reuses_answers_without_reapplying_actions_twice() {
    let a = Temp::new();
    let b = Temp::new();
    let config = Config::demo(4);
    run(&config, &a.0, 0, false).unwrap();
    let manifest = serde_json::from_slice(&fs::read(a.0.join("manifest.json")).unwrap()).unwrap();
    let calls;
    {
        let journal = Journal::open(&a.0, &manifest).unwrap();
        let (_, mut world) = journal.load_world().unwrap().unwrap();
        sternenepoche_agenten::window(&mut world, &config, &journal).unwrap();
        calls = journal.request_count().unwrap();
        // Simulate process exit after answers/actions but before checkpoint publication.
        // The mutated in-memory world is deliberately discarded.
    }
    let resumed = run(&config, &a.0, 1, false).unwrap();
    let straight = run(&config, &b.0, 1, false).unwrap();
    assert_eq!(resumed["hash"], straight["hash"]);
    assert_eq!(resumed["anfragen"], calls);
}

#[test]
fn pending_requests_are_not_sent_twice_and_finished_ones_are_cached() {
    let t = Temp::new();
    let j = Journal::open(&t.0, &json!({"run":1})).unwrap();
    let req = json!({"input":"observed"});
    assert!(j.reserve("a", &req, 2).unwrap().is_none());
    assert!(j.reserve("a", &req, 2).unwrap_err().contains("unbekannt"));
    j.finish("a", &json!({"text":"done"})).unwrap();
    assert_eq!(j.reserve("a", &req, 2).unwrap().unwrap()["text"], "done");
    assert!(j.reserve("a", &json!({"input":"changed"}), 2).is_err());
    assert!(j.reserve("b", &req, 2).unwrap().is_none());
    assert!(j
        .reserve("c", &req, 2)
        .unwrap_err()
        .contains("Anfragegrenze"));
}

#[test]
fn writer_lock_blocks_second_process_handle_and_config_is_immutable() {
    let t = Temp::new();
    let j = Journal::open(&t.0, &json!({"run":1})).unwrap();
    assert!(Journal::open(&t.0, &json!({"run":1})).is_err());
    drop(j);
    assert!(Journal::open(&t.0, &json!({"run":2})).is_err());
    assert!(Journal::open(&t.0, &json!({"run":1})).is_ok());
}

#[test]
fn checkpoints_verify_content_before_resume() {
    let t = Temp::new();
    let j = Journal::open(&t.0, &json!({})).unwrap();
    let w = world();
    j.save_world(0, &w).unwrap();
    assert_eq!(j.load_world().unwrap().unwrap().1.hash(), w.hash());
    fs::write(t.0.join("states/00000000.bin"), b"corruption").unwrap();
    assert!(j.load_world().is_err());
}

#[test]
fn hidden_opponent_economy_never_enters_prompts() {
    let mut w = world();
    let before = messages(&w, 0, Rolle::Verwalter);
    w.spieler[1].credits += 987654321;
    let foreign = w.spieler[1].heimat as usize;
    w.planeten[foreign].bestand[0] += 987654321;
    assert_eq!(before, messages(&w, 0, Rolle::Verwalter));
}

#[test]
fn envelope_rejects_role_spoofing_limits_and_unknown_fields() {
    let w = world();
    let mut d = json!({"begruendung":"kurz","abfragen":[],"aktionen":[],"prognose":"warten","notiz":"","wecker_stunden":24});
    assert!(Decision::parse(&d.to_string(), &w, Rolle::Diplomat, true).is_ok());
    d["aktionen"] = json!([{"typ":"steuersatz","prozent":25}]);
    assert!(Decision::parse(&d.to_string(), &w, Rolle::Diplomat, true).is_err());
    d["aktionen"] = json!([]);
    d["spieler"] = json!(1);
    assert!(Decision::parse(&d.to_string(), &w, Rolle::Diplomat, true).is_err());
    d.as_object_mut().unwrap().remove("spieler");
    d["wecker_stunden"] = json!(1e100);
    assert!(Decision::parse(&d.to_string(), &w, Rolle::Diplomat, true).is_err());
    d["wecker_stunden"] = json!(1);
    d["abfragen"] = json!([{"typ":"galaxie"}]);
    assert!(Decision::parse(&d.to_string(), &w, Rolle::Diplomat, false).is_err());
    assert!(sternenepoche_agenten::protocol::strict_json(
        r#"{"action":{"typ":"bauen","typ":"schenken"}}"#
    )
    .is_err());
    assert!(sternenepoche_agenten::protocol::strict_json(r#"{"x":1e999}"#).is_err());
}

/// Reads one HTTP request (headers plus Content-Length body) from a test socket.
fn read_request(socket: &mut std::net::TcpStream) -> String {
    use std::io::Read;
    socket.set_read_timeout(Some(std::time::Duration::from_secs(5))).unwrap();
    let mut all = Vec::new();
    let mut buf = [0; 8192];
    loop {
        let n = socket.read(&mut buf).unwrap();
        if n == 0 {
            break;
        }
        all.extend_from_slice(&buf[..n]);
        if let Some(end) = all.windows(4).position(|x| x == b"\r\n\r\n") {
            let header = String::from_utf8_lossy(&all[..end]).to_lowercase();
            let length = header
                .lines()
                .find_map(|s| s.strip_prefix("content-length: "))
                .unwrap()
                .parse::<usize>()
                .unwrap();
            if all.len() >= end + 4 + length {
                break;
            }
        }
    }
    String::from_utf8_lossy(&all).into_owned()
}

fn reply(socket: &mut std::net::TcpStream, status: u16, body: &str) {
    use std::io::Write;
    write!(socket, "HTTP/1.1 {status} Test\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
}

#[test]
fn unambiguous_failure_is_journaled_retried_and_the_run_continues() {
    use std::net::TcpListener;
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let decision = json!({"begruendung":"","abfragen":[],"aktionen":[],"prognose":"","notiz":"","wecker_stunden":null}).to_string();
    let ok = json!({"choices":[{"message":{"content":decision},"finish_reason":"stop"}]}).to_string();
    let server = std::thread::spawn(move || {
        let mut sockets = Vec::new();
        // No response until all four roles have connected: proves parallel I/O.
        for _ in 0..4 {
            sockets.push(listener.accept().unwrap().0);
        }
        for (i, mut socket) in sockets.into_iter().enumerate() {
            read_request(&mut socket);
            reply(&mut socket, if i == 0 { 500 } else { 200 }, &ok);
        }
        // The retry of the failed call arrives on a new connection.
        let (mut socket, _) = listener.accept().unwrap();
        assert!(read_request(&mut socket).contains("\"versuch\"") == false, "the provider body carries no journal fields");
        reply(&mut socket, 200, &ok);
    });
    let t = Temp::new();
    let mut c = Config::demo(1);
    c.parallel = 4;
    let p = c.anbieter.get_mut("offline").unwrap();
    p.art = "local".into();
    p.url = Some(format!("http://{address}/v1"));
    let summary = run(&c, &t.0, 1, true).unwrap();
    assert_eq!(summary["fenster"], 1);
    server.join().unwrap();
    let calls = t.0.join("calls");
    let responses: Vec<(String, serde_json::Value)> = fs::read_dir(&calls)
        .unwrap()
        .map(|p| p.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".response.json"))
        .map(|n| (n.clone(), serde_json::from_slice(&fs::read(calls.join(&n)).unwrap()).unwrap()))
        .collect();
    let failed: Vec<&(String, serde_json::Value)> = responses.iter().filter(|(_, v)| v.get("fehler").is_some()).collect();
    assert_eq!(failed.len(), 1, "the 500 is finished with its error, not left open");
    assert!(failed[0].1["fehler"].as_str().unwrap().contains("500"));
    let retry = failed[0].0.replace(".response.json", "-v2.response.json");
    assert!(responses.iter().any(|(n, v)| *n == retry && v.get("fehler").is_none()), "second attempt under its own key");
    assert_eq!(responses.len(), 5);
}

fn local_config(address: std::net::SocketAddr) -> Config {
    let mut c = Config::demo(1);
    c.parallel = 4;
    let p = c.anbieter.get_mut("offline").unwrap();
    p.art = "local".into();
    p.url = Some(format!("http://{address}/v1"));
    c
}

fn request_files(dir: &std::path::Path, suffix: &str) -> usize {
    fs::read_dir(dir.join("calls"))
        .unwrap()
        .filter(|p| p.as_ref().unwrap().file_name().to_string_lossy().ends_with(suffix))
        .count()
}

/// No credit (402), wrong key (401) or key limit (403): the run stops instead of letting every
/// role idle, the refused reservation is withdrawn, and the call is sent again on resume.
#[test]
fn refused_call_stops_the_run_and_is_sent_again_on_resume() {
    use std::net::TcpListener;
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let decision = json!({"begruendung":"","abfragen":[],"aktionen":[],"prognose":"","notiz":"","wecker_stunden":null}).to_string();
    let ok = json!({"choices":[{"message":{"content":decision},"finish_reason":"stop"}]}).to_string();
    let server = std::thread::spawn(move || {
        let mut sockets = Vec::new();
        for _ in 0..4 {
            sockets.push(listener.accept().unwrap().0);
        }
        for (i, mut socket) in sockets.into_iter().enumerate() {
            read_request(&mut socket);
            reply(&mut socket, if i == 0 { 402 } else { 200 }, &ok);
        }
        // Resume: only the refused call is sent again.
        let (mut socket, _) = listener.accept().unwrap();
        read_request(&mut socket);
        reply(&mut socket, 200, &ok);
    });
    let t = Temp::new();
    let c = local_config(address);
    let e = run(&c, &t.0, 1, true).unwrap_err();
    assert!(e.contains("402") && e.contains("Guthaben"), "{e}");
    assert_eq!(request_files(&t.0, ".request.json"), 3, "the refused reservation is withdrawn");
    assert_eq!(request_files(&t.0, ".response.json"), 3);
    let summary = run(&c, &t.0, 1, true).unwrap();
    assert_eq!(summary["fenster"], 1);
    server.join().unwrap();
    assert_eq!(request_files(&t.0, ".response.json"), 4);
}

/// A call that failed for good is skipped; asking the model for a correction would only repeat it.
#[test]
fn failed_call_gets_no_correction_round() {
    use std::net::TcpListener;
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let decision = json!({"begruendung":"","abfragen":[],"aktionen":[],"prognose":"","notiz":"","wecker_stunden":null}).to_string();
    let ok = json!({"choices":[{"message":{"content":decision},"finish_reason":"stop"}]}).to_string();
    let server = std::thread::spawn(move || {
        let mut sockets = Vec::new();
        for _ in 0..4 {
            sockets.push(listener.accept().unwrap().0);
        }
        for (i, mut socket) in sockets.into_iter().enumerate() {
            read_request(&mut socket);
            reply(&mut socket, if i == 0 { 500 } else { 200 }, &ok);
        }
    });
    let t = Temp::new();
    let mut c = local_config(address);
    c.anbieter.get_mut("offline").unwrap().versuche = 1;
    let summary = run(&c, &t.0, 1, true).unwrap();
    assert_eq!(summary["fenster"], 1);
    server.join().unwrap();
    assert_eq!(request_files(&t.0, ".request.json"), 4, "no correction request after a failed call");
}

#[test]
fn ambiguous_outcome_stops_the_run_and_is_never_retried() {
    use std::net::TcpListener;
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        // Each request arrives completely, then the connection closes without an answer.
        for _ in 0..4 {
            let (mut socket, _) = listener.accept().unwrap();
            read_request(&mut socket);
        }
    });
    let t = Temp::new();
    let mut c = Config::demo(1);
    c.parallel = 4;
    let p = c.anbieter.get_mut("offline").unwrap();
    p.art = "local".into();
    p.url = Some(format!("http://{address}/v1"));
    assert!(run(&c, &t.0, 1, true).unwrap_err().contains("unbekannt"));
    server.join().unwrap();
    // The listener is gone: this must stop at the durable pending reservation, not send again.
    assert!(run(&c, &t.0, 1, true).unwrap_err().contains("unbekannt"));
}

#[test]
fn remote_and_network_execution_require_explicit_opt_in() {
    let t = Temp::new();
    let mut c = Config::demo(4);
    let p = c.anbieter.get_mut("offline").unwrap();
    p.art = "local".into();
    p.url = Some("https://example.org/v1".into());
    assert!(c.validate().is_err());
    c.anbieter.get_mut("offline").unwrap().remote_erlaubt = true;
    assert!(c.validate().is_ok());
    assert!(run(&c, &t.0, 1, false).unwrap_err().contains("--execute"));
}

#[test]
fn real_parquet_files_have_rows_and_typed_columns() {
    use parquet::file::reader::{FileReader, SerializedFileReader};
    let t = Temp::new();
    let dir = t.0.join("run");
    run(&Config::demo(4), &dir, 8, false).unwrap();
    let out = t.0.join("export");
    let counts = sternenepoche_agenten::export::export(&dir, &out).unwrap();
    assert!(counts["entscheidungen"].as_u64().unwrap() > 0);
    for name in ["entscheidungen", "aktionen", "metriken"] {
        let reader =
            SerializedFileReader::new(fs::File::open(out.join(format!("{name}.parquet"))).unwrap())
                .unwrap();
        assert_eq!(
            reader.metadata().file_metadata().num_rows(),
            counts[name].as_i64().unwrap()
        );
        assert_eq!(
            reader
                .metadata()
                .file_metadata()
                .schema_descr()
                .num_columns(),
            7
        );
        assert!(reader.get_row_iter(None).unwrap().next().unwrap().is_ok());
    }
    assert!(sternenepoche_agenten::export::export(&dir, &out).is_err());
}

#[test]
fn local_http_transport_and_redirect_refusal() {
    use std::{
        io::{Read, Write},
        net::TcpListener,
    };
    for status in [200, 302] {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            socket
                .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                .unwrap();
            let mut all = Vec::new();
            let mut buf = [0; 1024];
            loop {
                let n = socket.read(&mut buf).unwrap();
                if n == 0 {
                    break;
                }
                all.extend_from_slice(&buf[..n]);
                if let Some(end) = all.windows(4).position(|x| x == b"\r\n\r\n") {
                    let header = String::from_utf8_lossy(&all[..end]).to_lowercase();
                    let length = header
                        .lines()
                        .find_map(|s| s.strip_prefix("content-length: "))
                        .unwrap()
                        .parse::<usize>()
                        .unwrap();
                    if all.len() >= end + 4 + length {
                        break;
                    }
                }
            }
            assert!(String::from_utf8_lossy(&all).starts_with("POST /v1/chat/completions"));
            let body=json!({"choices":[{"message":{"content":"{\"ok\":true}"},"finish_reason":"stop"}],"model":"mock-http","usage":{"total_tokens":5}}).to_string();
            write!(socket,"HTTP/1.1 {status} Test\r\nContent-Length: {}\r\nLocation: http://127.0.0.1:1/forbidden\r\nConnection: close\r\n\r\n{body}",body.len()).unwrap();
        });
        let mut p = Config::demo(1).anbieter["offline"].clone();
        p.art = "local".into();
        p.url = Some(format!("http://{address}/v1"));
        let result =
            sternenepoche_agenten::provider::call(&p, &[json!({"role":"user","content":"test"})]);
        if status == 200 {
            assert_eq!(result.unwrap()["usage"]["total_tokens"], 5);
        } else {
            assert!(result.unwrap_err().to_string().contains("302"));
        }
        server.join().unwrap();
    }
}

#[test]
fn openrouter_body_carries_schema_reasoning_and_provider_rules() {
    use sternenepoche_agenten::provider::koerper;
    let mut p = Config::demo(1).anbieter["offline"].clone();
    p.art = "openrouter".into();
    p.schema = true;
    p.denken = Some("mittel".into());
    p.provider = Some(json!({"order": ["deepinfra"], "data_collection": "deny"}));
    let schema = json!({"type": "object"});
    let m = [json!({"role": "user", "content": "x"})];
    let b = koerper(&p, &m, Some(&schema), false);
    assert_eq!(b["response_format"]["type"], "json_schema");
    assert_eq!(b["response_format"]["json_schema"]["strict"], true);
    assert_eq!(b["response_format"]["json_schema"]["schema"], schema);
    assert_eq!(b["provider"], json!({"allow_fallbacks": false, "require_parameters": true, "order": ["deepinfra"], "data_collection": "deny"}));
    assert_eq!(b["reasoning"], json!({"effort": "medium"}));
    assert_eq!(b["max_tokens"], 1024);
    // After an answer ended at the token limit: twice the room, lowest reasoning level.
    let b = koerper(&p, &m, Some(&schema), true);
    assert_eq!((b["max_tokens"].clone(), b["reasoning"].clone()), (json!(2048), json!({"effort": "low"})));
    // Reasoning switched off stays off.
    p.denken = Some("aus".into());
    assert_eq!(koerper(&p, &m, Some(&schema), true)["reasoning"], json!({"effort": "none"}));
    // Without schema in the config only json_object; local servers get no OpenRouter fields.
    p.schema = false;
    assert_eq!(koerper(&p, &m, Some(&schema), false)["response_format"], json!({"type": "json_object"}));
    p.art = "local".into();
    let b = koerper(&p, &m, Some(&schema), false);
    assert!(b.get("provider").is_none() && b.get("reasoning").is_none());
    // Unknown reasoning levels are rejected by the configuration.
    p.denken = Some("budget".into());
    assert!(p.validate().unwrap_err().contains("denken"));
}
