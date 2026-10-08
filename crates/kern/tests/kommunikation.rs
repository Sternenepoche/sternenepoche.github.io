use kern::{aktion::Aktion, *};
use serde_json::json;
fn world() -> Welt {
    let mut r = Regelwerk::laden(include_str!("../../../regeln/online-v1.ron")).unwrap();
    r.diplomatie.allianz_max = 12;
    let mut w = Welt::neu(r, 987, 10).unwrap();
    for (i, s) in w.spieler.iter_mut().enumerate() {
        s.name = format!("P{i}");
        s.ki = false;
    }
    w
}
fn alliance(w: &mut Welt, leader: u16, members: &[u16], name: &str) {
    w.allianz_gruenden(leader, name).unwrap();
    for sid in members {
        w.allianz_einladen(leader, &format!("P{sid}")).unwrap();
        w.allianz_beitreten(*sid, name).unwrap();
    }
}
#[test]
fn private_mail_read_reply_snapshot_and_archive() {
    let mut w = world();
    let old = w.zu_bytes();
    assert!(!old.starts_with(b"STERNEP9"));
    w.brief_senden(0, "privat", "P1", "Betreff", "Text", None)
        .unwrap();
    assert_eq!(w.offene_antworten(1), vec![1]);
    assert!(!w.brief_sichtbar(2, 1));
    assert!(w.brief_lesen(2, 1).is_err());
    w.brief_lesen(1, 1).unwrap();
    assert!(w
        .brief_senden(1, "privat", "P2", "Re", "Leak", Some(1))
        .is_err());
    w.brief_senden(1, "privat", "P0", "Re", "Hallo, ich bin P1", Some(1))
        .unwrap();
    assert!(w.offene_antworten(1).is_empty());
    assert!(w.offene_antworten(0).is_empty());
    let bytes = w.zu_bytes();
    assert!(bytes.starts_with(b"STERNEP9"));
    let restored = Welt::aus_bytes(&bytes).unwrap();
    assert_eq!(w.hash(), restored.hash());
    assert_eq!(restored.kommunikation.briefe[&2].antwort_auf, Some(1));
    let page = restored
        .werkzeug(
            1,
            &json!({"typ":"briefarchiv","vor_id":2,"limit":1,"kanal":"privat"}),
        )
        .unwrap();
    assert_eq!(page["briefe"].as_array().unwrap().len(), 1);
    assert_eq!(Welt::aus_bytes(&old).unwrap().zu_bytes(), old);
}
#[test]
fn alliance_chat_no_external_recipient_and_revocation() {
    let mut w = world();
    alliance(&mut w, 0, &[1, 2], "A");
    assert!(w.nachricht(0, &["P8".into()], true, "secret").is_err());
    assert!(w.nachrichten.is_empty());
    w.brief_senden(0, "allianz", "", "chat", "secret", None)
        .unwrap();
    assert!(w.brief_sichtbar(1, 1));
    assert!(!w.brief_sichtbar(8, 1));
    w.allianz_verlassen(1).unwrap();
    assert!(!w.brief_sichtbar(1, 1));
    w.allianz_einladen(0, "P3").unwrap();
    w.allianz_beitreten(3, "A").unwrap();
    assert!(!w.brief_sichtbar(3, 1));
    assert!(w.kommunikation_sicht(8)["allianzbereich"].is_null());
}
#[test]
fn leadership_is_exactly_four_and_rank_change_revokes() {
    let mut w = world();
    alliance(&mut w, 0, &[1, 2, 3, 4], "A");
    alliance(&mut w, 5, &[6, 7, 8, 9], "B");
    assert!(w
        .brief_senden(4, "diplomatie", "B", "secret", "secret", None)
        .is_err());
    w.brief_senden(0, "diplomatie", "B", "secret", "secret", None)
        .unwrap();
    for sid in [0, 1, 2, 3, 5, 6, 7, 8] {
        assert!(w.brief_sichtbar(sid, 1), "{sid}");
    }
    for sid in [4, 9] {
        assert!(!w.brief_sichtbar(sid, 1));
    }
    assert!(w.allianz_rolle(1, "P4", 2).is_err());
    w.allianz_rolle(0, "P4", 2).unwrap();
    assert!(!w.brief_sichtbar(1, 1));
    assert!(
        !w.brief_sichtbar(4, 1),
        "new leader does not retroactively read old mail"
    );
    w.brief_senden(4, "diplomatie", "B", "new", "hello", None)
        .unwrap();
    assert!(w.brief_sichtbar(4, 2));
    w.allianz_ausschliessen(0, "P4").unwrap();
    assert!(!w.brief_sichtbar(4, 2));
}
#[test]
fn request_acceptance_war_peace_and_nonrecipient_denied() {
    let mut w = world();
    w.diplomatie_anfrage(0, "P1", "nichtangriffspakt", "Pakt?")
        .unwrap();
    assert!(w.diplomatie_entscheiden(2, 1, true).is_err());
    w.diplomatie_entscheiden(1, 1, true).unwrap();
    assert!(w.vertrag_zwischen(0, 1, Vertragsart::Nichtangriffspakt));
    assert!(w.diplomatie_entscheiden(1, 1, true).is_err());
    w.diplomatie_anfrage(0, "P1", "krieg", "Krieg erklärt")
        .unwrap();
    assert!(!w.vertrag_zwischen(0, 1, Vertragsart::Nichtangriffspakt));
    w.diplomatie_anfrage(1, "P0", "frieden", "Frieden?")
        .unwrap();
    w.diplomatie_entscheiden(0, 3, true).unwrap();
    assert_eq!(w.kommunikation.anfragen[1].status, "beendet");
}
#[test]
fn automatic_answer_once_no_loops_and_no_acceptance() {
    let mut w = world();
    w.spieler[1].ki = true;
    w.brief_senden(0, "privat", "P1", "hello", "introduce yourself", None)
        .unwrap();
    w.zeit = STUNDE - 1;
    w.kommunikation_tick();
    assert_eq!(w.nachrichten.len(), 1);
    w.zeit = STUNDE;
    w.kommunikation_tick();
    w.kommunikation_tick();
    assert_eq!(w.nachrichten.len(), 2);
    assert!(w.kommunikation.briefe[&2].automatisch);
    assert!(w.nachrichten[1].text.contains("Ich bin P1"));
    w.spieler[0].ki = true;
    w.zeit += TAG;
    w.kommunikation_tick();
    assert_eq!(w.nachrichten.len(), 2);
}
#[test]
fn internal_trade_reserves_and_conserves_and_cannot_double_buy() {
    let mut w = world();
    alliance(&mut w, 0, &[1], "A");
    let p0 = w.spieler[0].heimat as usize;
    let p1 = w.spieler[1].heimat as usize;
    let k0 = w.planeten[p0].koord;
    let k1 = w.planeten[p1].koord;
    w.planeten[p0].gebaeude[Gebaeude::Markt.idx()] = 1;
    w.planeten[p0].bestand[Gut::Erz.idx()] = 1000 * M;
    w.intern_anbieten(0, k0, Gut::Erz, 100 * M, M).unwrap();
    assert_eq!(w.planeten[p0].bestand[Gut::Erz.idx()], 900 * M);
    let k2 = w.planeten[w.spieler[2].heimat as usize].koord;
    assert!(w.intern_kaufen(2, 1, k2).is_err());
    let credits = w.spieler[1].credits;
    w.intern_kaufen(1, 1, k1).unwrap();
    assert!(w.spieler[1].credits < credits);
    assert!(w.intern_kaufen(1, 1, k1).is_err());
    assert!(w.intern_storno(0, 1).is_err());
    let before = w.planeten[p1].bestand[Gut::Erz.idx()];
    w.intern_liefern(1);
    assert_eq!(w.planeten[p1].bestand[Gut::Erz.idx()], before + 100 * M);
    w.intern_liefern(1);
    assert_eq!(w.planeten[p1].bestand[Gut::Erz.idx()], before + 100 * M);
    assert_eq!(w.kommunikation.angebote[0].status, "geliefert");
    w.intern_anbieten(0, k0, Gut::Erz, 100 * M, 0).unwrap();
    w.allianz_verlassen(0).unwrap();
    assert_eq!(w.planeten[p0].bestand[Gut::Erz.idx()], 900 * M);
}
#[test]
fn delivery_blockade_keeps_transit_status() {
    let mut w = world();
    alliance(&mut w, 0, &[1], "A");
    let p = w.spieler[0].heimat as usize;
    let q = w.spieler[1].heimat as usize;
    w.planeten[p].gebaeude[Gebaeude::Markt.idx()] = 1;
    w.planeten[p].bestand[Gut::Erz.idx()] = 1000 * M;
    w.intern_anbieten(0, w.planeten[p].koord, Gut::Erz, 100 * M, 0)
        .unwrap();
    w.intern_kaufen(1, 1, w.planeten[q].koord).unwrap();
    w.planeten[q].blockade = Some(999);
    w.intern_liefern(1);
    assert_eq!(w.kommunikation.angebote[0].status, "unterwegs");
    w.planeten[q].blockade = None;
    w.intern_liefern(1);
    assert_eq!(w.kommunikation.angebote[0].status, "geliefert");
}
#[test]
fn help_private_and_successor_deterministic() {
    let mut w = world();
    alliance(&mut w, 0, &[1, 2], "A");
    let k = w.planeten[w.spieler[0].heimat as usize].koord;
    w.allianz_hilfe(0, k, "Bitte helfen").unwrap();
    assert!(w.allianz_hilfe_status(3, 1, false).is_err());
    w.allianz_hilfe_status(1, 1, false).unwrap();
    assert!(w.allianz_hilfe_status(1, 1, true).is_err());
    w.allianz_hilfe_status(0, 1, true).unwrap();
    w.allianz_verlassen(0).unwrap();
    assert_eq!(w.allianz_rang(1, 1), Some(0));
}
#[test]
fn all_new_actions_are_role_checked_and_serializable() {
    let mut w = world();
    let a = json!({"typ":"brief_senden","kanal":"privat","an":"P1","betreff":"test","text":"test","antwort_auf":null});
    assert!(serde_json::from_value::<Aktion>(a.clone()).is_ok());
    assert!(!w.handeln(0, Rolle::Verwalter, &a).0);
    assert!(w.handeln(0, Rolle::Diplomat, &a).0);
}
#[test]
fn alliance_treaties_and_colors_follow_current_relation_without_scanning() {
    let mut w = world();
    alliance(&mut w, 0, &[1, 2, 3, 4], "A");
    alliance(&mut w, 5, &[6, 7, 8, 9], "B");
    assert_eq!(w.beziehung(0, 5)["status"], "neutral");
    assert!(w.beziehung(0, 5)["farbe"].is_null());
    assert_eq!(w.beziehung(0, 1)["status"], "verbuendet");
    assert_eq!(w.beziehung(0, 5)["allianz"], "B");
    w.allianz_anfrage(0, "B", "handelsabkommen", "Handel?")
        .unwrap();
    assert!(w.diplomatie_entscheiden(9, 1, true).is_err());
    w.diplomatie_entscheiden(6, 1, true).unwrap();
    assert_eq!(w.beziehung(1, 9)["status"], "handelsabkommen");
    w.allianz_anfrage(0, "B", "nichtangriffspakt", "NAP?")
        .unwrap();
    w.diplomatie_entscheiden(5, 2, true).unwrap();
    assert_eq!(w.beziehung(4, 9)["status"], "nap");
    w.allianz_anfrage(0, "B", "verteidigungsbuendnis", "Bündnis?")
        .unwrap();
    w.diplomatie_entscheiden(5, 3, true).unwrap();
    assert_eq!(w.beziehung(4, 9)["status"], "verbuendet");
    w.allianz_anfrage(0, "B", "krieg", "Krieg erklärt").unwrap();
    assert_eq!(w.beziehung(4, 9)["status"], "feindlich");
    assert!(!w.verbuendet(4, 9));
    let k = w.planeten[w.spieler[9].heimat as usize].koord;
    let unknown = w.planetenwissen(0, k);
    assert_eq!(unknown["bekannt"], false);
    assert!(unknown["spieler"].is_null());
    assert!(unknown["beziehung"].is_null());
    w.ausscheiden.besiegt.insert(
        9,
        kern::ausscheiden::Niederlage {
            zeit: 0,
            grund: "Test".into(),
        },
    );
    assert_eq!(w.beziehung(4, 9)["status"], "ausgeschieden");
}
#[test]
fn alliance_nap_cancellation_observes_notice_period() {
    let mut w = world();
    alliance(&mut w, 0, &[1], "A");
    alliance(&mut w, 2, &[3], "B");
    w.allianz_anfrage(0, "B", "nichtangriffspakt", "NAP?")
        .unwrap();
    w.diplomatie_entscheiden(2, 1, true).unwrap();
    w.allianz_pakt_kuendigen(0, 1).unwrap();
    assert_eq!(w.beziehung(1, 3)["status"], "nap");
    w.zeit = w.kommunikation.anfragen[0].kuendigung_ende.unwrap();
    assert_eq!(w.beziehung(1, 3)["status"], "neutral");
    w.allianz_anfrage(0, "B", "nichtangriffspakt", "Neuer NAP?")
        .unwrap();
}
#[test]
fn private_peace_cannot_end_alliance_war() {
    let mut w = world();
    alliance(&mut w, 0, &[1], "A");
    alliance(&mut w, 2, &[3], "B");
    w.allianz_anfrage(0, "B", "krieg", "Krieg").unwrap();
    w.diplomatie_anfrage(0, "P2", "frieden", "Privater Frieden")
        .unwrap();
    w.diplomatie_entscheiden(2, 2, true).unwrap();
    assert_eq!(w.beziehung(1, 3)["status"], "feindlich");
    w.allianz_anfrage(0, "B", "frieden", "Allianzfrieden")
        .unwrap();
    w.diplomatie_entscheiden(2, 3, true).unwrap();
    assert_eq!(w.beziehung(1, 3)["status"], "neutral");
}
