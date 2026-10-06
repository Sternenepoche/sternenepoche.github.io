use std::path::Path;
use sternenepoche_agenten::{config::Config, run};

fn main() {
    if let Err(error) = cli() {
        eprintln!("{error}");
        std::process::exit(2);
    }
}

fn cli() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let command = args.first().map(String::as_str).unwrap_or("help");
    if command == "help" || command == "--help" {
        println!("sternenepoche-agenten demo --out ORDNER [--windows 96]\nsternenepoche-agenten run --config DATEI --out ORDNER [--windows 96] [--execute]\nsternenepoche-agenten validate --config DATEI\nsternenepoche-agenten export --journal ORDNER --out NEUER_ORDNER\nVorhandener Laufordner setzt automatisch fort. demo benötigt kein Netzwerk. --execute autorisiert konfigurierte Netzwerkaufrufe, keine automatischen Wiederholungen.");
        return Ok(());
    }
    let value = |flag: &str| -> Result<Option<&str>, String> {
        match args.iter().position(|s| s == flag) {
            Some(i) => args
                .get(i + 1)
                .map(|v| Some(v.as_str()))
                .ok_or_else(|| format!("Wert für {flag} fehlt")),
            None => Ok(None),
        }
    };
    if command == "export" {
        let input = value("--journal")?.ok_or("--journal fehlt")?;
        let output = value("--out")?.ok_or("--out fehlt")?;
        println!(
            "{}",
            sternenepoche_agenten::export::export(Path::new(input), Path::new(output))?
        );
        return Ok(());
    }
    let config = if command == "demo" {
        Config::demo(4)
    } else {
        let p = value("--config")?.ok_or("--config fehlt")?;
        serde_json::from_slice::<Config>(&std::fs::read(p).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?
    };
    config.validate()?;
    if command == "validate" {
        println!("Konfiguration gültig; keine Modellaufrufe.");
        return Ok(());
    }
    if !["run", "demo"].contains(&command) {
        return Err("Unbekannter Befehl; siehe --help".into());
    }
    let out = value("--out")?.ok_or("--out fehlt")?;
    let windows = value("--windows")?
        .unwrap_or("96")
        .parse::<usize>()
        .map_err(|_| "--windows braucht positive Ganzzahl")?;
    if windows == 0 {
        return Err("--windows muss positiv sein".into());
    }
    let summary = run(
        &config,
        Path::new(out),
        windows,
        args.iter().any(|s| s == "--execute"),
    )?;
    println!("{}", serde_json::to_string_pretty(&summary).unwrap());
    Ok(())
}
