//! Antwortschema gegen Kern: Was ein Modell unter erzwungenem Schema senden kann, muss der Kern
//! verstehen, und umgekehrt muss das Schema jedes Feld kennen, das der Kern liest. Sonst wird eine
//! Aktion immer abgelehnt (so ging es den vier Aktionen für Raketen und Verbandsangriff) oder eine
//! Angabe stillschweigend übergangen.

use kern::aktion::{antwortschema, erlaubte_typen, Aktion};
use kern::regeltext::aktionsbeispiel;
use kern::typen::*;
use kern::welt::*;
use kern::Regelwerk;
use serde_json::{json, Value};

const REGELN: &str = include_str!("../../../regeln/regelwerk.ron");
const ROLLEN: [Rolle; 5] = [Rolle::Alle, Rolle::Stratege, Rolle::Verwalter, Rolle::Feldherr, Rolle::Diplomat];

/// Ein gültiger Wert für ein Teilschema: erster erlaubter Wert, Koordinaten für Ortsfelder.
fn beispiel(feld: &str, s: &Value) -> Value {
    if let Some(varianten) = s.get("anyOf").and_then(Value::as_array) {
        return beispiel(feld, &varianten[0]);
    }
    if let Some(werte) = s.get("enum").and_then(Value::as_array) {
        return werte.iter().find(|w| !w.is_null()).cloned().unwrap_or(Value::Null);
    }
    let typen: Vec<&str> = match &s["type"] {
        Value::String(t) => vec![t.as_str()],
        Value::Array(a) => a.iter().filter_map(Value::as_str).collect(),
        _ => vec![],
    };
    let typ = typen.iter().find(|t| **t != "null").copied().unwrap_or("null");
    match typ {
        "object" => {
            let mut o = serde_json::Map::new();
            for (n, t) in s["properties"].as_object().into_iter().flatten() {
                o.insert(n.clone(), beispiel(n, t));
            }
            Value::Object(o)
        }
        "array" => json!([beispiel(feld, &s["items"])]),
        "string" if ["planet", "start", "ziel"].contains(&feld) => json!("1:1:1"),
        "string" => json!("x"),
        "integer" => json!(1),
        "number" => json!(1.0),
        "boolean" => json!(false),
        _ => Value::Null,
    }
}

/// Prüft einen Wert gegen das Teilschema. Ohne `streng` dürfen Felder fehlen (Beispiele im Regeltext
/// lassen Felder mit Standardwert weg), unbekannte Felder und falsche Typen oder Werte nie.
fn gilt(s: &Value, w: &Value, pfad: &str, streng: bool) -> Result<(), String> {
    if let Some(varianten) = s.get("anyOf").and_then(Value::as_array) {
        let fehler: Vec<String> = varianten.iter().filter_map(|v| gilt(v, w, pfad, streng).err()).collect();
        return if fehler.len() < varianten.len() { Ok(()) } else { Err(format!("{pfad}: keine Variante passt ({})", fehler.join("; "))) };
    }
    let typen: Vec<&str> = match &s["type"] {
        Value::String(t) => vec![t.as_str()],
        Value::Array(a) => a.iter().filter_map(Value::as_str).collect(),
        _ => vec![],
    };
    let passt = |t: &str| match t {
        "null" => w.is_null(),
        "boolean" => w.is_boolean(),
        "integer" => w.is_i64() || w.is_u64(),
        "number" => w.is_number(),
        "string" => w.is_string(),
        "array" => w.is_array(),
        "object" => w.is_object(),
        _ => false,
    };
    if !typen.is_empty() && !typen.iter().any(|t| passt(t)) {
        return Err(format!("{pfad}: {w} ist nicht vom Typ {typen:?}"));
    }
    if let Some(werte) = s.get("enum").and_then(Value::as_array) {
        if !werte.contains(w) {
            return Err(format!("{pfad}: {w} ist nicht erlaubt"));
        }
    }
    if let Some(x) = w.as_f64() {
        if s.get("minimum").and_then(Value::as_f64).is_some_and(|m| x < m) || s.get("maximum").and_then(Value::as_f64).is_some_and(|m| x > m) {
            return Err(format!("{pfad}: {w} liegt außerhalb der Grenzen"));
        }
    }
    if let (Some(props), Some(o)) = (s.get("properties").and_then(Value::as_object), w.as_object()) {
        for (n, v) in o {
            let t = props.get(n).ok_or_else(|| format!("{pfad}: unbekanntes Feld {n}"))?;
            gilt(t, v, &format!("{pfad}.{n}"), streng)?;
        }
        if streng {
            for n in s["required"].as_array().into_iter().flatten().filter_map(Value::as_str) {
                if !o.contains_key(n) {
                    return Err(format!("{pfad}: Pflichtfeld {n} fehlt"));
                }
            }
        }
    }
    if let (Some(items), Some(a)) = (s.get("items"), w.as_array()) {
        for (i, v) in a.iter().enumerate() {
            gilt(items, v, &format!("{pfad}[{i}]"), streng)?;
        }
    }
    Ok(())
}

fn varianten(schema: &Value, feld: &str) -> Vec<Value> {
    schema["properties"][feld]["items"]["anyOf"].as_array().cloned().unwrap_or_default()
}

fn typ_von(v: &Value) -> String {
    v["properties"]["typ"]["enum"][0].as_str().unwrap().to_string()
}

#[test]
fn jede_rolle_bekommt_genau_ihre_aktionen() {
    for rolle in ROLLEN {
        let schema = antwortschema(rolle);
        let mut im_schema: Vec<String> = varianten(&schema, "aktionen").iter().map(typ_von).collect();
        let mut erlaubt: Vec<String> = erlaubte_typen(rolle).iter().map(|t| t.to_string()).collect();
        im_schema.sort();
        // fertigen steht zweimal im Schema: mit Einheit oder mit Bauteil.
        im_schema.dedup();
        erlaubt.sort();
        assert_eq!(im_schema, erlaubt, "Rolle {rolle}");
    }
}

#[test]
fn was_das_schema_erlaubt_versteht_der_kern() {
    for rolle in ROLLEN {
        let schema = antwortschema(rolle);
        for v in varianten(&schema, "aktionen") {
            let typ = typ_von(&v);
            let grund = beispiel("", &v);
            assert!(gilt(&v, &grund, &typ, true).is_ok(), "{typ}: erzeugtes Beispiel verletzt das Schema");
            // Jeder erlaubte Wert jedes Aufzählungsfelds einzeln, auch null.
            let mut proben = vec![grund.clone()];
            for (feld, t) in v["properties"].as_object().unwrap() {
                for wert in t.get("enum").and_then(Value::as_array).into_iter().flatten() {
                    let mut p = grund.clone();
                    p[feld] = wert.clone();
                    proben.push(p);
                }
            }
            for p in proben {
                let a: Aktion = serde_json::from_value(p.clone()).unwrap_or_else(|e| panic!("Rolle {rolle}, {typ}: {p} nicht lesbar: {e}"));
                if rolle != Rolle::Alle {
                    assert!(a.zustaendig().contains(&rolle), "Rolle {rolle} darf {p} senden, ist aber nicht zuständig");
                }
                // Kein Schemafeld darf beim Einlesen verloren gehen (etwa ein Tippfehler bei einem Feld mit Standardwert).
                let zurueck = serde_json::to_value(&a).unwrap();
                for feld in v["properties"].as_object().unwrap().keys() {
                    assert!(zurueck.get(feld).is_some(), "{typ}: Schemafeld {feld} kennt der Kern nicht");
                }
            }
        }
    }
}

#[test]
fn beispiele_im_regeltext_passen_zum_schema() {
    let schema = antwortschema(Rolle::Alle);
    let alle = varianten(&schema, "aktionen");
    for typ in erlaubte_typen(Rolle::Alle) {
        // Nur das erste JSON-Objekt; manche Beispiele erklären danach Varianten im Fließtext.
        let b: Value = serde_json::Deserializer::from_str(aktionsbeispiel(typ))
            .into_iter::<Value>()
            .next()
            .unwrap()
            .unwrap_or_else(|e| panic!("{typ}: Beispiel nicht lesbar: {e}"));
        let v = alle.iter().find(|v| typ_von(v) == typ).unwrap_or_else(|| panic!("{typ} fehlt im Schema"));
        gilt(v, &b, typ, false).unwrap_or_else(|e| panic!("Beispiel für {typ}: {e}"));
        serde_json::from_value::<Aktion>(b.clone()).unwrap_or_else(|e| panic!("Beispiel für {typ} liest der Kern nicht: {e}"));
    }
}

#[test]
fn was_das_schema_abfragen_laesst_beantwortet_das_werkzeug() {
    let mut w = Welt::neu(Regelwerk::laden(REGELN).unwrap(), 42, 4).unwrap();
    w.kolonisationsregeln_v2_aktivieren();
    w.planeten[w.spieler[0].heimat as usize].einheiten[Einheit::LeichterJaeger.idx()] = 5;
    let heim = w.planeten[w.spieler[0].heimat as usize].koord.to_string();
    let fremd_k = w.planeten[w.spieler[1].heimat as usize].koord;
    let fremd = fremd_k.to_string();
    // Planning queries require an authorized survey in the current rule version.
    w.spieler[0].erkundet.insert(fremd_k, Erkundung { zeit: 0, felder: 180, zone: Zone::Leben,
        reich_erz: 1000, reich_kristall: 1000, nebel: false });
    // Ein Spionagebericht über das Ziel, damit der Kampfsimulator vollständig rechnet.
    let zeit = w.zeit;
    w.spieler[0].berichte.push(Spionagebericht {
        zeit,
        ziel: fremd_k,
        besitzer: 1,
        bestand: vec![0; GUETER],
        schiffe: Some(vec![0; SCHIFFE]),
        verteidigung: Some(vec![0; EINHEITEN - SCHIFFE]),
        gebaeude: None,
        forschung: None,
    });
    let schema = antwortschema(Rolle::Alle);
    for v in varianten(&schema, "abfragen") {
        let typ = typ_von(&v);
        let mut grund = beispiel("", &v);
        for feld in ["start", "planet"] {
            if grund.get(feld).is_some() {
                grund[feld] = json!(heim);
            }
        }
        if grund.get("ziel").is_some() {
            grund["ziel"] = json!(fremd);
        }
        if grund.get("schiffe").is_some() {
            for (_, n) in grund["schiffe"].as_object_mut().unwrap() {
                *n = json!(0);
            }
            grund["schiffe"]["leichter_jaeger"] = json!(5);
        }
        let mut proben = Vec::new();
        if typ == "kosten" {
            // Je eine Art der Kostenabfrage, die übrigen null, wie unter erzwungenem Schema.
            for (feld, wert) in [("gebaeude", "werft"), ("forschung", "astrophysik"), ("einheit", "kreuzer"), ("rakete", "interplanetar")] {
                let mut p = grund.clone();
                for f in ["gebaeude", "forschung", "einheit", "rakete"] {
                    p[f] = Value::Null;
                }
                p[feld] = json!(wert);
                p["stufe"] = Value::Null;
                p["anzahl"] = if feld == "rakete" { json!(3) } else { Value::Null };
                proben.push(p);
            }
        } else if typ == "galaxie" {
            grund["sektor"] = json!(1);
            grund["von"] = json!(1);
            grund["bis"] = json!(5);
            proben.push(grund);
        } else if typ == "regel" {
            grund["stichwort"] = json!("blockade");
            proben.push(grund);
        } else {
            proben.push(grund);
        }
        for p in proben {
            assert!(gilt(&v, &p, &typ, true).is_ok(), "{typ}: Probe verletzt das Schema: {p}");
            if let Err(e) = w.werkzeug(0, &p) {
                panic!("{typ}: {p} -> {e}");
            }
        }
    }
}

/// Die Grenzen im Schema sind genau die des Kerns: der Grenzwert scheitert nicht an der Grenze, der Wert
/// knapp daneben schon. Im echten Lauf schickte der Feldherr fertigen mit anzahl 0, solange das Schema keine
/// Grenze nannte.
#[test]
fn schemagrenzen_sind_die_grenzen_des_kerns() {
    let mut w = Welt::neu(Regelwerk::laden(REGELN).unwrap(), 3, 2).unwrap();
    let k = w.planeten[w.spieler[0].heimat as usize].koord.to_string();
    let schema = antwortschema(Rolle::Alle);
    let feld = |typ: &str, f: &str| -> Value {
        varianten(&schema, "aktionen").into_iter().find(|v| typ_von(v) == typ).unwrap()["properties"][f].clone()
    };
    let grenzen = |s: Value| (s["minimum"].as_f64(), s["maximum"].as_f64());
    let mut pruefe = |aktion: Value, wert: Value, feldname: &str, grund: &str, ok: bool| {
        let mut a = aktion.clone();
        a[feldname] = wert.clone();
        let (_, text) = w.handeln(0, Rolle::Alle, &a);
        assert_eq!(!text.contains(grund), ok, "{} {feldname} = {wert}: {text}", aktion["typ"]);
    };

    assert_eq!(grenzen(feld("fertigen", "anzahl")), (Some(1.0), Some(10_000.0)));
    let fertigen = json!({"typ": "fertigen", "planet": k, "einheit": "raketenwerfer", "anzahl": 1});
    for (n, ok) in [(0, false), (1, true), (10_000, true), (10_001, false)] {
        pruefe(fertigen.clone(), json!(n), "anzahl", "Anzahl muss", ok);
    }
    assert_eq!(grenzen(feld("steuersatz", "prozent")), (Some(0.0), Some(50.0)));
    for (n, ok) in [(0, true), (50, true), (51, false)] {
        pruefe(json!({"typ": "steuersatz", "prozent": 0}), json!(n), "prozent", "zwischen 0 und 50", ok);
    }
    assert_eq!(grenzen(feld("flotte_senden", "geschwindigkeit")), (Some(0.1), Some(1.0)));
    let flug = json!({"typ": "flotte_senden", "start": k, "ziel": "1:1:1", "mission": "transport", "schiffe": {"kleiner_transporter": 1},
                      "geschwindigkeit": 1.0, "ladung": {}, "haltedauer_stunden": 0});
    for (v, ok) in [(0.09, false), (0.1, true), (1.0, true), (1.01, false)] {
        pruefe(flug.clone(), json!(v), "geschwindigkeit", "geschwindigkeit muss", ok);
    }
    // Raketen: der Kern prüft die Anzahl in raketen_kosten, die Kostenabfrage erreicht sie ohne Silo.
    assert_eq!(grenzen(feld("raketen_bauen", "anzahl")), (Some(1.0), Some(1_000_000.0)));
    assert_eq!(grenzen(feld("raketen_starten", "anzahl")), (Some(1.0), None));
    for (n, ok) in [(0, false), (1, true), (1_000_000, true), (1_000_001, false)] {
        let antwort = w.werkzeug(0, &json!({"typ": "kosten", "rakete": "abfang", "anzahl": n}));
        assert_eq!(antwort.is_ok(), ok, "Raketen {n}: {antwort:?}");
    }
}

#[test]
fn fertigen_verlangt_genau_eines_von_einheit_und_bauteil() {
    let schema = antwortschema(Rolle::Feldherr);
    let fertigen: Vec<Value> = varianten(&schema, "aktionen").into_iter().filter(|v| typ_von(v) == "fertigen").collect();
    assert_eq!(fertigen.len(), 2);
    let leer = json!({"typ": "fertigen", "planet": "1:1:1", "einheit": null, "bauteil": null, "anzahl": 1});
    assert!(fertigen.iter().all(|v| gilt(v, &leer, "fertigen", true).is_err()), "beide leer darf das Schema nicht zulassen");
    let einheit = json!({"typ": "fertigen", "planet": "1:1:1", "einheit": "kreuzer", "anzahl": 2});
    let bauteil = json!({"typ": "fertigen", "planet": "1:1:1", "bauteil": "habitatmodul", "anzahl": 2});
    assert!(fertigen.iter().any(|v| gilt(v, &einheit, "fertigen", true).is_ok()));
    assert!(fertigen.iter().any(|v| gilt(v, &bauteil, "fertigen", true).is_ok()));
    assert!(fertigen.iter().all(|v| gilt(v, &json!({"typ": "fertigen", "planet": "1:1:1", "einheit": "kreuzer", "bauteil": "habitatmodul", "anzahl": 2}), "fertigen", true).is_err()));
}
