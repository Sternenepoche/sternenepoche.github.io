use std::path::PathBuf;
fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let mode = args.next().unwrap_or_else(|| "help".into());
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().map_err(|e|e.to_string())?;
    let db = args.next().map(PathBuf::from).unwrap_or_else(||root.join("wissen/current.json"));
    match mode.as_str() {
        "build" => { let output = wissen::rebuild(&root, &db)?; println!("{}", serde_json::to_string_pretty(&output).unwrap()); Ok(()) },
        "mcp" => wissen::service::serve_mcp(&db),
        "serve" => wissen::service::serve_http(&db, args.next().map(|s|s.parse::<u16>()).transpose().map_err(|e|e.to_string())?.unwrap_or(8197)),
        "query" => { let op=args.next().unwrap_or_else(||"status".into()); let params=serde_json::from_str(&args.next().unwrap_or_else(||"{}".into())).map_err(|e|e.to_string())?; let conn=wissen::Db::open(&db,true)?; println!("{}",wissen::query(&conn,&op,&params)?); Ok(()) },
        _ => { eprintln!("sternenepoche-wissen build|mcp|serve|query [database-path] [port|operation] [JSON args]"); Ok(()) }
    }
}
fn main() { if let Err(e)=run(){eprintln!("{e}"); std::process::exit(1);} }
