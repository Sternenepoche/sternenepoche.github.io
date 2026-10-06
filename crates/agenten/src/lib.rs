//! Direct Rust orchestration. Simulation time is frozen during every provider barrier.
pub mod config;
pub mod export;
pub mod journal;
pub mod labor;
pub mod protocol;
pub mod provider;

use config::Config;
use journal::Journal;
use kern::{Rolle, Welt};
use protocol::Decision;
use serde_json::{json, Value};
use std::path::Path;

pub const RULES: &str = include_str!("../../../regeln/regelwerk.ron");
const PROTOCOL_VERSION: &str = "rust-agenten-v1";

#[derive(Clone)]
struct Entry {
    sid: u16,
    role: Rolle,
    messages: Vec<Value>,
    decision: Option<Decision>,
    errors: Vec<String>,
    reasons: Vec<String>,
    cost: f64,
    results: Vec<Value>,
    /// The provider call failed for good: no correction is requested for it.
    failed: bool,
}

/// One call with bounded retries. Every attempt has its own journal key: an attempt that failed
/// unambiguously (HTTP status, empty or truncated answer, no connection) is finished with its error
/// and replays identically on resume. An ambiguous outcome stays open and, as before, blocks any
/// automatic retry, because the provider may have processed and billed it.
fn call_with_retries(
    journal: &Journal,
    key: &str,
    request: &Value,
    p: &config::Provider,
    e: &Entry,
    max: usize,
) -> Result<Value, String> {
    let schema = kern::aktion::antwortschema(e.role);
    let mut more_room = false;
    let mut last = String::new();
    for attempt in 1..=p.versuche {
        let (k, mut req) = if attempt == 1 {
            (key.to_string(), request.clone())
        } else {
            (format!("{key}-v{attempt}"), request.clone())
        };
        if attempt > 1 {
            req["versuch"] = json!(attempt);
            req["mehr_platz"] = json!(more_room);
        }
        let answer = match journal.reserve(&k, &req, max)? {
            Some(v) => v,
            None => {
                let v = match provider::anfrage(p, &e.messages, Some(&schema), more_room) {
                    Ok(v) => v,
                    Err(provider::Fehler::Unklar(t)) => return Err(t),
                    // Key, credit or key limit: retrying cannot help and every role would idle.
                    // The provider refused before processing, so the reservation is withdrawn and
                    // the call is sent again once the run continues.
                    Err(provider::Fehler::Eindeutig(t)) if provider::abgelehnt(&t) => {
                        journal.cancel(&k)?;
                        return Err(format!(
                            "{t}: der Anbieter lehnt ab – Schlüssel, Guthaben oder Schlüssellimit prüfen. \
                             Der Lauf hält an; beim Fortsetzen wird dieser Aufruf neu gestellt."
                        ));
                    }
                    Err(provider::Fehler::Eindeutig(t)) => json!({"fehler": t}),
                };
                journal.finish(&k, &v)?;
                if v.get("fehler").is_some() && attempt < p.versuche {
                    std::thread::sleep(std::time::Duration::from_millis(1500 * u64::from(attempt)));
                }
                v
            }
        };
        match answer.get("fehler").and_then(Value::as_str) {
            None => return Ok(answer),
            Some(t) => {
                more_room |= t == provider::TOKENLIMIT;
                last = t.to_string();
            }
        }
    }
    Ok(json!({"text": "", "fehler": format!("nach {} Versuchen aufgegeben: {last}", p.versuche)}))
}

/// Persist all reservations before their I/O. Join *every* worker before returning a
/// failure so successful concurrent requests remain reusable after restart. Calls that
/// failed unambiguously arrive as an answer with `fehler`; the role skips that call.
fn barrier(
    welt: &Welt,
    entries: &[Entry],
    config: &Config,
    journal: &Journal,
    phase: &str,
) -> Result<Vec<Value>, String> {
    let mut results: Vec<Option<Value>> = vec![None; entries.len()];
    let mut first_error = None;
    for (offset, chunk) in entries.chunks(config.parallel).enumerate() {
        std::thread::scope(|scope| {
            let jobs: Vec<_> = chunk
                .iter()
                .enumerate()
                .map(|(i, e)| {
                    let index = offset * config.parallel + i;
                    let p = &config.anbieter[&config.rollen[e.role.name()]];
                    let key = format!("{:012}-{:03}-{}-{phase}", welt.zeit, e.sid, e.role.name());
                    let request = json!({"zeit":welt.zeit, "spieler":e.sid, "rolle":e.role.name(), "phase":phase, "provider":p, "messages":e.messages});
                    let max = config.max_anfragen;
                    (index, scope.spawn(move || call_with_retries(journal, &key, &request, p, e, max)))
                })
                .collect();
            for (index, handle) in jobs {
                match handle.join() {
                    Ok(Ok(value)) => results[index] = Some(value),
                    Ok(Err(e)) => {
                        first_error.get_or_insert(e);
                    }
                    Err(_) => {
                        first_error.get_or_insert("Provider-Thread abgebrochen".into());
                    }
                }
            }
        });
        if first_error.is_some() {
            break;
        }
    }
    if let Some(e) = first_error {
        return Err(e);
    }
    results
        .into_iter()
        .map(|r| r.ok_or_else(|| "Unvollständige Entscheidungsbarriere".into()))
        .collect()
}

fn parse_round(welt: &Welt, entries: &mut [Entry], responses: Vec<Value>, queries: bool) {
    for (e, response) in entries.iter_mut().zip(responses) {
        let text = response["text"].as_str().unwrap_or("");
        e.cost += response["usage"]["cost"].as_f64().unwrap_or(0.0);
        e.messages.push(json!({"role":"assistant", "content":text}));
        if let Some(fehler) = response.get("fehler").and_then(Value::as_str) {
            // Der Aufruf ist endgültig gescheitert: die Rolle setzt aus, der Lauf geht weiter.
            e.errors.push(format!("Aufruf gescheitert: {fehler}"));
            e.decision = None;
            e.failed = true;
            continue;
        }
        match Decision::parse(text, welt, e.role, queries) {
            Ok(d) => e.decision = Some(d),
            Err(error) => {
                e.errors.push(error);
                e.decision = None;
            }
        }
    }
}

/// A complete decision window, including bounded read tools and one correction.
/// Model identity is never trusted for player or role: these come from `faellig`.
/// Returns one report per call: reasons, decision texts, every action with its result,
/// errors and the provider cost (`usage.cost`) of all rounds.
pub fn window(welt: &mut Welt, config: &Config, journal: &Journal) -> Result<Vec<Value>, String> {
    let mut entries: Vec<Entry> = welt
        .faellig
        .iter()
        .map(|f| Entry {
            sid: f.spieler,
            role: f.rolle,
            messages: protocol::messages(welt, f.spieler, f.rolle),
            decision: None,
            errors: Vec::new(),
            reasons: f.gruende.clone(),
            cost: 0.0,
            results: Vec::new(),
            failed: false,
        })
        .collect();
    if entries.is_empty() {
        return Ok(Vec::new());
    }
    let responses = barrier(welt, &entries, config, journal, "entscheidung")?;
    parse_round(welt, &mut entries, responses, true);
    let mut querying = Vec::new();
    let mut query_indices = Vec::new();
    for (index, e) in entries.iter().enumerate() {
        if let Some(d) = &e.decision {
            if !d.abfragen.is_empty() {
                let answers: Vec<Value> = d
                    .abfragen
                    .iter()
                    .map(|q| match welt.werkzeug(e.sid, q) {
                        Ok(a) => json!({"abfrage":q,"ergebnis":a}),
                        Err(error) => json!({"abfrage":q,"fehler":error}),
                    })
                    .collect();
                let mut next = e.clone();
                next.messages.push(json!({"role":"user", "content":format!("Ergebnisse deiner lesenden Abfragen: {}. Entscheide jetzt; abfragen muss [] sein.",json!(answers))}));
                querying.push(next);
                query_indices.push(index);
            }
        }
    }
    let responses = barrier(welt, &querying, config, journal, "abfragen")?;
    parse_round(welt, &mut querying, responses, false);
    for (index, e) in query_indices.into_iter().zip(querying) {
        entries[index] = e;
    }

    // All original observations and queries precede *any* action in the window.
    // faellig already follows the engine's seeded order.
    let mut corrections = Vec::new();
    let mut correction_indices = Vec::new();
    for (index, e) in entries.iter_mut().enumerate() {
        if let Some(d) = &e.decision {
            for action in &d.aktionen {
                let (ok, text) = welt.handeln(e.sid, e.role, action);
                e.results.push(json!({"aktion":action, "ok":ok, "text":text}));
                if !ok {
                    e.errors.push(format!("{}: {text}", action));
                }
            }
        }
        if !e.errors.is_empty() && !e.failed {
            let mut next = e.clone();
            next.messages.push(json!({"role":"user", "content":format!("Fehler: {}. Bereits angenommene Aktionen gelten. Eine Korrektur: sende nur Ersatz für abgelehnte Aktionen; abfragen muss [] sein.",json!(e.errors))}));
            next.errors.clear();
            corrections.push(next);
            correction_indices.push(index);
        }
    }
    let responses = barrier(welt, &corrections, config, journal, "korrektur")?;
    parse_round(welt, &mut corrections, responses, false);
    for (index, mut e) in correction_indices.into_iter().zip(corrections) {
        if let Some(d) = &e.decision {
            for action in &d.aktionen {
                let (ok, text) = welt.handeln(e.sid, e.role, action);
                e.results.push(json!({"aktion":action, "ok":ok, "text":text, "korrektur":true}));
                if !ok {
                    e.errors.push(text);
                }
            }
        }
        // Invalid correction must not erase a previously valid notebook.
        if e.decision.is_none() {
            e.decision = entries[index].decision.clone();
        }
        entries[index] = e;
    }
    let mut reports = Vec::new();
    for e in entries {
        let note = e.decision.as_ref().map(|d| d.notiz.as_str());
        let alarm = e
            .decision
            .as_ref()
            .and_then(|d| d.wecker_stunden)
            .map(|h| (h * 3600.0).round() as i64);
        reports.push(json!({
            "zeit": welt.zeit,
            "spieler": e.sid,
            "name": welt.spieler[e.sid as usize].name,
            "rolle": e.role.name(),
            "gruende": e.reasons,
            "begruendung": e.decision.as_ref().map(|d| d.begruendung.as_str()),
            "prognose": e.decision.as_ref().map(|d| d.prognose.as_str()),
            "notiz": note,
            "wecker_stunden": e.decision.as_ref().and_then(|d| d.wecker_stunden),
            "aktionen": e.results,
            "fehler": e.errors,
            "kosten": e.cost,
        }));
        welt.aufruf_ende(e.sid, e.role, note, alarm, &e.errors);
    }
    Ok(reports)
}

/// Resume from an immutable complete checkpoint. On interruption, an unfinished
/// window replays from its previous checkpoint using cached provider responses.
pub fn run(config: &Config, output: &Path, windows: usize, execute: bool) -> Result<Value, String> {
    config.validate()?;
    if !execute
        && config
            .rollen
            .values()
            .any(|name| config.anbieter[name].art != "mock")
    {
        return Err(
            "Netzwerkaufrufe benötigen --execute; demo arbeitet vollständig offline".into(),
        );
    }
    let mut rules = kern::Regelwerk::laden(RULES)?;
    rules.welt.epoche_tage = config.tage;
    let manifest = json!({"protocol":PROTOCOL_VERSION,"rules_hash":journal::hash(RULES.as_bytes()),"config":config});
    let journal = Journal::open(output, &manifest)?;
    let (mut index, mut world) = match journal.load_world()? {
        Some(x) => x,
        None => {
            let w = Welt::neu(rules, config.startwert, config.spieler)?;
            journal.save_world(0, &w)?;
            (0, w)
        }
    };
    let mut processed = 0;
    while processed < windows && !world.beendet() {
        // Mutations occur on a working copy; failed barriers leave the last
        // committed world intact, including if a correction request failed.
        let mut next = world.clone();
        window(&mut next, config, &journal)?;
        let log = next.log_abholen();
        journal.put(
            &format!("states/{index:08}.actions.json"),
            &serde_json::to_vec(&log).map_err(|e| e.to_string())?,
        )?;
        next.schritt();
        index += 1;
        journal.save_world(index, &next)?;
        world = next;
        processed += 1;
    }
    Ok(
        json!({"zeit":world.zeit,"tag":world.zeit/kern::TAG+1,"fenster":index,"anfragen":journal.request_count()?,"hash":world.hash(),"beendet":world.beendet(),"rangliste":world.rangliste()}),
    )
}
