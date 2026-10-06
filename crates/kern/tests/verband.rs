//! Allianzangriff: Zustimmung, gemeinsame Zeit, getrennte Kampfwerte und ein Beutepool.
use kern::{
    flotte::Flugauftrag,
    kampf::{kampf, Gruppe},
    welt::{strom, Flottenzustand, KANAL_KAMPF},
    *,
};

fn welt() -> Welt {
    let mut w = Welt::neu(
        Regelwerk::laden(include_str!("../../../regeln/regelwerk.ron")).unwrap(),
        42,
        3,
    )
    .unwrap();
    for sid in 0..3 {
        w.spieler[sid].schutz_bis = 0;
        w.spieler[sid].volk = Volk::Aurelianer;
        w.spieler[sid].stufe = 4;
        w.spieler[sid].forschung[Forschung::Computertechnik.idx()] = 10;
        let p = w.spieler[sid].heimat as usize;
        w.planeten[p].gebaeude[Gebaeude::Raumhafen.idx()] = 1;
        w.planeten[p].gebaeude[Gebaeude::Lager.idx()] = 12;
        w.planeten[p].bestand = [100_000 * M; GUETER];
        w.planeten[p].einheiten[Einheit::KleinerTransporter.idx()] = 100;
        w.planeten[p].einheiten[Einheit::Kreuzer.idx()] = 20;
        w.raten_neu(p);
    }
    w.spieler[0].allianz = Some(1);
    w.spieler[1].allianz = Some(1);
    let target = w.spieler[2].heimat as usize;
    w.planeten[target].einheiten = [0; EINHEITEN];
    w
}
fn senden(w: &mut Welt, sid: SpielerId, typ: Einheit, n: i64) -> FlottenId {
    let pid = w.spieler[sid as usize].heimat as usize;
    let target = w.spieler[2].heimat as usize;
    let mut schiffe = [0; SCHIFFE];
    schiffe[typ.idx()] = n;
    let id = w.naechste_flotte;
    w.flotte_senden(
        sid,
        Rolle::Alle,
        Flugauftrag {
            start: w.planeten[pid].koord,
            ziel: w.planeten[target].koord,
            mission: Mission::Angriff,
            schiffe,
            sigma_pm: 1000,
            ladung: [0; GUETER],
            haltedauer: 0,
        },
    )
    .unwrap();
    id
}
#[test]
fn beitritt_braucht_eigentuemer_zustimmung_allianz_und_ziel() {
    let mut w = welt();
    let a = senden(&mut w, 0, Einheit::KleinerTransporter, 2);
    let b = senden(&mut w, 1, Einheit::KleinerTransporter, 2);
    assert!(w.verband_beitreten(1, b, a).is_err());
    assert!(w.verband_oeffnen(1, a).is_err());
    w.verband_oeffnen(0, a).unwrap();
    assert!(w.verband_beitreten(0, b, a).is_err());
    w.spieler[1].allianz = None;
    assert!(w.verband_beitreten(1, b, a).is_err());
    w.spieler[1].allianz = Some(1);
    let ziel = w.flotten[&b].ziel;
    w.flotten.get_mut(&b).unwrap().ziel.position += 1;
    assert!(w.verband_beitreten(1, b, a).is_err());
    w.flotten.get_mut(&b).unwrap().ziel = ziel;
    let arrival = w.flotten[&a].ankunft.max(w.flotten[&b].ankunft);
    w.verband_beitreten(1, b, a).unwrap();
    assert_eq!(w.flotten[&a].ankunft, arrival);
    assert_eq!(w.flotten[&b].ankunft, arrival);
}
#[test]
fn keine_teleportation_oder_unbegrenzte_verzoegerung() {
    let mut w = welt();
    let a = senden(&mut w, 0, Einheit::KleinerTransporter, 2);
    let b = senden(&mut w, 1, Einheit::KleinerTransporter, 2);
    w.verband_oeffnen(0, a).unwrap();
    let original = w.flotten[&a].ankunft;
    w.flotten.get_mut(&b).unwrap().ankunft = original + 6 * STUNDE + 1;
    assert!(w.verband_beitreten(1, b, a).is_err());
    assert_eq!(w.flotten[&a].ankunft, original);
    assert_eq!(w.flotten[&b].verband, None);
}
#[test]
fn ein_gefecht_ein_beutepool_und_alte_ereignisse_wirkungslos() {
    let mut w = welt();
    let a = senden(&mut w, 0, Einheit::KleinerTransporter, 100);
    let b = senden(&mut w, 1, Einheit::KleinerTransporter, 100);
    let alt = w.flotten[&a].ankunft;
    w.verband_oeffnen(0, a).unwrap();
    w.verband_beitreten(1, b, a).unwrap();
    let ankunft = w.flotten[&a].ankunft;
    if alt < ankunft {
        w.zeit = alt;
        w.flotte_ankunft(a);
        assert!(w.kampfberichte.is_empty());
    }
    w.zeit = ankunft;
    let zp = w.spieler[2].heimat as usize;
    w.abrechnen(zp);
    let vorher = w.planeten[zp].bestand;
    w.flotte_ankunft(b);
    w.flotte_ankunft(a);
    w.flotte_ankunft(b);
    assert_eq!(w.kampfberichte.len(), 1);
    let bericht = &w.kampfberichte[0];
    assert_eq!(bericht.angreifer_gruppen.len(), 2);
    let quote = w
        .regeln
        .volk(Volk::Aurelianer)
        .pluenderquote
        .unwrap_or(w.regeln.kampf.pluenderquote);
    for g in 0..GUETER {
        let beute = w.flotten[&a].ladung[g] + w.flotten[&b].ladung[g];
        assert_eq!(beute, bericht.beute[g]);
        assert_eq!(vorher[g] - w.planeten[zp].bestand[g], beute);
        assert!(beute <= mal(vorher[g], quote));
    }
    for id in [a, b] {
        let f = &w.flotten[&id];
        assert_eq!(f.zustand, Flottenzustand::Rueckflug);
        let cap = w
            .flugplan(f.besitzer, f.start as usize, f.ziel, &f.schiffe, 1000)
            .unwrap()
            .kapazitaet;
        assert!(f.ladung.iter().sum::<i64>() <= cap);
    }
    assert_eq!(w.planeten[zp].mali.len(), 1);
}
#[test]
fn jeder_teilnehmer_kaempft_mit_eigener_forschung() {
    let mut w = welt();
    w.spieler[1].forschung[Forschung::Waffentechnik.idx()] = 15;
    w.spieler[1].forschung[Forschung::Panzerung.idx()] = 12;
    let a = senden(&mut w, 0, Einheit::Kreuzer, 4);
    let b = senden(&mut w, 1, Einheit::Kreuzer, 4);
    w.verband_oeffnen(0, a).unwrap();
    w.verband_beitreten(1, b, a).unwrap();
    let zp = w.spieler[2].heimat as usize;
    w.planeten[zp].einheiten[Einheit::Kreuzer.idx()] = 9;
    let groups: Vec<_> = [a, b]
        .iter()
        .map(|id| {
            let f = &w.flotten[id];
            let mut e = [0; EINHEITEN];
            e[..SCHIFFE].copy_from_slice(&f.schiffe);
            Gruppe::neu(&w.regeln, &w.spieler[f.besitzer as usize], e)
        })
        .collect();
    let defender = Gruppe::neu(&w.regeln, &w.spieler[2], w.planeten[zp].einheiten);
    let mut rng = strom(w.startwert, KANAL_KAMPF, w.kampf_nr);
    let expected = kampf(&w.regeln, &groups, &[defender], &mut rng);
    w.zeit = w.flotten[&a].ankunft;
    w.flotte_ankunft(a);
    let r = &w.kampfberichte[0];
    assert_eq!(r.angreifer_gruppen[0].nachher, expected.angreifer[0]);
    assert_eq!(r.angreifer_gruppen[1].nachher, expected.angreifer[1]);
    for g in &r.angreifer_gruppen {
        if let Some(f) = w.flotten.get(&g.flotte) {
            assert_eq!(f.schiffe.as_slice(), &g.nachher[..SCHIFFE]);
        } else {
            assert_eq!(g.nachher.iter().sum::<i64>(), 0);
        }
    }
}
#[test]
fn rueckruf_entfernt_nur_eigene_flottenteilnahme() {
    let mut w = welt();
    let a = senden(&mut w, 0, Einheit::KleinerTransporter, 2);
    let b = senden(&mut w, 1, Einheit::KleinerTransporter, 2);
    w.verband_oeffnen(0, a).unwrap();
    w.verband_beitreten(1, b, a).unwrap();
    let arrival = w.flotten[&b].ankunft;
    w.zeit = MINUTE;
    w.flotte_zurueckrufen(0, a).unwrap();
    assert_eq!(w.flotten[&a].verband, None);
    w.zeit = arrival;
    w.flotte_ankunft(a);
    assert!(w.kampfberichte.is_empty());
    w.flotte_ankunft(b);
    assert_eq!(w.kampfberichte.len(), 1);
    assert_eq!(w.kampfberichte[0].angreifer_gruppen.len(), 1);
    assert_eq!(w.kampfberichte[0].angreifer_gruppen[0].spieler, 1);
}
