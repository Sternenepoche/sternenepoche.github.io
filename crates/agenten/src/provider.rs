use crate::config::Provider;
use serde_json::{json, Value};
use std::io::Read;
use std::time::Duration;

/// Ausgang eines gescheiterten Aufrufs. `Eindeutig`: der Anbieter hat geantwortet (HTTP-Status, leere oder
/// abgeschnittene Antwort) oder die Verbindung kam gar nicht zustande; ein neuer Versuch ist sicher.
/// `Unklar`: die Anfrage ging hinaus, eine Antwort kam nicht vollständig an; dann wird nie automatisch
/// wiederholt, weil der Anbieter sie verarbeitet und berechnet haben kann.
#[derive(Debug)]
pub enum Fehler {
    Eindeutig(String),
    Unklar(String),
}

impl std::fmt::Display for Fehler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Fehler::Eindeutig(t) | Fehler::Unklar(t) => f.write_str(t),
        }
    }
}

/// Text des Fehlers, wenn eine Antwort am Tokenlimit endete; der nächste Versuch bekommt dann mehr Platz.
pub const TOKENLIMIT: &str = "Modellantwort wegen Tokenlimit abgeschnitten";

/// HTTP 401, 402 and 403: wrong key, no credit or key limit reached. The provider refused the
/// call before processing it; repeating it cannot succeed until someone acts.
pub fn abgelehnt(fehler: &str) -> bool {
    ["HTTP-Status 401", "HTTP-Status 402", "HTTP-Status 403"].contains(&fehler)
}

pub fn call(p: &Provider, messages: &[Value]) -> Result<Value, Fehler> {
    anfrage(p, messages, None, false)
}

/// Ein Aufruf. Mit `schema` und `p.schema` wird das Antwortschema erzwungen; `mehr_platz` verdoppelt das
/// Tokenlimit und senkt das Denken, nachdem eine Antwort am Limit endete.
pub fn anfrage(p: &Provider, messages: &[Value], schema: Option<&Value>, mehr_platz: bool) -> Result<Value, Fehler> {
    if p.art == "mock" {
        return Ok(mock(messages, &p.modell));
    }
    p.validate().map_err(Fehler::Eindeutig)?;
    let key = match &p.api_key_env {
        Some(name) if !name.is_empty() => {
            Some(std::env::var(name).map_err(|_| Fehler::Eindeutig("API-Key-Umgebungsvariable fehlt".to_string()))?)
        }
        _ => None,
    };
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(p.timeout_sekunden))
        .redirect(reqwest::redirect::Policy::none())
        .retry(reqwest::retry::never())
        .no_proxy()
        .build()
        .map_err(|_| Fehler::Eindeutig("HTTP-Client konnte nicht gestartet werden".to_string()))?;
    let body = koerper(p, messages, schema, mehr_platz);
    let url = format!(
        "{}/chat/completions",
        p.url.as_deref().unwrap().trim_end_matches('/')
    );
    let mut req = client.post(url).json(&body);
    if let Some(key) = key {
        req = req.bearer_auth(key);
    }
    let response = req.send().map_err(|e| {
        if e.is_connect() {
            // Keine Verbindung: die Anfrage hat den Anbieter nie erreicht.
            Fehler::Eindeutig("Verbindung zum Anbieter nicht möglich".to_string())
        } else {
            Fehler::Unklar("HTTP-Aufruf fehlgeschlagen; Ausgang unbekannt, keine automatische Wiederholung".to_string())
        }
    })?;
    let status = response.status().as_u16();
    if !(200..300).contains(&status) {
        return Err(Fehler::Eindeutig(format!("HTTP-Status {status}")));
    }
    let mut bytes = Vec::new();
    response
        .take(2_097_153)
        .read_to_end(&mut bytes)
        .map_err(|_| Fehler::Unklar("HTTP-Antwort unterbrochen".to_string()))?;
    if bytes.len() > 2_097_152 {
        return Err(Fehler::Eindeutig("HTTP-Antwort überschreitet 2 MiB".into()));
    }
    let raw: Value =
        serde_json::from_slice(&bytes).map_err(|_| Fehler::Eindeutig("HTTP-Antwort ist kein JSON".to_string()))?;
    if raw["choices"][0]["finish_reason"] == "length" {
        return Err(Fehler::Eindeutig(TOKENLIMIT.into()));
    }
    let text = raw["choices"][0]["message"]["content"]
        .as_str()
        .filter(|t| !t.trim().is_empty())
        .ok_or(Fehler::Eindeutig("Antwort enthält keinen Entscheidungstext".into()))?;
    Ok(
        json!({"text":text, "model":raw["model"], "id":raw["id"], "usage":raw["usage"], "provider":raw["provider"]}),
    )
}

/// Anfragekörper: Schema, Denkstufe und Providervorgaben nur bei OpenRouter; nach einer Antwort am
/// Tokenlimit (`mehr_platz`) doppelter Platz und die niedrigste Denkstufe.
pub fn koerper(p: &Provider, messages: &[Value], schema: Option<&Value>, mehr_platz: bool) -> Value {
    let max_tokens = if mehr_platz { p.max_tokens.saturating_mul(2) } else { p.max_tokens };
    let mut body = json!({"model":p.modell, "messages":messages, "max_tokens":max_tokens, "temperature":0, "stream":false, "response_format":{"type":"json_object"}});
    let erzwungen = p.schema && schema.is_some();
    if let (true, Some(s)) = (p.schema, schema) {
        body["response_format"] = json!({"type":"json_schema", "json_schema":{"name":"antwort", "strict":true, "schema":s}});
    }
    if p.art == "openrouter" {
        let mut vorgaben = json!({"allow_fallbacks":false});
        if erzwungen {
            // Nur Endpunkte, die das Schema wirklich erzwingen.
            vorgaben["require_parameters"] = json!(true);
        }
        for (k, v) in p.provider.as_ref().and_then(Value::as_object).into_iter().flatten() {
            vorgaben[k] = v.clone();
        }
        body["provider"] = vorgaben;
        // Denken: nach einer Antwort am Tokenlimit auf die niedrigste Stufe, die jedes Denkmodell kennt.
        let aufwand = match p.denken.as_deref() {
            Some("aus") => Some("none"),
            Some(_) if mehr_platz => Some("low"),
            Some("niedrig") => Some("low"),
            Some("mittel") => Some("medium"),
            Some("hoch") => Some("high"),
            _ => None,
        };
        if let Some(a) = aufwand {
            body["reasoning"] = json!({"effort": a});
        }
    }
    body
}

fn mock(messages: &[Value], model: &str) -> Value {
    let view: Value = serde_json::from_str(messages[1]["content"].as_str().unwrap_or("{}"))
        .unwrap_or(Value::Null);
    let role = view["rolle"].as_str().unwrap_or("");
    let mut actions = Vec::new();
    if messages.len() == 2 && role == "verwalter" {
        if let Some(planets) = view["planeten"].as_array() {
            for p in planets.iter().take(1) {
                if p["bauschleife"].as_array().is_some_and(Vec::is_empty) {
                    let building = if p["energie"]["erzeugung"].as_f64().unwrap_or(0.0)
                        <= p["energie"]["verbrauch"].as_f64().unwrap_or(0.0)
                    {
                        "solarkraftwerk"
                    } else if p["nahrung_deckung"].as_i64().unwrap_or(0) < 100 {
                        "farm"
                    } else {
                        "erzmine"
                    };
                    actions.push(json!({"typ":"bauen", "planet":p["koord"], "gebaeude":building}));
                }
            }
        }
    }
    let d = json!({"begruendung":"Offline-Regelagent: Versorgung prüfen und freie Bauschleife nutzen.", "abfragen":[], "aktionen":actions, "prognose":"Die Wirtschaft läuft bis zum nächsten Aufruf weiter.", "notiz":"Offline-Demo ohne Modellaufruf.", "wecker_stunden":24.0});
    json!({"text":d.to_string(), "model":model, "usage":{"prompt_tokens":0,"completion_tokens":0}, "offline":true})
}
