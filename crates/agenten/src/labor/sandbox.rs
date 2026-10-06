//! Docker is the executable-code boundary. Trusted mode has no command executor.
use super::{config::identifier, Result};
use serde_json::{json, Value};
use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    path::{Component, PathBuf, Prefix},
    process::{Child, Command, Stdio},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc, Arc,
    },
    thread,
    time::{Duration, Instant},
};

#[derive(Clone, Debug)]
pub struct Limits {
    pub memory_bytes: u64,
    pub cpus: f64,
    pub pids: u32,
    pub timeout_seconds: u64,
    pub output_bytes: usize,
    pub input_bytes: usize,
    pub workspace_bytes: u64,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            memory_bytes: 134_217_728,
            cpus: 0.5,
            pids: 32,
            timeout_seconds: 10,
            output_bytes: 2_097_152,
            input_bytes: 20_971_520,
            workspace_bytes: 16_777_216,
        }
    }
}
#[derive(Clone, Debug)]
pub struct Execution {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub elapsed_ms: u64,
}
pub struct DockerSandbox {
    image: String,
    limits: Limits,
    namespace: String,
}
static SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn command_output(args: &[&str]) -> Result<String> {
    // All host invocations are fixed Docker CLI arguments, never shell code.
    let mut child = Command::new("docker")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Docker CLI: {e}"))?;
    let start = Instant::now();
    loop {
        if child.try_wait().map_err(|e| e.to_string())?.is_some() {
            break;
        }
        if start.elapsed() > Duration::from_secs(20) {
            let _ = child.kill();
            let _ = child.wait();
            return Err("Docker Kontrollaufruf abgelaufen".into());
        }
        thread::sleep(Duration::from_millis(20));
    }
    let o = child.wait_with_output().map_err(|e| e.to_string())?;
    if !o.status.success() {
        return Err(format!("Docker: {}", String::from_utf8_lossy(&o.stderr)));
    }
    Ok(String::from_utf8_lossy(&o.stdout).trim().to_string())
}
impl DockerSandbox {
    pub fn new(image: &str, limits: Limits) -> Result<Self> {
        Self::with_workspace(image, limits, "ephemeral")
    }
    pub fn with_workspace(image: &str, limits: Limits, namespace: &str) -> Result<Self> {
        if !identifier(namespace)
            || image.is_empty()
            || image.starts_with('-')
            || image.bytes().any(|b| b.is_ascii_whitespace())
        {
            return Err("Ungültiger Sandbox-Namensraum oder Image".into());
        }
        if limits.memory_bytes < 16_777_216
            || !limits.cpus.is_finite()
            || limits.cpus <= 0.
            || limits.pids == 0
            || limits.timeout_seconds == 0
            || limits.output_bytes == 0
            || limits.workspace_bytes == 0
        {
            return Err("Ungültige Sandbox-Grenzen".into());
        }
        if command_output(&["info", "--format", "{{.OSType}}"])? != "linux" {
            return Err("Linux Docker erforderlich".into());
        }
        let pinned = command_output(&["image", "inspect", image, "--format", "{{.Id}}"])?;
        if !pinned.starts_with("sha256:") {
            return Err("Image nicht lokal und unveränderlich auflösbar".into());
        }
        Ok(Self {
            image: pinned,
            limits,
            namespace: namespace.into(),
        })
    }
    fn workspace_root() -> Result<PathBuf> {
        let root = std::env::var_os("STERNENEPOCHE_SANDBOX_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../laeufe/sandbox-workspaces")
            });
        if !root.is_absolute() || !on_d(&root) {
            return Err("Spielerbüros müssen auf lokalem D: liegen".into());
        }
        fs::create_dir_all(&root).map_err(|e| e.to_string())?;
        let canonical = fs::canonicalize(root).map_err(|e| e.to_string())?;
        if !on_d(&canonical) {
            return Err("Spielerbüro-Root verweist nicht auf D:".into());
        }
        Ok(canonical)
    }
    fn workspace(&self, owner: u16) -> Result<PathBuf> {
        let root = Self::workspace_root()?;
        let candidate = root.join(&self.namespace).join(format!("p{owner}"));
        fs::create_dir_all(&candidate).map_err(|e| e.to_string())?;
        let path = fs::canonicalize(candidate).map_err(|e| e.to_string())?;
        if !path.starts_with(&root) || path == root || !on_d(&path) {
            return Err("Spielerbüro verlässt erlaubten Root".into());
        }
        let exact = root.join(&self.namespace).join(format!("p{owner}"));
        if path != exact {
            return Err("Spielerbüro enthält eine Pfadumleitung".into());
        }
        Ok(path)
    }
    /// Fresh ephemeral container; a player's durable volume is only attached for worker RPC.
    pub fn execute(&self, owner: u16, argv: &[String], input: &[u8]) -> Result<Execution> {
        self.run(owner, argv, input, false)
    }
    fn run(
        &self,
        owner: u16,
        argv: &[String],
        input: &[u8],
        persistent: bool,
    ) -> Result<Execution> {
        if argv.is_empty() || input.len() > self.limits.input_bytes {
            return Err("Leerer Befehl oder Sandbox-Eingabe zu groß".into());
        }
        let name = container_name(owner);
        let args = self.command_args(owner, argv, &name, persistent)?;
        let mut child = Command::new("docker")
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| e.to_string())?;
        let mut stdin = child.stdin.take().unwrap();
        let payload = input.to_vec();
        let writer = thread::spawn(move || stdin.write_all(&payload));
        let exceeded = Arc::new(AtomicBool::new(false));
        let read_limited =
            |mut stream: Box<dyn Read + Send>, flag: Arc<AtomicBool>, limit: usize| {
                thread::spawn(move || {
                    let mut output = Vec::new();
                    let mut buf = [0; 8192];
                    loop {
                        match stream.read(&mut buf) {
                            Ok(0) | Err(_) => break,
                            Ok(n) => {
                                let keep = n.min(limit.saturating_sub(output.len()));
                                output.extend_from_slice(&buf[..keep]);
                                if keep < n {
                                    flag.store(true, Ordering::SeqCst);
                                    break;
                                }
                            }
                        }
                    }
                    output
                })
            };
        let stdout = read_limited(
            Box::new(child.stdout.take().unwrap()),
            exceeded.clone(),
            self.limits.output_bytes,
        );
        let stderr = read_limited(
            Box::new(child.stderr.take().unwrap()),
            exceeded.clone(),
            self.limits.output_bytes,
        );
        let start = Instant::now();
        let mut failed = None;
        let status = loop {
            if exceeded.load(Ordering::SeqCst) {
                failed = Some("Sandbox-Ausgabegrenze überschritten");
                break None;
            }
            if start.elapsed() > Duration::from_secs(self.limits.timeout_seconds) {
                failed = Some("Sandbox-Laufzeit überschritten");
                break None;
            }
            if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
                break Some(status);
            }
            thread::sleep(Duration::from_millis(20));
        };
        if failed.is_some() {
            let _ = command_output(&["rm", "-f", &name]);
            let _ = child.kill();
            let _ = child.wait();
        }
        let _ = writer.join();
        let out = stdout.join().map_err(|_| "Sandbox stdout reader")?;
        let err = stderr.join().map_err(|_| "Sandbox stderr reader")?;
        if let Some(error) = failed {
            return Err(error.into());
        }
        if exceeded.load(Ordering::SeqCst) {
            return Err("Sandbox-Ausgabegrenze überschritten".into());
        }
        Ok(Execution {
            stdout: String::from_utf8_lossy(&out).into(),
            stderr: String::from_utf8_lossy(&err).into(),
            exit_code: status.and_then(|s| s.code()).unwrap_or(-1),
            elapsed_ms: start.elapsed().as_millis() as u64,
        })
    }
    fn command_args(
        &self,
        owner: u16,
        argv: &[String],
        name: &str,
        persistent: bool,
    ) -> Result<Vec<String>> {
        let mut args = vec![
            "run".into(),
            "--rm".into(),
            "-i".into(),
            "--pull=never".into(),
            "--name".into(),
            name.to_string(),
            "--network=none".into(),
            "--read-only".into(),
            "--cap-drop=ALL".into(),
            "--security-opt=no-new-privileges:true".into(),
            "--user=65534:65534".into(),
            format!("--pids-limit={}", self.limits.pids),
            format!("--memory={}", self.limits.memory_bytes),
            format!("--memory-swap={}", self.limits.memory_bytes),
            format!("--cpus={}", self.limits.cpus),
            "--ulimit=nofile=64:64".into(),
            "--ulimit=fsize=29360128:29360128".into(),
            "--workdir=/workspace".into(),
            "--tmpfs=/tmp:rw,noexec,nosuid,nodev,size=8388608,mode=1777".into(),
            "--entrypoint".into(),
            argv[0].clone(),
        ];
        if persistent {
            let path = self.workspace(owner)?;
            // Docker's Windows bind parser needs an ordinary drive path, not \\?\.
            let ordinary = path
                .to_string_lossy()
                .trim_start_matches("\\\\?\\")
                .to_string();
            if ordinary.contains([',', '\n', '\r']) {
                return Err("Nicht darstellbarer Spielerbüro-Pfad".into());
            }
            args.extend([
                "--mount".into(),
                format!("type=bind,src={ordinary},dst=/workspace"),
            ]);
        } else {
            args.push(format!(
                "--tmpfs=/workspace:rw,noexec,nosuid,nodev,size={},mode=1777",
                self.limits.workspace_bytes
            ));
        }
        args.push(self.image.clone());
        args.extend_from_slice(&argv[1..]);
        Ok(args)
    }
    pub fn memory(&self, owner: u16, quota: u64, request: &Value) -> Result<Value> {
        let input = json!({"owner":owner,"quota":quota,"request":request}).to_string();
        let e = self.run(
            owner,
            &["/usr/local/bin/sternenepoche-worker".into()],
            input.as_bytes(),
            request["op"] != "probe",
        )?;
        if e.exit_code != 0 {
            return Err(format!("Worker exit {}: {}", e.exit_code, e.stderr));
        }
        let response = crate::protocol::strict_json(&e.stdout)?;
        if let Some(error) = response["error"].as_str() {
            return Err(error.into());
        }
        response
            .get("result")
            .cloned()
            .ok_or("Worker ohne Ergebnis".into())
    }
    pub fn destroy(&self, owner: u16) -> Result<()> {
        let root = Self::workspace_root()?;
        let path = self.workspace(owner)?;
        // Resolve and verify the exact private target immediately before recursive removal.
        if !path.starts_with(&root) || path != root.join(&self.namespace).join(format!("p{owner}"))
        {
            return Err("Fremdes Löschziel abgewiesen".into());
        }
        fs::remove_dir_all(path).map_err(|e| e.to_string())
    }
}
pub fn worker(
    image: &str,
    namespace: &str,
    owner: u16,
    quota: u64,
    request: &Value,
) -> Result<Value> {
    if request["op"] == "probe" {
        preflight(image)?;
    }
    let limits = Limits {
        output_bytes: 20_971_520,
        ..Limits::default()
    };
    DockerSandbox::with_workspace(image, limits, namespace)?.memory(owner, quota, request)
}

fn container_name(owner: u16) -> String {
    format!(
        "se-labor-{}-{owner}-{}",
        std::process::id(),
        SEQUENCE.fetch_add(1, Ordering::SeqCst)
    )
}
fn on_d(path: &std::path::Path) -> bool {
    matches!(path.components().next(),Some(Component::Prefix(p)) if matches!(p.kind(),Prefix::Disk(b'D'|b'd')|Prefix::VerbatimDisk(b'D'|b'd')))
}
fn frame(reader: &mut impl BufRead, limit: usize) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    loop {
        let buf = reader.fill_buf().map_err(|e| e.to_string())?;
        if buf.is_empty() {
            return if out.is_empty() {
                Err("Worker-Verbindung geschlossen".into())
            } else {
                Err("Unvollständige Worker-Antwort".into())
            };
        }
        let n = buf
            .iter()
            .position(|b| *b == b'\n')
            .map(|i| i + 1)
            .unwrap_or(buf.len());
        let done = buf[n - 1] == b'\n';
        if out.len() + n > limit {
            return Err("Worker-Ausgabegrenze überschritten".into());
        }
        out.extend_from_slice(&buf[..n]);
        reader.consume(n);
        if done {
            return Ok(out);
        }
    }
}
/// One live non-root container per player. Requests are sequential, replies bounded,
/// idle containers do no work; every RPC has an enforced wall-clock timeout.
pub struct WorkerSession {
    child: Child,
    name: String,
    owner: u16,
    quota: u64,
    limits: Limits,
    sender: mpsc::SyncSender<Vec<u8>>,
    receiver: mpsc::Receiver<Result<Value>>,
    failed: bool,
    stderr_overflow: Arc<AtomicBool>,
}
impl WorkerSession {
    pub fn start(image: &str, namespace: &str, owner: u16, quota: u64) -> Result<Self> {
        let limits = Limits {
            output_bytes: 20_971_520,
            ..Limits::default()
        };
        let sandbox = DockerSandbox::with_workspace(image, limits.clone(), namespace)?;
        let name = container_name(owner);
        let args = sandbox.command_args(
            owner,
            &[
                "/usr/local/bin/sternenepoche-worker".into(),
                "--serve".into(),
            ],
            &name,
            true,
        )?;
        let mut child = Command::new("docker")
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| e.to_string())?;
        let mut stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let mut stderr = child.stderr.take().unwrap();
        let (tx, rx) = mpsc::sync_channel::<Vec<u8>>(1);
        let (reply_tx, reply_rx) = mpsc::sync_channel(1);
        thread::spawn(move || {
            for payload in rx {
                if stdin
                    .write_all(&payload)
                    .and_then(|_| stdin.write_all(b"\n"))
                    .and_then(|_| stdin.flush())
                    .is_err()
                {
                    break;
                }
            }
        });
        let max = limits.output_bytes;
        thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            loop {
                let parsed = frame(&mut reader, max)
                    .and_then(|v| String::from_utf8(v).map_err(|e| e.to_string()))
                    .and_then(|s| crate::protocol::strict_json(&s));
                let stop = parsed.is_err();
                if reply_tx.send(parsed).is_err() || stop {
                    break;
                }
            }
        });
        let flag = Arc::new(AtomicBool::new(false));
        let stderr_flag = flag.clone();
        thread::spawn(move || {
            let mut seen = 0;
            let mut buf = [0; 8192];
            loop {
                match stderr.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        seen += n;
                        if seen > 65_536 {
                            stderr_flag.store(true, Ordering::SeqCst);
                            break;
                        }
                    }
                }
            }
        });
        Ok(Self {
            child,
            name,
            owner,
            quota,
            limits,
            sender: tx,
            receiver: reply_rx,
            failed: false,
            stderr_overflow: flag,
        })
    }
    fn stop(&mut self) {
        if !self.failed {
            self.failed = true;
            let _ = command_output(&["rm", "-f", &self.name]);
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
    pub fn request(&mut self, request: &Value) -> Result<Value> {
        if self.failed {
            return Err("Worker-Sitzung beendet; aus Checkpoint rekonstruieren".into());
        }
        let input = json!({"owner":self.owner,"quota":self.quota,"request":request})
            .to_string()
            .into_bytes();
        if input.len() > self.limits.input_bytes {
            return Err("Worker-Eingabe zu groß".into());
        }
        if self.stderr_overflow.load(Ordering::SeqCst) {
            self.stop();
            return Err("Worker stderr-Ausgabegrenze".into());
        }
        if self.sender.try_send(input).is_err() {
            self.stop();
            return Err("Worker-Eingang geschlossen".into());
        }
        let response = match self
            .receiver
            .recv_timeout(Duration::from_secs(self.limits.timeout_seconds))
        {
            Ok(Ok(response)) => response,
            Ok(Err(error)) => {
                self.stop();
                return Err(error);
            }
            Err(_) => {
                self.stop();
                return Err("Worker-RPC Timeout oder geschlossene Verbindung".into());
            }
        };
        if let Some(error) = response["error"].as_str() {
            return Err(error.into());
        }
        response
            .get("result")
            .cloned()
            .ok_or("Worker ohne Ergebnis".into())
    }
}
impl Drop for WorkerSession {
    fn drop(&mut self) {
        self.stop();
    }
}
pub fn preflight(image: &str) -> Result<Value> {
    let s = DockerSandbox::new(image, Limits::default())?;
    let result=s.execute(0,&["/bin/sh".into(),"-c".into(),"test \"$(id -u)\" = 65534 && test ! -e /var/run/docker.sock && test ! -e /host && ! touch /etc/sandbox-write && touch /workspace/test && grep -q 'CapEff:[[:space:]]*0000000000000000' /proc/self/status && grep -q 'NoNewPrivs:[[:space:]]*1' /proc/self/status && test \"$(ls /sys/class/net)\" = lo && printf isolation-ok".into()],b"")?;
    if result.exit_code != 0 || result.stdout != "isolation-ok" {
        return Err(format!(
            "Docker-Isolationsprobe fehlgeschlagen: {}",
            result.stderr
        ));
    }
    Ok(
        json!({"kind":"docker","image":s.image,"isolation_probe":"passed","private_workspace":"one exact player directory on D:","memory_worker":"/usr/local/bin/sternenepoche-worker"}),
    )
}
