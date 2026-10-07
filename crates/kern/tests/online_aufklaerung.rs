use kern::{flotte::Flugauftrag, welt::Flottenzustand, *};
use serde_json::json;
fn world() -> Welt {
    let mut r = Regelwerk::laden(include_str!("../../../regeln/online-v1.ron")).unwrap();
    r.welt.fenster_sekunden = 60;
    let mut w = Welt::neu(r, 42, 3).unwrap();
    w.ereignisse.clear();
    for s in &mut w.spieler {
        s.ki = false;
        s.schutz_bis = 0;
        s.forschung[Forschung::Computertechnik.idx()] = 10;
    }
    for p in &mut w.planeten {
        p.gebaeude[Gebaeude::Raumhafen.idx()] = 1;
        p.bestand = [1_000_000 * M; GUETER];
        p.rate = [0; GUETER];
        p.einheiten[Einheit::Spionagesonde.idx()] = 20;
        p.einheiten[Einheit::KleinerTransporter.idx()] = 3;
        p.einheiten[Einheit::LeichterJaeger.idx()] = 17;
    }
    w
}
#[test]
fn new_intelligence_research_requires_training_and_own_building() {
    let mut w=world();let p=w.spieler[0].heimat as usize;
    w.spieler[0].stufe=2;w.planeten[p].gebaeude[Gebaeude::Labor.idx()]=2;
    for f in [Forschung::Ueberwachungstechnik,Forschung::Abschirmtechnik] {
        let stock=w.planeten[p].bestand;assert!(!w.handeln(0,Rolle::Alle,&json!({"typ":"forschen","forschung":f.name()})).0);assert_eq!(stock,w.planeten[p].bestand);assert!(w.spieler[0].forschung_aktiv.is_none());
    }
    w.spieler[0].forschung[Forschung::Spionagetechnik.idx()]=1;
    assert!(!w.handeln(0,Rolle::Alle,&json!({"typ":"forschen","forschung":"ueberwachungstechnik"})).0);
    w.planeten[p].gebaeude[Gebaeude::Geheimdienst.idx()]=1;
    assert!(w.handeln(0,Rolle::Alle,&json!({"typ":"forschen","forschung":"ueberwachungstechnik"})).0);
}
fn home(w: &Welt, s: usize) -> Koord {
    w.planeten[w.spieler[s].heimat as usize].koord
}
fn fly(w: &mut Welt, s: usize, m: Mission, z: Koord, e: Einheit, n: i64, speed: i64) -> u32 {
    if m.feindlich() {
        let p = &w.planeten[*w.belegung.get(&z).unwrap() as usize];
        w.scans.insert(
            (s as u16, z),
            kern::sternkarte::Scan {
                zeit: w.zeit,
                technik: 1,
                besitzer: Some(p.besitzer),
                felder: p.felder,
                zone: p.zone,
                erz: [0, 800],
                kristall: [0, 800],
                bestand: vec![],
            },
        );
    }
    let id = w.naechste_flotte;
    w.flotte_senden(
        s as u16,
        Rolle::Alle,
        Flugauftrag {
            start: home(w, s),
            ziel: z,
            mission: m,
            schiffe: {
                let mut a = [0; SCHIFFE];
                a[e.idx()] = n;
                a
            },
            sigma_pm: speed,
            ladung: [0; GUETER],
            haltedauer: 0,
        },
    )
    .unwrap();
    id
}
fn advance(w: &mut Welt, t: i64) {
    while w.zeit < t {
        assert!(w.schritt());
    }
}
#[test]
fn exact_build_timing_survives_save_and_bauhof_changes() {
    let mut w=world();assert!(w.aufklaerung.neues_layout);
    let pid=w.spieler[0].heimat as usize;let k=w.planeten[pid].koord;
    w.planeten[pid].stabilitaet=80*M;
    w.bauen(0,Rolle::Alle,k,Gebaeude::Farm).unwrap();
    let duration=w.planeten[pid].bauschleife[0].dauer;assert!(duration>0);
    w.planeten[pid].gebaeude[Gebaeude::Bauhof.idx()]=15;
    let bytes=w.zu_bytes();assert!(bytes.starts_with(b"STERNEP7"));
    let loaded=Welt::aus_bytes(&bytes).unwrap();
    assert_eq!(loaded.planeten[pid].bauschleife[0].dauer,duration);
    let sight=loaded.sicht(0,Rolle::Alle);
    let job=&sight["planeten"][0]["bauschleife"][0];
    assert_eq!(job["dauer_sekunden"],duration);
    assert_eq!(job["fertig_sekunden"],w.planeten[pid].bauschleife[0].fertig.unwrap());
    // Existing V6 checkpoints still load with a safe duration fallback.
    let len=u64::from_le_bytes(bytes[8..16].try_into().unwrap()) as usize;
    let old=Welt::aus_bytes(&bytes[16..16+len]).unwrap();
    assert_eq!(old.planeten[pid].bauschleife[0].dauer,0);
    assert!(old.sicht(0,Rolle::Alle)["planeten"][0]["bauschleife"][0]["dauer_sekunden"].as_i64().unwrap()>0);
    w.spieler[0].forschung_aktiv=Some(kern::welt::Forschungsauftrag{forschung:Forschung::Agrarwissenschaft,stufe:1,fp_rest:100*M,begonnen:Some(0)});
    let restored=Welt::aus_bytes(&w.zu_bytes()).unwrap();
    assert_eq!(restored.spieler[0].forschung_aktiv.as_ref().unwrap().begonnen,Some(0));
}
fn attack(w: &mut Welt) -> u32 {
    let k = home(w, 1);
    fly(w, 0, Mission::Angriff, k, Einheit::LeichterJaeger, 17, 100)
}

#[test]
fn unknown_target_and_hidden_fleet_errors_cannot_be_used_as_free_scans() {
    let mut w = world();
    let start = home(&w, 0);
    let occupied = home(&w, 1);
    let empty = Koord::neu(
        occupied.sektor,
        occupied.system,
        if occupied.position == 1 { 2 } else { 1 },
    );
    let action = |z: Koord| json!({"typ":"flotte_senden","start":start.to_string(),"ziel":z.to_string(),"mission":"angriff","schiffe":{"leichter_jaeger":1}});
    let ships = w.planeten[0].einheiten;
    let stock = w.planeten[0].bestand;
    let a = w.handeln(0, Rolle::Alle, &action(occupied));
    let b = w.handeln(0, Rolle::Alle, &action(empty));
    assert!(!a.0);
    assert_eq!(a, b);
    assert_eq!(stock, w.planeten[0].bestand);
    assert_eq!(ships, w.planeten[0].einheiten);
    assert_eq!(
        w.eigener_planet(0, occupied)
            .unwrap_err()
            .replace(&occupied.to_string(), "ZIEL"),
        w.eigener_planet(0, empty)
            .unwrap_err()
            .replace(&empty.to_string(), "ZIEL")
    );
    let id = fly(
        &mut w,
        1,
        Mission::Saven,
        empty,
        Einheit::KleinerTransporter,
        1,
        1000,
    );
    assert_eq!(
        w.flotte_zurueckrufen(0, id).unwrap_err(),
        w.flotte_zurueckrufen(0, 999999).unwrap_err()
    );
}

#[test]
fn saves_reject_full_slots_missing_fuel_overflow_and_blockaded_departures() {
    let mut w = world();
    let start = home(&w, 0);
    let target = home(&w, 1);
    let request = |cargo: i64| Flugauftrag {
        start,
        ziel: target,
        mission: Mission::Saven,
        schiffe: {
            let mut a = [0; SCHIFFE];
            a[Einheit::KleinerTransporter.idx()] = 1;
            a
        },
        sigma_pm: 1000,
        ladung: {
            let mut a = [0; GUETER];
            a[Gut::Erz.idx()] = cargo;
            a
        },
        haltedauer: 0,
    };
    assert!(w.flotte_senden(0, Rolle::Alle, request(i64::MAX)).is_err());
    let fuel = w.planeten[0].bestand[Gut::Deuterium.idx()];
    w.planeten[0].bestand[Gut::Deuterium.idx()] = 0;
    assert!(w.flotte_senden(0, Rolle::Alle, request(0)).is_err());
    w.planeten[0].bestand[Gut::Deuterium.idx()] = fuel;
    w.spieler[0].forschung[Forschung::Computertechnik.idx()] = 0;
    fly(
        &mut w,
        0,
        Mission::Saven,
        target,
        Einheit::KleinerTransporter,
        1,
        1000,
    );
    assert!(w
        .flotte_senden(0, Rolle::Alle, request(0))
        .unwrap_err()
        .contains("Flottenplätze"));
    w.spieler[0].forschung[Forschung::Computertechnik.idx()] = 10;
    let fid = fly(
        &mut w,
        1,
        Mission::Angriff,
        start,
        Einheit::LeichterJaeger,
        1,
        1000,
    );
    w.flotten.get_mut(&fid).unwrap().zustand = Flottenzustand::ImOrbit;
    w.planeten[0].blockade = Some(fid);
    assert!(w
        .flotte_senden(0, Rolle::Alle, request(0))
        .unwrap_err()
        .contains("blockiert"));
}

#[test]
fn real_legacy_snapshot_keeps_exact_bytes_hash_and_old_rules() {
    let bytes = include_bytes!("fixtures/legacy-v4.bin");
    let mut w = Welt::aus_bytes(bytes).unwrap();
    assert_eq!(w.hash(), include_str!("fixtures/legacy-v4.sha256"));
    assert!(w.zu_bytes() == bytes, "Historical bytes changed");
    assert!(!w.aufklaerungsregeln());
    assert_eq!(w.warnzeit(w.spieler[0].heimat as usize), 30 * MINUTE);
    assert!(w
        .handeln(
            0,
            Rolle::Alle,
            &json!({"typ":"bauen","planet":home(&w,0).to_string(),"gebaeude":"geheimdienst"})
        )
        .1
        .contains("Partie"));
    assert!(w.schritt());
    assert_eq!(Welt::aus_bytes(&w.zu_bytes()).unwrap().hash(), w.hash());
}
#[test]
fn system_probe_discovers_no_planets_and_each_planet_needs_its_own_probe() {
    let mut w = world();
    let k = home(&w, 1);
    assert!(!w.system_erfasst(0, k));
    let rejected=w.handeln(0,Rolle::Alle,&json!({"typ":"flotte_senden","start":home(&w,0).to_string(),"ziel":k.to_string(),"mission":"spionage","schiffe":{"spionagesonde":1}}));
    assert!(!rejected.0);
    assert!(rejected.1.contains("Sonnensystem"));
    let id = fly(
        &mut w,
        0,
        Mission::SystemErkunden,
        k,
        Einheit::Spionagesonde,
        1,
        1000,
    );
    let arrival = w.flotten[&id].ankunft;
    advance(&mut w, arrival);
    assert!(w.system_erfasst(0, k));
    assert_eq!(w.planetenwissen(0, k)["bekannt"], false);
    let id = fly(
        &mut w,
        0,
        Mission::Spionage,
        k,
        Einheit::Spionagesonde,
        1,
        1000,
    );
    let arrival = w.flotten[&id].ankunft;
    advance(&mut w, arrival);
    assert_eq!(w.planetenwissen(0, k)["bekannt"], true);
    let other = Koord::neu(k.sektor, k.system, if k.position == 1 { 2 } else { 1 });
    assert_eq!(w.planetenwissen(0, other)["bekannt"], false);
    let unknown=w.werkzeug(0,&json!({"typ":"galaxie","sektor":home(&w,2).sektor,"von":home(&w,2).system,"bis":home(&w,2).system})).unwrap();
    assert!(unknown["systeme"][0]["nebel"].is_null());
    assert!(unknown["systeme"][0]["asteroidenguertel"].is_null());
    let remembered = w.planetenwissen(0, k);
    w.planeten[1].besitzer = 2;
    w.planeten[1].bestand = [0; GUETER];
    assert_eq!(remembered, w.planetenwissen(0, k));
}
#[test]
fn basic_warning_has_exact_time_but_no_identity_count_origin_or_cargo() {
    let mut w = world();
    let id = attack(&mut w);
    let arrival = w.flotten[&id].ankunft;
    assert!(arrival > 2 * STUNDE + MINUTE);
    w.zeit = arrival - 2 * STUNDE - 1;
    assert!(w.angriffswissen(1, id).is_none());
    w.zeit += 1;
    let v = w.angriffswissen(1, id).unwrap();
    assert_eq!(v["ankunft_sekunden"], arrival);
    assert_eq!(v["in_min"], 120);
    for field in ["von", "schiffe", "schiffstypen", "ladung", "start"] {
        assert!(v[field].is_null(), "{field} leaked");
    }
    let briefing = w.sicht(1, Rolle::Alle);
    assert!(briefing["angriffe"][0]["schiffe"].is_null());
    assert!(briefing["warnungen"]
        .as_array()
        .unwrap()
        .iter()
        .any(|v| v.as_str().unwrap().contains("Schiffszahl: unbekannt")));
    assert!(w.angriffswissen(2, id).is_none());
}
#[test]
fn detection_event_wakes_defender_before_impact_even_when_flight_is_short() {
    let mut w = world();
    let target = home(&w, 1);
    let id = fly(
        &mut w,
        0,
        Mission::Angriff,
        target,
        Einheit::Spionagesonde,
        1,
        1000,
    );
    assert!(w.flotten[&id].ankunft < 2 * STUNDE);
    assert!(w.spieler[1].weck[Rolle::Feldherr.idx()].dringend);
    assert!(w.angriffswissen(1, id).is_some());
    let mut w = world();
    let id = attack(&mut w);
    let time = w.flotten[&id].ankunft - 2 * STUNDE;
    advance(&mut w, time);
    assert!(w.spieler[1].weck[Rolle::Feldherr.idx()].dringend);
    assert_eq!(w.flotten[&id].zustand, Flottenzustand::Hinflug);
}
#[test]
fn upgrades_require_building_and_research_and_shielding_never_hides_arrival() {
    let mut w = world();
    let pid = w.spieler[1].heimat as usize;
    w.spieler[1].volk = Volk::Aurelianer;
    w.planeten[pid].gebaeude[Gebaeude::Sensorphalanx.idx()] = 8;
    assert_eq!(w.warnzeit(pid), 2 * STUNDE);
    w.planeten[pid].gebaeude[Gebaeude::Geheimdienst.idx()] = 8;
    w.spieler[1].forschung[Forschung::Ueberwachungstechnik.idx()] = 8;
    assert_eq!(w.warnzeit(pid), 2 * STUNDE);
    w.spieler[1].forschung[Forschung::Spionagetechnik.idx()] = 8;
    assert_eq!(w.warnzeit(pid), 6 * STUNDE);
    w.spieler[0].forschung[Forschung::Abschirmtechnik.idx()] = 7;
    let id = attack(&mut w);
    w.zeit = w.flotten[&id].ankunft - STUNDE;
    let view = w.angriffswissen(1, id).unwrap();
    assert!(view["schiffe"].is_null());
    assert!(view["schiffe_spanne"].is_array());
    w.spieler[0].forschung[Forschung::Abschirmtechnik.idx()] = 0;
    assert!(
        w.angriffswissen(1, id).unwrap()["schiffe"].is_null(),
        "retroactive refit"
    );
    w.aufklaerung.abschirmung.insert(id, 0);
    assert_eq!(w.angriffswissen(1, id).unwrap()["schiffe"], 17);
}
#[test]
fn fleet_probe_is_physical_private_shielded_and_stays_a_historical_report() {
    for shield in [0, 9] {
        let mut w = world();
        w.spieler[0].forschung[Forschung::Abschirmtechnik.idx()] = shield;
        let id = attack(&mut w);
        w.zeit = w.flotten[&id].ankunft - 2 * STUNDE;
        let before = w.planeten[1].einheiten[Einheit::Spionagesonde.idx()];
        let probe = w.naechste_flotte;
        let origin = home(&w, 1);
        w.flotte_ausspaehen(1, Rolle::Alle, origin, id, 1, 1000)
            .unwrap();
        assert_eq!(
            w.planeten[1].einheiten[Einheit::Spionagesonde.idx()],
            before - 1
        );
        assert!(w.eigene_flottenberichte(1).is_empty());
        let arrival = w.flotten[&probe].ankunft;
        advance(&mut w, arrival);
        let report = w.eigene_flottenberichte(1)[0].clone();
        assert_eq!(report["abgeschirmt"], shield > 0);
        if shield == 0 {
            assert_eq!(report["schiffe"], 17);
        } else {
            assert!(report["schiffe"].is_null());
        }
        assert!(w.eigene_flottenberichte(2).is_empty());
        w.flotten.get_mut(&id).unwrap().schiffe[Einheit::LeichterJaeger.idx()] = 2;
        assert_eq!(w.eigene_flottenberichte(1)[0]["schiffe"], report["schiffe"]);
        let mut restored = Welt::aus_bytes(&w.zu_bytes()).unwrap();
        assert_eq!(restored.hash(), w.hash());
        assert_eq!(
            restored.eigene_flottenberichte(1),
            w.eigene_flottenberichte(1)
        );
        let next = w.zeit + MINUTE;
        advance(&mut w, next);
        advance(&mut restored, next);
        assert_eq!(restored.hash(), w.hash());
    }
}
#[test]
fn unseen_and_too_late_fleet_probes_cannot_spend_resources_or_oracle_ids() {
    let mut w = world();
    let id = attack(&mut w);
    let origin = home(&w, 1);
    let before = w.planeten[1].bestand;
    let e = w
        .flotte_ausspaehen(1, Rolle::Alle, origin, id, 1, 1000)
        .unwrap_err();
    assert_eq!(
        e,
        w.flotte_ausspaehen(1, Rolle::Alle, origin, 999999, 1, 1000)
            .unwrap_err()
    );
    assert_eq!(before, w.planeten[1].bestand);
    w.zeit = w.flotten[&id].ankunft - MINUTE;
    assert!(w
        .flotte_ausspaehen(1, Rolle::Alle, origin, id, 1, 1000)
        .unwrap_err()
        .contains("nach dem Angriff"));
    assert_eq!(before, w.planeten[1].bestand);
}
#[test]
fn flight_quote_keeps_fractional_fuel_for_the_full_save_return_trip() {
    let mut w = world();
    let origin = home(&w, 0);
    let target = Koord { sektor: origin.sektor, system: if origin.system < 60 { origin.system + 1 } else { origin.system - 1 }, position: 6 };
    let quote = w.werkzeug(0, &json!({"typ":"flugzeit","start":origin.to_string(),"ziel":target.to_string(),
        "schiffe":{"kleiner_transporter":1,"spionagesonde":1},"geschwindigkeit":1})).unwrap();
    let fuel = quote["treibstoff_je_strecke_milli"].as_i64().unwrap();
    assert_ne!(fuel % M, 0, "regression needs fractional fuel");
    let reserve = (2 * fuel + M - 1) / M;
    let pid = w.spieler[0].heimat as usize;
    let cargo = 100 * M;
    w.planeten[pid].bestand[Gut::Deuterium.idx()] = cargo + reserve * M;
    let before = w.planeten[pid].bestand[Gut::Deuterium.idx()];
    assert!(w.handeln(0, Rolle::Alle, &json!({"typ":"flotte_senden","start":origin.to_string(),"ziel":target.to_string(),
        "mission":"saven","schiffe":{"kleiner_transporter":1,"spionagesonde":1},"ladung":{"deuterium":100},"geschwindigkeit":1,"haltedauer_stunden":3})).0);
    assert_eq!(before - w.planeten[pid].bestand[Gut::Deuterium.idx()], cargo + 2 * fuel);
    assert!(w.planeten[pid].bestand[Gut::Deuterium.idx()] < M);
}
#[test]
fn saven_preserves_cargo_and_ships_and_has_a_real_return_fuel_cost_and_no_delivery() {
    let mut w = world();
    let origin = home(&w, 0);
    let target = home(&w, 1);
    let before = w.planeten[0].bestand;
    let id = w.naechste_flotte;
    assert!(w.handeln(0,Rolle::Alle,&json!({"typ":"flotte_senden","start":origin.to_string(),"ziel":target.to_string(),
        "mission":"saven","schiffe":{"kleiner_transporter":2},"ladung":{"erz":500},"geschwindigkeit":0.1,"haltedauer_stunden":1})).0);
    let delta = before[Gut::Deuterium.idx()] - w.planeten[0].bestand[Gut::Deuterium.idx()];
    assert!(delta > 0);
    let target_before = w.planeten[1].bestand;
    let at = w.flotten[&id].ankunft;
    advance(&mut w, at);
    assert_eq!(w.flotten[&id].zustand, Flottenzustand::ImOrbit);
    assert_eq!(w.flotten[&id].ladung[Gut::Erz.idx()], 500 * M);
    assert_eq!(w.planeten[1].bestand, target_before);
    let end = w.flotten[&id].orbit_ende;
    advance(&mut w, end);
    let back = w.flotten[&id].rueckkehr;
    advance(&mut w, back);
    assert!(!w.flotten.contains_key(&id));
    assert_eq!(
        w.planeten[0].bestand[Gut::Erz.idx()],
        before[Gut::Erz.idx()]
    );
    assert_eq!(
        w.planeten[0].bestand[Gut::Deuterium.idx()],
        before[Gut::Deuterium.idx()] - delta
    );
    assert_eq!(
        w.planeten[0].einheiten[Einheit::KleinerTransporter.idx()],
        3
    );
    assert!(w.kampfberichte.is_empty());
}
