use kern::*;
use serde_json::json;
use sternenepoche_server::{token, Game};

#[test]
fn mixed_bot_world_recovers_repairable_empires_and_keeps_defeated_bots_stopped() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../laeufe/epochen-tests")
        .join(token());
    let mut g = Game::open(&root, 83).unwrap();
    g.admin(&json!({"world_id":g.runtime.world_id,"action":"settings",
        "versorgung_stunden":4,"stillstand_stunden":4,"bot_period_secs":900}))
        .unwrap();
    for bot in g.runtime.bots.values_mut() {
        bot.naechster = 0;
    }
    for sid in 0..8 {
        let p = g.world.spieler[sid].heimat as usize;
        g.world.planeten[p].gebaeude.fill(0);
        g.world.planeten[p].einheiten.fill(0);
        g.world.planeten[p].bestand.fill(0);
        g.world.spieler[sid].credits = 0;
        g.world.raten_neu(p);
    }
    let mut repairs = Vec::new();
    for sid in 8..16 {
        let p = g.world.spieler[sid].heimat as usize;
        let essential = if g.world.regeln.volk(g.world.spieler[sid].volk).ohne_nahrung {
            Gebaeude::Solarkraftwerk
        } else {
            Gebaeude::Farm
        };
        assert!(g.world.planeten[p].gebaeude[essential.idx()] > 0);
        g.world.planeten[p].bestand.fill(100_000 * M);
        g.world
            .kolonisation
            .integritaet
            .insert((p as PlanetId, essential), 0);
        g.world.raten_neu(p);
        repairs.push((sid as u16, p, essential));
    }
    g.world.ausscheiden_pruefen();
    for _ in 0..24 {
        g.advance(3600).unwrap();
    }
    for sid in 0..8 {
        assert!(g.world.ist_besiegt(sid));
    }
    for (sid, p, essential) in repairs {
        assert!(
            g.world.spieler_aktiv(sid),
            "repairable bot {sid} was eliminated"
        );
        assert_eq!(g.world.integritaet(p, essential), 1000);
    }
    for sid in 16..30 {
        assert!(g.world.spieler_aktiv(sid));
    }
    assert_eq!(g.world.ausscheiden.besiegt.len(), 8);
    assert_eq!(g.lobby()["freie_plaetze"], 20);
    assert!(g.admin_status()["invariant_errors"]
        .as_array()
        .unwrap()
        .is_empty());
    let frozen_actions = g.world.spieler[..8]
        .iter()
        .map(|s| s.statistik.aktionen)
        .collect::<Vec<_>>();
    let hash = g.world.hash();
    drop(g);
    let mut g = Game::open(&root, 999).unwrap();
    assert_eq!(g.world.hash(), hash);
    assert_eq!(g.world.ausscheiden.regeln.versorgung_stunden, 4);
    for _ in 0..24 {
        g.advance(3600).unwrap();
    }
    for sid in 0..8 {
        assert!(g.world.ist_besiegt(sid));
        assert_eq!(
            g.world.spieler[sid as usize].statistik.aktionen,
            frozen_actions[sid as usize]
        );
        let home = g.world.spieler[sid as usize].heimat as usize;
        assert_eq!(g.world.planeten[home].besitzer, sid);
        assert!(g.world.planeten[home].heimat);
        assert!(g.world.planeten[home].rate.iter().all(|n| *n == 0));
        assert!(!g.world.flotten.values().any(|f| f.besitzer == sid));
    }
}
