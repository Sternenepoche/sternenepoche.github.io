use fs2::FileExt;
use kern::typen::Rolle;
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    fs::OpenOptions,
    io::Read,
    net::IpAddr,
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};
use sternenepoche_server::{
    digest, now, password_hash, password_matches, token, Game, Reply, API_VERSION,
};
use tiny_http::{Header, Method, Request, Response, Server};
mod public_catalog;
mod web_assets;

struct Config {
    bind: String,
    admin_bind: String,
    root: PathBuf,
    origins: Vec<String>,
    proxy: bool,
}
fn args() -> Result<Config, String> {
    let mut c = Config {
        bind: "127.0.0.1:8890".into(),
        admin_bind: "127.0.0.1:8891".into(),
        root: "data/online".into(),
        origins: vec![
            "https://sternenepoche.github.io".into(),
            "http://127.0.0.1:8890".into(),
            "http://localhost:8890".into(),
        ],
        proxy: false,
    };
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--bind" => c.bind = args.next().ok_or("Adresse fehlt")?,
            "--admin-bind" => c.admin_bind = args.next().ok_or("Adminadresse fehlt")?,
            "--data" => c.root = args.next().ok_or("Datenverzeichnis fehlt")?.into(),
            "--origin" => c.origins.push(args.next().ok_or("Origin fehlt")?),
            "--trusted-proxy" => c.proxy = true,
            "--help" => {
                println!("sternenepoche-server [--data PFAD] [--bind 127.0.0.1:8890] [--admin-bind 127.0.0.1:8891] [--origin https://spiel.example] [--trusted-proxy]\nBeide Listener bleiben lokal. Öffentlicher Zugriff erfolgt über HTTPS-Reverse-Proxy. Adminzugriff beim VPS über SSH-Tunnel.");
                std::process::exit(0)
            }
            _ => return Err(format!("Unbekanntes Argument: {a}")),
        }
    }
    for s in [&c.bind, &c.admin_bind] {
        let a: std::net::SocketAddr = s
            .parse()
            .map_err(|_| "Adresse mit numerischer IP und Port angeben")?;
        if !a.ip().is_loopback() {
            return Err(
                "Listener nur auf Loopback; für Internetbetrieb HTTPS-Proxy davor schalten".into(),
            );
        }
    }
    if c.bind == c.admin_bind {
        return Err("Spiel und Verwaltung brauchen eigene Ports".into());
    }
    c.origins.push(format!("http://{}", c.bind));
    Ok(c)
}
fn header(req: &Request, name: &'static str) -> String {
    req.headers()
        .iter()
        .find(|h| h.field.equiv(name))
        .map(|h| h.value.to_string())
        .unwrap_or_default()
}
fn answer(req: Request, status: u16, body: Vec<u8>, mime: &str, origin: &str) {
    let mut r = Response::from_data(body).with_status_code(status);
    let cache=if status==200 && mime=="image/webp"{"public, max-age=86400"}else{"no-store"};
    for (k,v) in [("Content-Type",mime),("Cache-Control",cache),("X-Content-Type-Options","nosniff"),("Referrer-Policy","no-referrer"),("Cross-Origin-Resource-Policy","same-site"),
      ("Content-Security-Policy","default-src 'self'; script-src 'self'; style-src 'self'; connect-src 'self' https: http://127.0.0.1:* http://localhost:*; img-src 'self' data:; frame-ancestors 'none'; base-uri 'self'; form-action 'self'")] {r.add_header(Header::from_bytes(k,v).unwrap());}
    if !origin.is_empty() {
        r.add_header(Header::from_bytes("Access-Control-Allow-Origin", origin).unwrap());
        r.add_header(Header::from_bytes("Vary", "Origin").unwrap());
        r.add_header(
            Header::from_bytes(
                "Access-Control-Allow-Headers",
                "Authorization, Content-Type, X-Sternenepoche-Admin",
            )
            .unwrap(),
        );
        r.add_header(
            Header::from_bytes("Access-Control-Allow-Methods", "GET, POST, OPTIONS").unwrap(),
        );
    }
    let _ = req.respond(r);
}
fn json_reply(req: Request, result: Reply, origin: &str) {
    let (code, v) = match result {
        Ok(v) => (200, v),
        Err((code, e)) => (code, json!({"error":e})),
    };
    answer(
        req,
        code,
        v.to_string().into_bytes(),
        "application/json; charset=utf-8",
        origin,
    );
}
type Limits = Arc<Mutex<HashMap<String, (i64, u32)>>>;
fn allowed(limits: &Limits, key: String, max: u32) -> bool {
    let mut map = limits.lock().unwrap();
    let minute = now() / 60;
    map.retain(|_, v| v.0 >= minute - 1);
    // Bounding random-token/address churn also bounds the rate-limit table.
    if map.len() >= 10000 && !map.contains_key(&key) {
        return false;
    }
    let e = map.entry(key).or_insert((minute, 0));
    if e.0 != minute {
        *e = (minute, 0)
    }
    e.1 += 1;
    e.1 <= max
}
fn request_allowed(limits: &Limits, ip: &str, authorization: &str) -> bool {
    // Random Authorization headers must not create an unlimited sequence of fresh budgets.
    // Keep enough headroom for several legitimate players behind the same NAT.
    allowed(limits, format!("ip:{ip}"), 6000)
        && allowed(limits, format!("requests:{ip}:{}", digest(authorization)), 300)
}
fn serve(
    mut req: Request,
    admin: bool,
    game: &Arc<Mutex<Game>>,
    cfg: &Config,
    admin_key: &str,
    limits: &Limits,
) {
    let path = req.url().split('?').next().unwrap_or("").to_string();
    let origin = header(&req, "Origin");
    let host = header(&req, "Host");
    let local_admin = [
        cfg.admin_bind.clone(),
        cfg.admin_bind.replace("127.0.0.1", "localhost"),
    ];
    if admin
        && (!local_admin.contains(&host)
            || !req.remote_addr().is_some_and(|a| a.ip().is_loopback()))
    {
        json_reply(
            req,
            Err((
                403,
                "Verwaltung nur über lokale Adresse oder SSH-Tunnel".into(),
            )),
            "",
        );
        return;
    }
    let origin_allowed = origin.is_empty()
        || if admin {
            local_admin.iter().any(|h| origin == format!("http://{h}"))
        } else {
            cfg.origins.contains(&origin)
        };
    if !origin_allowed {
        json_reply(
            req,
            Err((
                403,
                "Diese Website ist nicht für den Server freigegeben".into(),
            )),
            "",
        );
        return;
    }
    if req.method() == &Method::Options {
        answer(req, 204, vec![], "application/json", &origin);
        return;
    }
    let ip = if cfg.proxy && !admin {
        let v = header(&req, "X-Real-IP");
        v.parse::<IpAddr>()
            .ok()
            .map(|ip| ip.to_string())
            .unwrap_or_else(|| "proxy".into())
    } else {
        req.remote_addr()
            .map(|a| a.ip().to_string())
            .unwrap_or_default()
    };
    if !request_allowed(limits, &ip, &header(&req, "Authorization")) {
        json_reply(
            req,
            Err((
                429,
                "Zu viele Anfragen; in einer Minute erneut versuchen".into(),
            )),
            &origin,
        );
        return;
    }
    let method = req.method().clone();
    if method == Method::Get {
        let asset = match (admin, path.as_str()) {
            (false, "/") | (false, "/index.html") => Some((
                include_bytes!("../../../web-client/index.html").as_slice(),
                "text/html; charset=utf-8",
            )),
            (false, "/app.js") => Some((
                include_bytes!("../../../web-client/app.js").as_slice(),
                "text/javascript; charset=utf-8",
            )),
            (false,"/presentation.js")=>Some((include_bytes!("../../../web-client/presentation.js").as_slice(),"text/javascript; charset=utf-8")),
            (false,"/game-ui.js")=>Some((include_bytes!("../../../web-client/game-ui.js").as_slice(),"text/javascript; charset=utf-8")),
            (false,"/game.css")=>Some((include_bytes!("../../../web-client/game.css").as_slice(),"text/css; charset=utf-8")),
            (false,"/art.js")=>Some((include_bytes!("../../../web-client/art.js").as_slice(),"text/javascript; charset=utf-8")),
            (_, "/style.css") => Some((
                include_bytes!("../../../web-client/style.css").as_slice(),
                "text/css; charset=utf-8",
            )),
            (true, "/") | (true, "/index.html") => Some((
                include_bytes!("../../../web-client/admin/index.html").as_slice(),
                "text/html; charset=utf-8",
            )),
            (true,"/monitoring.js")=>Some((include_bytes!("../../../web-client/admin/monitoring.js").as_slice(),"text/javascript; charset=utf-8")),
            (true, "/admin.js") => Some((
                include_bytes!("../../../web-client/admin/admin.js").as_slice(),
                "text/javascript; charset=utf-8",
            )),
            (false,p) => web_assets::get(p),
            _ => None,
        };
        if let Some((bytes, mime)) = asset {
            answer(req, 200, bytes.to_vec(), mime, &origin);
            return;
        }
        if path == "/config.js" && !admin {
            answer(
                req,
                200,
                format!("window.STERNENEPOCHE={{api:location.origin,version:{API_VERSION}}};").into_bytes(),
                "text/javascript; charset=utf-8",
                &origin,
            );
            return;
        }
        if path == "/api/admin/bootstrap" && admin {
            // Only the separate loopback UI can read this credential; public listener has no such route.
            json_reply(
                req,
                Ok(json!({"key":admin_key,"game_url":format!("http://{}",cfg.bind)})),
                &origin,
            );
            return;
        }
    }
    if admin && header(&req, "X-Sternenepoche-Admin") != admin_key {
        json_reply(req, Err((401, "Adminzugang erforderlich".into())), &origin);
        return;
    }
    if !admin && path.starts_with("/api/admin") {
        json_reply(req, Err((404, "Nicht gefunden".into())), &origin);
        return;
    }
    let mut body = Value::Null;
    if method == Method::Post {
        if !header(&req, "Content-Type").starts_with("application/json") {
            json_reply(req, Err((415, "JSON erwartet".into())), &origin);
            return;
        }
        if req.body_length().is_none() || req.body_length().is_some_and(|n| n > 65536) {
            json_reply(
                req,
                Err((
                    413,
                    "Befehle auf 64 KiB begrenzt; Content-Length erforderlich".into(),
                )),
                &origin,
            );
            return;
        }
        let mut bytes = Vec::new();
        if req.as_reader().take(65537).read_to_end(&mut bytes).is_err() || bytes.len() > 65536 {
            json_reply(req, Err((400, "Anfrage unvollständig".into())), &origin);
            return;
        }
        body = match serde_json::from_slice(&bytes) {
            Ok(v) => v,
            Err(_) => {
                json_reply(req, Err((400, "JSON nicht lesbar".into())), &origin);
                return;
            }
        };
    }
    if !admin && method == Method::Post && ["/api/register", "/api/login"].contains(&path.as_str())
    {
        if !allowed(limits, format!("auth:{ip}"), 12) {
            json_reply(
                req,
                Err((
                    429,
                    "Zu viele Anmeldungen; in einer Minute erneut versuchen".into(),
                )),
                &origin,
            );
            return;
        }
        let password = body["password"].as_str().unwrap_or("");
        // Expensive password work runs outside the world mutex.
        let result = if path == "/api/register" {
            match password_hash(password) {
                Ok(hash) => game.lock().unwrap().register(&body, hash),
                Err(e) => Err((400, e)),
            }
        } else {
            let account = game
                .lock()
                .unwrap()
                .account_by_name(body["name"].as_str().unwrap_or(""));
            let hash=account.as_ref().map(|a|a.password.clone()).unwrap_or_else(||"$argon2id$v=19$m=19456,t=2,p=1$dGVzdHNhbHQxMjM0NTY3OA$Z9XRhYVbhvSMjrgd8qVTuXFtrV89zhU4BJG5ULtdhdA".into());
            let valid = password.len() <= 128 && password_matches(password, &hash);
            match account.filter(|_| valid) {
                Some(a) => game.lock().unwrap().login_verified(a.id),
                None => Err((401, "Name oder Passwort falsch".into())),
            }
        };
        json_reply(req, result, &origin);
        return;
    }
    let raw = header(&req, "Authorization")
        .strip_prefix("Bearer ")
        .unwrap_or("")
        .to_string();
    let mut g = game.lock().unwrap();
    let result = if admin {
        match (method, path.as_str()) {
            (Method::Get, "/api/admin/status") => Ok(g.admin_status()),
            (Method::Post, "/api/admin/action") => g.admin(&body),
            _ => Err((404, "Nicht gefunden".into())),
        }
    } else {
        match (method, path.as_str()) {
            (Method::Get, "/api/lobby") | (Method::Get, "/api/health") => Ok(g.lobby()),
            (Method::Get, "/api/rules") => Ok(
                json!({"api_version":API_VERSION,"schema":kern::aktion::antwortschema(Rolle::Alle),"volk":VolkNames::values(),"catalog":public_catalog::catalog(&g.world.regeln),"text":format!("{}\n{}\n{}",kern::regeltext::regeltext(&g.world.regeln,Rolle::Alle),kern::kolonisation::REGELTEXT_V2,kern::ausscheiden::REGELTEXT)}),
            ),
            (method, path) => {
                match g.authenticate(&raw) {
                    Err(e) => Err(e),
                    Ok(a) => match (method, path) {
                        (Method::Get, "/api/me") => Ok(g.me(&a)),
                        (Method::Get, "/api/view") => g.view(&a, Rolle::Alle),
                        (Method::Post, "/api/context") => {
                            let role = body["rolle"]
                                .as_str()
                                .and_then(Rolle::aus_name)
                                .filter(|r| *r != Rolle::Alle);
                            match role {Some(r)=>g.view(&a,r).map(|view|json!({"view":view,"schema":kern::aktion::antwortschema(r),"text":kern::regeltext::regeltext(&g.world.regeln,r)})),None=>Err((400,"Regierungsrolle auswählen".into()))}
                        }
                        (Method::Post, "/api/claim") => g.claim(&a, &body),
                        (Method::Post, "/api/waitlist/leave") => g.leave_waitlist(&a, &body),
                        (Method::Post, "/api/tool") => g.tool(&a, &body),
                        (Method::Post, "/api/command") => g.command(&a, &body),
                        (Method::Post, "/api/lease") => g.lease(&a, &body),
                        (Method::Post, "/api/agent-report") => g.agent_report(&a, &body),
                        (Method::Post, "/api/logout") => g.logout(&a, &raw),
                        _ => Err((404, "Nicht gefunden".into())),
                    },
                }
            }
        }
    };
    drop(g);
    json_reply(req, result, &origin);
}
struct VolkNames;
impl VolkNames {
    fn values() -> Vec<&'static str> {
        kern::typen::Volk::ALLE
            .into_iter()
            .map(|v| v.name())
            .collect()
    }
}
fn run() -> Result<(), String> {
    let cfg = Arc::new(args()?);
    std::fs::create_dir_all(&cfg.root).map_err(|e| e.to_string())?;
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(cfg.root.join("server.lock"))
        .map_err(|e| e.to_string())?;
    lock.try_lock_exclusive()
        .map_err(|_| "Diese Datenbank wird bereits von einem Server benutzt")?;
    let game = Arc::new(Mutex::new(Game::open(&cfg.root, rand::random())?));
    let keyfile = cfg.root.join("admin-token.txt");
    let key = if keyfile.exists() {
        std::fs::read_to_string(&keyfile)
            .map_err(|e| e.to_string())?
            .trim()
            .to_string()
    } else {
        let key = token();
        let mut opts = OpenOptions::new();
        opts.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600);
        }
        use std::io::Write;
        let mut f = opts.open(&keyfile).map_err(|e| e.to_string())?;
        f.write_all(key.as_bytes()).map_err(|e| e.to_string())?;
        key
    };
    if key.len() != 64 {
        return Err("Admin-Token beschädigt".into());
    }
    let key = Arc::new(key);
    for (admin, addr, count) in [(false, &cfg.bind, 4), (true, &cfg.admin_bind, 2)] {
        // Public request pressure must not fill the private dashboard's limiter.
        let limits: Limits = Arc::new(Mutex::new(HashMap::new()));
        let server = Arc::new(Server::http(addr).map_err(|e| e.to_string())?);
        for _ in 0..count {
            let (s, g, c, k, l) = (
                server.clone(),
                game.clone(),
                cfg.clone(),
                key.clone(),
                limits.clone(),
            );
            thread::spawn(move || {
                for req in s.incoming_requests() {
                    serve(req, admin, &g, &c, &k, &l);
                }
            });
        }
    }
    println!(
        "Spiel: http://{} | Lokales Dashboard: http://{} | Daten: {}",
        cfg.bind,
        cfg.admin_bind,
        cfg.root.display()
    );
    let mut last = Instant::now();
    let mut remainder = 0.0;
    loop {
        thread::sleep(Duration::from_millis(200));
        let elapsed = last.elapsed().as_secs_f64();
        last = Instant::now();
        let mut g = game.lock().unwrap();
        if g.runtime.paused || g.world.beendet() {
            remainder = 0.0;
            continue;
        }
        // Resume never interprets machine sleep as an unattended raid catch-up.
        // Normal work/SQLite contention must not silently lose elapsed time.
        // A gap beyond one minute is treated as machine suspension, not catch-up.
        remainder += (if elapsed > 60.0 {0.2} else {elapsed}) * g.runtime.tempo as f64;
        let steps = remainder.floor().min(3600.0) as u32;
        if steps > 0 {
            remainder -= steps as f64;
            if let Err((_, e)) = g.advance(steps) {
                eprintln!("{e}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rotating_invalid_tokens_cannot_bypass_the_ip_budget() {
        let limits: Limits = Arc::new(Mutex::new(HashMap::new()));
        for n in 0..6000 {
            assert!(request_allowed(&limits, "test-ip", &format!("invalid-token-{n}")));
        }
        assert!(!request_allowed(&limits, "test-ip", "another-invalid-token"));
        assert!(request_allowed(&limits, "other-ip", "legitimate-session"));
    }
}
fn main() {
    if let Err(e) = run() {
        eprintln!("Serverstart fehlgeschlagen: {e}");
        std::process::exit(1)
    }
}
