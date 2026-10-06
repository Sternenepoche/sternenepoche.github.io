//! Native Windows LPAC + Job Object boundary, without a VM or container daemon.
use super::Result;
use serde_json::Value;
use std::path::Path;
pub fn default_worker() -> Result<std::path::PathBuf> {
    Ok(std::env::current_exe()
        .map_err(|e| e.to_string())?
        .parent()
        .ok_or("Executable ohne Elternordner")?
        .join("sternenepoche-player-worker.exe"))
}
#[cfg(not(windows))]
pub fn preflight(_: &Path) -> Result<Value> {
    Err("Native AppContainer benötigt Windows".into())
}

#[cfg(not(windows))]
pub struct NativeSession;
#[cfg(not(windows))]
impl NativeSession {
    pub fn start(_: &Path, _: &str, _: u16, _: u64) -> Result<Self> {
        Err("Native AppContainer benötigt Windows".into())
    }
    pub fn request(&mut self, _: &Value) -> Result<Value> {
        Err("Native AppContainer benötigt Windows".into())
    }
}

#[cfg(windows)]
pub use windows::{preflight, NativeSession};
#[cfg(windows)]
mod windows {
    use super::*;
    use crate::labor::config::identifier;
    use serde_json::json;
    use std::{
        fs,
        io::{BufRead, BufReader, Read, Write},
        mem::size_of,
        os::windows::{ffi::OsStrExt, io::FromRawHandle},
        path::{Component, PathBuf, Prefix},
        ptr::{null, null_mut},
        sync::{
            atomic::{AtomicBool, Ordering},
            mpsc, Arc, Mutex,
        },
        thread,
        time::Duration,
    };
    use windows_sys::Win32::{
        Foundation::*,
        Security::{Authorization::*, Isolation::*, *},
        Storage::FileSystem::{
            DELETE, FILE_GENERIC_EXECUTE, FILE_GENERIC_READ, FILE_GENERIC_WRITE, WRITE_DAC,
            WRITE_OWNER,
        },
        System::{JobObjects::*, Pipes::CreatePipe, Threading::*},
    };
    const FRAME: usize = 20_971_520;
    fn wide(s: impl AsRef<std::ffi::OsStr>) -> Vec<u16> {
        s.as_ref().encode_wide().chain(Some(0)).collect()
    }
    fn error(context: &str) -> String {
        format!("{context}: {}", std::io::Error::last_os_error())
    }
    fn ok(value: i32, context: &str) -> Result<()> {
        if value == 0 {
            Err(error(context))
        } else {
            Ok(())
        }
    }
    struct Handle(HANDLE);
    impl Handle {
        fn new(h: HANDLE, context: &str) -> Result<Self> {
            if h.is_null() || h == INVALID_HANDLE_VALUE {
                Err(error(context))
            } else {
                Ok(Self(h))
            }
        }
        fn file(self) -> fs::File {
            let h = self.0;
            std::mem::forget(self);
            unsafe { fs::File::from_raw_handle(h) }
        }
    }
    impl Drop for Handle {
        fn drop(&mut self) {
            unsafe {
                CloseHandle(self.0);
            }
        }
    }
    struct Profile {
        name: Vec<u16>,
        sid: PSID,
    }
    impl Profile {
        fn create(namespace: &str, owner: u16) -> Result<Self> {
            let digest = crate::journal::hash(format!("{namespace}:{owner}").as_bytes());
            let name = wide(format!("Sternenepoche.{}", &digest[..32]));
            let mut sid = null_mut();
            unsafe {
                let hr = CreateAppContainerProfile(
                    name.as_ptr(),
                    name.as_ptr(),
                    name.as_ptr(),
                    null(),
                    0,
                    &mut sid,
                );
                if hr < 0 {
                    if hr as u32 != 0x800700b7 {
                        return Err(format!("CreateAppContainerProfile HRESULT {hr:#x}"));
                    }
                    let derive = DeriveAppContainerSidFromAppContainerName(name.as_ptr(), &mut sid);
                    if derive < 0 {
                        return Err(format!("DeriveAppContainerSid HRESULT {derive:#x}"));
                    }
                }
            }
            Ok(Self { name, sid })
        }
    }
    impl Drop for Profile {
        fn drop(&mut self) {
            unsafe {
                DeleteAppContainerProfile(self.name.as_ptr());
                FreeSid(self.sid);
            }
        }
    }
    fn on_d(p: &Path) -> bool {
        matches!(p.components().next(),Some(Component::Prefix(p)) if matches!(p.kind(),Prefix::Disk(b'D'|b'd')|Prefix::VerbatimDisk(b'D'|b'd')))
    }
    fn no_reparse(path: &Path) -> Result<()> {
        use std::os::windows::fs::MetadataExt;
        match fs::symlink_metadata(path) {
            Ok(m) if m.file_attributes() & 0x400 != 0 => {
                Err("Sandboxpfad enthält einen Reparsepoint".into())
            }
            Ok(_) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }
    fn workspace(namespace: &str, owner: u16) -> Result<PathBuf> {
        if !identifier(namespace) {
            return Err("Ungültiger nativer Sandbox-Namensraum".into());
        }
        let root = std::env::var_os("STERNENEPOCHE_SANDBOX_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../laeufe/sandbox-workspaces")
            });
        if !root.is_absolute() || !on_d(&root) {
            return Err("Native Spielerbüros müssen auf D: liegen".into());
        }
        fs::create_dir_all(&root).map_err(|e| e.to_string())?;
        let root = fs::canonicalize(root).map_err(|e| e.to_string())?;
        if !on_d(&root) {
            return Err("Native Root-Umleitung außerhalb D:".into());
        }
        let namespace_dir = root.join(namespace);
        no_reparse(&namespace_dir)?;
        if !namespace_dir.exists() {
            fs::create_dir(&namespace_dir).map_err(|e| e.to_string())?;
        }
        let candidate = namespace_dir.join(format!("p{owner}"));
        no_reparse(&candidate)?;
        if !candidate.exists() {
            fs::create_dir(&candidate).map_err(|e| e.to_string())?;
        }
        let canonical = fs::canonicalize(&candidate).map_err(|e| e.to_string())?;
        if canonical != candidate || !canonical.starts_with(&root) {
            return Err("Native Spielerbüro enthält Pfadumleitung".into());
        }
        Ok(canonical)
    }
    pub fn preflight(worker: &Path) -> Result<Value> {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos();
        let namespace = format!("native-preflight-{}-{nonce}", std::process::id());
        let outside = workspace(&namespace, u16::MAX)?;
        let canary = outside.join("known-canary.txt");
        let denied = outside.join("denied.txt");
        fs::write(&canary, b"private peer data").map_err(|e| e.to_string())?;
        let package_canary = outside.join("all-packages-canary.txt");
        fs::write(&package_canary, b"all packages read").map_err(|e| e.to_string())?;
        unsafe {
            let mut sid = null_mut();
            ok(
                ConvertStringSidToSidW(wide("S-1-15-2-1").as_ptr(), &mut sid),
                "All packages SID",
            )?;
            let result = grant(&package_canary, sid, false);
            LocalFree(sid);
            result?;
        }
        let result = (|| -> Result<Value> {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
            let mut session = NativeSession::start(worker, &namespace, 0, 65_536)?;
            let proof=session.request(&json!({"op":"isolation_probe","forbidden_read":canary,"forbidden_write":denied,"package_read":package_canary,"network":listener.local_addr().map_err(|e|e.to_string())?.to_string(),"parent_pid":std::process::id()}))?;
            if proof["app_container"] != true
                || proof["low_privilege_app_container"] != true
                || proof["capability_count"] != 0
                || proof["own_workspace_writable"] != true
                || proof["foreign_read_error"] != 5
                || proof["foreign_write_error"] != 5
                || proof["foreign_hardlink_allowed"] != false
                || proof["worker_binary_read_allowed"] != true
                || proof["worker_binary_write_error"] != 5
                || proof["network_connected"] != false
                || ![10013, 10107].iter().any(|e| proof["network_error"] == *e)
                || proof["child_spawned"] != false
                || proof["parent_process_access"] != false
                || proof["job_active_process_limit"] != 1
                || proof["job_process_memory_limit"] != 134_217_728
                || proof["job_cpu_rate"] != 5000
                || proof["job_cpu_control_flags"] != 5
            {
                return Err(format!("Native Isolationsprüfung fehlgeschlagen: {proof}"));
            }
            let worker = fs::canonicalize(worker).map_err(|e| e.to_string())?;
            let bytes = fs::read(&worker).map_err(|e| e.to_string())?;
            Ok(
                json!({"isolation":"windows-lpac-job-object","worker":worker,"worker_sha256":crate::journal::hash(&bytes),"worker_bytes":bytes.len(),"proof":proof}),
            )
        })();
        let _ = fs::remove_file(&canary);
        let _ = fs::remove_file(&package_canary);
        let _ = fs::remove_dir(&outside);
        result
    }
    unsafe fn grant(path: &Path, sid: PSID, write: bool) -> Result<()> {
        set_acl(path, sid, write, false)
    }
    unsafe fn set_acl(path: &Path, sid: PSID, write: bool, immutable_binary: bool) -> Result<()> {
        let path = wide(path);
        let mut sd = null_mut();
        let mut old_acl = null_mut();
        let result = GetNamedSecurityInfoW(
            path.as_ptr(),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            null_mut(),
            null_mut(),
            &mut old_acl,
            null_mut(),
            &mut sd,
        );
        if result != 0 {
            return Err(format!("Read sandbox ACL: {result}"));
        }
        let mut entries = vec![EXPLICIT_ACCESS_W {
            grfAccessPermissions: if write {
                GENERIC_ALL
            } else {
                FILE_GENERIC_READ | FILE_GENERIC_EXECUTE
            },
            grfAccessMode: GRANT_ACCESS,
            grfInheritance: if write {
                SUB_CONTAINERS_AND_OBJECTS_INHERIT
            } else {
                0
            },
            Trustee: TRUSTEE_W {
                TrusteeForm: TRUSTEE_IS_SID,
                TrusteeType: TRUSTEE_IS_UNKNOWN,
                ptstrName: sid.cast(),
                ..Default::default()
            },
        }];
        if immutable_binary {
            entries.push(EXPLICIT_ACCESS_W {
                grfAccessPermissions: FILE_GENERIC_WRITE | DELETE | WRITE_DAC | WRITE_OWNER,
                grfAccessMode: DENY_ACCESS,
                grfInheritance: 0,
                Trustee: TRUSTEE_W {
                    TrusteeForm: TRUSTEE_IS_SID,
                    ptstrName: sid.cast(),
                    ..Default::default()
                },
            });
        }
        // A directory owner can update its DACL, but an inherited Modify ACE does
        // not include WRITE_OWNER, which SetNamedSecurityInfo(LABEL) requires.
        // Grant that right only to our own host identity on this private directory.
        let mut user_storage = Vec::<usize>::new();
        if write {
            let mut raw_token = null_mut();
            ok(
                OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut raw_token),
                "Host token",
            )?;
            let token = Handle::new(raw_token, "Host token")?;
            let mut length = 0;
            GetTokenInformation(token.0, TokenUser, null_mut(), 0, &mut length);
            user_storage.resize((length as usize).div_ceil(size_of::<usize>()), 0);
            ok(
                GetTokenInformation(
                    token.0,
                    TokenUser,
                    user_storage.as_mut_ptr().cast(),
                    length,
                    &mut length,
                ),
                "Host identity",
            )?;
            let user = &*(user_storage.as_ptr().cast::<TOKEN_USER>());
            entries.push(EXPLICIT_ACCESS_W {
                grfAccessPermissions: WRITE_OWNER,
                grfAccessMode: GRANT_ACCESS,
                grfInheritance: 0,
                Trustee: TRUSTEE_W {
                    TrusteeForm: TRUSTEE_IS_SID,
                    TrusteeType: TRUSTEE_IS_USER,
                    ptstrName: user.User.Sid.cast(),
                    ..Default::default()
                },
            });
        }
        let mut acl = null_mut();
        let result = SetEntriesInAclW(entries.len() as u32, entries.as_ptr(), old_acl, &mut acl);
        LocalFree(sd);
        if result != 0 {
            return Err(format!("Build sandbox ACL: {result}"));
        }
        let result = SetNamedSecurityInfoW(
            path.as_ptr(),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            null_mut(),
            null_mut(),
            acl,
            null(),
        );
        LocalFree(acl.cast());
        if result != 0 {
            return Err(format!("Set sandbox ACL: {result}"));
        }
        if write {
            let mut descriptor = null_mut();
            let sddl = wide("S:(ML;OICI;NW;;;LW)");
            ok(
                ConvertStringSecurityDescriptorToSecurityDescriptorW(
                    sddl.as_ptr(),
                    1,
                    &mut descriptor,
                    null_mut(),
                ),
                "Low integrity descriptor",
            )?;
            let mut sacl = null_mut();
            let mut present = 0;
            let mut defaulted = 0;
            let read =
                GetSecurityDescriptorSacl(descriptor, &mut present, &mut sacl, &mut defaulted);
            if read == 0 {
                LocalFree(descriptor);
                return Err(error("Low integrity label"));
            }
            let result = SetNamedSecurityInfoW(
                path.as_ptr(),
                SE_FILE_OBJECT,
                LABEL_SECURITY_INFORMATION,
                null_mut(),
                null_mut(),
                null(),
                sacl,
            );
            LocalFree(descriptor);
            if result != 0 {
                return Err(format!("Set sandbox low integrity: {result}"));
            }
        }
        Ok(())
    }
    unsafe fn pipe() -> Result<(Handle, Handle)> {
        let attributes = SECURITY_ATTRIBUTES {
            nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: null_mut(),
            bInheritHandle: 1,
        };
        let mut read = null_mut();
        let mut write = null_mut();
        ok(
            CreatePipe(&mut read, &mut write, &attributes, 0),
            "RPC pipe",
        )?;
        Ok((
            Handle::new(read, "Pipe read")?,
            Handle::new(write, "Pipe write")?,
        ))
    }
    struct Attributes {
        storage: Vec<usize>,
        ptr: LPPROC_THREAD_ATTRIBUTE_LIST,
    }
    impl Attributes {
        unsafe fn new() -> Result<Self> {
            let mut size = 0;
            InitializeProcThreadAttributeList(null_mut(), 5, 0, &mut size);
            let mut storage = vec![0usize; size.div_ceil(size_of::<usize>())];
            let ptr = storage.as_mut_ptr().cast();
            ok(
                InitializeProcThreadAttributeList(ptr, 5, 0, &mut size),
                "Process attributes",
            )?;
            Ok(Self { storage, ptr })
        }
        unsafe fn set<T>(&self, key: u32, value: &T) -> Result<()> {
            ok(
                UpdateProcThreadAttribute(
                    self.ptr,
                    0,
                    key as usize,
                    (value as *const T).cast(),
                    size_of::<T>(),
                    null_mut(),
                    null(),
                ),
                "Sandbox process attribute",
            )
        }
    }
    impl Drop for Attributes {
        fn drop(&mut self) {
            unsafe {
                DeleteProcThreadAttributeList(self.ptr);
            }
            let _ = &self.storage;
        }
    }
    fn frame(reader: &mut impl BufRead) -> Result<Vec<u8>> {
        let mut out = Vec::new();
        loop {
            let buf = reader.fill_buf().map_err(|e| e.to_string())?;
            if buf.is_empty() {
                return Err("Native Worker-Verbindung geschlossen".into());
            }
            let n = buf
                .iter()
                .position(|b| *b == b'\n')
                .map(|i| i + 1)
                .unwrap_or(buf.len());
            let done = buf[n - 1] == b'\n';
            if out.len() + n > FRAME {
                return Err("Native Worker-Ausgabegrenze".into());
            }
            out.extend_from_slice(&buf[..n]);
            reader.consume(n);
            if done {
                return Ok(out);
            }
        }
    }
    pub struct NativeSession {
        process: Handle,
        job: Handle,
        profile: Profile,
        _lock: fs::File,
        owner: u16,
        quota: u64,
        pid: u32,
        sender: mpsc::SyncSender<Vec<u8>>,
        receiver: mpsc::Receiver<Result<Value>>,
        stderr_overflow: Arc<AtomicBool>,
        stderr: Arc<Mutex<Vec<u8>>>,
        failed: bool,
        timeout: Duration,
    }
    impl NativeSession {
        pub fn start(worker: &Path, namespace: &str, owner: u16, quota: u64) -> Result<Self> {
            Self::start_with_timeout(worker, namespace, owner, quota, Duration::from_secs(10))
        }
        pub fn start_with_timeout(
            worker: &Path,
            namespace: &str,
            owner: u16,
            quota: u64,
            timeout: Duration,
        ) -> Result<Self> {
            if !(65_536..=16_777_216).contains(&quota) || timeout.is_zero() {
                return Err("Ungültiges natives Workerbudget".into());
            }
            let worker = fs::canonicalize(worker).map_err(|e| e.to_string())?;
            if !worker.is_file() || !on_d(&worker) {
                return Err("Native Workerdatei muss auf D: liegen".into());
            }
            let workspace = workspace(namespace, owner)?;
            let lock_path = workspace.join(".host-session-lock");
            no_reparse(&lock_path)?;
            let lock = fs::OpenOptions::new()
                .create(true)
                .truncate(false)
                .read(true)
                .write(true)
                .open(&lock_path)
                .map_err(|e| e.to_string())?;
            fs2::FileExt::try_lock_exclusive(&lock)
                .map_err(|_| "Spielerbüro bereits von einem Worker geöffnet")?;
            let profile = Profile::create(namespace, owner)?;
            unsafe {
                grant(&workspace, profile.sid, true)?;
                // Read/execute only the immutable host executable, using the
                // built-in restricted-package SID. Never overwrite a path in
                // the untrusted writable workspace with host permissions.
                let mut packages = null_mut();
                ok(
                    ConvertStringSidToSidW(wide("S-1-15-2-2").as_ptr(), &mut packages),
                    "Restricted packages SID",
                )?;
                let binary_acl = set_acl(&worker, packages, false, true);
                LocalFree(packages);
                binary_acl?;
                let job = Handle::new(CreateJobObjectW(null(), null()), "Create Job Object")?;
                let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
                limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
                    | JOB_OBJECT_LIMIT_ACTIVE_PROCESS
                    | JOB_OBJECT_LIMIT_PROCESS_MEMORY
                    | JOB_OBJECT_LIMIT_JOB_MEMORY;
                limits.BasicLimitInformation.ActiveProcessLimit = 1;
                limits.ProcessMemoryLimit = 134_217_728;
                limits.JobMemoryLimit = 134_217_728;
                ok(
                    SetInformationJobObject(
                        job.0,
                        JobObjectExtendedLimitInformation,
                        (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                        size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                    ),
                    "Job resource limits",
                )?;
                let mut cpu = JOBOBJECT_CPU_RATE_CONTROL_INFORMATION::default();
                cpu.ControlFlags =
                    JOB_OBJECT_CPU_RATE_CONTROL_ENABLE | JOB_OBJECT_CPU_RATE_CONTROL_HARD_CAP;
                cpu.Anonymous.CpuRate = 5000;
                ok(
                    SetInformationJobObject(
                        job.0,
                        JobObjectCpuRateControlInformation,
                        (&cpu as *const JOBOBJECT_CPU_RATE_CONTROL_INFORMATION).cast(),
                        size_of::<JOBOBJECT_CPU_RATE_CONTROL_INFORMATION>() as u32,
                    ),
                    "Job CPU cap",
                )?;
                let (child_stdin, parent_stdin) = pipe()?;
                let (parent_stdout, child_stdout) = pipe()?;
                let (parent_stderr, child_stderr) = pipe()?;
                for handle in [&parent_stdin, &parent_stdout, &parent_stderr] {
                    ok(
                        SetHandleInformation(handle.0, HANDLE_FLAG_INHERIT, 0),
                        "RPC handle inheritance",
                    )?;
                }
                let attributes = Attributes::new()?;
                let capabilities = SECURITY_CAPABILITIES {
                    AppContainerSid: profile.sid,
                    ..Default::default()
                };
                attributes.set(PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES, &capabilities)?;
                attributes.set(PROC_THREAD_ATTRIBUTE_ALL_APPLICATION_PACKAGES_POLICY, &1u32)?;
                // Job ActiveProcessLimit=1 denies children. CHILD_PROCESS_POLICY
                // breaks Windows DLL initialization on this host (0xc0000142).
                let handles = [child_stdin.0, child_stdout.0, child_stderr.0];
                attributes.set(PROC_THREAD_ATTRIBUTE_HANDLE_LIST, &handles)?;
                let jobs = [job.0];
                attributes.set(PROC_THREAD_ATTRIBUTE_JOB_LIST, &jobs)?;
                let mut startup = STARTUPINFOEXW::default();
                startup.StartupInfo.cb = size_of::<STARTUPINFOEXW>() as u32;
                startup.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
                startup.StartupInfo.hStdInput = child_stdin.0;
                startup.StartupInfo.hStdOutput = child_stdout.0;
                startup.StartupInfo.hStdError = child_stderr.0;
                startup.lpAttributeList = attributes.ptr;
                let exe = wide(&worker);
                let mut command = wide(format!("\"{}\" --serve", worker.display()));
                let current = wide(&workspace);
                let root = std::env::var("SystemRoot").map_err(|_| "Windows SystemRoot fehlt")?;
                let mut environment = Vec::new();
                for variable in [
                    format!("LOCALAPPDATA={}", workspace.display()),
                    format!("PATH={root}\\System32"),
                    format!("SystemRoot={root}"),
                    format!("TEMP={}", workspace.display()),
                    format!("TMP={}", workspace.display()),
                    format!("WINDIR={root}"),
                ] {
                    environment.extend(wide(variable));
                }
                environment.push(0);
                let mut info = PROCESS_INFORMATION::default();
                ok(
                    CreateProcessW(
                        exe.as_ptr(),
                        command.as_mut_ptr(),
                        null(),
                        null(),
                        1,
                        EXTENDED_STARTUPINFO_PRESENT
                            | CREATE_NO_WINDOW
                            | CREATE_UNICODE_ENVIRONMENT,
                        environment.as_ptr().cast(),
                        current.as_ptr(),
                        &startup.StartupInfo,
                        &mut info,
                    ),
                    "Create isolated native worker",
                )?;
                let process = Handle::new(info.hProcess, "Worker process")?;
                let thread_handle = Handle::new(info.hThread, "Worker thread")?;
                drop(thread_handle);
                drop(child_stdin);
                drop(child_stdout);
                drop(child_stderr);
                let mut input = parent_stdin.file();
                let output = parent_stdout.file();
                let mut error_stream = parent_stderr.file();
                let (tx, rx) = mpsc::sync_channel::<Vec<u8>>(1);
                let (reply_tx, reply_rx) = mpsc::sync_channel(1);
                thread::spawn(move || {
                    for payload in rx {
                        if input
                            .write_all(&payload)
                            .and_then(|_| input.write_all(b"\n"))
                            .and_then(|_| input.flush())
                            .is_err()
                        {
                            break;
                        }
                    }
                });
                thread::spawn(move || {
                    let mut reader = BufReader::new(output);
                    loop {
                        let response = frame(&mut reader)
                            .and_then(|v| String::from_utf8(v).map_err(|e| e.to_string()))
                            .and_then(|s| crate::protocol::strict_json(&s));
                        let stop = response.is_err();
                        if reply_tx.send(response).is_err() || stop {
                            break;
                        }
                    }
                });
                let flag = Arc::new(AtomicBool::new(false));
                let error_flag = flag.clone();
                let stderr = Arc::new(Mutex::new(Vec::new()));
                let captured = stderr.clone();
                thread::spawn(move || {
                    let mut seen = 0;
                    let mut buf = [0; 8192];
                    loop {
                        match error_stream.read(&mut buf) {
                            Ok(0) | Err(_) => break,
                            Ok(n) => {
                                seen += n;
                                if seen > 65_536 {
                                    error_flag.store(true, Ordering::SeqCst);
                                    break;
                                }
                                if let Ok(mut data) = captured.lock() {
                                    data.extend_from_slice(&buf[..n]);
                                }
                            }
                        }
                    }
                });
                Ok(Self {
                    process,
                    job,
                    profile,
                    _lock: lock,
                    owner,
                    quota,
                    pid: info.dwProcessId,
                    sender: tx,
                    receiver: reply_rx,
                    stderr_overflow: flag,
                    stderr,
                    failed: false,
                    timeout,
                })
            }
        }
        pub fn pid(&self) -> u32 {
            self.pid
        }
        pub fn memory_usage(&self) -> Result<Value> {
            use windows_sys::Win32::System::ProcessStatus::{
                K32GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS_EX,
            };
            let mut info = PROCESS_MEMORY_COUNTERS_EX::default();
            info.cb = size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32;
            unsafe {
                ok(
                    K32GetProcessMemoryInfo(
                        self.process.0,
                        (&mut info as *mut PROCESS_MEMORY_COUNTERS_EX).cast(),
                        info.cb,
                    ),
                    "Own worker memory measurement",
                )?;
            }
            Ok(json!({"working_set_bytes":info.WorkingSetSize,"private_bytes":info.PrivateUsage}))
        }
        fn stop(&mut self) {
            if !self.failed {
                self.failed = true;
                unsafe {
                    TerminateJobObject(self.job.0, 1);
                    WaitForSingleObject(self.process.0, 2000);
                }
            }
        }
        pub fn request(&mut self, request: &Value) -> Result<Value> {
            if self.failed {
                return Err("Native Worker beendet; aus Checkpoint rekonstruieren".into());
            }
            let input = json!({"owner":self.owner,"quota":self.quota,"request":request})
                .to_string()
                .into_bytes();
            if input.len() > FRAME {
                return Err("Native Worker-Eingabegrenze".into());
            }
            if self.stderr_overflow.load(Ordering::SeqCst) {
                self.stop();
                return Err("Native Worker stderr-Grenze".into());
            }
            if self.sender.try_send(input).is_err() {
                self.stop();
                return Err("Native Worker-Eingang geschlossen".into());
            }
            let response = match self.receiver.recv_timeout(self.timeout) {
                Ok(Ok(v)) => v,
                Ok(Err(e)) => {
                    let mut exit = 0;
                    unsafe {
                        WaitForSingleObject(self.process.0, 100);
                        GetExitCodeProcess(self.process.0, &mut exit);
                    }
                    self.stop();
                    let stderr = self
                        .stderr
                        .lock()
                        .map(|v| String::from_utf8_lossy(&v).to_string())
                        .unwrap_or_default();
                    return Err(format!("{e}; exit={exit:#x}; {stderr}"));
                }
                Err(_) => {
                    self.stop();
                    return Err("Native Worker RPC Timeout".into());
                }
            };
            if self.stderr_overflow.load(Ordering::SeqCst) {
                self.stop();
                return Err("Native Worker stderr-Grenze".into());
            }
            if let Some(error) = response["error"].as_str() {
                return Err(error.into());
            }
            response
                .get("result")
                .cloned()
                .ok_or("Native Worker ohne Ergebnis".into())
        }
    }
    impl Drop for NativeSession {
        fn drop(&mut self) {
            self.stop();
            let _ = &self.profile;
        }
    }
}
