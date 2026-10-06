use kern::{typen::*, welt::*, Regelwerk};
use serde_json::json;
fn world() -> Welt {
    let mut w = Welt::neu(
        Regelwerk::laden(include_str!("../../../regeln/regelwerk.ron")).unwrap(),
        42,
        2,
    )
    .unwrap();
    w.kolonisationsregeln_aktivieren();
    for sid in 0..2 {
        let sp = &mut w.spieler[sid];
        sp.stufe = 4;
        sp.schutz_bis = 0;
        sp.forschung[Forschung::Astrophysik.idx()] = 5;
        sp.forschung[Forschung::Computertechnik.idx()] = 5;
        sp.toepfe = [1_000_000_000 * M; TOEPFE];
        let p = &mut w.planeten[sp.heimat as usize];
        p.bestand = [100_000 * M; GUETER];
        p.bevoelkerung = 20_000 * M;
        p.gebaeude[Gebaeude::Raumhafen.idx()] = 1;
        p.gebaeude[Gebaeude::Lager.idx()] = 8;
        p.einheiten[Einheit::Kolonieschiff.idx()] = 3;
        p.einheiten[Einheit::Kreuzer.idx()] = 100;
    }
    w
}
fn colony(w: &mut Welt) -> usize {
    let mut p = w.planeten[w.spieler[1].heimat as usize].clone();
    p.id = w.planeten.len() as PlanetId;
    p.heimat = false;
    p.koord.position = 8;
    p.einheiten = [0; EINHEITEN];
    p.gebaeude[Gebaeude::Erzmine.idx()] = 10;
    w.belegung.insert(p.koord, p.id);
    w.spieler[1].planeten.push(p.id);
    let id = p.id as usize;
    w.planeten.push(p);
    id
}
fn occupy(w: &mut Welt, pid: usize) -> FlottenId {
    let home = w.spieler[0].heimat;
    let mut ships = [0; SCHIFFE];
    ships[Einheit::Kolonieschiff.idx()] = 1;
    ships[Einheit::Kreuzer.idx()] = 100;
    let f = Flotte {
        id: 900,
        besitzer: 0,
        start: home,
        start_koord: w.planeten[home as usize].koord,
        ziel: w.planeten[pid].koord,
        mission: Mission::Kampfkolonisieren,
        schiffe: ships,
        ladung: [0; GUETER],
        siedler: 0,
        abflug: 0,
        ankunft: 0,
        flugdauer: 0,
        rueckkehr: 0,
        orbit_ende: 0,
        haltedauer: 0,
        zustand: Flottenzustand::ImOrbit,
        gemeldet: true,
        verband: None,
    };
    w.flotten.insert(f.id, f);
    w.planeten[pid].blockade = Some(900);
    w.bombardement_abschluss(pid, 900);
    900
}
#[test]
fn original_home_is_protected_but_colony_keeps_levels_and_two_reactions() {
    let mut w = world();
    let home = w.spieler[1].heimat as usize;
    w.planeten[home].einheiten = [0; EINHEITEN];
    occupy(&mut w, home);
    assert!(w.kolonisation.besetzungen.is_empty());
    w.flotten.clear();
    w.planeten[home].blockade = None;
    let p = colony(&mut w);
    let levels = w.planeten[p].gebaeude;
    occupy(&mut w, p);
    assert_eq!(w.kolonisation.besetzungen.len(), 1);
    w.schritt();
    assert_eq!(w.zeit, 900);
    assert_eq!(w.planeten[p].besitzer, 1);
    w.schritt();
    assert_eq!(w.zeit, 1800);
    assert_eq!(w.planeten[p].besitzer, 1);
    // Defender can still react at t=1800. Ownership changes only after that window closes.
    w.schritt();
    assert_eq!(w.planeten[p].besitzer, 0);
    assert_eq!(w.planeten[p].gebaeude, levels);
    assert_eq!(w.integritaet(p, Gebaeude::Erzmine), 300);
    assert_eq!(w.planeten[p].einheiten[Einheit::Kolonieschiff.idx()], 0);
}
#[test]
fn last_window_recall_or_rebuilt_shield_prevents_capture() {
    for rebuild in [false, true] {
        let mut w = world();
        let p = colony(&mut w);
        let f = occupy(&mut w, p);
        w.schritt();
        w.schritt();
        if rebuild {
            w.planeten[p].einheiten[EINHEITEN - 1] = 1;
        } else {
            let (ok, msg) = w.handeln(
                0,
                Rolle::Alle,
                &json!({"typ":"flotte_zurueckrufen","flotte":f}),
            );
            assert!(ok, "{msg}");
        }
        w.schritt();
        assert_eq!(w.planeten[p].besitzer, 1);
        assert!(w.kolonisation.besetzungen.is_empty());
    }
}
#[test]
fn scout_escort_and_supplies_are_required_before_any_spending() {
    let mut w = world();
    let start = w.planeten[w.spieler[0].heimat as usize].koord;
    let target = Koord::neu(start.sektor, start.system, 8);
    let mut ships = [0; SCHIFFE];
    ships[Einheit::Kolonieschiff.idx()] = 1;
    ships[Einheit::Kreuzer.idx()] = 1;
    let cargo = w.koloniefracht(0);
    assert!(w
        .koloniefracht_pruefen(0, target, &ships, &cargo)
        .unwrap_err()
        .contains("Spionagesonde"));
    let before = w.planeten[w.spieler[0].heimat as usize].clone();
    let result=w.handeln(0,Rolle::Alle,&json!({"typ":"flotte_senden","start":start.to_string(),"ziel":target.to_string(),"mission":"kolonisieren","schiffe":{"kolonieschiff":1,"kreuzer":1}}));
    assert!(!result.0);
    let after = &w.planeten[w.spieler[0].heimat as usize];
    assert_eq!(after.bestand, before.bestand);
    assert_eq!(after.einheiten, before.einheiten);
    assert_eq!(after.bevoelkerung, before.bevoelkerung);
    assert!(w.flotten.is_empty());
    w.spieler[0].erkundet.insert(
        target,
        Erkundung {
            zeit: 0,
            felder: 200,
            zone: Zone::Leben,
            reich_erz: 1000,
            reich_kristall: 1000,
            nebel: false,
        },
    );
    assert!(w
        .koloniefracht_pruefen(0, target, &ships, &[0; GUETER])
        .is_err());
    ships[Einheit::Kreuzer.idx()] = 0;
    assert!(w.koloniefracht_pruefen(0, target, &ships, &cargo).is_err());
    ships[Einheit::Kreuzer.idx()] = 1;
    assert!(w.koloniefracht_pruefen(0, target, &ships, &cargo).is_ok());
}
#[test]
fn shields_and_capture_limit_are_rechecked_and_repair_survives_checkpoint() {
    let mut w = world();
    let p = colony(&mut w);
    w.planeten[p].einheiten[EINHEITEN - 1] = 1;
    occupy(&mut w, p);
    assert!(w.kolonisation.besetzungen.is_empty());
    assert_eq!(w.integritaet(p, Gebaeude::Erzmine), 1000);
    w.planeten[p].einheiten[EINHEITEN - 1] = 0;
    w.bombardement_abschluss(p, 900);
    w.spieler[0].forschung[Forschung::Astrophysik.idx()] = 0;
    w.schritt();
    assert!(w.kolonisation.besetzungen.is_empty());
    w.planeten[p].blockade = None;
    w.flotten.clear();
    let k = w.planeten[p].koord;
    let before = w.planeten[p].bestand;
    w.reparieren(1, Rolle::Alle, k, Gebaeude::Solarkraftwerk)
        .unwrap();
    assert!(w.planeten[p]
        .bestand
        .iter()
        .zip(before)
        .any(|(a, b)| *a < b));
    let end = w.kolonisation.reparaturen[&(p as PlanetId, Gebaeude::Solarkraftwerk)].fertig;
    assert!(w
        .reparieren(1, Rolle::Alle, k, Gebaeude::Solarkraftwerk)
        .is_err());
    let mut restored = Welt::aus_bytes(&w.zu_bytes()).unwrap();
    assert_eq!(w.hash(), restored.hash());
    restored.zeit = end;
    restored.kolonisation_fensterabschluss();
    assert_eq!(restored.integritaet(p, Gebaeude::Solarkraftwerk), 1000);
}
#[test]
fn legacy_binary_remains_legacy_and_new_state_affects_hash() {
    let w = Welt::neu(
        Regelwerk::laden(include_str!("../../../regeln/regelwerk.ron")).unwrap(),
        5,
        2,
    )
    .unwrap();
    let bytes = bincode::serialize(&w).unwrap();
    assert_eq!(bytes, w.zu_bytes());
    assert_eq!(Welt::aus_bytes(&bytes).unwrap().hash(), w.hash());
    let mut next = w.clone();
    next.kolonisationsregeln_aktivieren();
    assert_ne!(next.hash(), w.hash());
    let bytes = next.zu_bytes();
    assert!(bytes.starts_with(b"STERNEP3"));
    assert_eq!(Welt::aus_bytes(&bytes).unwrap().hash(), next.hash());
}
#[test]
fn real_battle_destroys_every_shield_layer_and_starts_occupation() {
    let mut w = world();
    let p = colony(&mut w);
    let target = w.planeten[p].koord;
    let start = w.planeten[w.spieler[0].heimat as usize].koord;
    w.planeten[p].einheiten[Einheit::Planetenschild.idx()] = 3;
    w.planeten[p].einheiten[Einheit::Raketenwerfer.idx()] = 4;
    w.planeten[w.spieler[0].heimat as usize].einheiten[Einheit::Zerstoerer.idx()] = 1000;
    w.planeten[w.spieler[0].heimat as usize].bestand[Gut::Deuterium.idx()] = 1_000_000 * M;
    let (ok,msg)=w.handeln(0,Rolle::Alle,&json!({"typ":"flotte_senden","start":start.to_string(),"ziel":target.to_string(),"mission":"kampfkolonisieren","schiffe":{"kolonieschiff":1,"zerstoerer":1000}}));
    assert!(ok, "{msg}");
    let f = *w.flotten.keys().next().unwrap();
    w.zeit = w.flotten[&f].ankunft;
    w.flotte_ankunft(f);
    assert_eq!(w.planeten[p].einheiten[Einheit::Planetenschild.idx()], 0);
    assert!(w.planeten[p].einheiten.iter().all(|n| *n == 0));
    assert_eq!(w.integritaet(p, Gebaeude::Erzmine), 300);
    assert!(w.kolonisation.besetzungen.contains_key(&(p as PlanetId)));
}
#[test]
fn probe_then_supplied_escorted_expedition_creates_colony() {
    let mut w = world();
    let home = w.spieler[0].heimat as usize;
    let start = w.planeten[home].koord;
    let target = Koord::neu(start.sektor, start.system, 8);
    w.planeten[home].einheiten[Einheit::Spionagesonde.idx()] = 1;
    let (ok,msg)=w.handeln(0,Rolle::Alle,&json!({"typ":"flotte_senden","start":start.to_string(),"ziel":target.to_string(),"mission":"spionage","schiffe":{"spionagesonde":1}}));
    assert!(ok, "{msg}");
    let f = *w.flotten.keys().next().unwrap();
    w.zeit = w.flotten[&f].ankunft;
    w.flotte_ankunft(f);
    assert!(w.spieler[0].erkundet.contains_key(&target));
    let cargo = w.koloniefracht(0);
    let cargo: serde_json::Map<String, serde_json::Value> = Gut::ALLE
        .iter()
        .filter(|g| cargo[g.idx()] > 0)
        .map(|g| (g.name().into(), json!(cargo[g.idx()] as f64 / M as f64)))
        .collect();
    let (ok,msg)=w.handeln(0,Rolle::Alle,&json!({"typ":"flotte_senden","start":start.to_string(),"ziel":target.to_string(),"mission":"kolonisieren","schiffe":{"kolonieschiff":1,"kreuzer":1},"ladung":cargo}));
    assert!(ok, "{msg}");
    let f = w
        .flotten
        .values()
        .find(|f| f.mission == Mission::Kolonisieren)
        .unwrap()
        .id;
    w.zeit = w.flotten[&f].ankunft;
    w.flotte_ankunft(f);
    let p = &w.planeten[*w.belegung.get(&target).unwrap() as usize];
    assert_eq!(p.besitzer, 0);
    assert!(!p.heimat);
    assert_eq!(p.einheiten[Einheit::Kreuzer.idx()], 1);
    assert!(p.bestand[Gut::Nahrung.idx()] > 0);
}
