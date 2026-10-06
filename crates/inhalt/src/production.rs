//! Explicit local-only ComfyUI execution. Preparing workflows never uses the network.
//! A persisted submitting record deliberately blocks retry after an ambiguous POST.
use crate::catalog::{digest, read_json, safe_relative, Asset, Catalog};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    net::TcpStream,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProductionConfig {
    pub gpu_released: bool,
    #[serde(default)]
    pub release_note: Option<String>,
    #[serde(default)]
    pub reason: String,
    pub comfy_url: String,
    pub baseline: PathBuf,
    #[serde(default)]
    pub external_reservation: Option<PathBuf>,
}
impl ProductionConfig {
    pub fn require_release(&self) -> Result<(), String> {
        if !self.gpu_released {
            return Err(format!("GPU nicht freigegeben: {}", self.reason));
        }
        if self.release_note.as_deref().unwrap_or("").trim().is_empty() {
            return Err("Explizite Nutzerfreigabe als release_note erforderlich".into());
        }
        local_endpoint(&self.comfy_url)?;
        Ok(())
    }
}
pub fn make_graph(baseline: &Value, a: &Asset, pilot: bool) -> Result<Value, String> {
    if a.gate["generator"] == "image_gen" { return Err("External image_gen asset: no ComfyUI workflow".into()); }
    let mut graph = baseline.clone();
    for (id, class) in [
        ("1", "UNETLoader"),
        ("2", "CLIPLoader"),
        ("3", "VAELoader"),
        ("4", "TextEncodeQwenImage21"),
        ("5", "QwenImage21Cache"),
        ("6", "EmptyLatentImage"),
        ("7", "KSampler"),
        ("8", "VAEDecode"),
        ("9", "SaveImage"),
    ] {
        if graph[id]["class_type"].as_str() != Some(class) {
            return Err(format!("Unexpected baseline node {id}: expected {class}"));
        }
    }
    let (mut w, mut h) = (a.width, a.height);
    if pilot {
        let scale = 512. / w.max(h) as f64;
        w = ((w as f64 * scale / 64.).round() as u32 * 64).max(64);
        h = ((h as f64 * scale / 64.).round() as u32 * 64).max(64);
    }
    graph["4"]["inputs"]["prompt"] = json!(a.prompt);
    graph["4"]["inputs"]["negative_prompt"] = json!(a.negative_prompt);
    graph["4"]["inputs"]["resolution"] = json!(w.min(h));
    graph["6"]["inputs"]["width"] = json!(w);
    graph["6"]["inputs"]["height"] = json!(h);
    graph["6"]["inputs"]["batch_size"] = json!(1);
    graph["7"]["inputs"]["seed"] = json!(a.seed);
    graph["7"]["inputs"]["steps"] = json!(25);
    graph["7"]["inputs"]["cfg"] = json!(1.0);
    graph["9"]["inputs"]["filename_prefix"] = json!(format!(
        "sternenepoche/{}/{}",
        if pilot { "pilot" } else { "final" },
        a.id
    ));
    Ok(graph)
}
pub fn prepare(content: &Path) -> Result<usize, String> {
    let catalog = Catalog::load(content)?;
    catalog.validate()?;
    let config: ProductionConfig = read_json(&content.join("production.json"))?;
    let baseline: Value = read_json(&config.baseline)?;
    for a in &catalog.assets {
        if a.gate["generator"] == "image_gen" { continue; }
        for profile in ["pilot", "final"] {
            save_json(
                &content
                    .join("workflows")
                    .join(profile)
                    .join(format!("{}.json", a.id)),
                &make_graph(&baseline, a, profile == "pilot")?,
            )?;
        }
    }
    Ok(catalog.assets.len())
}
fn nonce() -> String {
    format!(
        "{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    )
}
pub fn save_json(path: &Path, data: &Value) -> Result<(), String> {
    let parent = path.parent().ok_or("Missing parent path")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let temp = parent.join(format!(
        ".{}.{}.tmp",
        path.file_name().unwrap().to_string_lossy(),
        nonce()
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
        .map_err(|e| e.to_string())?;
    file.write_all(&serde_json::to_vec_pretty(data).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    drop(file);
    fs::rename(&temp, path).map_err(|e| format!("Atomic save {}: {e}", path.display()))
}
struct Lock(PathBuf);
impl Lock {
    fn acquire(path: PathBuf) -> Result<Self, String> {
        let mut f = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|e| {
                format!(
                    "Batch lock {}: {e}. Existing lock needs explicit crash review.",
                    path.display()
                )
            })?;
        f.write_all(nonce().as_bytes()).map_err(|e| e.to_string())?;
        f.sync_all().map_err(|e| e.to_string())?;
        Ok(Self(path))
    }
}
impl Drop for Lock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
#[derive(Debug, PartialEq, Eq)]
pub enum BatchOutcome {
    Completed { asset: String, file: PathBuf },
    Pending { prompt_id: String },
    Busy,
    AlreadyComplete,
}

/// Advances exactly one asset one step; never waits or enqueues an entire batch.
/// Call repeatedly only while the user-approved GPU release remains in force.
pub fn execute_one(content: &Path, asset_id: &str, profile: &str) -> Result<BatchOutcome, String> {
    if !["pilot", "final"].contains(&profile) {
        return Err("Profile must be pilot or final".into());
    }
    let config: ProductionConfig = read_json(&content.join("production.json"))?;
    config.require_release()?;
    let catalog = Catalog::load(content)?;
    catalog.validate()?;
    let asset = catalog
        .asset(asset_id)
        .ok_or_else(|| format!("Unknown asset {asset_id}"))?;
    if asset.gate["generator"] == "image_gen" { return Err("External image_gen asset: regenerate with its recorded generator".into()); }
    let _lock = Lock::acquire(content.join(".batch.lock"))?;
    let state_path = content.join("batch-state.json");
    let mut state: Value = if state_path.exists() {
        read_json(&state_path)?
    } else {
        json!({"version":2,"jobs":{}})
    };
    if !state["jobs"].is_object() {
        return Err("Invalid batch journal".into());
    }
    let graph: Value = read_json(
        &content
            .join("workflows")
            .join(profile)
            .join(format!("{asset_id}.json")),
    )?;
    let fingerprint = digest(&serde_json::to_vec(&graph).map_err(|e| e.to_string())?);
    let key = format!("{profile}:{asset_id}");
    let mut entry = state["jobs"][&key].clone();
    if !entry.is_null() {
        if entry["workflow_rust_sha256"].as_str() != Some(&fingerprint) {
            return Err("Existing job has a different workflow or legacy fingerprint; explicit reconciliation required, no resubmit".into());
        }
        if entry["status"] == "complete" {
            let output = contained(
                content,
                entry["file"].as_str().ok_or("Missing completed file")?,
            )?;
            let bytes = fs::read(&output).map_err(|e| e.to_string())?;
            if entry["sha256"].as_str() != Some(&digest(&bytes)) {
                return Err("Completed image changed; no resubmit".into());
            }
            return Ok(BatchOutcome::AlreadyComplete);
        }
        if entry["status"] == "failed" || entry["status"] == "rejected" {
            return Err("Previous job failed or was rejected; explicit review required".into());
        }
        if entry["prompt_id"].as_str().is_none() {
            return Err(
                "Ambiguous previous POST: reconcile client_id in ComfyUI; never blindly retry"
                    .into(),
            );
        }
    } else {
        let queue = api_json(&config, "/queue", None)?;
        if !queue["queue_running"].is_array() || !queue["queue_pending"].is_array() {
            return Err("Invalid ComfyUI queue response".into());
        }
        if !queue["queue_running"].as_array().unwrap().is_empty()
            || !queue["queue_pending"].as_array().unwrap().is_empty()
        {
            return Ok(BatchOutcome::Busy);
        }
        // Re-read after queue probe, so revocation takes effect before submission.
        let live: ProductionConfig = read_json(&content.join("production.json"))?;
        live.require_release()?;
        if live.comfy_url != config.comfy_url {
            return Err("ComfyUI configuration changed during execution".into());
        }
        entry = json!({"status":"submitting","client_id":format!("sternenepoche-rust-{}",nonce()),"workflow_rust_sha256":fingerprint,"asset":asset_id,"profile":profile});
        state["jobs"][&key] = entry.clone();
        save_json(&state_path, &state)?;
        let response = api_json(
            &config,
            "/prompt",
            Some(&json!({"prompt":graph,"client_id":entry["client_id"]})),
        )?;
        if response["prompt_id"].as_str().is_none()
            || response
                .get("node_errors")
                .is_some_and(|v| v.as_object().is_some_and(|m| !m.is_empty()))
        {
            entry["status"] = json!("rejected");
            entry["response"] = response;
            state["jobs"][&key] = entry;
            save_json(&state_path, &state)?;
            return Err("ComfyUI rejected workflow; no retry".into());
        }
        entry["prompt_id"] = response["prompt_id"].clone();
        entry["status"] = json!("queued");
        state["jobs"][&key] = entry.clone();
        save_json(&state_path, &state)?;
    }
    let prompt_id = entry["prompt_id"].as_str().unwrap().to_owned();
    if !prompt_id
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'-')
    {
        return Err("Invalid prompt_id".into());
    }
    let history = api_json(&config, &format!("/history/{prompt_id}"), None)?;
    let job = &history[&prompt_id];
    if job["status"]["status_str"] == "error" {
        entry["status"] = json!("failed");
        entry["history"] = job.clone();
        state["jobs"][&key] = entry;
        save_json(&state_path, &state)?;
        return Err("ComfyUI generation failed; no automatic retry".into());
    }
    if job["status"]["completed"] != true {
        return Ok(BatchOutcome::Pending { prompt_id });
    }
    let images = job["outputs"]["9"]["images"]
        .as_array()
        .ok_or("Missing SaveImage result")?;
    if images.len() != 1 {
        return Err("Expected one generated image".into());
    }
    let image = &images[0];
    let mut fields = Vec::new();
    for name in ["filename", "subfolder", "type"] {
        let v = image[name].as_str().ok_or("Missing image descriptor")?;
        fields.push(format!("{name}={}", percent(v)));
    }
    let bytes = http(
        &config.comfy_url,
        &format!("/view?{}", fields.join("&")),
        None,
    )?;
    let (w, h) = png_dimensions(&bytes)?;
    if Some(w as u64) != graph["6"]["inputs"]["width"].as_u64()
        || Some(h as u64) != graph["6"]["inputs"]["height"].as_u64()
    {
        return Err("Unexpected image dimensions".into());
    }
    let relative = if profile == "final" {
        asset.file.clone()
    } else {
        format!("pilots/{asset_id}.png")
    };
    let output = contained(content, &relative)?;
    fs::create_dir_all(output.parent().unwrap()).map_err(|e| e.to_string())?;
    if output.exists() {
        if fs::read(&output).map_err(|e| e.to_string())? != bytes {
            return Err("Existing image will not be overwritten".into());
        }
    } else {
        let mut f = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&output)
            .map_err(|e| e.to_string())?;
        f.write_all(&bytes).map_err(|e| e.to_string())?;
        f.sync_all().map_err(|e| e.to_string())?;
    }
    entry["status"] = json!("complete");
    entry["file"] = json!(relative);
    entry["sha256"] = json!(digest(&bytes));
    entry["reviewed"] = json!(false);
    let mut provenance = entry.clone();
    provenance["workflow"] = graph;
    provenance["comfy_image"] = image.clone();
    save_json(&output.with_extension("provenance.json"), &provenance)?;
    state["jobs"][&key] = entry;
    save_json(&state_path, &state)?;
    Ok(BatchOutcome::Completed {
        asset: asset_id.into(),
        file: output,
    })
}
fn contained(root: &Path, name: &str) -> Result<PathBuf, String> {
    let relative = safe_relative(name)?;
    let root = fs::canonicalize(root).map_err(|e| e.to_string())?;
    let path = root.join(relative);
    // Reject existing symlink/junction escapes, including ancestors of a new file.
    let mut ancestor = path.as_path();
    while !ancestor.exists() {
        ancestor = ancestor.parent().ok_or("Missing root")?;
    }
    if !fs::canonicalize(ancestor)
        .map_err(|e| e.to_string())?
        .starts_with(&root)
    {
        return Err("Asset path leaves content root".into());
    }
    Ok(path)
}
pub fn png_dimensions(bytes: &[u8]) -> Result<(u32, u32), String> {
    if bytes.len() < 24 || &bytes[..8] != b"\x89PNG\r\n\x1a\n" || &bytes[12..16] != b"IHDR" {
        return Err("Response is not PNG".into());
    }
    Ok((
        u32::from_be_bytes(bytes[16..20].try_into().unwrap()),
        u32::from_be_bytes(bytes[20..24].try_into().unwrap()),
    ))
}
fn percent(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b"-._~".contains(&b) {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}
fn local_endpoint(url: &str) -> Result<(String, u16), String> {
    let authority = url
        .strip_prefix("http://")
        .ok_or("Only local HTTP ComfyUI permitted")?
        .trim_end_matches('/');
    let (host, port) = authority
        .rsplit_once(':')
        .ok_or("Explicit local port required")?;
    if !["127.0.0.1", "localhost", "[::1]"].contains(&host) {
        return Err("Only local loopback ComfyUI permitted".into());
    }
    Ok((
        host.trim_matches(['[', ']']).into(),
        port.parse::<u16>().map_err(|_| "Invalid port")?,
    ))
}
fn api_json(c: &ProductionConfig, path: &str, body: Option<&Value>) -> Result<Value, String> {
    serde_json::from_slice(&http(&c.comfy_url, path, body)?).map_err(|e| e.to_string())
}
fn http(url: &str, path: &str, body: Option<&Value>) -> Result<Vec<u8>, String> {
    let (host, port) = local_endpoint(url)?;
    if !path.starts_with('/') || path.contains(['\r', '\n']) {
        return Err("Invalid HTTP path".into());
    }
    // Resolve localhost ourselves: never trust DNS or an HTTP proxy for model submissions.
    let ip = if host == "::1" {
        std::net::IpAddr::V6(std::net::Ipv6Addr::LOCALHOST)
    } else {
        std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)
    };
    let mut stream = TcpStream::connect_timeout(
        &std::net::SocketAddr::new(ip, port),
        Duration::from_secs(10),
    )
    .map_err(|e| e.to_string())?;
    stream
        .set_read_timeout(Some(Duration::from_secs(30)))
        .map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(Duration::from_secs(30)))
        .map_err(|e| e.to_string())?;
    let payload = body
        .map(serde_json::to_vec)
        .transpose()
        .map_err(|e| e.to_string())?
        .unwrap_or_default();
    let request=format!("{} {path} HTTP/1.1\r\nHost: {host}:{port}\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",if body.is_some(){"POST"}else{"GET"},payload.len());
    stream
        .write_all(request.as_bytes())
        .and_then(|_| stream.write_all(&payload))
        .map_err(|e| e.to_string())?;
    let mut response = Vec::new();
    stream
        .take(128 * 1024 * 1024 + 1)
        .read_to_end(&mut response)
        .map_err(|e| e.to_string())?;
    if response.len() > 128 * 1024 * 1024 {
        return Err("HTTP result exceeds 128 MiB".into());
    }
    let split = response
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or("Truncated HTTP response")?;
    let headers = std::str::from_utf8(&response[..split]).map_err(|e| e.to_string())?;
    let status = headers
        .lines()
        .next()
        .and_then(|s| s.split_whitespace().nth(1))
        .and_then(|s| s.parse::<u16>().ok())
        .ok_or("Missing HTTP status")?;
    if !(200..300).contains(&status) {
        return Err(format!("ComfyUI HTTP status {status}; no retry"));
    }
    let data = &response[split + 4..];
    if headers.lines().any(|s| {
        s.to_ascii_lowercase()
            .starts_with("transfer-encoding: chunked")
    }) {
        return decode_chunked(data);
    }
    if let Some(n) = headers.lines().find_map(|s| {
        s.to_ascii_lowercase()
            .strip_prefix("content-length:")
            .and_then(|v| v.trim().parse::<usize>().ok())
    }) {
        if n != data.len() {
            return Err("Truncated HTTP body".into());
        }
    }
    Ok(data.to_vec())
}
fn decode_chunked(mut data: &[u8]) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    loop {
        let end = data
            .windows(2)
            .position(|w| w == b"\r\n")
            .ok_or("Malformed chunk")?;
        let length = std::str::from_utf8(&data[..end])
            .map_err(|e| e.to_string())?
            .split(';')
            .next()
            .unwrap();
        let n = usize::from_str_radix(length, 16).map_err(|e| e.to_string())?;
        data = &data[end + 2..];
        if n == 0 {
            return Ok(out);
        }
        if n > data.len().saturating_sub(2) || &data[n..n + 2] != b"\r\n" {
            return Err("Truncated chunk".into());
        }
        out.extend_from_slice(&data[..n]);
        data = &data[n + 2..];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(url: &str) -> (PathBuf, String) {
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content");
        let root = source.join("test-output").join(nonce());
        fs::create_dir_all(&root).unwrap();
        fs::copy(source.join("catalog.json"), root.join("catalog.json")).unwrap();
        let asset = Catalog::load(&root).unwrap().assets[0].id.clone();
        fs::create_dir_all(root.join("workflows/pilot")).unwrap();
        fs::copy(
            source.join("workflows/pilot").join(format!("{asset}.json")),
            root.join("workflows/pilot").join(format!("{asset}.json")),
        )
        .unwrap();
        save_json(&root.join("production.json"),&json!({"gpu_released":true,"release_note":"Isolated mock HTTP test; no real models","comfy_url":url,"baseline":"unused"})).unwrap();
        (root, asset)
    }
    fn request(stream: &mut TcpStream) -> String {
        let mut data = Vec::new();
        let mut byte = [0];
        while !data.ends_with(b"\r\n\r\n") {
            stream.read_exact(&mut byte).unwrap();
            data.push(byte[0]);
        }
        let headers = String::from_utf8(data.clone()).unwrap();
        let n = headers
            .lines()
            .find_map(|s| {
                s.strip_prefix("Content-Length: ")
                    .and_then(|s| s.parse::<usize>().ok())
            })
            .unwrap();
        let mut body = vec![0; n];
        stream.read_exact(&mut body).unwrap();
        data.extend(body);
        String::from_utf8(data).unwrap()
    }
    /// Waits at most 20 s for the client. If execute_one fails before connecting (e.g. catalog
    /// validation), the server thread panics with a message instead of blocking join() forever.
    fn accept(listener: &std::net::TcpListener) -> TcpStream {
        listener.set_nonblocking(true).unwrap();
        let deadline = std::time::Instant::now() + Duration::from_secs(20);
        loop {
            match listener.accept() {
                Ok((s, _)) => {
                    s.set_nonblocking(false).unwrap();
                    return s;
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock && std::time::Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(10))
                }
                Err(e) => panic!("mock server got no connection: {e}"),
            }
        }
    }
    fn reply(stream: &mut TcpStream, body: &str) {
        write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
        .unwrap();
    }
    #[test]
    fn occupied_queue_never_submits() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let (root, asset) = fixture(&url);
        let server = std::thread::spawn(move || {
            let mut s = accept(&listener);
            assert!(request(&mut s).starts_with("GET /queue "));
            reply(&mut s, r#"{"queue_running":[[1]],"queue_pending":[]}"#);
        });
        assert_eq!(
            execute_one(&root, &asset, "pilot").unwrap(),
            BatchOutcome::Busy
        );
        assert!(!root.join("batch-state.json").exists());
        server.join().unwrap();
    }
    #[test]
    fn lost_post_response_is_persisted_and_not_retried() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let (root, asset) = fixture(&url);
        let server = std::thread::spawn(move || {
            let mut s = accept(&listener);
            request(&mut s);
            reply(&mut s, r#"{"queue_running":[],"queue_pending":[]}"#);
            drop(s);
            let mut s = accept(&listener);
            assert!(request(&mut s).starts_with("POST /prompt ")); /* Simulate accepted request with lost reply. */
        });
        assert!(execute_one(&root, &asset, "pilot").is_err());
        server.join().unwrap();
        let state: Value = read_json(&root.join("batch-state.json")).unwrap();
        assert_eq!(
            state["jobs"][format!("pilot:{asset}")]["status"],
            "submitting"
        );
        assert!(execute_one(&root, &asset, "pilot")
            .unwrap_err()
            .contains("Ambiguous previous POST"));
    }
    #[test]
    fn queued_job_resumes_history_without_second_post() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let (root, asset) = fixture(&url);
        let server = std::thread::spawn(move || {
            for (method, body) in [
                ("GET /queue ", r#"{"queue_running":[],"queue_pending":[]}"#),
                (
                    "POST /prompt ",
                    r#"{"prompt_id":"test-id","node_errors":{}}"#,
                ),
                ("GET /history/test-id ", "{}"),
                ("GET /history/test-id ", "{}"),
            ] {
                let mut s = accept(&listener);
                assert!(request(&mut s).starts_with(method));
                reply(&mut s, body);
            }
        });
        assert_eq!(
            execute_one(&root, &asset, "pilot").unwrap(),
            BatchOutcome::Pending {
                prompt_id: "test-id".into()
            }
        );
        assert_eq!(
            execute_one(&root, &asset, "pilot").unwrap(),
            BatchOutcome::Pending {
                prompt_id: "test-id".into()
            }
        );
        server.join().unwrap();
    }
    #[test]
    fn release_required_before_network() {
        let mut c: ProductionConfig =
            read_json(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content/production.json"))
                .unwrap();
        c.gpu_released = false;
        assert!(c.require_release().is_err());
    }
    #[test]
    fn remote_endpoints_rejected() {
        for s in [
            "https://127.0.0.1:8189",
            "http://example.com:8189",
            "http://127.0.0.1.evil:80",
            "http://localhost:8189/path",
        ] {
            assert!(local_endpoint(s).is_err());
        }
    }
    #[test]
    fn chunked_http_decodes() {
        assert_eq!(decode_chunked(b"4\r\ntest\r\n0\r\n\r\n").unwrap(), b"test");
        assert!(decode_chunked(b"a\r\ntest\r\n").is_err());
    }
    #[test]
    fn png_header_round_trip() {
        let img = crate::planet::render_orbit(32, &Default::default()).unwrap();
        assert_eq!(png_dimensions(&img.png().unwrap()).unwrap(), (32, 32));
    }
}
