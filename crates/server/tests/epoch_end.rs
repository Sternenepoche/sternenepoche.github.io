use kern::Rolle;
use serde_json::json;
use sternenepoche_server::{password_hash, token, Game};

#[test]
fn epoch_end_is_read_only_and_admission_recovers_after_reset() {
    let root = std::env::temp_dir().join(format!("epoch-end-{}", token()));
    let mut g = Game::open(&root, 42).unwrap();
    let raw = g
        .register(
            &json!({"name":"EndProbe"}),
            password_hash("nur-ein-test-passwort").unwrap(),
        )
        .unwrap();
    let a = g.authenticate(raw["token"].as_str().unwrap()).unwrap();
    g.claim(&a, &json!({"mode":"gemischt","volk":"krath"}))
        .unwrap();
    let a = g.account_by_name("EndProbe").unwrap();
    let wid = g.runtime.world_id.clone();
    g.admin(&json!({"world_id":wid,"action":"settings","bots_enabled":false,"epoche_tage":1}))
        .unwrap();
    assert_eq!(g.lobby()["freie_zugaenge"], 2);
    assert_eq!(g.view(&a, Rolle::Alle).unwrap()["beendet"], false);
    for _ in 0..24 {
        g.advance(3600).unwrap();
    }
    let hash = g.world.hash();
    assert_eq!(g.lobby()["beendet"], true);
    assert_eq!(g.lobby()["freie_zugaenge"], 0);
    assert_eq!(g.lobby()["freie_plaetze"], 19);
    assert_eq!(g.view(&a, Rolle::Alle).unwrap()["beendet"], true);
    assert_eq!(g.command(&a,&json!({"world_id":wid,"request_id":"ended","aktionen":[{"typ":"steuersatz","prozent":0}]})).unwrap_err().0,409);
    assert_eq!(
        g.lease(
            &a,
            &json!({"world_id":wid,"action":"start","roles":["verwalter"]})
        )
        .unwrap_err()
        .0,
        409
    );
    g.advance(86400).unwrap();
    assert_eq!(hash, g.world.hash());
    drop(g);
    let mut g = Game::open(&root, 83).unwrap();
    let a = g.account_by_name("EndProbe").unwrap();
    assert_eq!(g.lobby()["freie_zugaenge"], 0);
    assert_eq!(g.view(&a, Rolle::Alle).unwrap()["beendet"], true);
    g.admin(&json!({"world_id":wid,"action":"reset","confirm":format!("RESET {wid}")}))
        .unwrap();
    assert_eq!(g.lobby()["freie_zugaenge"], 3);
    assert_eq!(g.lobby()["beendet"], false);
    assert_eq!(g.account_by_name("EndProbe").unwrap().sid, None);
}
