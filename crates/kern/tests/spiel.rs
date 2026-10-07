//! Tests des Spielkerns: Formeln aus dem Konzept, Erhaltung von Mengen, die Wege
//! Plünderung, Blockade und Eroberung, Verträge, Markt und Schnappschüsse.

use kern::typen::*;
use kern::welt::*;
use kern::Regelwerk;
use serde_json::{json, Value};

const REGELN: &str = include_str!("../../../regeln/regelwerk.ron");

fn welt(spieler: usize) -> Welt {
    Welt::neu(Regelwerk::laden(REGELN).expect("Regelwerk"), 42, spieler).expect("Welt")
}

fn tu_fn(w: &mut Welt, sid: SpielerId, a: Value) -> String {
    let (ok, text) = w.handeln(sid, Rolle::Alle, &a);
    assert!(ok, "abgelehnt: {text} bei {a}");
    text
}

fn abgelehnt_fn(w: &mut Welt, sid: SpielerId, a: Value) -> String {
    let (ok, text) = w.handeln(sid, Rolle::Alle, &a);
    assert!(!ok, "unerwartet angenommen: {a}");
    text
}

// Die Aktion wird erst gebaut (sie liest oft aus der Welt), dann angewandt.
macro_rules! tu {
    ($w:ident, $sid:expr, $a:expr) => {{
        let a = $a;
        tu_fn(&mut $w, $sid, a)
    }};
}
macro_rules! abgelehnt {
    ($w:ident, $sid:expr, $a:expr) => {{
        let a = $a;
        abgelehnt_fn(&mut $w, $sid, a)
    }};
}

fn vor(w: &mut Welt, sekunden: i64) {
    let ziel = w.zeit + sekunden;
    while w.zeit < ziel {
        assert!(w.schritt());
    }
}

fn heimat(w: &Welt, sid: SpielerId) -> usize {
    w.spieler[sid as usize].heimat as usize
}

fn koord(w: &Welt, sid: SpielerId) -> String {
    w.planeten[heimat(w, sid)].koord.to_string()
}

/// Macht einen Spieler raumfahrtfähig, ohne die Aufbauphase zu spielen.
fn ruesten(w: &mut Welt, sid: SpielerId) {
    let pid = heimat(w, sid);
    let sp = &mut w.spieler[sid as usize];
    sp.stufe = 4;
    sp.schutz_bis = 0;
    sp.credits = 1_000_000 * M;
    sp.forschung[Forschung::Computertechnik.idx()] = 5;
    sp.forschung[Forschung::Astrophysik.idx()] = 3;
    let p = &mut w.planeten[pid];
    p.gebaeude[Gebaeude::Raumhafen.idx()] = 1;
    p.gebaeude[Gebaeude::Werft.idx()] = 4;
    p.gebaeude[Gebaeude::Lager.idx()] = 8;
    p.gebaeude[Gebaeude::Markt.idx()] = 2;
    p.gebaeude[Gebaeude::Wohnblock.idx()] = 14;
    p.bevoelkerung = 20_000 * M;
    for g in [Gut::Erz, Gut::Kristall, Gut::Deuterium] {
        p.bestand[g.idx()] = 100_000 * M;
    }
    w.raten_neu(pid);
}

fn gib(w: &mut Welt, sid: SpielerId, e: Einheit, n: i64) {
    let pid = heimat(w, sid);
    w.planeten[pid].einheiten[e.idx()] += n;
}

fn summe(w: &Welt, gut: Gut) -> i64 {
    w.planeten.iter().map(|p| p.bestand[gut.idx()]).sum::<i64>() + w.flotten.values().map(|f| f.ladung[gut.idx()]).sum::<i64>()
}

#[test]
fn flugzeiten_wie_im_konzept() {
    let w = welt(4);
    // Kleiner Transporter mit Tempo 10.000: ins Nachbarsystem rund 40 Minuten, in den anderen Sektor knapp zwei Stunden.
    let nachbar = w.flugdauer(w.entfernung(Koord::neu(1, 27, 7), Koord::neu(1, 28, 7)), 10_000, 1000);
    assert!((38 * 60..=42 * 60).contains(&nachbar), "{nachbar}");
    let sektor = w.flugdauer(w.entfernung(Koord::neu(1, 27, 7), Koord::neu(2, 27, 7)), 10_000, 1000);
    assert!((100 * 60..120 * 60).contains(&sektor), "{sektor}");
    // Die kürzeste Flugzeit liegt über der Fensterbreite: jeder erkannte Angriff lässt eine Reaktion zu.
    let kurz = w.flugdauer(w.entfernung(Koord::neu(1, 27, 7), Koord::neu(1, 27, 8)), 1_000_000, 1000);
    assert_eq!(kurz, w.regeln.flug.min_sekunden);
    assert!(kurz > w.regeln.fenster());
}

#[test]
fn startplaetze_sind_fair() {
    let w = welt(50);
    let heimaten: Vec<Koord> = w.spieler.iter().map(|s| w.planeten[s.heimat as usize].koord).collect();
    for (i, a) in heimaten.iter().enumerate() {
        assert!(!w.system(*a).unwrap().nebel);
        for b in &heimaten[i + 1..] {
            if a.sektor == b.sektor {
                assert!((a.system as i32 - b.system as i32).abs() >= w.regeln.welt.start_abstand as i32, "{a} und {b}");
            }
        }
    }
    // Abstand zum nächsten Nebel weicht zwischen den Spielern um höchstens 20 Prozent ab.
    let naechster_nebel = |k: Koord| {
        w.systeme.iter().filter(|s| s.nebel && s.sektor == k.sektor).map(|s| w.entfernung(k, Koord::neu(s.sektor, s.nummer, 6))).min().unwrap()
    };
    let d: Vec<i64> = heimaten.iter().map(|k| naechster_nebel(*k)).collect();
    let (min, max) = (*d.iter().min().unwrap(), *d.iter().max().unwrap());
    assert!(max * 100 <= min * 120, "Nebelabstand {min} bis {max}");
    // Völker per Los in möglichst gleich großen Gruppen: 13, 13, 12, 12.
    let mut gruppen: Vec<usize> = Volk::ALLE.iter().map(|v| w.spieler.iter().filter(|s| s.volk == *v).count()).collect();
    gruppen.sort();
    assert_eq!(gruppen, vec![12, 12, 13, 13]);
    assert_eq!(w.systeme.iter().filter(|s| s.nebel).count(), 12);
}

#[test]
fn bestaende_werden_nie_negativ_und_schnappschuss_ist_exakt() {
    let mut w = welt(6);
    tu!(w, 0, json!({"typ": "bauen", "planet": koord(&w, 0), "gebaeude": "erzmine"}));
    vor(&mut w, 3 * TAG);
    for p in &w.planeten {
        assert!(p.bestand.iter().all(|b| *b >= 0));
        assert!(p.bevoelkerung > 0);
    }
    // Schnappschuss laden und beide Welten weiterrechnen: gleicher Hash.
    let mut kopie = Welt::aus_bytes(&w.zu_bytes()).unwrap();
    assert_eq!(kopie.hash(), w.hash());
    vor(&mut w, TAG);
    vor(&mut kopie, TAG);
    assert_eq!(kopie.hash(), w.hash());
}

#[test]
fn stufenaufstieg_braucht_48_stunden_ohne_unterbrechung() {
    let mut w = welt(4);
    let pid = heimat(&w, 0);
    let p = &mut w.planeten[pid];
    p.gebaeude[Gebaeude::Erzmine.idx()] = 5;
    p.gebaeude[Gebaeude::Kristallmine.idx()] = 5;
    p.gebaeude[Gebaeude::Solarkraftwerk.idx()] = 8;
    p.gebaeude[Gebaeude::Farm.idx()] = 3;
    p.gebaeude[Gebaeude::Wohnblock.idx()] = 5;
    p.gebaeude[Gebaeude::Raumhafen.idx()] = 1;
    p.bevoelkerung = 2_500 * M;
    p.bestand[Gut::Erz.idx()] = 9_000 * M;
    p.bestand[Gut::Kristall.idx()] = 9_000 * M;
    w.raten_neu(pid);
    vor(&mut w, 24 * STUNDE);
    let grund = abgelehnt!(w, 0, json!({"typ": "stufenaufstieg"}));
    assert!(grund.contains("von 48 Stunden"), "{grund}");
    vor(&mut w, 25 * STUNDE);
    tu!(w, 0, json!({"typ": "stufenaufstieg"}));
    assert_eq!(w.spieler[0].stufe, 2);
    // Neue Gebäude sind frei, ältere Sperren bleiben genau begründet.
    tu!(w, 0, json!({"typ": "bauen", "planet": koord(&w, 0), "gebaeude": "giesserei"}));
    let grund = abgelehnt!(w, 0, json!({"typ": "bauen", "planet": koord(&w, 0), "gebaeude": "markt"}));
    assert!(grund.contains("Zivilisationsstufe 3"), "{grund}");
}

#[test]
fn syntheten_hungern_ohne_strom() {
    let mut w = welt(8);
    let sid = w.spieler.iter().find(|s| s.volk == Volk::Syntheten).unwrap().id;
    let pid = heimat(&w, sid);
    w.planeten[pid].gebaeude[Gebaeude::Solarkraftwerk.idx()] = 0;
    w.raten_neu(pid);
    let vorher = w.planeten[pid].bevoelkerung;
    vor(&mut w, 2 * TAG);
    assert!(w.planeten[pid].bevoelkerung < vorher, "ohne Strom schrumpft die Bevölkerung der Syntheten");
    assert_eq!(w.planeten[pid].rate[Gut::Nahrung.idx()].min(0), 0, "Syntheten verbrauchen keine Nahrung");
}

#[test]
fn transport_erhaelt_mengen_und_wird_als_geschenk_gezaehlt() {
    let mut w = welt(4);
    ruesten(&mut w, 0);
    gib(&mut w, 0, Einheit::GrosserTransporter, 2);
    for pid in 0..w.planeten.len() {
        w.raten_neu(pid);
        w.planeten[pid].rate = [0; GUETER];
    }
    let kristall = summe(&w, Gut::Kristall);
    let text = tu!(w, 0, json!({"typ": "flotte_senden", "start": koord(&w, 0), "ziel": koord(&w, 1), "mission": "transport",
        "schiffe": {"grosser_transporter": 2}, "ladung": {"kristall": 30000}}));
    assert!(text.contains("gestartet"));
    assert_eq!(summe(&w, Gut::Kristall), kristall, "Ladung wechselt nur den Ort");
    let dauer = w.flotten.values().next().unwrap().flugdauer;
    // Zwischen zwei Ticks rechnen die Planeten mit Rate 0: die Summe bleibt exakt.
    while w.flotten.values().any(|f| f.zustand == Flottenzustand::Hinflug) {
        for p in w.planeten.iter_mut() {
            p.rate[Gut::Kristall.idx()] = 0;
        }
        vor(&mut w, 900);
    }
    assert!(w.zeit >= dauer);
    assert_eq!(w.spieler[0].statistik.geschenkt, w.spieler[1].statistik.erhalten);
    assert!(w.spieler[1].statistik.erhalten >= 45_000 * M);
}

#[test]
fn angriff_pluendert_nach_beutequote_und_bricht_den_pakt() {
    let mut w = welt(4);
    ruesten(&mut w, 0);
    ruesten(&mut w, 1);
    gib(&mut w, 0, Einheit::LeichterJaeger, 40);
    gib(&mut w, 0, Einheit::GrosserTransporter, 20);
    let (a, b) = (w.spieler[0].name.clone(), w.spieler[1].name.clone());
    tu!(w, 0, json!({"typ": "vertrag_anbieten", "partner": b, "art": "nichtangriffspakt", "kaution": 500}));
    tu!(w, 1, json!({"typ": "vertrag_annehmen", "vertrag": 1}));
    assert!(w.vertrag_zwischen(0, 1, Vertragsart::Nichtangriffspakt));
    let credits_opfer = w.spieler[1].credits;
    let ziel = heimat(&w, 1);
    w.planeten[ziel].gebaeude[Gebaeude::Bunker.idx()] = 2;
    let schutz = w.regeln.bunkerschutz(2)[Gut::Erz.idx()];
    tu!(w, 0, json!({"typ": "flotte_senden", "start": koord(&w, 0), "ziel": koord(&w, 1), "mission": "angriff",
        "schiffe": {"leichter_jaeger": 40, "grosser_transporter": 20}}));
    // Das Opfer sieht die Flotte erst im Sensorbereich, dann aber sicher vor der Ankunft.
    let ankunft = w.flotten.values().next().unwrap().ankunft;
    while w.zeit + w.regeln.fenster() < ankunft {
        vor(&mut w, 900);
    }
    assert_eq!(w.sichtbare_angriffe(1).len(), 1);
    let erz_vorher = w.bestand_jetzt(ziel)[Gut::Erz.idx()];
    vor(&mut w, 900);
    let bericht = w.kampfberichte.last().expect("Kampfbericht");
    assert_eq!(bericht.sieger, Sieger::Angreifer);
    let beute = bericht.beute[Gut::Erz.idx()];
    let quote = w.regeln.volk(w.spieler[0].volk).pluenderquote.unwrap_or(w.regeln.kampf.pluenderquote);
    let erwartet = mal(erz_vorher - schutz, quote);
    assert!((beute - erwartet).abs() < 300 * M, "Beute {beute}, erwartet etwa {erwartet}");
    // Der Bruch steht im Register, beide Kautionen gehen an das Opfer.
    assert!(!w.vertrag_zwischen(0, 1, Vertragsart::Nichtangriffspakt));
    assert!(w.register.iter().any(|e| e.vorgang.contains("gebrochen") && e.vorgang.contains(&a)));
    assert!(w.spieler[1].credits >= credits_opfer + 1000 * M);
    assert_eq!(w.spieler[0].statistik.vertragsbrueche, 1);
    assert_eq!(w.planeten[ziel].mali.len(), 1);
    // Beide Seiten lesen den Bericht wie im Spiel: Namen, Ausgang, Verluste und Beute je Typ.
    // Das Lagebild des Orchestrators (lagebild.py, _kampf) liest genau diese Felder.
    for (sid, seite) in [(0, "angriff"), (1, "verteidigung")] {
        let k = &w.sicht(sid, Rolle::Feldherr)["kampfberichte"][0];
        assert_eq!(k["seite"], seite);
        assert_eq!(k["angreifer"], json!([a]));
        assert_eq!(k["verteidiger"], json!([b]));
        assert_eq!(k["sieger"], "angreifer");
        assert_eq!(k["mission"], "angriff");
        assert!(k["runden"].is_u64(), "ein wehrloses Ziel ergibt 0 Runden");
        assert!(k["verluste_angreifer"].is_object() && k["verluste_verteidiger"].is_object());
        assert!(k["beute"]["erz"].as_i64().unwrap() > 0);
        assert!(k["truemmer"]["erz"].is_i64() && k["zeit"].as_str().unwrap().starts_with("Tag "));
    }
    assert!(w.sicht(2, Rolle::Feldherr)["kampfberichte"].as_array().unwrap().is_empty(), "Unbeteiligte sehen nichts");
}

#[test]
fn anfaengerschutz_und_rollenrechte() {
    let mut w = welt(4);
    ruesten(&mut w, 0);
    gib(&mut w, 0, Einheit::LeichterJaeger, 5);
    let grund = abgelehnt!(w, 0, json!({"typ": "flotte_senden", "start": koord(&w, 0), "ziel": koord(&w, 1), "mission": "angriff",
        "schiffe": {"leichter_jaeger": 5}}));
    assert!(grund.contains("Anfängerschutz"), "{grund}");
    // Der Verwalter darf nicht angreifen und keine Kampfschiffe bestellen, der Feldherr nicht bauen.
    let (ok, text) = w.handeln(0, Rolle::Verwalter, &json!({"typ": "flotte_senden", "start": koord(&w, 0), "ziel": koord(&w, 1),
        "mission": "angriff", "schiffe": {"leichter_jaeger": 5}}));
    assert!(!ok && text.contains("feldherr"), "{text}");
    let (ok, text) = w.handeln(0, Rolle::Feldherr, &json!({"typ": "bauen", "planet": koord(&w, 0), "gebaeude": "erzmine"}));
    assert!(!ok && text.contains("verwalter"), "{text}");
    // Der Topf begrenzt die Ausgaben einer Rolle, auch wenn die Güter da sind.
    w.spieler[0].toepfe[Topf::Militaer.idx()] = 0;
    let (ok, text) = w.handeln(0, Rolle::Feldherr, &json!({"typ": "fertigen", "planet": koord(&w, 0), "einheit": "leichter_jaeger", "anzahl": 3}));
    assert!(!ok && text.contains("Topf militaer"), "{text}");
}

#[test]
fn kolonie_blockade_und_eroberung() {
    let mut w = welt(4);
    ruesten(&mut w, 0);
    ruesten(&mut w, 1);
    gib(&mut w, 0, Einheit::Kolonieschiff, 1);
    let h0 = w.planeten[heimat(&w, 0)].koord;
    let ziel = Koord::neu(h0.sektor, h0.system, 8);
    let bev = w.planeten[heimat(&w, 0)].bevoelkerung;
    tu!(w, 0, json!({"typ": "flotte_senden", "start": h0.to_string(), "ziel": ziel.to_string(), "mission": "kolonisieren",
        "schiffe": {"kolonieschiff": 1}, "ladung": {"erz": 4000}}));
    // Siedler gehen an Bord und fehlen der Heimatwelt: Kolonisierung vervielfacht keine Menschen.
    let siedler = w.regeln.wirtschaft.siedler * M;
    assert_eq!(w.planeten[heimat(&w, 0)].bevoelkerung, bev - siedler);
    vor(&mut w, 2 * STUNDE);
    let kolonie = *w.belegung.get(&ziel).expect("Kolonie gegründet") as usize;
    assert_eq!(w.spieler[0].planeten.len(), 2);
    // Die Kolonie hat noch keine Farm: ohne mitgebrachte Nahrung hungert sie vom ersten Tag an.
    let bev_kolonie = w.planeten[kolonie].bevoelkerung;
    assert!(bev_kolonie > mal(siedler, 0.97) && bev_kolonie < siedler + 300 * M, "{bev_kolonie}");
    assert_eq!(w.planeten[kolonie].bestand[Gut::Erz.idx()], 4000 * M);
    w.planeten[kolonie].gebaeude[Gebaeude::Erzmine.idx()] = 10;

    // Heimatwelten lassen sich nicht erobern, Kolonien schon.
    gib(&mut w, 1, Einheit::Kreuzer, 10);
    gib(&mut w, 1, Einheit::Truppentransporter, 3);
    let schiffe = json!({"kreuzer": 10, "truppentransporter": 3});
    let grund = abgelehnt!(w, 1, json!({"typ": "flotte_senden", "start": koord(&w, 1), "ziel": h0.to_string(), "mission": "invasion", "schiffe": schiffe}));
    assert!(grund.contains("Heimatwelten"), "{grund}");
    tu!(w, 1, json!({"typ": "flotte_senden", "start": koord(&w, 1), "ziel": ziel.to_string(), "mission": "invasion", "schiffe": schiffe}));
    vor(&mut w, 6 * STUNDE);
    assert!(w.planeten[kolonie].blockade.is_some(), "die Flotte hält den Orbit");
    // Während der Blockade startet von dort nur ein Angriff auf die Blockadeflotte.
    w.planeten[kolonie].gebaeude[Gebaeude::Raumhafen.idx()] = 1;
    w.planeten[kolonie].einheiten[Einheit::KleinerTransporter.idx()] = 1;
    w.planeten[kolonie].bestand[Gut::Deuterium.idx()] = 1000 * M;
    let grund = abgelehnt!(w, 0, json!({"typ": "flotte_senden", "start": ziel.to_string(), "ziel": h0.to_string(), "mission": "transport",
        "schiffe": {"kleiner_transporter": 1}}));
    assert!(grund.contains("blockiert"), "{grund}");
    // Die Belagerung senkt die Stabilität über Tage, dann wechselt die Kolonie den Besitzer.
    let mut tage = 0;
    while w.planeten[kolonie].besitzer == 0 && tage < 20 {
        vor(&mut w, TAG);
        tage += 1;
    }
    assert_eq!(w.planeten[kolonie].besitzer, 1, "nach {tage} Tagen nicht erobert");
    assert!(tage >= 3, "eine Belagerung dauert mehrere Tage, hier {tage}");
    assert_eq!(w.planeten[kolonie].gebaeude[Gebaeude::Erzmine.idx()], 7, "30 Prozent Stufenverlust");
    assert_eq!(w.spieler[0].planeten.len(), 1);
    assert_eq!(w.spieler[1].planeten.len(), 2);
    assert!(w.planeten[kolonie].einheiten[Einheit::Kreuzer.idx()] > 0, "die Flotte ist auf der Kolonie stationiert");
}

#[test]
fn verbuendete_halten_und_verteidigen_mit() {
    let mut w = welt(4);
    for sid in 0..3 {
        ruesten(&mut w, sid);
    }
    let (n1, n2) = (w.spieler[1].name.clone(), w.spieler[2].name.clone());
    tu!(w, 1, json!({"typ": "allianz_gruenden", "name": "Nordbund"}));
    tu!(w, 1, json!({"typ": "allianz_einladen", "spieler": n2}));
    tu!(w, 2, json!({"typ": "allianz_beitreten", "allianz": "nordbund"}));
    assert!(w.verbuendet(1, 2));
    tu!(w, 1, json!({"typ": "nachricht", "allianz": true, "text": "Ich schicke Kreuzer."}));
    gib(&mut w, 2, Einheit::Kreuzer, 20);
    tu!(w, 2, json!({"typ": "flotte_senden", "start": koord(&w, 2), "ziel": koord(&w, 1), "mission": "halten",
        "schiffe": {"kreuzer": 20}, "haltedauer_stunden": 48}));
    vor(&mut w, 4 * STUNDE);
    // Ein Angreifer trifft jetzt auf die haltende Flotte des Verbündeten.
    gib(&mut w, 0, Einheit::LeichterJaeger, 30);
    tu!(w, 0, json!({"typ": "flotte_senden", "start": koord(&w, 0), "ziel": koord(&w, 1), "mission": "angriff",
        "schiffe": {"leichter_jaeger": 30}}));
    vor(&mut w, 4 * STUNDE);
    let bericht = w.kampfberichte.last().expect("Kampfbericht");
    assert_eq!(bericht.sieger, Sieger::Verteidiger);
    assert_eq!(bericht.verteidiger, vec![1, 2]);
    assert!(w.spieler[0].statistik.verluste > 0);
    assert!(!w.truemmer.is_empty(), "zerstörte Schiffe hinterlassen ein Trümmerfeld");
    // Wer ein Mitglied der eigenen Allianz angreift, fliegt hinaus.
    let _ = n1;
    gib(&mut w, 2, Einheit::LeichterJaeger, 5);
    tu!(w, 2, json!({"typ": "flotte_senden", "start": koord(&w, 2), "ziel": koord(&w, 1), "mission": "angriff", "schiffe": {"leichter_jaeger": 5}}));
    vor(&mut w, 4 * STUNDE);
    assert!(w.spieler[2].allianz.is_none());
}

#[test]
fn markt_tribut_und_recycling() {
    let mut w = welt(4);
    ruesten(&mut w, 0);
    ruesten(&mut w, 1);
    let credits = w.spieler[0].credits + w.spieler[1].credits;
    tu!(w, 0, json!({"typ": "markt_order", "planet": koord(&w, 0), "gut": "kristall", "seite": "verkauf", "menge": 5000, "preis": 2.0}));
    let ziel = heimat(&w, 1);
    let vorher = w.bestand_jetzt(ziel)[Gut::Kristall.idx()];
    let text = tu!(w, 1, json!({"typ": "markt_order", "planet": koord(&w, 1), "gut": "kristall", "seite": "kauf", "menge": 3000, "preis": 2.5}));
    assert!(text.contains("3000 kristall sofort gehandelt"), "{text}");
    // Gehandelt wird zum Preis der älteren Order; die Gebühr beider Seiten verschwindet aus dem Spiel.
    let wert = 6000 * M;
    let geb = |sid: SpielerId| mal(wert, w.gebuehr(sid, None));
    assert_eq!(w.spieler[0].credits + w.spieler[1].credits, credits - geb(0) - geb(1));
    assert_eq!(w.orders.len(), 1, "der Rest der Verkaufsorder steht im Buch");
    vor(&mut w, 3 * STUNDE);
    assert!(w.bestand_jetzt(ziel)[Gut::Kristall.idx()] >= vorher + 3000 * M, "die Handelsflotte hat geliefert");
    tu!(w, 0, json!({"typ": "markt_storno", "order": 1}));

    // Tribut: täglich, und wer nicht zahlen kann, bricht den Vertrag.
    let b = w.spieler[1].name.clone();
    tu!(w, 0, json!({"typ": "vertrag_anbieten", "partner": b, "art": "tribut", "kaution": 0, "tribut_menge": 400, "tribut_tage": 5}));
    let id = w.vertraege.last().unwrap().id;
    tu!(w, 1, json!({"typ": "vertrag_annehmen", "vertrag": id}));
    let c1 = w.spieler[1].credits;
    vor(&mut w, TAG);
    assert!(w.spieler[1].credits >= c1 + 400 * M);
    w.spieler[0].credits = 0;
    w.spieler[0].steuersatz = 0;
    vor(&mut w, TAG);
    assert_eq!(w.vertraege.last().unwrap().status, Vertragsstatus::Gebrochen);

    // Recycler sammeln ein Trümmerfeld ein.
    let h0 = w.planeten[heimat(&w, 0)].koord;
    let feld = Koord::neu(h0.sektor, h0.system, 2);
    w.truemmer.insert(feld, [30_000 * M, 10_000 * M]);
    gib(&mut w, 0, Einheit::Recycler, 1);
    tu!(w, 0, json!({"typ": "flotte_senden", "start": h0.to_string(), "ziel": feld.to_string(), "mission": "recyceln", "schiffe": {"recycler": 1}}));
    vor(&mut w, 3 * STUNDE);
    let rest = w.truemmer.get(&feld).copied().unwrap_or([0, 0]);
    let fasst = mal(20_000 * M, w.regeln.volk(w.spieler[0].volk).ladung);
    assert_eq!(rest[0] + rest[1], 40_000 * M - fasst, "ein Recycler fasst 20.000 mal Ladefaktor des Volks");
}

#[test]
fn abbau_spionage_und_kampfsimulator() {
    let mut w = welt(4);
    ruesten(&mut w, 0);
    ruesten(&mut w, 1);
    let h0 = w.planeten[heimat(&w, 0)].koord;
    let guertel = w.systeme.iter().filter(|s| s.guertel && s.sektor == h0.sektor).min_by_key(|s| (s.nummer as i32 - h0.system as i32).abs()).unwrap().nummer;
    gib(&mut w, 0, Einheit::Bergbauschiff, 3);
    gib(&mut w, 0, Einheit::Spionagesonde, 6);
    tu!(w, 0, json!({"typ": "flotte_senden", "start": h0.to_string(), "ziel": format!("{}:{}:0", h0.sektor, guertel), "mission": "abbau",
        "schiffe": {"bergbauschiff": 3}, "haltedauer_stunden": 10}));
    tu!(w, 0, json!({"typ": "flotte_senden", "start": h0.to_string(), "ziel": koord(&w, 1), "mission": "spionage", "schiffe": {"spionagesonde": 4}}));
    // Erkundung eines freien Platzes zeigt seine Werte.
    let frei = Koord::neu(h0.sektor, h0.system, 3);
    tu!(w, 0, json!({"typ": "flotte_senden", "start": h0.to_string(), "ziel": frei.to_string(), "mission": "spionage", "schiffe": {"spionagesonde": 1}}));
    vor(&mut w, TAG);
    assert!(w.spieler[0].erkundet.contains_key(&frei));
    let bericht = w.spieler[0].berichte.last().expect("Spionagebericht");
    assert!(bericht.schiffe.is_some() && bericht.verteidigung.is_some());
    assert!(w.spieler[1].vorfaelle.iter().any(|v| v.art == "spionage"), "das Ziel bemerkt den Versuch");
    assert!(w.spieler[0].vorfaelle.iter().any(|v| v.art == "flotte" && v.text.contains("abbau") && v.text.contains("erz")), "Abbau bringt Erz heim");
    // Der Simulator rechnet gegen den Bericht, nicht gegen die Wahrheit, und ist wiederholbar.
    let abfrage = json!({"typ": "kampfsimulator", "ziel": koord(&w, 1), "schiffe": {"leichter_jaeger": 10}});
    let a = w.werkzeug(0, &abfrage).unwrap();
    let b = w.werkzeug(0, &abfrage).unwrap();
    assert_eq!(a, b);
    assert_eq!(a["siegchance_prozent"], 100);
    assert!(w.werkzeug(0, &json!({"typ": "kampfsimulator", "ziel": koord(&w, 2), "schiffe": {}})).is_err());
    let flug = w.werkzeug(0, &json!({"typ": "flugzeit", "start": h0.to_string(), "ziel": koord(&w, 1), "schiffe": {"leichter_jaeger": 3}, "geschwindigkeit": 0.5})).unwrap();
    assert!(flug["dauer_min"].as_i64().unwrap() >= 20);
    let regel = w.werkzeug(0, &json!({"typ": "regel", "stichwort": "blockade"})).unwrap();
    assert!(regel["text"].as_str().unwrap().contains("Blockade"));
}

#[test]
fn unterhalt_und_desertion() {
    let mut w = welt(4);
    ruesten(&mut w, 0);
    gib(&mut w, 0, Einheit::Kreuzer, 100);
    w.spieler[0].credits = 0;
    w.spieler[0].steuersatz = 0;
    vor(&mut w, 5 * TAG);
    let rest = w.planeten[heimat(&w, 0)].einheiten[Einheit::Kreuzer.idx()];
    assert!(rest < 100 && rest >= 70, "nach drei unbezahlten Tagen desertieren täglich zehn Prozent, übrig {rest}");
}

#[test]
fn wecker_und_ereignisse_fruehestens_nach_halbem_takt() {
    let mut w = welt(4);
    w.fenster_vorbereiten();
    let jetzt = w.zeit;
    // Ein Wecker auf eine Stunde wird auf den halben Takt des Strategen verschoben.
    w.aufruf_ende(0, Rolle::Stratege, None, Some(STUNDE), &[]);
    let halb = w.regeln.agenten.takt_stunden[&Rolle::Stratege] * STUNDE / 2;
    assert_eq!(w.spieler[0].weck[Rolle::Stratege.idx()].wecker, Some(jetzt + halb));
    // Ein nicht dringendes Ereignis weckt ihn vorher nicht, ein dringendes sofort.
    let faellig = |w: &Welt| w.faellig.iter().any(|f| f.spieler == 0 && f.rolle == Rolle::Stratege);
    w.wecke(0, Rolle::Stratege, "Meldung vom verwalter", false);
    w.fenster_vorbereiten();
    assert!(!faellig(&w));
    w.wecke(0, Rolle::Stratege, "Blockade", true);
    w.fenster_vorbereiten();
    assert!(faellig(&w));
}

#[test]
fn feldherr_zahlt_bei_warnung_auch_aus_der_reserve() {
    let mut w = welt(4);
    ruesten(&mut w, 0);
    ruesten(&mut w, 1);
    gib(&mut w, 0, Einheit::LeichterJaeger, 40);
    let fertigen = json!({"typ": "fertigen", "planet": koord(&w, 1), "einheit": "raketenwerfer", "anzahl": 5});
    // Militärtopf leer, Reserve gefüllt: ohne Warnung zahlt die Reserve nicht.
    w.spieler[1].toepfe[Topf::Militaer.idx()] = 0;
    w.spieler[1].toepfe[Topf::Reserve.idx()] = 1_000_000 * M;
    let (ok, text) = w.handeln(1, Rolle::Feldherr, &fertigen);
    assert!(!ok && text.contains("Topf"), "ohne Warnung: {text}");
    tu!(w, 0, json!({"typ": "flotte_senden", "start": koord(&w, 0), "ziel": koord(&w, 1), "mission": "angriff", "schiffe": {"leichter_jaeger": 40}}));
    let ankunft = w.flotten.values().next().unwrap().ankunft;
    while w.zeit + w.regeln.fenster() < ankunft {
        vor(&mut w, 900);
    }
    assert!(w.warnung_aktiv(1));
    // Das Einkommen füllt den Militärtopf weiter; für den Nachweis wieder leeren.
    w.spieler[1].toepfe[Topf::Militaer.idx()] = 0;
    let reserve = w.spieler[1].toepfe[Topf::Reserve.idx()];
    let (ok, text) = w.handeln(1, Rolle::Feldherr, &fertigen);
    assert!(ok, "bei sichtbarer feindlicher Flotte zahlt die Reserve mit: {text}");
    assert!(w.spieler[1].toepfe[Topf::Reserve.idx()] < reserve);
    // Andere Rollen bekommen die Ausnahme nicht.
    w.spieler[1].toepfe[Topf::Wirtschaft.idx()] = 0;
    let (ok, _) = w.handeln(1, Rolle::Verwalter, &json!({"typ": "bauen", "planet": koord(&w, 1), "gebaeude": "erzmine"}));
    assert!(!ok);
}

/// Junge Reiche sehen die Voraussetzungen; erreichte Schritte werden abgehakt.
#[test]
fn kolonie_weg_ab_spielbeginn() {
    let mut w = welt(4);
    let frueh = w.sicht(0, Rolle::Alle)["kolonie_weg"].as_array().cloned().unwrap();
    assert_eq!(frueh.len(), 8);
    assert_eq!(frueh[0]["erledigt"], false);
    w.spieler[0].stufe = 3;
    let weg = w.sicht(0, Rolle::Alle)["kolonie_weg"].as_array().cloned().unwrap();
    assert_eq!(weg.len(), 8);
    assert_eq!(weg[0]["erledigt"], false);
    w.spieler[0].stufe = 4;
    assert_eq!(w.sicht(0, Rolle::Alle)["kolonie_weg"][0]["erledigt"], true);
}
