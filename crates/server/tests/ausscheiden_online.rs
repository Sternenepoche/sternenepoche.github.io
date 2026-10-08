use kern::*;
use serde_json::json;
use sternenepoche_server::{password_hash, token, Game};
#[test]
fn defeated_account_and_bot_stay_inactive_across_restart() {
    let root = std::env::temp_dir().join(format!("defeat-online-{}", token()));
    let mut g = Game::open(&root, 83).unwrap();
    let raw = g
        .register(
            &json!({"name":"UntergangProbe"}),
            password_hash("nur-ein-test-passwort").unwrap(),
        )
        .unwrap();
    let a = g.authenticate(raw["token"].as_str().unwrap()).unwrap();
    g.claim(&a, &json!({"mode":"gemischt","volk":"aurelianer"}))
        .unwrap();
    let a = g.account_by_name("UntergangProbe").unwrap();
    let sid = a.sid.unwrap();
    let wid = g.runtime.world_id.clone();
    let lease = g
        .lease(
            &a,
            &json!({"world_id":wid,"action":"start","roles":["verwalter"]}),
        )
        .unwrap();
    assert!(lease["lease"].is_string());
    g.admin(
        &json!({"world_id":wid,"action":"settings","versorgung_stunden":1,"stillstand_stunden":1}),
    )
    .unwrap();
    for id in [sid, 0] {
        let pid = g.world.spieler[id as usize].heimat as usize;
        g.world.planeten[pid].gebaeude.fill(0);
        g.world.planeten[pid].einheiten.fill(0);
        g.world.planeten[pid].bestand.fill(0);
        g.world.spieler[id as usize].credits = 0;
        g.world.raten_neu(pid);
    }
    g.world.ausscheiden_pruefen();
    g.advance(3600).unwrap();
    assert!(g.world.ist_besiegt(sid));
    assert!(g.world.ist_besiegt(0));
    let actions = g.world.spieler[0].statistik.aktionen;
    assert_eq!(
        g.lease(
            &a,
            &json!({"world_id":wid,"action":"start","roles":["verwalter"]})
        )
        .unwrap_err()
        .0,
        409
    );
    assert_eq!(g.command(&a,&json!({"world_id":wid,"request_id":"after-defeat","aktionen":[{"typ":"steuersatz","prozent":0}]})).unwrap_err().0,409);
    assert_eq!(
        g.view(&a, Rolle::Alle).unwrap()["reich_status"]["status"],
        "besiegt"
    );
    assert_eq!(
        g.lobby()["freie_zugaenge"],
        4,
        "defeat must not recycle an occupied epoch seat"
    );
    g.advance(3600).unwrap();
    assert_eq!(g.world.spieler[0].statistik.aktionen, actions);
    drop(g);
    let mut g = Game::open(&root, 1).unwrap();
    let a = g.account_by_name("UntergangProbe").unwrap();
    assert_eq!(a.sid, Some(sid));
    assert_eq!(g.me(&a)["reich_status"]["status"], "besiegt");
    g.advance(3600).unwrap();
    assert_eq!(g.world.spieler[0].statistik.aktionen, actions);
    assert_eq!(g.world.ausscheiden.regeln.versorgung_stunden, 1);
}
