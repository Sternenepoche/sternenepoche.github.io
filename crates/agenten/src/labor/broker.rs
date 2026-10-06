use super::{
    config::{Budget, Model},
    Result,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    io::Read,
    time::{Duration, Instant},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: Value,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Reply {
    pub message: Value,
    pub calls: Vec<ToolCall>,
    pub text: String,
    pub model: String,
    pub usage: Value,
    pub cost_micro_usd: Option<u64>,
    pub elapsed_ms: u64,
    pub failure: Option<String>,
}
fn client(timeout: u64) -> Result<reqwest::blocking::Client> {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(timeout))
        .redirect(reqwest::redirect::Policy::none())
        .retry(reqwest::retry::never())
        .no_proxy()
        .build()
        .map_err(|e| e.to_string())
}
fn read(response: reqwest::blocking::Response) -> Result<Value> {
    let mut bytes = Vec::new();
    response
        .take(2_097_153)
        .read_to_end(&mut bytes)
        .map_err(|_| "Antwort abgerissen; Ausgang unbekannt")?;
    if bytes.len() > 2_097_152 {
        return Err("Antwort >2MiB; Ausgang unbekannt".into());
    }
    let text = std::str::from_utf8(&bytes).map_err(|_| "Antwort nicht UTF-8; Ausgang unbekannt")?;
    crate::protocol::strict_json(text)
        .map_err(|_| "Antwort nicht eindeutig lesbar; Ausgang unbekannt".into())
}
pub fn inventory(base: &str) -> Result<Value> {
    let probe = Model {
        kind: "ollama".into(),
        model: "inventory".into(),
        url: Some(base.into()),
        api_key_env: None,
        digest: None,
        context: 4096,
        timeout_seconds: 5,
        reserve_micro_usd: 0,
        resident_bytes: None,
        provider: None,
        cpu_only: false,
        thinking: false,
    };
    probe.validate()?;
    let c = client(5)?;
    let fetch = |p: &str| -> Result<Value> {
        let r = c
            .get(format!("{}{p}", base.trim_end_matches('/')))
            .send()
            .map_err(|_| "Ollama nicht erreichbar")?;
        if !r.status().is_success() {
            return Err(format!("Inventar HTTP {}", r.status()));
        }
        read(r)
    };
    Ok(
        json!({"installed":fetch("/api/tags")?,"running":fetch("/api/ps")?,"capacity":"Measure loaded footprint at the configured context and parallelism; disk size is not VRAM."}),
    )
}
pub fn preflight(m: &Model) -> Result<Value> {
    m.validate()?;
    if m.kind == "mock" {
        return Ok(json!({"kind":"mock"}));
    }
    if m.kind == "openrouter" {
        std::env::var(m.api_key_env.as_ref().unwrap())
            .map_err(|_| "OpenRouter-Schlüsselvariable fehlt")?;
        return Ok(json!({"kind":"openrouter","model":m.model,"network_probe":false}));
    }
    let base = m.url.as_ref().unwrap();
    let inv = inventory(base)?;
    let tag = inv["installed"]["models"]
        .as_array()
        .and_then(|a| {
            a.iter()
                .find(|x| x["name"] == m.model || x["model"] == m.model)
        })
        .ok_or("Konfiguriertes Ollama-Modell nicht installiert")?;
    if m.digest.as_ref().is_some_and(|d| tag["digest"] != *d) {
        return Err("Ollama-Digest weicht vom Laufvertrag ab".into());
    }
    let r = client(10)?
        .post(format!("{}/api/show", base.trim_end_matches('/')))
        .json(&json!({"model":m.model}))
        .send()
        .map_err(|_| "Ollama-Modellabfrage fehlgeschlagen")?;
    if !r.status().is_success() {
        return Err("Ollama-Modellabfrage abgelehnt".into());
    }
    let details = read(r)?;
    if !local_weights(tag, &details) {
        return Err(
            "Ollama-Cloudmodell ausgeschlossen: benötigt installierte lokale Gewichte".into(),
        );
    }
    if !details["capabilities"]
        .as_array()
        .is_some_and(|c| c.iter().any(|x| x == "tools"))
    {
        return Err(
            "Modell meldet keine native Toolfähigkeit; expliziter getesteter Adapter erforderlich"
                .into(),
        );
    }
    Ok(json!({"kind":"ollama","installed":tag,"details":details,"running":inv["running"]}))
}

pub fn local_weights(tag: &Value, details: &Value) -> bool {
    tag["size"].as_u64().unwrap_or(0) > 1_000_000
        && !tag["name"].as_str().unwrap_or("").ends_with(":cloud")
        && [tag, details].iter().all(|v| {
            ["remote_model", "remote_host"]
                .iter()
                .all(|k| v.get(k).is_none_or(|x| x.is_null() || x == ""))
        })
        && details["model_info"]
            .as_object()
            .is_some_and(|o| !o.is_empty())
}

/// Resolve fallback once per immutable run, never after an individual call fails.
pub fn resolve_policy(c: &super::config::Config) -> Result<super::config::Config> {
    if c.provider_mode != "local_or_remote" {
        return Ok(c.clone());
    }
    let base = c
        .models
        .values()
        .find(|m| m.kind == "ollama")
        .and_then(|m| m.url.as_deref())
        .unwrap_or("http://127.0.0.1:11434");
    let explicit = c
        .models
        .iter()
        .filter(|(_, m)| m.kind == "ollama")
        .collect::<Vec<_>>();
    let mut out = c.clone();
    let mut local = std::collections::BTreeMap::new();
    for (name, m) in &explicit {
        match preflight(m) {
            Ok(_) => {
                local.insert((**name).clone(), (**m).clone());
            }
            Err(e)
                if e == "Ollama nicht erreichbar"
                    || e == "Konfiguriertes Ollama-Modell nicht installiert" => {}
            Err(e) => {
                return Err(format!(
                    "Lokale Modellauswahl unklar; kein Remote-Fallback: {e}"
                ))
            }
        }
    }
    if local.is_empty() {
        match local_config(c.players.len(),base,false) {
            Ok(discovered) if explicit.is_empty() => local=discovered.models,
            Ok(_) => return Err("Konfigurierte lokale Modelle fehlen, aber andere lokale Toolmodelle sind verfügbar. Lokale Auswahl korrigieren; kein Remote-Fallback".into()),
            Err(e) if e=="Ollama nicht erreichbar" || e=="Kein installiertes Toolmodell mit mindestens 8K Kontext gefunden" => {},
            Err(e) => return Err(format!("Lokales Inventar ungeklärt; kein Remote-Fallback: {e}")),
        }
    }
    if local.is_empty() {
        out.models.retain(|_, m| m.kind == "openrouter");
    } else {
        out.models = local;
    }
    let name = out
        .models
        .keys()
        .next()
        .ok_or("Keine lokalen Toolmodelle und kein OpenRouter-Fallback konfiguriert")?
        .clone();
    for (sid, h) in out.players.iter_mut().enumerate() {
        for r in &mut h.roles {
            if !out.models.contains_key(&r.model) {
                r.model = name.clone();
            }
        }
        if out
            .primary_models
            .get(&(sid as u16))
            .is_none_or(|m| !out.models.contains_key(m))
        {
            out.primary_models
                .insert(sid as u16, h.roles[0].model.clone());
        }
    }
    out.validate()?;
    Ok(out)
}

pub fn request_body(m: &Model, messages: &[Value], tools: &[Value], b: &Budget) -> Value {
    let mut body = json!({"model":m.model,"messages":messages,"tools":tools,"stream":false});
    if m.kind == "ollama" {
        body["options"] =
            json!({"temperature":0,"num_ctx":m.context,"num_predict":b.output_tokens});
        if m.cpu_only {
            body["options"]["num_gpu"] = json!(0);
        }
        body["think"] = json!(m.thinking);
        body["keep_alive"] = json!("5m");
    } else {
        body["max_tokens"] = json!(b.output_tokens);
        body["temperature"] = json!(0);
        let mut provider = json!({"allow_fallbacks":false,"require_parameters":true});
        if let Some(p) = &m.provider {
            provider["order"] = json!([p]);
        }
        body["provider"] = provider;
    }
    body
}

/// HTTP/transport failure never invents a valid game decision. Unknown outcomes leave the
/// caller's journal reservation open and require explicit reconciliation.
pub fn call(m: &Model, messages: &[Value], tools: &[Value], b: &Budget) -> Result<Reply> {
    if m.kind == "mock" {
        return Ok(mock(messages, &m.model));
    }
    let body = request_body(m, messages, tools, b);
    let mut request = client(m.timeout_seconds)?
        .post(format!(
            "{}{}",
            m.url.as_ref().unwrap().trim_end_matches('/'),
            if m.kind == "ollama" {
                "/api/chat"
            } else {
                "/chat/completions"
            }
        ))
        .json(&body);
    if let Some(env) = &m.api_key_env {
        let key = std::env::var(env).map_err(|_| "Schlüsselvariable fehlt")?;
        request = request.bearer_auth(key);
    }
    let start = Instant::now();
    let response = match request.send() {
        Ok(r) => r,
        Err(e) if e.is_connect() => return Ok(failed(m, "connect_refused", start)),
        Err(_) => return Err("Provider-Ausgang unbekannt; keine automatische Wiederholung".into()),
    };
    let status = response.status().as_u16();
    if [400, 401, 402, 403, 404, 429].contains(&status) {
        return Ok(failed(m, &format!("http_{status}"), start));
    }
    if !(200..300).contains(&status) {
        let detail = read(response)
            .ok()
            .map(|v| v.to_string())
            .unwrap_or_default();
        return Err(format!(
            "HTTP {status}; Verarbeitung/Abrechnung ungeklärt; {}",
            detail.chars().take(2000).collect::<String>()
        ));
    }
    let raw = read(response)?;
    let mut message = if m.kind == "ollama" {
        raw["message"].clone()
    } else {
        raw["choices"][0]["message"].clone()
    };
    if !message.is_object() {
        return Err("Provider ohne vollständige Nachricht; Ausgang unklar".into());
    }
    let mut calls = Vec::new();
    let mut ids = std::collections::BTreeSet::new();
    if let Some(items) = message["tool_calls"].as_array_mut() {
        if items.len() > 64 {
            return Err("Provider liefert zu viele Tools; Antwort zur Klärung gesperrt".into());
        }
        for (i, t) in items.iter_mut().enumerate() {
            let id = t["id"]
                .as_str()
                .map(str::to_string)
                .unwrap_or_else(|| format!("tool_{i}"));
            if id.len() > 200 || !ids.insert(id.clone()) {
                return Err(
                    "Providerantwort mit mehrdeutigen Tool-IDs; Protokollklärung erforderlich"
                        .into(),
                );
            }
            t["id"] = json!(id);
            t["type"] = json!("function");
            let args = match &t["function"]["arguments"] {
                Value::String(s) => {
                    crate::protocol::strict_json(s).unwrap_or_else(|e| json!({"_parse_error":e}))
                }
                v if v.is_object() => v.clone(),
                _ => json!({"_parse_error":"Toolargumente fehlen"}),
            };
            calls.push(ToolCall {
                id,
                name: t["function"]["name"].as_str().unwrap_or("").into(),
                arguments: args,
            });
        }
    }
    let mut usage = if m.kind == "ollama" {
        json!({"input_tokens":raw["prompt_eval_count"],"output_tokens":raw["eval_count"],
        "load_duration_ns":raw["load_duration"],"prompt_eval_duration_ns":raw["prompt_eval_duration"],
        "eval_duration_ns":raw["eval_duration"],"total_duration_ns":raw["total_duration"],"local_inference":true})
    } else {
        raw["usage"].clone()
    };
    if !usage.is_object() {
        usage = json!({"reported_usage":usage});
    }
    let finish = if m.kind == "ollama" {
        raw["done_reason"].clone()
    } else {
        raw["choices"][0]["finish_reason"].clone()
    };
    usage["output_truncated"] = json!(finish == "length");
    usage["finish_reason"] = finish;
    usage["provider_request_id"] = raw["id"].clone();
    let cost = if m.kind == "mock" {
        Some(0)
    } else {
        usage["cost"]
            .as_f64()
            .filter(|v| v.is_finite() && *v >= 0.)
            .map(|v| (v * 1_000_000.).ceil() as u64)
    };
    let text = message["content"].as_str().unwrap_or("").to_string();
    Ok(Reply {
        message,
        calls,
        text,
        model: raw["model"].as_str().unwrap_or(&m.model).into(),
        usage,
        cost_micro_usd: cost,
        elapsed_ms: start.elapsed().as_millis() as u64,
        failure: None,
    })
}
fn failed(m: &Model, error: &str, start: Instant) -> Reply {
    Reply {
        message: json!({"role":"assistant","content":""}),
        calls: vec![],
        text: String::new(),
        model: m.model.clone(),
        usage: Value::Null,
        cost_micro_usd: Some(0),
        elapsed_ms: start.elapsed().as_millis() as u64,
        failure: Some(error.into()),
    }
}
fn mock(messages: &[Value], model: &str) -> Reply {
    let mut calls = vec![];
    if messages.len() == 2 {
        let briefing: Value = serde_json::from_str(messages[1]["content"].as_str().unwrap_or("{}"))
            .unwrap_or(Value::Null);
        if let Some(p) = briefing["view"]["planeten"]
            .as_array()
            .and_then(|v| v.first())
        {
            let building = if p["energie"]["erzeugung"].as_i64().unwrap_or(0)
                <= p["energie"]["verbrauch"].as_i64().unwrap_or(0)
            {
                "solarkraftwerk"
            } else {
                "erzmine"
            };
            calls.push(ToolCall {
                id: "build".into(),
                name: "game_bauen".into(),
                arguments: json!({"planet":p["koord"],"gebaeude":building}),
            });
        }
        let revision = briefing["memory"]
            .as_array()
            .and_then(|v| v.iter().find(|v| v["key"] == "strategy"))
            .and_then(|v| v["revision"].as_u64())
            .unwrap_or(0);
        calls.push(ToolCall{id:"remember".into(),name:"memory_write".into(),arguments:json!({"kind":"plan","key":"strategy","revision":revision,"value":{"goal":"Versorgung und Wachstum","next":"Wirkung prüfen; Nachbarn beobachten","source":"own_observation"}})});
    }
    let message = json!({"role":"assistant","content":"Versorgung prüfen und den laufenden Plan fortsetzen.","tool_calls":calls.iter().map(|c|json!({"id":c.id,"type":"function","function":{"name":c.name,"arguments":c.arguments.to_string()}})).collect::<Vec<_>>()});
    Reply {
        message,
        calls,
        text: "Versorgung prüfen und den laufenden Plan fortsetzen.".into(),
        model: model.into(),
        usage: json!({"mock":true}),
        cost_micro_usd: Some(0),
        elapsed_ms: 0,
        failure: None,
    }
}

/// Work is admitted in bounded, deterministic model groups. Unknown local capacity is serial.
/// Resident model sharing only; simultaneous different local models need a measured footprint.
pub fn batch_width(models: &[&Model], configured: usize, capacity: u64) -> usize {
    if models.iter().all(|m| m.kind != "ollama") {
        return configured.max(1).min(models.len().max(1));
    }
    if capacity == 0 || models.iter().any(|m| m.resident_bytes.is_none()) {
        return 1;
    }
    let sum = models
        .iter()
        .take(configured)
        .map(|m| m.resident_bytes.unwrap_or(u64::MAX))
        .try_fold(0u64, |a, b| a.checked_add(b));
    if sum.is_some_and(|n| n <= capacity) {
        configured.min(models.len()).max(1)
    } else {
        1
    }
}

/// Confirm the configured resident estimates against live Ollama measurements. With unknown
/// capacity/weights/context, serialize. Never infer RAM or VRAM from the download size.
pub fn admitted_width(models: &[&Model], configured: usize, capacity: u64) -> usize {
    let models = &models[..models.len().min(configured.max(1))];
    if models.iter().all(|m| m.kind != "ollama") {
        return batch_width(models, configured, capacity);
    }
    if capacity == 0 {
        return serial_local_width(models, configured);
    }
    let mut measured = Vec::new();
    let mut probes = std::collections::BTreeMap::new();
    for m in models.iter().take(configured) {
        let mut model = (*m).clone();
        if m.kind == "ollama" {
            let base = m.url.as_deref().unwrap_or("");
            if !probes.contains_key(base) {
                let Ok(inv) = inventory(base) else {
                    return 1;
                };
                probes.insert(base, inv);
            }
            let Some(p) = probes[base]["running"]["models"].as_array().and_then(|a| {
                a.iter().find(|p| {
                    p["name"] == m.model
                        && p["context_length"]
                            .as_u64()
                            .is_some_and(|n| n >= u64::from(m.context))
                        && m.digest.as_ref().is_none_or(|d| p["digest"] == *d)
                })
            }) else {
                return 1;
            };
            let Some(bytes) = p["size"].as_u64().filter(|n| *n > 0) else {
                return 1;
            };
            // Count each concurrent request with a full resident allowance plus 25% headroom.
            model.resident_bytes = Some(
                bytes
                    .saturating_add(bytes / 4)
                    .max(m.resident_bytes.unwrap_or(0)),
            );
        } else {
            model.resident_bytes = Some(0);
        }
        measured.push(model);
    }
    let mut other = 0u64;
    for inv in probes.values() {
        if let Some(running) = inv["running"]["models"].as_array() {
            for p in running {
                if !models.iter().take(configured).any(|m| p["name"] == m.model) {
                    other = other.saturating_add(p["size"].as_u64().unwrap_or(capacity));
                }
            }
        }
    }
    batch_width(
        &measured.iter().collect::<Vec<_>>(),
        configured,
        capacity.saturating_sub(other),
    )
}

/// Unknown GPU capacity still permits remote work alongside one local request.
pub fn serial_local_width(models: &[&Model], configured: usize) -> usize {
    let mut local = 0;
    let mut width = 0;
    for m in models.iter().take(configured.max(1)) {
        if m.kind == "ollama" {
            local += 1;
            if local > 1 {
                break;
            }
        }
        width += 1;
    }
    width.max(1)
}

/// Discover, do not download or load weights. All chosen identities are pinned in the new config.
pub fn local_config(players: usize, base: &str, cpu_only: bool) -> Result<super::config::Config> {
    let inv = inventory(base)?;
    let c = client(10)?;
    let mut models = Vec::new();
    let mut digests = std::collections::BTreeSet::new();
    for tag in inv["installed"]["models"]
        .as_array()
        .ok_or("Modellliste fehlt")?
    {
        let Some(name) = tag["name"].as_str() else {
            continue;
        };
        let Some(digest) = tag["digest"].as_str() else {
            continue;
        };
        if digests.contains(digest) {
            continue;
        }
        let response = c
            .post(format!("{}/api/show", base.trim_end_matches('/')))
            .json(&json!({"model":name}))
            .send()
            .map_err(|_| "Inventar-Detailabfrage fehlgeschlagen")?;
        if !response.status().is_success() {
            return Err(format!("Inventar-Detail HTTP {}", response.status()));
        }
        let detail = read(response)?;
        if !detail["capabilities"]
            .as_array()
            .is_some_and(|a| a.iter().any(|x| x == "tools"))
        {
            continue;
        }
        let max_context = detail["model_info"]
            .as_object()
            .and_then(|o| o.iter().find(|(k, _)| k.ends_with(".context_length")))
            .and_then(|(_, v)| v.as_u64())
            .unwrap_or(4096);
        if max_context < 8192 || !local_weights(tag, &detail) {
            continue;
        }
        digests.insert(digest.to_string());
        models.push((
            tag["size"].as_u64().unwrap_or(u64::MAX),
            Model {
                kind: "ollama".into(),
                model: name.into(),
                url: Some(base.into()),
                api_key_env: None,
                digest: Some(digest.into()),
                context: 8192,
                timeout_seconds: 300,
                reserve_micro_usd: 0,
                resident_bytes: None,
                provider: None,
                cpu_only,
                thinking: false,
            },
        ));
    }
    models.sort_by_key(|x| x.0);
    if models.is_empty() {
        return Err("Kein installiertes Toolmodell mit mindestens 8K Kontext gefunden".into());
    }
    // Prefer a conventional 7-9B footprint, while keeping a small-model option on smaller hosts.
    // This is a candidate selection, not a claim that disk bytes equal GPU residency.
    let preferred = models
        .iter()
        .position(|(size, _)| (3_800_000_000..=5_600_000_000).contains(size))
        .unwrap_or(0);
    let chosen = models.remove(preferred).1;
    let mut config = super::config::Config::v4(players);
    config.provider_mode = "local_only".into();
    config.sandbox = "native".into();
    config.budget.output_tokens = 1024;
    config.models.clear();
    config.models.insert("primary".into(), chosen);
    let names = config.models.keys().cloned().collect::<Vec<_>>();
    for (i, h) in config.players.iter_mut().enumerate() {
        for r in &mut h.roles {
            r.model = names[i % names.len()].clone();
        }
    }
    config.validate()?;
    Ok(config)
}
