use std::path::Path;
use sternenepoche_agenten::labor::{self, config::Config};
fn main() {
    if let Err(e) = cli() {
        eprintln!("{e}");
        std::process::exit(2);
    }
}
fn cli() -> labor::Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let cmd = args.first().map(String::as_str).unwrap_or("help");
    let value = |flag: &str| -> labor::Result<&str> {
        let i = args
            .iter()
            .position(|s| s == flag)
            .ok_or_else(|| format!("{flag} fehlt"))?;
        args.get(i + 1)
            .map(String::as_str)
            .ok_or_else(|| format!("Wert für {flag} fehlt"))
    };
    let result = match cmd {
        "help" | "--help" => {
            println!(
                r#"sternenepoche-labor config --players N [--out DATEI]
sternenepoche-labor local-config --players N --out DATEI [--model NAME --context 8192] [--cpu] [--url URL]
sternenepoche-labor validate --config DATEI
sternenepoche-labor run --config DATEI --out ORDNER --windows N [--execute]
sternenepoche-labor inventory [--url http://127.0.0.1:11434]
sternenepoche-labor status|verify|replay --run ORDNER
sternenepoche-labor capacity --run ORDNER
sternenepoche-labor plan-capacity --config DATEI --sources 'LAUF1;LAUF2' [--out NEUE_KONFIG]
sternenepoche-labor retry --run ORDNER --call BESTAETIGT_ABGELEHNTE_CALL_ID
sternenepoche-labor export --run ORDNER --out NEUER_ORDNER
sternenepoche-labor matrix --config DATEI --out ORDNER --seeds 1,2 --windows N [--execute]
sternenepoche-labor balance-report
sternenepoche-labor factorial --config DATEI --harnesses JSON_DATEI --models ALIAS1,ALIAS2 --out ORDNER --seeds 1,2 --windows N [--execute]
sternenepoche-labor package --concepts DATEI --sources 'LAUF1;LAUF2' --out NEUE_DATEI
Keine UI. Config erzeugt einen Mock-Kontrolllauf mit nativen Rust-Spielerbüros; Worker vorher über tools/sandbox/build-native.ps1 bauen.
local_only erlaubt ausschließlich lokale Ollama-Gewichte. Trusted ist keine OS-Sandbox. Docker bleibt ein optionaler Altadapter.
run addiert Fenster, matrix setzt eine feste Zielzahl pro Match. Resume nur bei identischem Manifest."#
            );
            return Ok(());
        }
        "config" => {
            let n = value("--players")?
                .parse()
                .map_err(|_| "Spielerzahl ungültig")?;
            let mut c = Config::v4(n);
            c.sandbox = "native".into();
            c.validate()?;
            if let Ok(out) = value("--out") {
                if Path::new(out).exists() {
                    return Err("Konfiguration existiert bereits".into());
                }
                std::fs::write(out, serde_json::to_vec_pretty(&c).unwrap())
                    .map_err(|e| e.to_string())?;
            }
            serde_json::json!(c)
        }
        "inventory" => {
            labor::broker::inventory(value("--url").unwrap_or("http://127.0.0.1:11434"))?
        }
        "local-config" => {
            let n = value("--players")?
                .parse()
                .map_err(|_| "Spielerzahl ungültig")?;
            let mut c = labor::broker::local_config(
                n,
                value("--url").unwrap_or("http://127.0.0.1:11434"),
                args.iter().any(|s| s == "--cpu"),
            )?;
            if let Ok(name) = value("--model") {
                let m = c.models.get_mut("primary").unwrap();
                m.model = name.into();
                m.digest = None;
                if let Ok(context) = value("--context") {
                    m.context = context.parse().map_err(|_| "Kontext ungültig")?;
                }
                let proof = labor::broker::preflight(m)?;
                m.digest = proof["installed"]["digest"].as_str().map(str::to_string);
            }
            c.validate()?;
            let out = value("--out")?;
            if Path::new(out).exists() {
                return Err("Konfiguration existiert bereits".into());
            }
            std::fs::write(out, serde_json::to_vec_pretty(&c).unwrap())
                .map_err(|e| e.to_string())?;
            serde_json::json!({"config":out,"models":c.models.len(),"weights_loaded":false})
        }
        "balance-report" => labor::balance::report()?,
        "factorial" => {
            let c: Config = serde_json::from_slice(
                &std::fs::read(value("--config")?).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
            let harnesses = serde_json::from_slice(
                &std::fs::read(value("--harnesses")?).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
            let models = value("--models")?
                .split(',')
                .map(str::to_string)
                .collect::<Vec<_>>();
            let seeds = value("--seeds")?
                .split(',')
                .map(|s| s.parse().map_err(|_| "Seed ungültig".to_string()))
                .collect::<labor::Result<Vec<u64>>>()?;
            let windows = value("--windows")?
                .parse()
                .map_err(|_| "Fensterzahl ungültig")?;
            labor::research::factorial(
                &c,
                &harnesses,
                &models,
                Path::new(value("--out")?),
                &seeds,
                windows,
                args.iter().any(|s| s == "--execute"),
            )?
        }
        "status" => labor::runtime::status(Path::new(value("--run")?))?,
        "capacity" => labor::runtime::capacity(Path::new(value("--run")?))?,
        "plan-capacity" => {
            let c: Config = serde_json::from_slice(
                &std::fs::read(value("--config")?).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
            let sources = value("--sources")?
                .split(';')
                .map(str::to_string)
                .collect::<Vec<_>>();
            if let Ok(out) = value("--out") {
                labor::capacity::fit(&c, &sources, Path::new(out))?
            } else {
                labor::capacity::plan(&c, &sources)?
            }
        }
        "verify" => labor::audit::verify(Path::new(value("--run")?))?,
        "replay" => labor::runtime::replay(Path::new(value("--run")?))?,
        "retry" => labor::runtime::retry(Path::new(value("--run")?), value("--call")?)?,
        "export" => labor::audit::export(Path::new(value("--run")?), Path::new(value("--out")?))?,
        "matrix" => {
            let c: Config = serde_json::from_slice(
                &std::fs::read(value("--config")?).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
            let seeds = value("--seeds")?
                .split(',')
                .map(|s| s.parse().map_err(|_| "Seed ungültig".to_string()))
                .collect::<labor::Result<Vec<u64>>>()?;
            let windows = value("--windows")?
                .parse()
                .map_err(|_| "Fensterzahl ungültig")?;
            labor::experiment::matrix(
                &c,
                Path::new(value("--out")?),
                &seeds,
                windows,
                args.iter().any(|s| s == "--execute"),
            )?
        }
        "package" => {
            let sources = value("--sources")?
                .split(';')
                .map(str::to_string)
                .collect::<Vec<_>>();
            labor::experiment::package(
                Path::new(value("--concepts")?),
                &sources,
                Path::new(value("--out")?),
            )?
        }
        "validate" | "run" => {
            let c: Config = serde_json::from_slice(
                &std::fs::read(value("--config")?).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
            c.validate()?;
            if cmd == "validate" {
                serde_json::json!({"valid":true,"network_calls":false})
            } else {
                let windows = value("--windows")?
                    .parse()
                    .map_err(|_| "Fensterzahl ungültig")?;
                labor::run(
                    &c,
                    Path::new(value("--out")?),
                    windows,
                    args.iter().any(|s| s == "--execute"),
                )?
            }
        }
        _ => return Err("Unbekannter Befehl; siehe --help".into()),
    };
    println!("{}", serde_json::to_string_pretty(&result).unwrap());
    Ok(())
}
