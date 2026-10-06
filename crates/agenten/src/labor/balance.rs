//! Bounded V4 acceptance measurements, not a long-match balance optimizer.
use super::{config::Config, world, Result};
use kern::{kampf::{kampf, Gruppe}, typen::*, welt::{strom, Sieger}, Welt};
use serde_json::{json, Value};
use std::collections::BTreeMap;

const COUNTS: [usize; 6] = [2, 3, 10, 11, 20, 50];
const SEEDS: [u64; 3] = [1, 42, 20261005];

fn check(id: &str, passed: bool, measurement: Value) -> Value {
    json!({"criterion":id,"passed":passed,"measurement":measurement})
}
fn home_signature(w: &Welt, sid: usize) -> Value {
    let s=&w.spieler[sid];let p=&w.planeten[s.heimat as usize];
    json!({"faction":s.volk,"credits":s.credits,"research":s.forschung.to_vec(),"stage":s.stufe,
        "budgets":s.toepfe,"population":p.bevoelkerung,"fields":p.felder,"factors":p.faktor,
        "inventory":p.bestand,"buildings":p.gebaeude.to_vec(),"units":p.einheiten.to_vec(),
        "rates":p.rate,"research_rate":p.fp_rate,"stability":p.stabilitaet,
        "energy":[p.energie_erzeugung,p.energie_verbrauch],"housing":p.wohnraum})
}
fn local_site_signature(w:&Welt,home:Koord)->Vec<(i64,u16,String)> {
    let mut sites=Vec::new();
    for position in 1..=w.regeln.welt.plaetze_je_system {
        let k=Koord::neu(home.sektor,home.system,position);
        if !w.belegung.contains_key(&k) {if let Some(p)=w.platz(k) {sites.push((w.entfernung(home,k),p.felder,p.zone.name().into()));}}
    }sites.sort();sites
}
fn map_case(n:usize,seed:u64)->Result<Value> {
    let mut c=Config::v4(n);c.seed=seed;let (w,manifest)=world::create(&c)?;
    let sig=home_signature(&w,0);
    let equal=(0..n).all(|sid|home_signature(&w,sid)==sig);
    let homes=w.planeten.iter().all(|p|w.belegung.get(&p.koord)==Some(&p.id) && w.system(p.koord).is_some() && w.platz(p.koord).is_some()) && w.belegung.len()==n;
    let free=w.plaetze.len()-w.belegung.len();let capacity=w.regeln.wirtschaft.kolonien_max as usize*n;
    let mut travel=Vec::new();
    for sid in 0..n {
        let home=w.planeten[w.spieler[sid].heimat as usize].koord;
        let distances=(0..n).filter(|s|*s!=sid).map(|s|w.entfernung(home,w.planeten[w.spieler[s].heimat as usize].koord)).collect::<Vec<_>>();
        let nearest=*distances.iter().min().ok_or("Kein Nachbar")?;
        let reference= [Einheit::Spionagesonde,Einheit::Kreuzer,Einheit::Kolonieschiff].iter().map(|ship| {
            let t=w.flugdauer(nearest,w.tempo(sid as u16,*ship),1000);
            (ship.name(),json!({"seconds":t,"windows":t as f64/w.regeln.fenster() as f64}))
        }).collect::<BTreeMap<_,_>>();
        let mut zones=BTreeMap::<String,i64>::new();
        for (distance,_,zone) in local_site_signature(&w,home) {zones.entry(zone).and_modify(|d|*d=(*d).min(distance)).or_insert(distance);}
        travel.push(json!({"player":sid,"home":home.to_string(),"nearest_distance":nearest,
            "nearest_neighbors":distances.iter().filter(|d|**d==nearest).count(),"ships":reference,"nearest_local_free_zone_distance":zones}));
    }
    let mut pairs=Vec::new();let mut triples=Vec::new();let mut pair_ok=true;let mut whole_pair_symmetry=true;
    for group in manifest["starts"].as_array().ok_or("Startgruppen fehlen")? {
        let members=group["members"].as_array().ok_or("Startmitglieder fehlen")?;
        let ids=members.iter().map(|m|m["player"].as_u64().map(|n|n as usize).ok_or_else(||"Spieler-ID fehlt".to_string())).collect::<Result<Vec<_>>>()?;
        if ids.len()==2 {
            let a=w.planeten[w.spieler[ids[0]].heimat as usize].koord;let b=w.planeten[w.spieler[ids[1]].heimat as usize].koord;
            let target=Koord::neu(a.sektor,a.system,6);let peak=w.platz(target).ok_or("Umkämpfter Platz fehlt")?;
            let da=w.entfernung(a,target);let db=w.entfernung(b,target);
            let local=da==db && !w.belegung.contains_key(&target) && peak.felder==w.regeln.zonen[&peak.zone].felder_max;
            let whole=local_site_signature(&w,a)==local_site_signature(&w,b);pair_ok &= local;whole_pair_symmetry &= whole;
            pairs.push(json!({"players":ids,"contested":target.to_string(),"fields":peak.felder,"distance":[da,db],"local_contest_equal":local,"entire_local_territory_symmetric":whole}));
        } else if ids.len()==3 {
            let pressures=ids.iter().map(|sid|travel[*sid]["nearest_neighbors"].as_u64().unwrap()).collect::<Vec<_>>();
            triples.push(json!({"players":ids,"nearest_neighbor_counts":pressures,"single_seat_equal_pressure":pressures.windows(2).all(|x|x[0]==x[1]),
                "interpretation":"Middle seat has two equally near rivals, outer seats one. Seed shuffling changes identities but does not prove equal single-run seats."}));
        } else {return Err("Startgruppe hat unerwartete Größe".into());}
    }
    let same_nearest=travel.iter().all(|row|row["nearest_distance"]==travel[0]["nearest_distance"]);
    let reaction=travel.iter().all(|row|row["ships"].as_object().unwrap().values().all(|ship|ship["seconds"].as_i64().unwrap()>w.regeln.fenster()));
    let checks=vec![check("v4_rules",w.kolonisationsregeln_v2(),json!(w.kolonisation.version)),
        check("identical_start_economy",equal,sig),check("valid_unique_home_sites",homes,json!(w.belegung.len())),
        check("equal_nearest_rival_distance",same_nearest,json!(travel[0]["nearest_distance"])),
        check("paired_contested_site",pair_ok,json!({"pairs":pairs.len()})),
        check("reference_flight_exceeds_one_decision_window",reaction,json!({"window_seconds":w.regeln.fenster(),"minimum_flight_seconds":w.regeln.flug.min_sekunden})),
        check("expansion_is_possible_but_scarce",free>=n && free<capacity,json!({"free_sites":free,"free_per_player":free as f64/n as f64,"maximum_total_colonies":capacity,"site_to_max_colony_ratio":free as f64/capacity as f64}))];
    let hard_pass=checks.iter().all(|v|v["passed"]==true);
    Ok(json!({"players":n,"seed":seed,"world_hash":w.hash(),"rules_hash":w.regeln.hash,"systems":w.systeme.len(),
        "passed":hard_pass,"checks":checks,"travel":travel,"pairs":pairs,"triples":triples,
        "full_single_seat_territorial_fairness":{"passed":whole_pair_symmetry && triples.is_empty(),"required_for_local_acceptance":false,
            "reason":"Identical starting economy and one equal contested site do not imply identical zone/boundary access; odd triples also have unequal rival pressure."}}))
}
fn economy_threshold()->Result<Value> {
    let (mut w,_)=world::create(&Config::v4(2))?;let sid=0usize;let pid=w.spieler[sid].heimat as usize;
    let start=w.planeten[pid].koord;let target=Koord::neu(start.sektor,start.system,6);
    let sp=&mut w.spieler[sid];sp.stufe=4;sp.forschung[Forschung::Astrophysik.idx()]=3;sp.forschung[Forschung::Computertechnik.idx()]=3;sp.toepfe=[1_000_000*M;TOEPFE];
    let p=&mut w.planeten[pid];p.bevoelkerung=10000*M;p.bestand=[100000*M;GUETER];p.gebaeude[Gebaeude::Raumhafen.idx()]=1;
    for (e,n) in [(Einheit::Kolonieschiff,1),(Einheit::Kreuzer,1),(Einheit::GrosserTransporter,20)] {p.einheiten[e.idx()]=n;}
    let site=w.platz(target).ok_or("Kolonieziel fehlt")?.clone();let sys=w.system(target).ok_or("Koloniesystem fehlt")?.clone();
    w.spieler[sid].erkundet.insert(target,kern::welt::Erkundung{zeit:0,felder:site.felder,zone:site.zone,reich_erz:sys.reich_erz,reich_kristall:sys.reich_kristall,nebel:sys.nebel});
    let mut cargo=w.koloniefracht(sid as u16);cargo[Gut::Erz.idx()]+=2000*M;cargo[Gut::Kristall.idx()]+=2000*M;
    let cargo=Gut::ALLE.iter().filter(|g|cargo[g.idx()]>0).map(|g|(g.name(),cargo[g.idx()] as f64/M as f64)).collect::<BTreeMap<_,_>>();
    let mut query=json!({"typ":"kolonieplan","start":start.to_string(),"ziel":target.to_string(),"schiffe":{
        "kolonieschiff":1,"kreuzer":1,"grosser_transporter":20},"ladung":cargo,"aufbau":[]});
    let hash=w.hash();let bare=w.kolonieplan(sid as u16,&query)?;
    query["aufbau"]=json!(["solarkraftwerk","farm","farm"]);let built=w.kolonieplan(sid as u16,&query)?;
    let passed=bare["start_moeglich"]==true && bare["versorgung_im_modell_gedeckt"]==false
        && built["aufbau_vollstaendig"]==true && built["grundversorgung_im_modell_gedeckt"]==true
        && built["versorgung_im_modell_gedeckt"]==false && hash==w.hash();
    Ok(json!({"criterion":"founding_is_not_self_sufficient_economy","passed":passed,"bare_colony":bare,"solar_and_two_farms":built,
        "scope":"Chosen static build sequence, population/research fixed. Food/energy coverage is not consumption-goods coverage or a long-run growth guarantee."}))
}
fn combat_threshold()->Result<Value> {
    let (w,_)=world::create(&Config::v4(2))?;let mut defense=[0;EINHEITEN];defense[Einheit::Planetenschild.idx()]=1;
    let d=Gruppe::neu(&w.regeln,&w.spieler[1],defense);let mut rows=Vec::new();let mut passed=true;
    for seed in SEEDS {for count in [50,100,150,250,500] {
        let mut ships=[0;EINHEITEN];ships[Einheit::Kreuzer.idx()]=count;let a=Gruppe::neu(&w.regeln,&w.spieler[0],ships);
        let result=kampf(&w.regeln,&[a.clone()],&[d.clone()],&mut strom(seed,700,0));
        if count==50 {passed &= result.sieger==Sieger::Unentschieden && result.verteidiger[0][Einheit::Planetenschild.idx()]==1;}
        if count==500 {passed &= result.sieger==Sieger::Angreifer && result.verteidiger[0][Einheit::Planetenschild.idx()]==0;}
        let ship_cost=w.regeln.wert(&w.regeln.kosten_einheit(Einheit::Kreuzer,w.spieler[0].volk))*count;
        let shield_cost=w.regeln.wert(&w.regeln.kosten_einheit(Einheit::Planetenschild,w.spieler[1].volk));
        rows.push(json!({"seed":seed,"cruisers":count,"winner":result.sieger,"rounds":result.runden,
            "surviving_cruisers":result.angreifer[0][Einheit::Kreuzer.idx()],"surviving_shields":result.verteidiger[0][Einheit::Planetenschild.idx()],
            "attack_per_round":a.angriff[Einheit::Kreuzer.idx()]*count,"shield_regeneration_per_round":d.schild[Einheit::Planetenschild.idx()],
            "attacker_cost":ship_cost,"defender_cost":shield_cost,"cost_ratio":ship_cost as f64/shield_cost as f64}));
    }}
    Ok(json!({"criterion":"shield_regeneration_requires_force_threshold","passed":passed,"rows":rows,
        "scope":"Three deterministic combat streams; no repair recovery, economy replenishment, mixed fleets or researched weapons. Counts bracket a threshold, not an optimal army or exact global minimum."}))
}
/// Pure deterministic report: no provider calls, worker starts, filesystem writes or full matches.
pub fn report()->Result<Value> {
    let mut maps=Vec::new();for n in COUNTS {for seed in SEEDS {maps.push(map_case(n,seed)?);}}
    let tactical=vec![economy_threshold()?,combat_threshold()?];
    let passed=maps.iter().all(|m|m["passed"]==true) && tactical.iter().all(|t|t["passed"]==true);
    Ok(json!({"format":"labor-balance-v1","labor_version":4,"status":if passed{"PASS_WITH_LIMITS"}else{"FAIL"},"passed":passed,
        "player_counts":COUNTS,"seeds":SEEDS,"map_cases":maps,"tactical_cases":tactical,
        "criteria_rationale":["Exact home equality isolates model decisions from starting economic luck.",
            "One equidistant valuable free site tests the paired local contest, not all territorial opportunities.",
            "Reference travel longer than 900s permits at least one decision phase before arrival.",
            "At least one free site per player allows expansion; fewer sites than aggregate colony caps forces competition."],
        "limits":["Full single-run seat fairness is not passed: pairs have unequal boundary/zone geometry; triples add unequal neighbor pressure.",
            "Seat identity shuffling over three seeds is not a statistical fairness proof. Research comparisons need complete seat rotation.",
            "Uniform Aurelianer terrain is a controlled benchmark, not faction balance.",
            "No long matches, empirical win rates, model-quality ranking, GPU use or balance optimization."]}))
}
