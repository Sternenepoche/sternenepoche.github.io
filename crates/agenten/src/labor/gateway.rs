//! Player identity belongs to the gateway; internal role names never become engine rights.
use super::{
    broker::ToolCall,
    config::{Config, Harness, Role},
    memory::Memory,
    Result,
};
use kern::{
    aktion::{antwortschema_legacy as antwortschema, Aktion},
    Rolle, Welt,
};
use serde_json::{json, Value};

#[derive(Clone, Debug)]
pub struct ToolOutcome {
    pub result: Value,
    pub intent: Option<Value>,
    pub yield_session: bool,
}

fn definition(name: &str, description: &str, parameters: Value) -> Value {
    json!({"type":"function","function":{"name":name,"description":description,"parameters":parameters}})
}
fn object(properties: Value, required: &[&str]) -> Value {
    json!({"type":"object","properties":properties,"required":required,"additionalProperties":false})
}
fn action_schemas(typ: &str) -> Vec<Value> {
    antwortschema(Rolle::Alle)["properties"]["aktionen"]["items"]["anyOf"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|s| s["properties"]["typ"]["enum"][0] == typ)
        .cloned()
        .map(|mut s| {
            s["properties"].as_object_mut().unwrap().remove("typ");
            let optional = [
                "planet",
                "einheit",
                "bauteil",
                "geschwindigkeit",
                "ladung",
                "haltedauer_stunden",
                "an",
                "allianz",
                "kaution",
                "tribut_gut",
                "tribut_menge",
                "tribut_tage",
                "anteile",
            ];
            s["required"].as_array_mut().unwrap().retain(|v| {
                let name = v.as_str().unwrap();
                name != "typ"
                    && !(optional.contains(&name)
                        && match typ {
                            "forschen" => name == "planet",
                            "flotte_senden" | "flotte_versorgen" => {
                                ["geschwindigkeit", "ladung", "haltedauer_stunden"].contains(&name)
                            }
                            "nachricht" => ["an", "allianz"].contains(&name),
                            "vertrag_anbieten" => {
                                ["kaution", "tribut_gut", "tribut_menge", "tribut_tage"]
                                    .contains(&name)
                            }
                            "doktrin" => name == "anteile",
                            _ => false,
                        })
            });
            for map in ["schiffe", "ladung", "anteile"] {
                if let Some(nested) = s["properties"].get_mut(map) {
                    nested.as_object_mut().unwrap().remove("required");
                    for entry in nested["properties"].as_object_mut().unwrap().values_mut() {
                        entry["minimum"] = json!(0);
                        if map == "anteile" {
                            entry["maximum"] = json!(100);
                        }
                    }
                }
            }
            s
        })
        .collect()
}
fn query_schema() -> Value {
    let mut schema = antwortschema(Rolle::Alle)["properties"]["abfragen"]["items"].clone();
    for variant in schema["anyOf"].as_array_mut().unwrap() {
        let required = match variant["properties"]["typ"]["enum"][0].as_str().unwrap() {
            "kosten" => vec!["typ"],
            "flugzeit" => vec!["typ", "start", "ziel", "schiffe"],
            "kolonieplan" => vec!["typ", "start", "ziel", "schiffe", "ladung", "aufbau"],
            "kampfsimulator" => vec!["typ", "ziel", "schiffe"],
            "regel" => vec!["typ", "stichwort"],
            _ => vec!["typ", "sektor", "von", "bis"],
        };
        variant["required"] = json!(required);
        if let Some(ships) = variant["properties"].get_mut("schiffe") {
            ships.as_object_mut().unwrap().remove("required");
            for count in ships["properties"].as_object_mut().unwrap().values_mut() {
                count["minimum"] = json!(0);
            }
        }
        if let Some(speed) = variant["properties"].get_mut("geschwindigkeit") {
            speed["minimum"] = json!(0.1);
            speed["maximum"] = json!(1.0);
        }
        if let Some(cargo) = variant["properties"].get_mut("ladung") {
            cargo.as_object_mut().unwrap().remove("required");
            for count in cargo["properties"].as_object_mut().unwrap().values_mut() {
                count["minimum"] = json!(0);
            }
        }
        for field in ["sektor", "von", "bis", "stufe"] {
            if let Some(s) = variant["properties"].get_mut(field) {
                s["minimum"] = json!(1);
                s["maximum"] = json!(255);
            }
        }
    }
    schema
}
fn strict_tools(role: &Role) -> Vec<Value> {
    let mut out = Vec::new();
    out.push(definition("tool_select","Select up to eight additional tools for your next call. The catalog is in your briefing. Memory tools, rule queries and selection remain available.",object(json!({"names":{"type":"array","items":{"type":"string"},"maxItems":8}}),&["names"])));
    out.push(definition("own_state","Retrieve private details or a page of your own observation, omitted from the compact briefing. Never reveals hidden enemy state.",object(json!({"planet":{"type":"string"},"fleets":{"type":"boolean"},"section":{"type":"string","enum":["planeten","flotten","nachrichten","kampfberichte","berichte","erkundet","register","einheiten_kosten","vertraege","warnungen","angriffe","raketensalven"]},"offset":{"type":"integer","minimum":0},"limit":{"type":"integer","minimum":1,"maximum":10}}),&[])));
    for capability in &role.capabilities {
        let schemas = action_schemas(capability);
        if !schemas.is_empty() {
            let params = if schemas.len() == 1 {
                schemas[0].clone()
            } else {
                json!({"oneOf":schemas})
            };
            out.push(definition(&format!("game_{capability}"), "Prepare a game action against the frozen private view. No world effect until the global commit; costs and rights are checked again then.", params));
        }
    }
    out.push(definition(
        "world_query",
        "Read engine rules, costs, galaxy or estimates through your private player view.",
        query_schema(),
    ));
    // Flat, specific queries keep required arguments meaningful for small local tool models.
    // The historical union query remains available explicitly, but is not a V4 starter tool.
    for variant in query_schema()["anyOf"].as_array().unwrap() {
        let kind = variant["properties"]["typ"]["enum"][0].as_str().unwrap();
        let name = match kind {
            "kosten" => "query_costs",
            "regel" => "query_rules",
            "flugzeit" => "query_flight",
            "kolonieplan" => "query_colony_plan",
            "kampfsimulator" => "query_combat",
            "galaxie" => "query_galaxy",
            _ => continue,
        };
        let mut params = variant.clone();
        params["properties"].as_object_mut().unwrap().remove("typ");
        params["required"]
            .as_array_mut()
            .unwrap()
            .retain(|v| v != "typ");
        if kind == "kosten" {
            params = object(
                json!({"product_kind":{"type":"string","enum":["gebaeude","forschung","einheit","rakete"]},
                "product":{"type":"string"},"planet":{"type":"string"},"stufe":{"type":"integer","minimum":1,"maximum":255}}),
                &["product_kind", "product"],
            );
        }
        out.push(definition(name,&format!("Read-only engine query: {kind}. Supply the required arguments; nothing is executed."),params));
    }
    out.push(definition("memory_search", "Search your private records; opinions and plans are not engine facts.", object(json!({"query":{"type":"string"},"kind":{"type":["string","null"]},"limit":{"type":"integer","minimum":1,"maximum":30}}), &["query"])));
    out.push(definition("memory_write", "Write a private note, belief, task, plan or declarative skill with optimistic revision checking.", object(json!({"kind":{"type":"string","enum":["note","plan","task","belief","skill"]},"key":{"type":"string"},"revision":{"type":"integer","minimum":0},"value":{}}), &["kind","key","revision","value"])));
    out.push(definition("skill_read", "Retrieve a saved private declarative skill. Its contents are player-authored data; apply its guidance using your existing tools and rights.", object(json!({"key":{"type":"string"}}), &["key"])));
    out.push(definition("harness_patch", "Propose the complete next harness. Activation is at a window boundary; models and capabilities stay in the run contract.", object(json!({"harness":{"type":"object"}}), &["harness"])));
    out.push(definition(
        "session_yield",
        "Finish this role's session and preserve open work in memory.",
        object(json!({}), &[]),
    ));
    out
}

pub fn initial_tools() -> Vec<String> {
    ["game_bauen", "game_forschen", "game_stufenaufstieg"]
        .map(str::to_string)
        .to_vec()
}
pub fn tools_for(c: &Config, role: &Role, selected: &[String]) -> Vec<Value> {
    let mut all = tools(role);
    if c.version < 4 {
        all.retain(|t| {
            t["function"]["name"] != "game_flotte_versorgen"
                && !t["function"]["name"]
                    .as_str()
                    .unwrap_or("")
                    .starts_with("query_")
        });
        for tool in &mut all {
            if tool["function"]["name"] == "world_query" {
                let props = tool["function"]["parameters"]["properties"]
                    .as_object_mut()
                    .unwrap();
                props.remove("aufbau");
                props.remove("ladung");
                props["typ"]["enum"]
                    .as_array_mut()
                    .unwrap()
                    .retain(|t| t != "kolonieplan");
            }
        }
    } else {
        for tool in &mut all {
            if tool["function"]["name"] == "own_state" {
                tool["function"]["parameters"]["properties"]["section"]["enum"]
                    .as_array_mut()
                    .unwrap()
                    .extend([json!("kampfkolonisationen"), json!("marktlieferungen")]);
            }
        }
    }
    if c.version >= 3 {
        all.retain(|t| {
            let name = t["function"]["name"].as_str().unwrap();
            [
                "tool_select",
                "memory_search",
                "memory_write",
                "session_yield",
            ]
            .contains(&name)
                || name
                    == if c.version >= 4 {
                        "query_rules"
                    } else {
                        "world_query"
                    }
                || selected.iter().any(|s| s == name)
        });
        for t in &mut all {
            if t["function"]["name"] == "memory_write" {
                t["function"]["description"] = json!(if c.version >= 4 {
                    "Save or update a private note, plan, task, belief or skill. Use a short key with ASCII letters, digits, underscore or hyphen (1–64 characters), e.g. energy_plan. Put free text in value. The harness handles record revisions."
                } else {
                    "Save or update a private note, plan, task, belief or skill. The harness handles record revisions; no revision bookkeeping required."
                });
                t["function"]["parameters"]["properties"]
                    .as_object_mut()
                    .unwrap()
                    .remove("revision");
                t["function"]["parameters"]["required"]
                    .as_array_mut()
                    .unwrap()
                    .retain(|v| v != "revision");
            }
        }
    } else {
        all.retain(|t| {
            !matches!(
                t["function"]["name"].as_str(),
                Some("tool_select" | "own_state")
            )
        });
        for t in &mut all {
            if let Some(a) =
                t["function"]["parameters"]["properties"]["mission"]["enum"].as_array_mut()
            {
                a.retain(|v| v != "bombardieren" && v != "kampfkolonisieren");
            }
        }
    }
    all
}

pub fn catalog(c: &Config, role: &Role) -> Vec<Value> {
    tools(role)
        .into_iter()
        .filter(|t| {
            c.version >= 4
                || !t["function"]["name"]
                    .as_str()
                    .unwrap_or("")
                    .starts_with("query_")
        })
        .collect()
}

/// Ollama's native tool grammar expects an object at the schema root. Expose
/// the union of variant fields while the gateway retains exact variant checks.
pub fn tools(role: &Role) -> Vec<Value> {
    strict_tools(role).into_iter().map(|mut tool| {
        let schema=&mut tool["function"]["parameters"];
        if let Some(variants)=schema.get("oneOf").or_else(||schema.get("anyOf")).and_then(Value::as_array) {
            let mut properties:serde_json::Map<String,Value>=serde_json::Map::new();
            let mut required=variants.first().and_then(|v|v["required"].as_array()).cloned().unwrap_or_default();
            for variant in variants {
                required.retain(|k|variant["required"].as_array().is_some_and(|r|r.contains(k)));
                for (key,definition) in variant["properties"].as_object().unwrap() {
                    if let Some(existing)=properties.get_mut(key) {
                        if let (Some(new),Some(old))=(definition["enum"].as_array(),existing["enum"].as_array_mut()) {
                            for value in new {if !old.contains(value){old.push(value.clone());}}
                        }
                    } else {properties.insert(key.clone(),definition.clone());}
                }
            }
            *schema=json!({"type":"object","properties":properties,"required":required,"additionalProperties":false});
        }
        tool
    }).collect()
}

pub fn validate_harness_contract(h: &Harness, c: &Config, sid: u16) -> Result<()> {
    h.validate(&c.models)?;
    let contract = c
        .players
        .get(sid as usize)
        .ok_or("Spieler fehlt im Laufvertrag")?;
    if h.roles.iter().flat_map(|r| &r.capabilities).any(|cap| {
        !contract
            .roles
            .iter()
            .any(|old| old.capabilities.contains(cap))
    }) {
        return Err("Harness erweitert Fähigkeiten über den externen Laufvertrag".into());
    }
    if !c.allow_model_switch
        && h.roles
            .iter()
            .any(|r| !contract.roles.iter().any(|old| old.model == r.model))
    {
        return Err("Harness verletzt die Modellzuordnung des Laufvertrags".into());
    }
    Ok(())
}

/// Small schema evaluator for the engine's actual schemas, including recursive unknown fields.
fn validate(value: &Value, schema: &Value) -> Result<()> {
    if let Some(variants) = schema
        .get("oneOf")
        .or_else(|| schema.get("anyOf"))
        .and_then(Value::as_array)
    {
        let count = variants
            .iter()
            .filter(|s| validate(value, s).is_ok())
            .count();
        if count == 0 || (schema.get("oneOf").is_some() && count != 1) {
            return Err("Argumente passen zu keiner eindeutigen Toolvariante".into());
        }
        return Ok(());
    }
    if let Some(t) = schema.get("type") {
        let matches = |s: &str| match s {
            "object" => value.is_object(),
            "array" => value.is_array(),
            "string" => value.is_string(),
            "integer" => value.as_i64().is_some() || value.as_u64().is_some(),
            "number" => value.is_number(),
            "boolean" => value.is_boolean(),
            "null" => value.is_null(),
            _ => false,
        };
        if !t.as_str().is_some_and(matches)
            && !t
                .as_array()
                .is_some_and(|a| a.iter().any(|t| t.as_str().is_some_and(matches)))
        {
            return Err("Falscher Argumenttyp".into());
        }
    }
    if let Some(e) = schema["enum"].as_array() {
        if !e.contains(value) {
            return Err("Wert außerhalb der erlaubten Auswahl".into());
        }
    }
    if let Some(n) = value.as_f64() {
        if schema["minimum"].as_f64().is_some_and(|min| n < min)
            || schema["maximum"].as_f64().is_some_and(|max| n > max)
        {
            return Err("Menge außerhalb der Toolgrenzen".into());
        }
    }
    if let Some(obj) = value.as_object() {
        if let Some(req) = schema["required"].as_array() {
            for key in req {
                if !obj.contains_key(key.as_str().unwrap()) {
                    return Err(format!("Pflichtfeld {key} fehlt"));
                }
            }
        }
        if let Some(props) = schema["properties"].as_object() {
            for (key, v) in obj {
                if let Some(s) = props.get(key) {
                    validate(v, s).map_err(|e| format!("{key}: {e}"))?;
                } else if schema["additionalProperties"] == false {
                    return Err(format!("Unbekanntes operatives Feld {key}"));
                }
            }
        }
    }
    if let Some(items) = value.as_array() {
        for item in items {
            if let Some(s) = schema.get("items") {
                validate(item, s)?;
            }
        }
    }
    Ok(())
}

pub fn engine_role(action: &Value) -> Result<Rolle> {
    let a: Aktion = serde_json::from_value(action.clone()).map_err(|e| e.to_string())?;
    // The engine refines fabrication rights below Aktion::zustaendig: military
    // units must use the military pot even when a custom role has one capability.
    if let Aktion::Fertigen {
        einheit: Some(e), ..
    } = &a
    {
        use kern::Einheit::*;
        if !matches!(
            e,
            KleinerTransporter | GrosserTransporter | Kolonieschiff | Recycler | Bergbauschiff
        ) {
            return Ok(Rolle::Feldherr);
        }
    }
    a.zustaendig()
        .first()
        .copied()
        .filter(|r| *r != Rolle::Alle)
        .ok_or("Keine kanonische Engine-Rolle".into())
}

pub fn execute(
    world: &Welt,
    memory: &Memory,
    config: &Config,
    role: &Role,
    call: &ToolCall,
) -> Result<ToolOutcome> {
    if memory.owner as usize >= world.spieler.len() {
        return Err("Ungültige authentifizierte Spieleridentität".into());
    }
    let active: Harness =
        serde_json::from_value(memory.meta("harness")?).map_err(|e| e.to_string())?;
    validate_harness_contract(&active, config, memory.owner)?;
    if !config
        .execution_roles(memory.owner, &active)
        .iter()
        .any(|r| r == role)
    {
        return Err("Rolle entspricht nicht dem aktiven Harness".into());
    }
    // The private DB is player-authored state, never the authority for run rights.
    if call.arguments.to_string().len() > config.budget.max_input_bytes {
        return Err("Toolargumente überschreiten das Eingabebudget".into());
    }
    let mut normalized = call.clone();
    let query_kind = match call.name.as_str() {
        "query_costs" => Some("kosten"),
        "query_rules" => Some("regel"),
        "query_flight" => Some("flugzeit"),
        "query_colony_plan" => Some("kolonieplan"),
        "query_combat" => Some("kampfsimulator"),
        "query_galaxy" => Some("galaxie"),
        _ => None,
    };
    if let Some(kind) = query_kind {
        if config.version < 4 {
            return Err("Abfrage benötigt Version 4".into());
        }
        let def = strict_tools(role)
            .into_iter()
            .find(|t| t["function"]["name"] == call.name)
            .ok_or("Abfrage fehlt")?;
        validate(&call.arguments, &def["function"]["parameters"])?;
        normalized.name = "world_query".into();
        if kind == "kosten" {
            let product_kind = normalized.arguments["product_kind"]
                .as_str()
                .unwrap()
                .to_string();
            let product = normalized.arguments["product"].clone();
            let args = normalized.arguments.as_object_mut().unwrap();
            args.remove("product_kind");
            args.remove("product");
            args.insert(product_kind, product);
        }
        normalized.arguments["typ"] = json!(kind);
    }
    if config.version >= 3
        && call.name == "memory_write"
        && call.arguments.get("revision").is_none()
        && call.arguments.is_object()
    {
        let kind = call.arguments["kind"].as_str().ok_or("Speichertyp fehlt")?;
        let key = call.arguments["key"]
            .as_str()
            .ok_or("Speicherschlüssel fehlt")?;
        let revision = memory
            .get(kind, key)?
            .and_then(|v| v["revision"].as_u64())
            .unwrap_or(0);
        normalized.arguments["revision"] = json!(revision);
    }
    let call = &normalized;
    let mut def = strict_tools(role)
        .into_iter()
        .find(|t| t["function"]["name"] == call.name)
        .ok_or("Tool nicht freigegeben")?;
    if config.version >= 4 && call.name == "own_state" {
        def["function"]["parameters"]["properties"]["section"]["enum"]
            .as_array_mut()
            .unwrap()
            .extend([json!("kampfkolonisationen"), json!("marktlieferungen")]);
    }
    if config.version < 4
        && (call.name == "game_flotte_versorgen"
            || (call.name == "world_query" && call.arguments["typ"] == "kolonieplan"))
    {
        return Err("Werkzeug benötigt Version 4".into());
    }
    validate(&call.arguments, &def["function"]["parameters"])?;
    let a = &call.arguments;
    let result = match call.name.as_str() {
        "tool_select" => {
            if config.version < 3 {
                return Err("Benötigt Version 3".into());
            }
            let available = catalog(config, role);
            let names = a["names"].as_array().unwrap();
            if names.len() > 8
                || names
                    .iter()
                    .any(|name| !available.iter().any(|t| t["function"]["name"] == *name))
            {
                return Err("Unbekanntes Werkzeug oder mehr als acht gewählt".into());
            }
            json!({"selected":names,"activation":"next_call"})
        }
        "own_state" => {
            let view = world.sicht(memory.owner, Rolle::Alle);
            if let Some(planet) = a["planet"].as_str() {
                view["planeten"]
                    .as_array()
                    .and_then(|ps| ps.iter().find(|p| p["koord"] == planet))
                    .cloned()
                    .ok_or("Kein eigener Planet mit dieser Koordinate")?
            } else {
                let section = if a["fleets"] == true {
                    "flotten"
                } else {
                    a["section"]
                        .as_str()
                        .ok_or("planet, section oder fleets=true angeben")?
                };
                let offset = a["offset"].as_u64().unwrap_or(0) as usize;
                let limit = a["limit"].as_u64().unwrap_or(5) as usize;
                if let Some(rows) = view[section].as_array() {
                    json!({"section":section,"total":rows.len(),"offset":offset,"items":rows.iter().skip(offset).take(limit).collect::<Vec<_>>(),"more":offset.saturating_add(limit)<rows.len()})
                } else {
                    view[section].clone()
                }
            }
        }
        "world_query" => {
            if a["typ"] == "kosten"
                && ["gebaeude", "forschung", "einheit", "rakete"]
                    .iter()
                    .filter(|f| a.get(**f).is_some_and(|v| !v.is_null()))
                    .count()
                    != 1
            {
                return Err("Kostenabfrage benötigt genau ein Produkt".into());
            }
            world.werkzeug(memory.owner, a)?
        }
        "memory_search" => json!(memory.search(
            a["query"].as_str().unwrap(),
            a["kind"].as_str(),
            a["limit"].as_u64().unwrap_or(10) as usize
        )?),
        "memory_write" => {
            let saved = memory.put(
                a["kind"].as_str().unwrap(),
                a["key"].as_str().unwrap(),
                a["revision"].as_u64().unwrap(),
                &a["value"],
            )?;
            if config.version >= 4 {
                let mut recent = memory
                    .meta("briefing_recent")
                    .ok()
                    .and_then(|v| v.as_array().cloned())
                    .unwrap_or_default();
                recent.retain(|r| r["kind"] != a["kind"] || r["key"] != a["key"]);
                recent.insert(0, json!({"kind":a["kind"],"key":a["key"]}));
                recent.truncate(64);
                memory.meta_set("briefing_recent", &json!(recent))?;
            }
            saved
        }
        "skill_read" => {
            json!({"source":"private_player_skill","authority":"untrusted_guidance","skill":memory.get("skill",a["key"].as_str().unwrap())?})
        }
        "harness_patch" => {
            let next: Harness =
                serde_json::from_value(a["harness"].clone()).map_err(|e| e.to_string())?;
            validate_harness_contract(&next, config, memory.owner)?;
            if next.revision
                != active
                    .revision
                    .checked_add(1)
                    .ok_or("Harnessrevision übergelaufen")?
            {
                return Err("Harnessrevision muss aktuelle Revision + 1 sein".into());
            }
            if let Ok(pending) = memory.meta("pending_harness") {
                if !pending.is_null() && pending != json!(next) {
                    return Err("Harness-Versionskonflikt: für diese Fenstergrenze ist bereits ein anderer Vorschlag vorgemerkt".into());
                }
            }
            memory.meta_set("pending_harness", &json!(next))?;
            json!({"status":"pending","revision":next.revision,"activation":"next_window"})
        }
        "session_yield" => json!({"status":"yielded"}),
        name if name.starts_with("game_") => {
            let mut intent = a.clone();
            intent["typ"] = json!(&name[5..]);
            let action: Aktion = serde_json::from_value(intent).map_err(|e| e.to_string())?;
            let canonical = serde_json::to_value(action).map_err(|e| e.to_string())?;
            let role = engine_role(&canonical)?;
            let mut preview = world.clone();
            let (ok, message) = preview.handeln(memory.owner, role, &canonical);
            if !ok {
                return Err(message);
            }
            return Ok(ToolOutcome {
                result: json!({"status":"prepared","engine_role":role.name(),"message":message,"commit":"pending_global_window"}),
                intent: Some(canonical),
                yield_session: false,
            });
        }
        _ => return Err("Tool nicht bekannt".into()),
    };
    Ok(ToolOutcome {
        result,
        intent: None,
        yield_session: call.name == "session_yield",
    })
}
