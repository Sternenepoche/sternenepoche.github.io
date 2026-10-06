use inhalt::{
    catalog::{digest, Catalog},
    planet::{render_orbit, surface_maps, PlanetOptions},
    production::{self, save_json},
};
use kern::Zone;
use serde_json::json;
use std::{env, fs, path::PathBuf};

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "validate".into());
    let mut content = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../content");
    let mut output = None;
    let mut asset = None;
    let mut profile = "pilot".to_string();
    let mut seed = 2406;
    let mut width = 512;
    let mut execute = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--content" => {
                content = PathBuf::from(args.next().ok_or("--content needs a directory")?)
            }
            "--output" => {
                output = Some(PathBuf::from(
                    args.next().ok_or("--output needs a directory")?,
                ))
            }
            "--asset" => asset = Some(args.next().ok_or("--asset needs an id")?),
            "--profile" => profile = args.next().ok_or("--profile needs pilot/final")?,
            "--seed" => {
                seed = args
                    .next()
                    .ok_or("--seed needs integer")?
                    .parse()
                    .map_err(|_| "Invalid seed")?
            }
            "--width" => {
                width = args
                    .next()
                    .ok_or("--width needs integer")?
                    .parse()
                    .map_err(|_| "Invalid width")?
            }
            "--execute" => execute = true,
            _ => return Err(format!("Unknown option {arg}")),
        }
    }
    match command.as_str() {
        "sync-engine" => println!("{} building motifs added; engine rule gates synchronized in Rust.",inhalt::catalog::sync_engine(&content)?),
        "validate"=>{let c=Catalog::load(&content)?;c.validate()?;println!("{} assets cover all engine factions, goods, buildings, research and units; headless has no asset dependency.",c.assets.len());
            let rules=content.join("../regeln/regelwerk.ron");if rules.exists()&&!c.rules_current(rules)? {println!("NOTICE: rules changed since catalog provenance; gates require review.");}}
        "prepare"=>println!("{} assets x pilot/final prepared in Rust. No inference.",production::prepare(&content)?),
        "step"=>{
            if !execute{return Err("step requires --execute and production.json GPU release; no inference performed".into());}
            println!("{:?}",production::execute_one(&content,&asset.ok_or("step requires --asset")?,&profile)?);
        }
        "export-planets"=>{
            let output=output.unwrap_or_else(||content.join("procedural-rust"));fs::create_dir_all(&output).map_err(|e|e.to_string())?;
            let mut records=Vec::new();
            for zone in Zone::ALLE {
                let opts=PlanetOptions{seed,zone,rotation:0.65,..Default::default()};let maps=surface_maps(width,&opts)?;
                let orbit=render_orbit((width*3/4).max(4),&opts)?;
                for (name,image) in [("albedo",maps.albedo),("heightmap",maps.heightmap),("roughness",maps.roughness),("clouds",maps.clouds),("orbital",orbit)] {
                    let file=format!("{}_{}.png",zone.name(),name);let bytes=image.png()?;fs::write(output.join(&file),&bytes).map_err(|e|e.to_string())?;
                    records.push(json!({"file":file,"zone":zone.name(),"kind":name,"width":image.width,"height":image.height,"sha256":digest(&bytes),"note":"Visual material only; no gameplay resource deposits."}));
                }
            }
            save_json(&output.join("manifest.json"),&json!({"generator":"inhalt::planet / Rust CPU spherical value noise","version":1,"seed":seed,"gameplay_deposits":false,"maps":records}))?;
            println!("15 Rust CPU PNG maps exported to {}. No GPU used.",output.display());
        }
        "help"|"--help"=>println!("sternenepoche-inhalt validate|sync-engine|prepare|export-planets|step\nOptions: --content DIR --output DIR --seed N --width N\nstep requires --execute --asset ID [--profile pilot|final] and confirmed GPU release.\nEach step submits at most one job or resumes its saved prompt ID. No polling daemon."),
        _=>return Err(format!("Unknown command {command}")),
    }
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(2);
    }
}
