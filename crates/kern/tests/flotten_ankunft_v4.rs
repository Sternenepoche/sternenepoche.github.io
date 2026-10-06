use kern::{
    flotte::Flugauftrag,
    kolonisation::{Besetzung, Reparatur},
    typen::*,
    welt::*,
    Regelwerk,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BinaryHeap};

fn world() -> Welt {
    let mut r = Regelwerk::laden(include_str!("../../../regeln/regelwerk.ron")).unwrap();
    r.diplomatie.schwachenschutz_anteil = 0.0;
    let mut w = Welt::neu(r, 42, 3).unwrap();
    w.kolonisationsregeln_v2_aktivieren();
    for sid in 0..3 {
        let sp = &mut w.spieler[sid];
        sp.stufe = 4;
        sp.schutz_bis = 0;
        sp.forschung[Forschung::Astrophysik.idx()] = 5;
        sp.forschung[Forschung::Computertechnik.idx()] = 10;
        sp.toepfe = [1_000_000_000 * M; TOEPFE];
        let p = &mut w.planeten[sp.heimat as usize];
        p.bestand = [1_000_000 * M; GUETER];
        p.bevoelkerung = 20_000 * M;
        p.gebaeude[Gebaeude::Raumhafen.idx()] = 1;
        p.einheiten = [0; EINHEITEN];
        p.einheiten[Einheit::Kreuzer.idx()] = 100;
        p.einheiten[Einheit::Kolonieschiff.idx()] = 3;
        p.einheiten[Einheit::GrosserTransporter.idx()] = 20;
    }
    w
}
fn colony(w: &mut Welt, sid: SpielerId) -> usize {
    let mut p = w.planeten[w.spieler[sid as usize].heimat as usize].clone();
    p.id = w.planeten.len() as PlanetId;
    p.heimat = false;
    p.koord.position = 8;
    p.einheiten = [0; EINHEITEN];
    w.belegung.insert(p.koord, p.id);
    w.spieler[sid as usize].planeten.push(p.id);
    let pid = p.id as usize;
    w.planeten.push(p);
    pid
}
fn fleet(
    w: &mut Welt,
    id: FlottenId,
    sid: SpielerId,
    pid: usize,
    mission: Mission,
    state: Flottenzustand,
    cruisers: i64,
    colonies: i64,
) {
    let home = w.spieler[sid as usize].heimat;
    let mut ships = [0; SCHIFFE];
    ships[Einheit::Kreuzer.idx()] = cruisers;
    ships[Einheit::Kolonieschiff.idx()] = colonies;
    w.flotten.insert(
        id,
        Flotte {
            id,
            besitzer: sid,
            start: home,
            start_koord: w.planeten[home as usize].koord,
            ziel: w.planeten[pid].koord,
            mission,
            schiffe: ships,
            ladung: [0; GUETER],
            siedler: 0,
            abflug: 0,
            ankunft: w.zeit,
            flugdauer: 600,
            rueckkehr: 0,
            orbit_ende: 0,
            haltedauer: 0,
            zustand: state,
            gemeldet: true,
            verband: None,
        },
    );
}
fn order(w: &Welt, pid: usize, mission: Mission) -> Flugauftrag {
    let mut schiffe = [0; SCHIFFE];
    schiffe[Einheit::GrosserTransporter.idx()] = 1;
    Flugauftrag {
        start: w.planeten[w.spieler[0].heimat as usize].koord,
        ziel: w.planeten[pid].koord,
        mission,
        schiffe,
        sigma_pm: 1000,
        ladung: [0; GUETER],
        haltedauer: 0,
    }
}
fn occupy(w: &mut Welt, pid: usize) {
    fleet(
        w,
        900,
        0,
        pid,
        Mission::Kampfkolonisieren,
        Flottenzustand::ImOrbit,
        100,
        1,
    );
    w.planeten[pid].blockade = Some(900);
    w.bombardement_abschluss(pid, 900);
    assert!(w.kolonisation.besetzungen.contains_key(&(pid as PlanetId)));
}

#[test]
fn old_extension_bytes_and_hash_are_exact_and_v4_roundtrips() {
    #[derive(Serialize)]
    struct Old<'a> {
        aktiv: bool,
        integritaet: &'a BTreeMap<(PlanetId, Gebaeude), u16>,
        besetzungen: &'a BTreeMap<PlanetId, Besetzung>,
        reparaturen: &'a BTreeMap<(PlanetId, Gebaeude), Reparatur>,
    }
    let mut w = world();
    w.kolonisationsregeln_aktivieren();
    w.kolonisation.integritaet.insert((0, Gebaeude::Farm), 300);
    let base = bincode::serialize(&w).unwrap();
    let old = Old {
        aktiv: true,
        integritaet: &w.kolonisation.integritaet,
        besetzungen: &w.kolonisation.besetzungen,
        reparaturen: &w.kolonisation.reparaturen,
    };
    let extension = bincode::serialize(&old).unwrap();
    let mut historical = b"STERNEP3".to_vec();
    historical.extend((base.len() as u64).to_le_bytes());
    historical.extend(base);
    historical.extend(&extension);
    assert_eq!(w.zu_bytes(), historical);
    assert_eq!(Welt::aus_bytes(&historical).unwrap().zu_bytes(), historical);
    let mut copy = w.clone();
    let events = std::mem::replace(&mut copy.ereignisse, BinaryHeap::new()).into_sorted_vec();
    let mut hash = Sha256::new();
    hash.update(bincode::serialize(&copy).unwrap());
    hash.update(bincode::serialize(&events).unwrap());
    hash.update(b"kolonisation-v1");
    hash.update(extension);
    assert_eq!(w.hash(), format!("{:x}", hash.finalize()));
    w.kolonisation.aktiv = false;
    let bare = bincode::serialize(&w).unwrap();
    assert_eq!(w.zu_bytes(), bare);
    assert_eq!(Welt::aus_bytes(&bare).unwrap().zu_bytes(), bare);
    w.kolonisationsregeln_v2_aktivieren();
    let bytes = w.zu_bytes();
    assert!(bytes.starts_with(b"STERNEP4"));
    assert_eq!(Welt::aus_bytes(&bytes).unwrap().hash(), w.hash());
    let h = w.hash();
    w.vorfall(0, "test", "flüchtig".into());
    assert_ne!(w.hash(), h); // ordinary notices remain historical state
    let h = w.hash();
    assert!(!w.fachereignisse_abholen().is_empty());
    assert_eq!(w.hash(), h);
}

#[test]
fn transport_binding_returns_cargo_after_owner_change() {
    let mut w = world();
    let p = colony(&mut w, 1);
    let mut a = order(&w, p, Mission::Transport);
    a.ladung[Gut::Erz.idx()] = 100 * M;
    let fid = w.naechste_flotte;
    w.flotte_senden(0, Rolle::Feldherr, a).unwrap();
    let view = w.sicht(0, Rolle::Feldherr);
    let fv = view["flotten"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["flotte"] == fid)
        .unwrap();
    assert_eq!(fv["gebundener_empfaenger"]["spieler"], 1);
    assert_eq!(fv["gebundener_empfaenger"]["art"], "planet");
    assert!(w.sicht(1, Rolle::Feldherr)["flotten"]
        .as_array()
        .unwrap()
        .iter()
        .all(|f| f["flotte"] != fid));
    w.planeten[p].besitzer = 2;
    let stock = w.planeten[p].bestand;
    w.zeit = w.flotten[&fid].ankunft;
    w.flotte_ankunft(fid);
    assert_eq!(w.planeten[p].bestand, stock);
    assert_eq!(w.flotten[&fid].ladung[Gut::Erz.idx()], 100 * M);
    assert_eq!(w.spieler[2].statistik.erhalten, 0);
}

#[test]
fn explicit_supply_reaches_bound_fleet_and_ordinary_transport_cannot_feed_enemy_planet() {
    let mut w = world();
    let p = colony(&mut w, 1);
    occupy(&mut w, p);
    let mut a = order(&w, p, Mission::Transport);
    a.ladung[Gut::Erz.idx()] = 100 * M;
    let stock = w.planeten[p].bestand;
    let first = w.naechste_flotte;
    w.flotte_senden(0, Rolle::Feldherr, a.clone()).unwrap();
    w.zeit = w.flotten[&first].ankunft;
    w.flotte_ankunft(first);
    assert_eq!(w.planeten[p].bestand, stock);
    assert_eq!(w.flotten[&first].ladung[Gut::Erz.idx()], 100 * M);
    let next = w.naechste_flotte;
    w.flotte_versorgen(0, Rolle::Feldherr, a, 900).unwrap();
    let view = w.sicht(0, Rolle::Feldherr);
    let fv = view["flotten"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["flotte"] == next)
        .unwrap();
    assert_eq!(fv["gebundener_empfaenger"]["flotte"], 900);
    assert_eq!(fv["gebundener_empfaenger"]["art"], "flotte");
    w.zeit = w.flotten[&next].ankunft;
    w.flotte_ankunft(next);
    assert_eq!(w.flotten[&900].ladung[Gut::Erz.idx()], 100 * M);
    assert_eq!(w.flotten[&next].ladung, [0; GUETER]);
    assert_eq!(w.planeten[p].bestand, stock);
    let mut a = order(&w, p, Mission::Transport);
    a.ladung[Gut::Erz.idx()] = 100 * M;
    let lost = w.naechste_flotte;
    w.flotte_versorgen(0, Rolle::Feldherr, a, 900).unwrap();
    w.flotten.remove(&900);
    fleet(
        &mut w,
        901,
        0,
        p,
        Mission::Blockade,
        Flottenzustand::ImOrbit,
        100,
        0,
    );
    w.planeten[p].blockade = Some(901);
    w.zeit = w.flotten[&lost].ankunft;
    w.flotte_ankunft(lost);
    assert_eq!(w.flotten[&lost].ladung[Gut::Erz.idx()], 100 * M);
    assert_eq!(w.flotten[&901].ladung, [0; GUETER]);
}

#[test]
fn single_and_group_relief_fight_real_blockader_and_leave_allied_planet_intact() {
    for grouped in [false, true] {
        let mut w = world();
        let p = colony(&mut w, 1);
        w.spieler[0].allianz = Some(7);
        w.spieler[1].allianz = Some(7);
        w.spieler[1].schutz_bis = 10000;
        fleet(
            &mut w,
            800,
            2,
            p,
            Mission::Blockade,
            Flottenzustand::ImOrbit,
            1,
            0,
        );
        w.planeten[p].blockade = Some(800);
        // Sending checks the real blockader, not the protected ally being rescued.
        let mut a = order(&w, p, Mission::Angriff);
        a.schiffe = [0; SCHIFFE];
        a.schiffe[Einheit::Kreuzer.idx()] = 80;
        let fid = w.naechste_flotte;
        w.flotte_senden(0, Rolle::Feldherr, a).unwrap();
        if grouped {
            w.flotten.get_mut(&fid).unwrap().verband = Some(fid);
            fleet(
                &mut w,
                901,
                1,
                p,
                Mission::Angriff,
                Flottenzustand::Hinflug,
                80,
                0,
            );
            let eta = w.flotten[&fid].ankunft;
            let extra = w.flotten.get_mut(&901).unwrap();
            extra.ankunft = eta;
            extra.verband = Some(fid);
        }
        let units = w.planeten[p].einheiten;
        w.zeit = w.flotten[&fid].ankunft;
        w.abrechnen(p);
        let cargo = w.planeten[p].bestand;
        w.flotte_ankunft(fid);
        assert!(!w.flotten.contains_key(&800));
        assert_eq!(w.planeten[p].blockade, None);
        assert_eq!(w.planeten[p].einheiten, units);
        assert_eq!(w.planeten[p].bestand, cargo);
        assert_eq!(w.kampfberichte.last().unwrap().verteidiger, vec![2]);
        assert!(!w.spieler[0].angegriffen.contains(&1));
        assert!(w.spieler[0].angegriffen.contains(&2));
    }
}

#[test]
fn changed_alliance_or_protected_blockader_cancels_both_attack_paths() {
    for grouped in [false, true] {
        for new_ally in [false, true] {
            let mut w = world();
            let p = colony(&mut w, 1);
            fleet(
                &mut w,
                800,
                2,
                p,
                Mission::Blockade,
                Flottenzustand::ImOrbit,
                1,
                0,
            );
            w.planeten[p].blockade = Some(800);
            fleet(
                &mut w,
                900,
                0,
                p,
                Mission::Angriff,
                Flottenzustand::Hinflug,
                100,
                0,
            );
            if grouped {
                w.flotten.get_mut(&900).unwrap().verband = Some(900);
            }
            if new_ally {
                w.spieler[0].allianz = Some(7);
                w.spieler[2].allianz = Some(7);
            } else {
                w.spieler[2].schutz_bis = 10000;
            }
            w.flotte_ankunft(900);
            assert_eq!(w.kampf_nr, 0);
            assert_eq!(w.flotten[&900].zustand, Flottenzustand::Rueckflug);
            assert!(w.flotten.contains_key(&800));
        }
    }
}

#[test]
fn return_redirects_from_actual_old_destination_and_waits_under_blockade() {
    let mut w = world();
    let p = colony(&mut w, 0);
    let home = w.spieler[0].heimat as usize;
    fleet(
        &mut w,
        900,
        0,
        p,
        Mission::Transport,
        Flottenzustand::Rueckflug,
        1,
        0,
    );
    {
        let f = w.flotten.get_mut(&900).unwrap();
        f.start = p as PlanetId;
        f.start_koord = w.planeten[p].koord;
        f.rueckkehr = 0;
    }
    w.planeten[p].besitzer = 1;
    let units = w.planeten[home].einheiten;
    w.flotte_rueckkehr(900);
    assert_eq!(w.planeten[home].einheiten, units);
    assert!(w.flotten[&900].rueckkehr > 0);
    fleet(
        &mut w,
        800,
        2,
        home,
        Mission::Blockade,
        Flottenzustand::ImOrbit,
        1,
        0,
    );
    w.planeten[home].blockade = Some(800);
    w.zeit = w.flotten[&900].rueckkehr;
    w.flotte_rueckkehr(900);
    assert_eq!(w.flotten[&900].zustand, Flottenzustand::ImOrbit);
    assert!(w.kolonisation.rueckwege[&900].wartend);
    let view = w.sicht(0, Rolle::Feldherr);
    let fv = view["flotten"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["flotte"] == 900)
        .unwrap();
    assert_eq!(fv["rueckkehr"]["wartend"], true);
    assert_eq!(fv["rueckkehr"]["ziel"], w.planeten[home].koord.to_string());
    assert!(w.sicht(1, Rolle::Feldherr)["flotten"]
        .as_array()
        .unwrap()
        .iter()
        .all(|f| f["flotte"] != 900));
    assert_eq!(w.planeten[home].einheiten, units);
    w.flotten.remove(&800);
    w.planeten[home].blockade = None;
    w.zeit = w.flotten[&900].orbit_ende;
    w.orbit_ende(900);
    assert!(!w.flotten.contains_key(&900));
    assert_eq!(
        w.planeten[home].einheiten[Einheit::Kreuzer.idx()],
        units[Einheit::Kreuzer.idx()] + 1
    );
}

#[test]
fn demolition_removes_dead_integrity_and_releases_repair_site_for_queued_build() {
    let mut w = world();
    let p = colony(&mut w, 1);
    let k = w.planeten[p].koord;
    w.planeten[p].gebaeude[Gebaeude::Farm.idx()] = 1;
    w.kolonisation
        .integritaet
        .insert((p as PlanetId, Gebaeude::Farm), 300);
    w.reparieren(1, Rolle::Verwalter, k, Gebaeude::Farm)
        .unwrap();
    w.bauen(1, Rolle::Verwalter, k, Gebaeude::Erzmine).unwrap();
    w.abreissen(1, k, Gebaeude::Farm).unwrap();
    assert_eq!(w.integritaet(p, Gebaeude::Farm), 1000);
    assert!(!w
        .kolonisation
        .reparaturen
        .contains_key(&(p as PlanetId, Gebaeude::Farm)));
    assert!(w.planeten[p].bauschleife[0].fertig.is_some());
    assert!(w.bauen(1, Rolle::Verwalter, k, Gebaeude::Farm).is_ok());
}

#[test]
fn capture_cancels_planet_work_events_and_research_fallback_rechecks_laboratory() {
    let mut w = world();
    let p = colony(&mut w, 1);
    occupy(&mut w, p);
    w.planeten[p].bauschleife.push(Bauauftrag {
        gebaeude: Gebaeude::Farm,
        stufe: 2,
        fertig: Some(10000),
        topf: None,
    });
    w.planeten[p].fertigung[0].push(Fertigung {
        produkt: Produkt::Einheit(Einheit::GrosserTransporter),
        rest: 10,
        dauer: 600,
    });
    w.planeten[p].fertigung_naechste[0] = 10000;
    for art in [
        EreignisArt::BauFertig {
            planet: p as PlanetId,
        },
        EreignisArt::FertigungFertig {
            planet: p as PlanetId,
            schleife: 0,
        },
        EreignisArt::RaketenFertig {
            planet: p as PlanetId,
            besitzer: 1,
            art: 0,
            anzahl: 1,
        },
    ] {
        w.plane(10000, art);
    }
    w.kolonisation.reparaturen.insert(
        (p as PlanetId, Gebaeude::Farm),
        Reparatur {
            besitzer: 1,
            fertig: 10000,
        },
    );
    let home = w.spieler[1].heimat as usize;
    w.planeten[home].gebaeude[Gebaeude::Labor.idx()] = 0;
    assert!(w.regeln.forsch(Forschung::Astrophysik).labor > 0);
    w.spieler[1].forschung_aktiv = Some(Forschungsauftrag {
        forschung: Forschung::Waffentechnik,
        stufe: 2,
        fp_rest: 100000000,
    });
    w.spieler[1]
        .forschung_schlange
        .push((Forschung::Astrophysik, p as PlanetId, None));
    w.zeit = 900;
    w.kolonisation_fensterabschluss();
    w.zeit = 1800;
    w.kolonisation_fensterabschluss();
    assert_eq!(w.planeten[p].besitzer, 0);
    assert!(w.planeten[p].bauschleife.is_empty());
    assert!(w.planeten[p].fertigung[0].is_empty());
    assert_eq!(w.planeten[p].fertigung_naechste, [-1, -1]);
    assert!(w.ereignisse.iter().all(|e|!matches!(e.art,EreignisArt::BauFertig{planet}|EreignisArt::FertigungFertig{planet,..}|EreignisArt::RaketenFertig{planet,..} if planet==p as PlanetId)));
    assert!(!w
        .kolonisation
        .reparaturen
        .keys()
        .any(|(pid, _)| *pid == p as PlanetId));
    assert_eq!(
        w.spieler[1].forschung_aktiv.as_ref().unwrap().fp_rest,
        100000000
    );
    assert!(w.spieler[0].forschung_aktiv.is_none());
    w.spieler[1].forschung_aktiv.as_mut().unwrap().fp_rest = 0;
    w.abrechnen(home);
    let captured_stock = w.planeten[p].bestand;
    let home_stock = w.planeten[home].bestand;
    w.tick();
    assert!(w.spieler[1].forschung_aktiv.is_none());
    assert!(w.spieler[1].forschung_schlange.is_empty());
    assert_eq!(w.planeten[p].bestand, captured_stock);
    assert_eq!(w.planeten[home].bestand, home_stock);
    assert!(w.spieler[1].vorfaelle.iter().any(|v| v.art == "forschung"
        && v.text.contains("nicht begonnen")
        && v.text.contains("Labor")));
}

#[test]
fn occupation_boundary_is_idempotent_even_after_more_same_clock_defense() {
    let mut w = world();
    let p = colony(&mut w, 1);
    occupy(&mut w, p);
    w.planeten[p].einheiten[Einheit::LeichterJaeger.idx()] = 1;
    w.zeit = 900;
    w.kolonisation_fensterabschluss();
    assert_eq!(w.kampf_nr, 1);
    w.planeten[p].einheiten[Einheit::LeichterJaeger.idx()] = 1;
    w.kolonisation_fensterabschluss();
    assert_eq!(w.kampf_nr, 1);
    assert_eq!(w.planeten[p].einheiten[Einheit::LeichterJaeger.idx()], 1);
    w.zeit = 1800;
    w.kolonisation_fensterabschluss();
    assert_eq!(w.kampf_nr, 2);
    assert_eq!(w.planeten[p].besitzer, 0);
}

#[test]
fn neutral_market_delivery_stays_bound_to_buyer_redirects_and_waits() {
    let mut w = world();
    let p = colony(&mut w, 0);
    let buyerhome = w.spieler[0].heimat as usize;
    let sellerhome = w.spieler[1].heimat as usize;
    for pid in [p, sellerhome] {
        w.planeten[pid].gebaeude[Gebaeude::Markt.idx()] = 1;
    }
    w.spieler[0].credits = 1000000000 * M;
    w.markt_order(
        1,
        Rolle::Verwalter,
        w.planeten[sellerhome].koord,
        Gut::Erz,
        Marktseite::Verkauf,
        10 * M,
        1000,
    )
    .unwrap();
    w.markt_order(
        0,
        Rolle::Verwalter,
        w.planeten[p].koord,
        Gut::Erz,
        Marktseite::Kauf,
        10 * M,
        1000,
    )
    .unwrap();
    let event = w
        .ereignisse
        .iter()
        .find(|e| {
            matches!(
                e.art,
                EreignisArt::MarktlieferungGebunden { empfaenger: 0, .. }
            )
        })
        .unwrap()
        .clone();
    assert!(
        matches!(event.art,EreignisArt::MarktlieferungGebunden{planet,empfaenger:0,..} if planet==p as PlanetId)
    );
    w.ereignisse.retain(|e| e.seq != event.seq);
    w.planeten[p].besitzer = 2;
    let stock = w.planeten[p].bestand;
    w.zeit = event.zeit;
    w.marktlieferung_gebunden(p, 0, Gut::Erz, 10 * M);
    assert_eq!(w.planeten[p].bestand, stock);
    let redirected = w
        .ereignisse
        .iter()
        .find(|e| matches!(e.art, EreignisArt::MarktlieferungGebunden { .. }))
        .unwrap()
        .clone();
    assert!(redirected.zeit > event.zeit);
    assert!(
        matches!(redirected.art,EreignisArt::MarktlieferungGebunden{planet,empfaenger:0,..} if planet==buyerhome as PlanetId)
    );
    w.ereignisse.retain(|e| e.seq != redirected.seq);
    w.planeten[buyerhome].blockade = Some(1234);
    w.zeit = redirected.zeit;
    let old = w.planeten[buyerhome].bestand;
    w.marktlieferung_gebunden(buyerhome, 0, Gut::Erz, 10 * M);
    assert_eq!(w.planeten[buyerhome].bestand, old);
    let waiting = w
        .ereignisse
        .iter()
        .find(|e| matches!(e.art, EreignisArt::MarktlieferungGebunden { .. }))
        .unwrap()
        .clone();
    assert_eq!(waiting.zeit, w.zeit + STUNDE);
    w.planeten[buyerhome].blockade = None;
    w.zeit = waiting.zeit;
    w.abrechnen(buyerhome);
    let old = w.planeten[buyerhome].bestand;
    w.marktlieferung_gebunden(buyerhome, 0, Gut::Erz, 10 * M);
    assert_eq!(
        w.planeten[buyerhome].bestand[Gut::Erz.idx()],
        old[Gut::Erz.idx()] + 10 * M
    );
    assert_eq!(
        bincode::serialize(&EreignisArt::Marktlieferung {
            planet: 0,
            gut: Gut::Erz,
            menge: 1
        })
        .unwrap()[..4],
        7u32.to_le_bytes()
    );
    assert_eq!(
        bincode::serialize(&EreignisArt::RaketenAnkunft {
            von: 0,
            ziel: w.planeten[p].koord,
            anzahl: 1,
            zieltyp: Einheit::LeichterJaeger
        })
        .unwrap()[..4],
        10u32.to_le_bytes()
    );
    assert_eq!(
        bincode::serialize(&EreignisArt::MarktlieferungGebunden {
            planet: 0,
            empfaenger: 0,
            gut: Gut::Erz,
            menge: 1
        })
        .unwrap()[..4],
        11u32.to_le_bytes()
    );
}

#[test]
fn occupation_ignores_farm_repair_and_probe_counts_full_unique_windows_and_preserves_people() {
    let mut w = world();
    let p = colony(&mut w, 1);
    w.zeit = 100;
    occupy(&mut w, p);
    w.kolonisation
        .integritaet
        .remove(&(p as PlanetId, Gebaeude::Farm));
    w.planeten[p].einheiten[Einheit::Spionagesonde.idx()] = 1;
    w.zeit = 900;
    w.kolonisation_fensterabschluss();
    assert_eq!(
        w.kolonisation.besetzungen[&(p as PlanetId)].reaktionsfenster,
        1
    );
    w.zeit = 1800;
    w.kolonisation_fensterabschluss();
    w.kolonisation_fensterabschluss();
    assert_eq!(
        w.kolonisation.besetzungen[&(p as PlanetId)].reaktionsfenster,
        2
    );
    assert_eq!(w.planeten[p].besitzer, 1);
    w.abrechnen(p);
    w.zeit = 2700;
    w.abrechnen(p);
    let people = w.planeten[p].bevoelkerung;
    w.kolonisation_fensterabschluss();
    assert_eq!(w.planeten[p].besitzer, 0);
    assert_eq!(w.planeten[p].bevoelkerung, people);
}

#[test]
fn new_armed_defense_is_actually_fought_before_capture() {
    let mut w = world();
    let p = colony(&mut w, 1);
    occupy(&mut w, p);
    w.zeit = 900;
    w.kolonisation_fensterabschluss();
    w.planeten[p].einheiten[Einheit::LeichterJaeger.idx()] = 1;
    w.zeit = 1800;
    w.kolonisation_fensterabschluss();
    assert_eq!(w.kampf_nr, 1);
    assert_eq!(w.planeten[p].besitzer, 0);
    assert!(w
        .fachereignisse
        .iter()
        .any(|e| e["event_type"] == "combat_resolved"));
}

#[test]
fn repair_and_normal_build_share_one_site_without_deadlock() {
    let mut w = world();
    let p = colony(&mut w, 1);
    let k = w.planeten[p].koord;
    w.kolonisation
        .integritaet
        .insert((p as PlanetId, Gebaeude::Farm), 300);
    w.kolonisation
        .integritaet
        .insert((p as PlanetId, Gebaeude::Solarkraftwerk), 300);
    w.reparieren(1, Rolle::Verwalter, k, Gebaeude::Farm)
        .unwrap();
    assert!(w
        .reparieren(1, Rolle::Verwalter, k, Gebaeude::Solarkraftwerk)
        .is_err());
    w.bauen(1, Rolle::Verwalter, k, Gebaeude::Erzmine).unwrap();
    assert!(w.planeten[p].bauschleife[0].fertig.is_none());
    w.zeit = w.kolonisation.reparaturen[&(p as PlanetId, Gebaeude::Farm)].fertig;
    w.kolonisation_fensterabschluss();
    assert!(w.planeten[p].bauschleife[0].fertig.is_some());
    assert!(w
        .reparieren(1, Rolle::Verwalter, k, Gebaeude::Solarkraftwerk)
        .is_err());
}

#[test]
fn colony_ship_casualty_aborts_old_clock_and_replacement_begins_fresh() {
    let mut w = world();
    let p = colony(&mut w, 1);
    occupy(&mut w, p);
    w.zeit = 900;
    w.kolonisation_fensterabschluss();
    fleet(
        &mut w,
        901,
        1,
        p,
        Mission::Angriff,
        Flottenzustand::Hinflug,
        10_000,
        0,
    );
    w.flotte_ankunft(901);
    assert!(!w.flotten.contains_key(&900));
    assert!(!w.kolonisation.besetzungen.contains_key(&(p as PlanetId)));
    assert!(w
        .fachereignisse
        .iter()
        .any(|e| e["event_type"] == "occupation_aborted"
            && e["payload"]["reason"] == "Kolonieschiffbestand im Gefecht gesunken"));
    for sid in [0, 1] {
        assert!(w.spieler[sid]
            .vorfaelle
            .iter()
            .any(|v| v.art == "kampfkolonisation" && v.text.contains("abgebrochen")));
    }
    fleet(
        &mut w,
        902,
        0,
        p,
        Mission::Kampfkolonisieren,
        Flottenzustand::ImOrbit,
        100,
        1,
    );
    w.planeten[p].blockade = Some(902);
    w.bombardement_abschluss(p, 902);
    let b = &w.kolonisation.besetzungen[&(p as PlanetId)];
    assert_eq!(b.seit, 900);
    assert_eq!(b.fruehestens, 2700);
    assert_eq!(b.reaktionsfenster, 0);
}

#[test]
fn both_colonization_missions_share_capacity_and_same_target_reinforcement_remains_possible() {
    for free_first in [false, true] {
        let mut w = world();
        w.spieler[0].forschung[Forschung::Astrophysik.idx()] = 1;
        let p = colony(&mut w, 1);
        let start = w.planeten[w.spieler[0].heimat as usize].koord;
        let free = Koord::neu(start.sektor, start.system, 7);
        w.spieler[0].erkundet.insert(
            free,
            Erkundung {
                zeit: 0,
                felder: 200,
                zone: Zone::Leben,
                reich_erz: 1000,
                reich_kristall: 1000,
                nebel: false,
            },
        );
        let mut fighting = order(&w, p, Mission::Kampfkolonisieren);
        fighting.schiffe = [0; SCHIFFE];
        fighting.schiffe[Einheit::Kolonieschiff.idx()] = 1;
        fighting.schiffe[Einheit::Kreuzer.idx()] = 1;
        let mut peaceful = fighting.clone();
        peaceful.ziel = free;
        peaceful.mission = Mission::Kolonisieren;
        peaceful.ladung = w.koloniefracht(0);
        peaceful.schiffe[Einheit::GrosserTransporter.idx()] = 10;
        let (first, second) = if free_first {
            (peaceful, fighting.clone())
        } else {
            (fighting.clone(), peaceful)
        };
        w.flotte_senden(0, Rolle::Feldherr, first).unwrap();
        let before = w.planeten[w.spieler[0].heimat as usize].bestand;
        assert!(w.flotte_senden(0, Rolle::Feldherr, second).is_err());
        assert_eq!(w.planeten[w.spieler[0].heimat as usize].bestand, before);
        if !free_first {
            assert!(w.flotte_senden(0, Rolle::Feldherr, fighting).is_ok());
        }
    }
}

#[test]
fn reinforcement_cannot_merge_after_target_becomes_allied_and_valid_merge_preserves_clock() {
    for mode in 0..3 {
        let mut w = world();
        let p = colony(&mut w, 1);
        occupy(&mut w, p);
        w.zeit = 900;
        w.kolonisation_fensterabschluss();
        fleet(
            &mut w,
            901,
            0,
            p,
            Mission::Kampfkolonisieren,
            Flottenzustand::Hinflug,
            10,
            1,
        );
        if mode == 1 {
            w.spieler[0].allianz = Some(7);
            w.spieler[1].allianz = Some(7);
        }
        if mode == 2 {
            w.planeten[p].besitzer = 2;
            w.spieler[2].schutz_bis = 10000;
        }
        w.flotte_ankunft(901);
        if mode > 0 {
            assert_eq!(w.flotten[&901].zustand, Flottenzustand::Rueckflug);
            assert_eq!(w.flotten[&900].schiffe[Einheit::Kolonieschiff.idx()], 1);
        } else {
            assert!(!w.flotten.contains_key(&901));
            let b = &w.kolonisation.besetzungen[&(p as PlanetId)];
            assert_eq!(b.seit, 0);
            assert_eq!(b.reaktionsfenster, 1);
            assert_eq!(w.flotten[&900].schiffe[Einheit::Kolonieschiff.idx()], 2);
        }
    }
}
