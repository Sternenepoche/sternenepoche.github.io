use kern::{welt::Spionagebericht,*};
use serde_json::json;
#[test]
fn simulator_uses_known_defender_faction_modifiers() {
    let mut r=Regelwerk::laden(include_str!("../../../regeln/online-v1.ron")).unwrap();
    r.voelker.get_mut(&Volk::Krath).unwrap().waffen=1000.0;
    r.voelker.get_mut(&Volk::Krath).unwrap().panzerung=1000.0;
    let mut w=Welt::neu(r,43,2).unwrap();
    w.spieler[0].volk=Volk::Aurelianer;
    let target=w.planeten[w.spieler[1].heimat as usize].koord;
    w.spieler[1].volk=Volk::Krath;
    let mut ships=vec![0;SCHIFFE];ships[Einheit::LeichterJaeger.idx()]=5;
    w.spieler[0].berichte.push(Spionagebericht{zeit:0,ziel:target,besitzer:1,bestand:vec![0;GUETER],schiffe:Some(ships),verteidigung:Some(vec![0;EINHEITEN-SCHIFFE]),gebaeude:None,forschung:None});
    let query=json!({"typ":"kampfsimulator","ziel":target.to_string(),"schiffe":{"leichter_jaeger":1000}});
    let strong=w.werkzeug(0,&query).unwrap();assert_eq!(strong["siegchance_prozent"],0);
    w.spieler[1].volk=Volk::Aurelianer;
    let normal=w.werkzeug(0,&query).unwrap();assert!(normal["siegchance_prozent"].as_u64().unwrap()>0);
}
