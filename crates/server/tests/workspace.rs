use serde_json::{json,Value};
use sternenepoche_server::{Game,token,Account};
fn account(g:&mut Game,name:&str)->Account {let r=g.register(&json!({"name":name}),"test-only".into()).unwrap();g.authenticate(r["token"].as_str().unwrap()).unwrap()}
fn state()->Value {json!({"models":[{"id":"nova","name":"Nova","provider":"ollama","url":"http://127.0.0.1:11434","model":"test"}],"roles":{"stratege":{"model":"nova","soul":"Freundlich","instructions":"Plane"},"verwalter":{"model":"nova","soul":"Sorgfältig","instructions":"Versorge"},"feldherr":{"model":"nova","soul":"Vorsichtig","instructions":"Schütze"},"diplomat":{"model":"nova","soul":"Verbindlich","instructions":"Handle"}},"boards":[{"id":"plan","title":"Gemeinsamer Plan","world_id":"","notes":[]}],"delay":60,"limit":100,"cost_limit":1.0})}
#[test]
fn workspace_private_bounded_versioned_and_survives_restart(){
 let root=std::env::temp_dir().join(format!("team-test-{}",token()));let mut g=Game::open(&root,42).unwrap();let a=account(&mut g,"TeamAlpha");let b=account(&mut g,"TeamBeta");let wid=g.runtime.world_id.clone();
 assert_eq!(g.workspace(&a).unwrap()["revision"],0);
 let body=json!({"world_id":wid,"revision":0,"state":state()});g.save_workspace(&a,&body).unwrap();
 assert_eq!(g.workspace(&b).unwrap()["state"],Value::Null);
 assert_eq!(g.save_workspace(&a,&body).unwrap_err().0,409);
 let mut bad=state();bad["models"][0]["key"]=json!("must-not-be-stored");assert_eq!(g.save_workspace(&a,&json!({"world_id":wid,"revision":1,"state":bad})).unwrap_err().0,400);
 let mut bad=state();bad["roles"]["diplomat"]["model"]=json!("unknown");assert_eq!(g.save_workspace(&a,&json!({"world_id":wid,"revision":1,"state":bad})).unwrap_err().0,400);
 drop(g);let g=Game::open(&root,99).unwrap();let a=g.account_by_name("TeamAlpha").unwrap();assert_eq!(g.workspace(&a).unwrap()["state"],state());assert_eq!(g.workspace(&a).unwrap()["revision"],1);
}
#[test]
fn turn_memory_and_actions_are_atomic_and_replay_is_exactly_once(){
 let root=std::env::temp_dir().join(format!("team-turn-test-{}",token()));let mut g=Game::open(&root,42).unwrap();let a=account(&mut g,"TeamGamma");g.claim(&a,&json!({"mode":"gemischt","volk":"krath"})).unwrap();let a=g.account_by_name("TeamGamma").unwrap();let wid=g.runtime.world_id.clone();
 g.admin(&json!({"world_id":wid,"action":"settings","paused":false,"bots_enabled":false})).unwrap();
 g.save_workspace(&a,&json!({"world_id":wid,"revision":0,"state":state()})).unwrap();
 let lease=g.lease(&a,&json!({"world_id":wid,"action":"start","roles":["verwalter"]})).unwrap();
 let pin=json!({"board_id":"plan","note":{"id":"trade","title":"Handel","text":"Lyra: Kristallbedarf mit Nova abstimmen","to":"Lyra","status":"offen"}});
 let mut cmd=json!({"world_id":wid,"request_id":"team-command","lease":lease["lease"],"rolle":"verwalter","aktionen":[{"typ":"steuersatz","prozent":12}],"team":{"revision":0,"model_id":"nova","summary":"Steuern anpassen, Handel noch offen","pins":[pin]}});
 let hash=g.world.hash();assert_eq!(g.command(&a,&cmd).unwrap_err().0,409);assert_eq!(g.world.hash(),hash,"stale memory executed a game action");
 cmd["team"]["revision"]=json!(1);cmd["team"]["pins"][0]["board_id"]=json!("missing");assert_eq!(g.command(&a,&cmd).unwrap_err().0,400);assert_eq!(g.world.hash(),hash);
 cmd["team"]["pins"][0]["board_id"]=json!("plan");let result=g.command(&a,&cmd).unwrap();assert_eq!(result["ergebnisse"][0]["ok"],true);
 assert_eq!(g.command(&a,&cmd).unwrap(),result);let memory=g.workspace(&a).unwrap();assert_eq!(memory["revision"],2);assert_eq!(memory["journal"].as_array().unwrap().len(),1);assert_eq!(memory["state"]["boards"][0]["notes"].as_array().unwrap().len(),1);
 drop(g);let g=Game::open(&root,5).unwrap();let a=g.account_by_name("TeamGamma").unwrap();assert_eq!(g.workspace(&a).unwrap(),memory);
}
