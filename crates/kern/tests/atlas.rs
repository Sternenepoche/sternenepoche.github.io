use kern::*;
use serde_json::json;

fn world() -> Welt {
    let r=Regelwerk::laden(include_str!("../../../regeln/online-v1.ron")).unwrap();
    let mut w=Welt::neu(r,987,8).unwrap();
    for (i,s) in w.spieler.iter_mut().enumerate(){s.name=format!("P{i}");s.ki=false;}
    w
}
fn home(w:&Welt,s:usize)->Koord{w.planeten[w.spieler[s].heimat as usize].koord}

#[test]
fn map_colors_distinguish_ownership_pacts_foreign_alliances_and_actual_war(){
    let mut w=world();
    assert_eq!(w.beziehung(0,0)["karte_status"],"eigen");
    assert_eq!(w.beziehung(0,1)["karte_farbe"],"#ffffff");
    w.allianz_gruenden_mit_tag(0,"Nordbund",Some("NORD")).unwrap();
    w.allianz_gruenden_mit_tag(2,"Südbund",Some("SUED")).unwrap();
    assert_eq!(w.beziehung(0,2)["status"],"neutral","red does not declare a war");
    assert_eq!(w.beziehung(0,2)["karte_status"],"fremde_allianz");
    assert_eq!(w.beziehung(0,2)["karte_farbe"],"#ff6470");
    w.allianz_einladen(0,"P1").unwrap();w.allianz_beitreten(1,"Nordbund").unwrap();
    assert_eq!(w.beziehung(0,1)["karte_status"],"verbuendet");
    assert_ne!(w.beziehung(0,0)["karte_farbe"],w.beziehung(0,1)["karte_farbe"]);
    w.allianz_anfrage(0,"Südbund","nichtangriffspakt","NAP").unwrap();
    w.diplomatie_entscheiden(2,1,true).unwrap();
    assert_eq!(w.beziehung(1,2)["karte_farbe"],"#f3d365");
    w.allianz_anfrage(0,"Südbund","verteidigungsbuendnis","Bündnis").unwrap();
    w.diplomatie_entscheiden(2,2,true).unwrap();
    assert_eq!(w.beziehung(1,2)["karte_status"],"verbuendet");
    w.allianz_anfrage(0,"Südbund","krieg","Krieg").unwrap();
    assert_eq!(w.beziehung(1,2)["karte_status"],"feindlich");
    assert_eq!(w.beziehung(1,2)["karte_farbe"],"#ff6470");
}

#[test]
fn alliance_tags_are_unique_persisted_and_allow_members_of_every_house(){
    let mut w=world();w.spieler[0].volk=Volk::Veyari;w.spieler[1].volk=Volk::Krath;
    assert!(w.handeln(0,Rolle::Diplomat,&json!({"typ":"allianz_gruenden","name":"Sternenbund","tag":"nova"})).0);
    assert!(w.allianz_beitreten(1,"Sternenbund").is_err());
    w.allianz_einladen(0,"P1").unwrap();w.allianz_beitreten(1,"Sternenbund").unwrap();
    assert_eq!(w.beziehung(0,1)["allianz_tag"],"NOVA");
    let old=w.zu_bytes();let hash=w.hash();
    assert!(w.allianz_gruenden_mit_tag(2,"Andere",Some("nova")).is_err());
    assert!(w.allianz_gruenden_mit_tag(2,"Andere",Some("<x>")).is_err());
    assert_eq!(old,w.zu_bytes(),"invalid tag must not partially create an alliance");
    let loaded=Welt::aus_bytes(&old).unwrap();assert_eq!(hash,loaded.hash());
    let directory=loaded.allianz_verzeichnis(0);
    assert_eq!(directory[0]["tag"],"NOVA");
    assert_eq!(directory[0]["voelker"],json!(["krath","veyari"]));
    assert!(directory[0].get("planeten").is_none());
    // Reading older JSON worlds supplies stable tags without rewriting the saved state.
    w.kommunikation.allianz_tags.clear();let old=w.zu_bytes();
    let loaded=Welt::aus_bytes(&old).unwrap();
    assert_eq!(loaded.allianz_tag(1),"STER1");assert_eq!(loaded.zu_bytes(),old);
}

#[test]
fn alliance_system_shares_only_residents_and_revokes_access_after_leaving(){
    let mut w=world();let ally=home(&w,1);let pid=w.spieler[2].heimat as usize;
    let old=w.planeten[pid].koord;
    let position=(1..=w.regeln.welt.plaetze_je_system).find(|p|!w.belegung.contains_key(&Koord::neu(ally.sektor,ally.system,*p))).unwrap();
    let foreign=Koord::neu(ally.sektor,ally.system,position);
    w.belegung.remove(&old);w.belegung.insert(foreign,pid as u32);w.planeten[pid].koord=foreign;
    assert!(w.planetenwissen(0,ally)["spieler"].is_null());
    assert!(w.planetenwissen(0,foreign)["spieler"].is_null());
    w.allianz_gruenden(0,"A").unwrap();w.allianz_einladen(0,"P1").unwrap();w.allianz_beitreten(1,"A").unwrap();
    for (k,name) in [(ally,"P1"),(foreign,"P2")] {
        let p=w.planetenwissen(0,k);
        assert_eq!(p["bekannt"],false);assert_eq!(p["spieler"],name);
        assert_eq!(p["besiedlung_bekannt"],true);
        for secret in ["zone","felder","bestand_spannen","ertrag_promille","heimat"]{assert!(p[secret].is_null(),"{secret} leaked");}
        assert!(w.planetenwissen(3,k)["spieler"].is_null());
        assert!(w.atlas_sicht(0).iter().any(|p|p["koord"]==k.to_string()&&p["spieler"]==name));
    }
    assert!(!w.system_erfasst(0,ally),"sharing residents is not a system probe");
    let bytes=w.zu_bytes();let loaded=Welt::aus_bytes(&bytes).unwrap();assert_eq!(w.atlas_sicht(0),loaded.atlas_sicht(0));
    w.allianz_verlassen(1).unwrap();
    assert!(w.planetenwissen(0,foreign)["spieler"].is_null());
    assert!(!w.atlas_sicht(0).iter().any(|p|p["koord"]==foreign.to_string()));
    assert_eq!(w.planetenwissen(0,home(&w,0))["beziehung"]["karte_status"],"eigen");
}
