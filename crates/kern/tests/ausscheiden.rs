use kern::{welt::Bauauftrag, *};
use serde_json::json;
fn world() -> Welt {
    let r = Regelwerk::laden(include_str!("../../../regeln/online-v1.ron")).unwrap();
    let mut w = Welt::neu(r, 42, 3).unwrap();
    w.ereignisse.clear();
    for s in &mut w.spieler {
        s.ki = false;
    }
    w
}
fn hunger(w: &mut Welt, sid: usize) {
    let pid = w.spieler[sid].heimat as usize;
    w.planeten[pid].gebaeude[Gebaeude::Farm.idx()] = 0;
    w.planeten[pid].bestand[Gut::Nahrung.idx()] = 0;
    w.raten_neu(pid);
}
fn stranded(w: &mut Welt, sid: usize) {
    let pid = w.spieler[sid].heimat as usize;
    w.planeten[pid].gebaeude.fill(0);
    w.planeten[pid].einheiten.fill(0);
    w.planeten[pid].bestand.fill(0);
    w.planeten[pid].bestand[Gut::Nahrung.idx()] = 100_000 * M;
    w.spieler[sid].credits = 0;
    w.raten_neu(pid);
}
#[test]
fn starvation_grace_recovery_and_restart() {
    let mut w = world();
    hunger(&mut w, 0);
    w.ausscheiden_pruefen();
    assert_eq!(w.reich_status(0)["status"], "kritisch");
    w.zeit = 20 * STUNDE;
    let mut w = Welt::aus_bytes(&w.zu_bytes()).unwrap();
    assert_eq!(
        w.reich_status(0)["rettungsfristen"][0]["frist"],
        72 * STUNDE
    );
    let pid = w.spieler[0].heimat as usize;
    w.planeten[pid].bestand[Gut::Nahrung.idx()] = 100_000 * M;
    w.raten_neu(pid);
    w.ausscheiden_pruefen();
    assert_eq!(w.reich_status(0)["status"], "aktiv");
    w.zeit = 80 * STUNDE;
    hunger(&mut w, 0);
    w.ausscheiden_pruefen();
    assert_eq!(
        w.reich_status(0)["rettungsfristen"][0]["frist"],
        152 * STUNDE
    );
    w.zeit = 151 * STUNDE;
    w.ausscheiden_pruefen();
    assert!(!w.ist_besiegt(0));
    w.zeit = 152 * STUNDE;
    w.ausscheiden_pruefen();
    assert!(w.ist_besiegt(0));
    assert_eq!(w.reich_status(0)["grund"], "Versorgungskollaps");
    assert!(
        !w.handeln(0, Rolle::Alle, &json!({"typ":"steuersatz","prozent":0}))
            .0
    );
    let hash = w.hash();
    let bytes = w.zu_bytes();
    assert!(bytes.starts_with(b"STERNEP8"));
    let mut w = Welt::aus_bytes(&bytes).unwrap();
    assert_eq!(w.hash(), hash);
    w.planeten[pid].bestand[Gut::Nahrung.idx()] = 1_000_000 * M;
    w.raten_neu(pid);
    w.tick();
    assert!(w.ist_besiegt(0));
    assert_eq!(w.planeten[pid].rate, [0; GUETER]);
    assert_eq!(w.belegung.get(&w.planeten[pid].koord), Some(&(pid as u32)));
}
#[test]
fn unpaid_queue_cannot_conceal_permanent_resource_deadlock() {
    let mut w = world();
    stranded(&mut w, 0);
    let pid = w.spieler[0].heimat as usize;
    w.planeten[pid].bauschleife.push(Bauauftrag {
        gebaeude: Gebaeude::Solarkraftwerk,
        stufe: 1,
        fertig: None,
        dauer: 0,
        topf: None,
    });
    w.ausscheiden_pruefen();
    assert_eq!(
        w.reich_status(0)["rettungsfristen"][0]["frist"],
        48 * STUNDE
    );
    w.zeit = 47 * STUNDE;
    w.ausscheiden_pruefen();
    assert!(!w.ist_besiegt(0));
    w.zeit = 48 * STUNDE;
    w.ausscheiden_pruefen();
    assert!(w.ist_besiegt(0));
    assert_eq!(w.reich_status(0)["grund"], "Wirtschaftlicher Stillstand");
    assert!(w.planeten[pid].bauschleife.is_empty());
}
#[test]
fn affordable_solar_repair_is_a_real_recovery_path() {
    let mut w = world();
    let pid = w.spieler[0].heimat as usize;
    w.kolonisation
        .integritaet
        .insert((pid as u32, Gebaeude::Solarkraftwerk), 0);
    w.planeten[pid].bestand.fill(10_000 * M);
    w.raten_neu(pid);
    w.ausscheiden_pruefen();
    assert_eq!(w.reich_status(0)["status"], "aktiv");
    assert!(w
        .reparieren(
            0,
            Rolle::Alle,
            w.planeten[pid].koord,
            Gebaeude::Solarkraftwerk
        )
        .is_ok());
}
#[test]
fn healthy_colony_and_market_prevent_false_elimination() {
    let mut w = world();
    stranded(&mut w, 0);
    let pid = w.spieler[1].heimat as usize;
    w.spieler[1].planeten.clear();
    w.planeten[pid].besitzer = 0;
    w.spieler[0].planeten.push(pid as u32);
    w.raten_neu(pid);
    w.ausscheiden_pruefen();
    assert_eq!(w.reich_status(0)["status"], "aktiv");
    let mut w = world();
    stranded(&mut w, 0);
    let pid = w.spieler[0].heimat as usize;
    w.planeten[pid].gebaeude[Gebaeude::Markt.idx()] = 1;
    w.spieler[0].credits = 1000 * M;
    w.ausscheiden_pruefen();
    assert_eq!(w.reich_status(0)["status"], "aktiv");
}
#[test]
fn synthetics_need_energy_even_with_food_in_storage() {
    let mut w = world();
    stranded(&mut w, 0);
    w.spieler[0].volk = Volk::Syntheten;
    let pid = w.spieler[0].heimat as usize;
    w.planeten[pid].bestand.fill(10_000 * M);
    w.raten_neu(pid);
    w.ausscheiden_pruefen();
    assert_eq!(
        w.reich_status(0)["rettungsfristen"][0]["frist"],
        72 * STUNDE
    );
    w.zeit = 72 * STUNDE;
    w.ausscheiden_pruefen();
    assert_eq!(w.reich_status(0)["grund"], "Versorgungskollaps");
}
#[test]
fn repeated_bombardment_reaches_zero_without_freeing_home() {
    let mut w = world();
    let pid = w.spieler[0].heimat as usize;
    let k = w.planeten[pid].koord;
    for _ in 0..7 {
        w.bombardement_abschluss(pid, 9999);
    }
    assert_eq!(w.integritaet(pid, Gebaeude::Solarkraftwerk), 0);
    w.planeten[pid].bestand.fill(0);
    w.spieler[0].credits = 0;
    w.raten_neu(pid);
    w.ausscheiden_pruefen();
    w.zeit = 48 * STUNDE;
    w.ausscheiden_pruefen();
    assert!(w.ist_besiegt(0));
    let known = w.planetenwissen(0, k);
    assert_eq!(known["besiegt"], true);
    assert_eq!(known["heimat"], true);
    assert_eq!(known["status"], "eigen");
    w.punkte_neu();
    assert_eq!(w.rangliste().len(), 3);
    assert_eq!(w.rangliste().last().unwrap().1, w.spieler[0].name);
    // A surviving enemy cannot treat the occupied protected home as a free colony.
    assert!(w.belegung.contains_key(&k));
    assert!(w.planeten[pid].heimat);
}
#[test]
fn reserved_seats_and_legacy_worlds_do_not_enter_crises() {
    let mut w = world();
    w.startplatz_reservieren(2).unwrap();
    stranded(&mut w, 2);
    w.ausscheiden_pruefen();
    w.zeit = 100 * STUNDE;
    w.ausscheiden_pruefen();
    assert!(!w.ist_besiegt(2));
    assert!(!w.ausscheiden.krisen.contains_key(&2));
    let r = Regelwerk::laden(include_str!("../../../regeln/regelwerk.ron")).unwrap();
    let mut w = Welt::neu(r, 42, 3).unwrap();
    stranded(&mut w, 0);
    w.ausscheiden_pruefen();
    assert!(!w.ausscheiden.erweitert());
}
