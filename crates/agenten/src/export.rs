//! Native Parquet output, independent of Python, Arrow runtimes or a JSON pipe.
//! Core identity columns are typed; heterogeneous engine payloads remain JSON strings.
use crate::journal::Journal;
use parquet::{
    data_type::{ByteArray, ByteArrayType, Int32Type, Int64Type},
    file::{properties::WriterProperties, writer::SerializedFileWriter},
    schema::parser::parse_message_type,
};
use serde_json::{json, Value};
use std::{
    fs::{self, OpenOptions},
    path::Path,
    sync::Arc,
};

#[derive(Clone)]
struct Row {
    zeit: i64,
    spieler: i32,
    rolle: String,
    phase: String,
    input: String,
    output: String,
    tokens: i64,
}

fn write(path: &Path, rows: &[Row]) -> Result<(), String> {
    let schema = Arc::new(parse_message_type("message sternenepoche { REQUIRED INT64 zeit; REQUIRED INT32 spieler; REQUIRED BINARY rolle (UTF8); REQUIRED BINARY phase (UTF8); REQUIRED BINARY eingabe (UTF8); REQUIRED BINARY ergebnis (UTF8); REQUIRED INT64 tokens; }").map_err(|e|e.to_string())?);
    let f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    let props = Arc::new(WriterProperties::builder().build());
    let mut writer = SerializedFileWriter::new(f, schema, props).map_err(|e| e.to_string())?;
    for batch in rows.chunks(1024) {
        let mut group = writer.next_row_group().map_err(|e| e.to_string())?;
        for index in 0..7 {
            let mut col = group
                .next_column()
                .map_err(|e| e.to_string())?
                .ok_or("Parquet-Spalte fehlt")?;
            match index {
                0 | 6 => {
                    let data: Vec<i64> = batch
                        .iter()
                        .map(|r| if index == 0 { r.zeit } else { r.tokens })
                        .collect();
                    col.typed::<Int64Type>()
                        .write_batch(&data, None, None)
                        .map_err(|e| e.to_string())?;
                }
                1 => {
                    let data: Vec<i32> = batch.iter().map(|r| r.spieler).collect();
                    col.typed::<Int32Type>()
                        .write_batch(&data, None, None)
                        .map_err(|e| e.to_string())?;
                }
                _ => {
                    let data: Vec<ByteArray> = batch
                        .iter()
                        .map(|r| {
                            ByteArray::from(match index {
                                2 => r.rolle.as_str(),
                                3 => r.phase.as_str(),
                                4 => r.input.as_str(),
                                _ => r.output.as_str(),
                            })
                        })
                        .collect();
                    col.typed::<ByteArrayType>()
                        .write_batch(&data, None, None)
                        .map_err(|e| e.to_string())?;
                }
            }
            col.close().map_err(|e| e.to_string())?;
        }
        group.close().map_err(|e| e.to_string())?;
    }
    writer.close().map_err(|e| e.to_string())?;
    Ok(())
}

/// Export only player-filtered prompts as training inputs. Full checkpoints are
/// used solely for explicitly labelled metrics, never joined into model prompts.
pub fn export(input: &Path, output: &Path) -> Result<Value, String> {
    let manifest: Value =
        serde_json::from_slice(&fs::read(input.join("manifest.json")).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let journal = Journal::open(input, &manifest)?;
    fs::create_dir(output).map_err(|e| format!("Neuer Exportordner benötigt: {e}"))?;
    let mut paths: Vec<_> = fs::read_dir(input.join("calls"))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    paths.sort_by_key(|p| p.file_name());
    let mut decisions = Vec::new();
    for p in paths {
        let name = p.file_name().to_string_lossy().into_owned();
        if !name.ends_with(".request.json") {
            continue;
        }
        let req: Value = serde_json::from_slice(&fs::read(p.path()).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        let res = match journal.read(&format!(
            "calls/{}",
            name.replace(".request.json", ".response.json")
        ))? {
            Some(bytes) => serde_json::from_slice(&bytes).map_err(|e| e.to_string())?,
            None => json!({"status":"pending_unknown"}),
        };
        decisions.push(Row {
            zeit: req["zeit"].as_i64().unwrap_or(0),
            spieler: req["spieler"].as_i64().unwrap_or(-1) as i32,
            rolle: req["rolle"].as_str().unwrap_or("").into(),
            phase: req["phase"].as_str().unwrap_or("").into(),
            input: req["messages"].to_string(),
            tokens: res["usage"]["total_tokens"].as_i64().unwrap_or(0),
            output: res.to_string(),
        });
    }
    let mut states: Vec<_> = fs::read_dir(input.join("states"))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    states.sort_by_key(|p| p.file_name());
    let (mut actions, mut metrics) = (Vec::new(), Vec::new());
    for p in states {
        let name = p.file_name().to_string_lossy().into_owned();
        if name.ends_with(".actions.json") {
            let index = name.trim_end_matches(".actions.json").parse::<usize>().map_err(|_|"Ungültiger Aktionsindex")?;
            // A journal may contain a pre-commit action log after interruption.
            // Only export actions whose resulting checkpoint was published.
            if journal.read(&format!("states/{:08}.json",index+1))?.is_none() { continue; }
            let rows: Vec<Value> =
                serde_json::from_slice(&fs::read(p.path()).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            for v in rows {
                actions.push(Row {
                    zeit: v["zeit"].as_i64().unwrap_or(0),
                    spieler: v["spieler"].as_i64().unwrap_or(-1) as i32,
                    rolle: v["rolle"].as_str().unwrap_or("").into(),
                    phase: "aktion".into(),
                    input: v["aktion"].as_str().unwrap_or("").into(),
                    output: v.to_string(),
                    tokens: 0,
                });
            }
        } else if name.ends_with(".json") {
            let stem = name.trim_end_matches(".json");
            if stem.parse::<usize>().is_err() {
                continue;
            }
            let bytes = journal
                .read(&format!("states/{stem}.bin"))?
                .ok_or("Checkpoint fehlt")?;
            let marker: Value =
                serde_json::from_slice(&fs::read(p.path()).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            if marker["sha256"] != crate::journal::hash(&bytes) {
                return Err("Checkpoint-Prüfsumme falsch".into());
            }
            let w = kern::Welt::aus_bytes(&bytes)?;
            for s in &w.spieler {
                let v = json!({"punkte":s.punkte.gesamt(),"stufe":s.stufe,"rang":s.rang,"planeten":s.planeten.len(),"credits":s.credits,"credits_einheit":"tausendstel","label_only":true});
                metrics.push(Row {
                    zeit: w.zeit,
                    spieler: s.id as i32,
                    rolle: String::new(),
                    phase: "zustandsmetrik_label".into(),
                    input: String::new(),
                    output: v.to_string(),
                    tokens: 0,
                });
            }
        }
    }
    write(&output.join("entscheidungen.parquet"), &decisions)?;
    write(&output.join("aktionen.parquet"), &actions)?;
    write(&output.join("metriken.parquet"), &metrics)?;
    Ok(
        json!({"entscheidungen":decisions.len(),"aktionen":actions.len(),"metriken":metrics.len(),"ordner":output}),
    )
}
