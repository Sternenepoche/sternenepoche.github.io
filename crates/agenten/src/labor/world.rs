use super::{config::Config, Result};
use kern::{Koord, Volk, Welt};
use serde_json::{json, Value};

/// Versioned benchmark map; legacy Welt::neu is unchanged.
pub fn create(c: &Config) -> Result<(Welt, Value)> {
    c.validate()?;
    let n = c.players.len();
    // One shared system per pair makes the eight-colony cap compete for actual sites.
    // V2/V3 retain their larger uniform maps for reproducible historical runs.
    let total = if c.version >= 4 {
        (n / 2).max(1)
    } else {
        (n * 12).div_ceil(5).max(6)
    };
    let sectors = total.div_ceil(60);
    let per = total.div_ceil(sectors);
    let mut rules = kern::Regelwerk::laden(crate::RULES)?;
    rules.welt.sektoren = sectors as u8;
    rules.welt.systeme_je_sektor = per as u8;
    rules.welt.epoche_tage = c.days;
    rules.hash = crate::journal::hash(&serde_json::to_vec(&rules).map_err(|e| e.to_string())?);
    let mut w = if c.version >= 4 {
        // Legacy constructor requires one start system per player. Bootstrap its player records,
        // then install the compact, paired map before exposing any world state.
        let mut bootstrap = rules.clone();
        bootstrap.welt.sektoren = 1;
        bootstrap.welt.systeme_je_sektor = (n * 3 + 6).min(255) as u8;
        let mut w = Welt::neu(bootstrap, c.seed, n)?;
        w.systeme.truncate(total);
        w.plaetze
            .truncate(total * rules.welt.plaetze_je_system as usize);
        w.regeln = std::sync::Arc::new(rules);
        w
    } else {
        Welt::neu(rules, c.seed, n)?
    };
    if c.version >= 4 {
        w.kolonisationsregeln_v2_aktivieren();
    } else if c.version >= 3 {
        w.kolonisationsregeln_aktivieren();
    }
    // Uniform benchmark terrain eliminates resource luck as a confound. Exploratory random maps
    // remain available in the legacy runner, but are not silently mixed into these comparisons.
    for sys in &mut w.systeme {
        sys.reich_erz = 1000;
        sys.reich_kristall = 1000;
        // Equal access to late-game xenocrystals; no faction is locked out of research.
        sys.nebel = c.version >= 4;
        sys.guertel = true;
    }
    for p in &mut w.plaetze {
        let r = &w.regeln.zonen[&p.zone];
        p.felder = (r.felder_min + r.felder_max) / 2;
    }
    if c.version >= 4 {
        for (index, p) in w.plaetze.iter_mut().enumerate() {
            // Position six is contested equally by starts at five and seven.
            if index % w.regeln.welt.plaetze_je_system as usize == 5 {
                p.felder = w.regeln.zonen[&p.zone].felder_max;
            }
        }
    }
    let mut ids: Vec<u16> = (0..n as u16).collect();
    kern::welt::mischen(&mut kern::welt::strom(c.seed, 91, 0), &mut ids);
    let groups = n / 2;
    let mut starts = Vec::new();
    let mut cursor = 0;
    w.belegung.clear();
    for g in 0..groups {
        let sector = g % sectors;
        let count = (groups + sectors - 1 - sector) / sectors;
        let index = g / sectors;
        let system = 1 + (index * per + per / 2) / count;
        let size = if n % 2 == 1 && g + 1 == groups { 3 } else { 2 };
        let positions = if size == 3 { vec![4, 6, 8] } else { vec![5, 7] };
        let mut members = Vec::new();
        for position in positions {
            let sid = ids[cursor];
            cursor += 1;
            let coord = Koord::neu(sector as u8 + 1, system as u8, position);
            let pid = w.spieler[sid as usize].heimat;
            w.planeten[pid as usize].koord = coord;
            w.spieler[sid as usize].volk = Volk::Aurelianer;
            w.belegung.insert(coord, pid);
            members.push(json!({"player":sid,"coordinate":coord.to_string()}));
        }
        starts.push(json!({"group":g,"members":members}));
    }
    for pid in 0..n {
        w.raten_neu(pid);
    }
    w.punkte_neu();
    w.fenster_vorbereiten();
    let mut nearest = Vec::new();
    for sid in 0..n {
        let a = w.planeten[w.spieler[sid].heimat as usize].koord;
        let distance = (0..n)
            .filter(|i| *i != sid)
            .map(|i| w.entfernung(a, w.planeten[w.spieler[i].heimat as usize].koord))
            .min()
            .unwrap();
        let ship = kern::Einheit::ALLE
            .iter()
            .copied()
            .find(|e| e.ist_schiff())
            .unwrap();
        nearest.push(
            json!({"player":sid,"nearest_distance":distance,"reference_ship":ship.name(),
            "reference_travel_seconds":w.flugdauer(distance,w.tempo(sid as u16,ship),1000)}),
        );
    }
    let manifest = json!({"generator":if c.version>=4{"paired-contested-v2"}else{"paired-uniform-v1"},"seed":c.seed,"systems":per*sectors,
        "sectors":sectors,"systems_per_sector":per,"starts":starts,"travel":nearest,
        "faction":"aurelianer","odd_group_note":if n%2==1 {Some("Dreiergruppe: gleicher nächster Abstand, mittlerer Sitz hat zwei direkte Nachbarn; Sitze über Seeds rotieren.")}else{None},
        "world_hash":w.hash()});
    Ok((w, manifest))
}
