use serde_json::json;
use sternenepoche_server::Game;
fn main(){
    let root=std::env::args().nth(1).expect("Neues Testdatenverzeichnis angeben");
    assert!(!std::path::Path::new(&root).exists(),"Nur neues, isoliertes Testverzeichnis verwenden");
    let mut g=Game::open(std::path::Path::new(&root),83).unwrap();
    for hour in 1..=180*24 {
        g.advance(3600).unwrap();
        if hour%720==0 {let s=g.admin_status();println!("{}",json!({"tage":hour/24,"frei":s["lobby"]["freie_plaetze"],"aktionen":s["bot_actions"],"abgelehnt":s["bot_rejected"],"bots":s["bots"],"monitor":s["monitor"],"invariant_errors":s["invariant_errors"]}));assert!(s["invariant_errors"].as_array().unwrap().is_empty());}
    }
}
