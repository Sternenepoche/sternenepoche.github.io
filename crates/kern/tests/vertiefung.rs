//! Regressions for the interactive barrier, accounting and adversarial quantities.
use kern::{Gebaeude, Gut, Regelwerk, Rolle, Topf, Vertragsart, Welt, M, TAG};
use kern::welt::Vertragsstatus;
use serde_json::{json, Value};

const RULES: &str = include_str!("../../../regeln/regelwerk.ron");

fn world() -> Welt {
    Welt::neu(Regelwerk::laden(RULES).unwrap(), 42, 4).unwrap()
}

#[test]
fn unrest_preserves_exact_remaining_build_time_and_charges_only_once() {
    let mut rules = Regelwerk::laden(RULES).unwrap();
    rules.welt.fenster_sekunden = 1;
    let mut w = Welt::neu(rules, 42, 4).unwrap();
    // Isolate building time from hourly demographics/production.
    w.ereignisse.clear();
    let pid = w.spieler[0].heimat as usize;
    let coord = w.planeten[pid].koord;
    let before_level = w.planeten[pid].gebaeude[Gebaeude::Erzmine.idx()];
    let costs = w.regeln.kosten_gebaeude(Gebaeude::Erzmine, before_level + 1);
    w.planeten[pid].bestand = [1_000_000 * M; kern::GUETER];
    w.planeten[pid].rate = [0; kern::GUETER];
    w.spieler[0].toepfe[Topf::Wirtschaft.idx()] = 1_000_000_000;
    let stock_before = w.planeten[pid].bestand;
    let pot_before = w.spieler[0].toepfe[Topf::Wirtschaft.idx()];
    w.bauen(0, Rolle::Verwalter, coord, Gebaeude::Erzmine).unwrap();
    let finish = w.planeten[pid].bauschleife[0].fertig.unwrap();
    assert!(finish > 2);
    for _ in 0..finish / 3 { assert!(w.schritt()); }
    let remaining = finish - w.zeit;
    w.planeten[pid].stabilitaet = 0;
    // Cross the stale original finish event while paused.
    for _ in 0..finish + 17 { assert!(w.schritt()); }
    assert_eq!(w.planeten[pid].gebaeude[Gebaeude::Erzmine.idx()], before_level);
    assert_eq!(w.planeten[pid].bauschleife[0].fertig.unwrap() - w.zeit, remaining);
    w.planeten[pid].stabilitaet = 100 * M;
    for _ in 1..remaining { assert!(w.schritt()); }
    assert_eq!(w.planeten[pid].gebaeude[Gebaeude::Erzmine.idx()], before_level);
    assert!(w.schritt());
    assert_eq!(w.planeten[pid].gebaeude[Gebaeude::Erzmine.idx()], before_level + 1);
    assert!(w.planeten[pid].bauschleife.is_empty());
    for g in 0..kern::GUETER { assert_eq!(w.planeten[pid].bestand[g], stock_before[g] - costs[g]); }
    assert_eq!(w.spieler[0].toepfe[Topf::Wirtschaft.idx()], pot_before - w.regeln.wert(&costs));
}

#[test]
fn accepting_old_alliance_offer_rechecks_both_partners_limit() {
    for full_receiver in [false, true] {
        let mut rules = Regelwerk::laden(RULES).unwrap();
        rules.diplomatie.buendnisse_max = 1;
        let mut w = Welt::neu(rules,42,4).unwrap();
        let n1 = w.spieler[1].name.clone();
        let n2 = w.spieler[2].name.clone();
        w.vertrag_anbieten(0,&n1,Vertragsart::Verteidigungsbuendnis,0,None,0,0).unwrap();
        if full_receiver {
            w.vertrag_anbieten(2,&n1,Vertragsart::Verteidigungsbuendnis,0,None,0,0).unwrap();
        } else {
            w.vertrag_anbieten(0,&n2,Vertragsart::Verteidigungsbuendnis,0,None,0,0).unwrap();
        }
        w.vertrag_annehmen(1,1).unwrap();
        let receiver = if full_receiver {1} else {2};
        let credits = w.spieler[receiver].credits;
        assert!(w.vertrag_annehmen(receiver as u16,2).unwrap_err().contains("Bündnislimit"));
        assert_eq!(w.vertraege[1].status,Vertragsstatus::Angeboten);
        assert_eq!(w.spieler[receiver].credits,credits);
    }
}

#[test]
fn malformed_and_overflowing_action_quantities_never_mint_resources_or_panic() {
    let mut w = world();
    let pid = w.spieler[0].heimat as usize;
    let origin = w.planeten[pid].koord.to_string();
    let target = w.planeten[w.spieler[1].heimat as usize].koord.to_string();
    let recipient = w.spieler[1].name.clone();
    w.planeten[pid].gebaeude[Gebaeude::Markt.idx()] = 2;
    w.planeten[pid].gebaeude[Gebaeude::Raumhafen.idx()] = 1;
    w.planeten[pid].einheiten[kern::Einheit::KleinerTransporter.idx()] = 2;
    let mut cases = Vec::new();
    for bad in [-1.0,1e300] {
        cases.push(json!({"typ":"schenken","an":recipient,"credits":bad}));
        cases.push(json!({"typ":"vertrag_anbieten","partner":recipient,"art":"handelsabkommen","kaution":bad}));
        cases.push(json!({"typ":"markt_order","planet":origin,"gut":"erz","seite":"kauf","menge":bad,"preis":1}));
        cases.push(json!({"typ":"markt_order","planet":origin,"gut":"erz","seite":"kauf","menge":1,"preis":bad}));
        cases.push(json!({"typ":"flotte_senden","start":origin,"ziel":target,"mission":"transport","schiffe":{"kleiner_transporter":1},"ladung":{"erz":bad}}));
    }
    cases.push(json!({"typ":"markt_order","planet":origin,"gut":"erz","seite":"kauf","menge":1e12,"preis":1e12}));
    for count in [-1, i64::MAX] {
        cases.push(json!({"typ":"flotte_senden","start":origin,"ziel":target,"mission":"transport","schiffe":{"kleiner_transporter":count}}));
        cases.push(json!({"typ":"fertigen","planet":origin,"einheit":"kleiner_transporter","anzahl":count}));
    }
    for speed in [0.0,0.09,1.01,1e300] {
        cases.push(json!({"typ":"flotte_senden","start":origin,"ziel":target,"mission":"transport","schiffe":{"kleiner_transporter":1},"geschwindigkeit":speed}));
    }
    // Non-finite JSON numbers cannot be represented; malformed/null numeric input is rejected too.
    cases.push(json!({"typ":"schenken","an":recipient,"credits":Value::Null}));
    for action in cases {
        let credits: Vec<_> = w.spieler.iter().map(|s|s.credits).collect();
        let stock = w.planeten[pid].bestand;
        let ships = w.planeten[pid].einheiten;
        let (ok,text) = w.handeln(0,Rolle::Alle,&action);
        assert!(!ok,"unexpectedly accepted {action}: {text}");
        assert_eq!(w.spieler.iter().map(|s|s.credits).collect::<Vec<_>>(),credits);
        assert_eq!(w.planeten[pid].bestand,stock);
        assert_eq!(w.planeten[pid].einheiten,ships);
        assert!(w.orders.is_empty());
        assert!(w.flotten.is_empty());
    }
    // Exercise the fixed-point order API itself, not only JSON conversion.
    assert!(w.markt_order(0,Rolle::Alle,w.planeten[pid].koord,Gut::Erz,kern::Marktseite::Kauf,i64::MAX,i64::MAX).is_err());
}

#[test]
fn interactive_clock_waits_for_every_due_role() {
    let mut w = world();
    let hash = w.hash();
    assert!(!w.faellig.is_empty());
    assert!(w.schritt_wenn_bereit().is_err());
    assert_eq!(hash,w.hash());
    let due = w.faellig.clone();
    for f in &due[..due.len()-1] {
        w.aufruf_ende(f.spieler,f.rolle,None,None,&[]);
    }
    assert!(w.schritt_wenn_bereit().is_err());
    let last = due.last().unwrap();
    w.aufruf_ende(last.spieler,last.rolle,None,None,&[]);
    assert!(w.schritt_wenn_bereit().unwrap());
    assert_eq!(w.zeit,w.regeln.fenster());
}

#[test]
fn last_partial_window_stops_exactly_at_epoch_boundary() {
    let mut rules = Regelwerk::laden(RULES).unwrap();
    rules.welt.epoche_tage = 1;
    rules.welt.fenster_sekunden = 1000;
    let mut w = Welt::neu(rules,42,4).unwrap();
    while w.schritt() { assert!(w.zeit <= TAG); }
    assert_eq!(w.zeit,TAG);
    assert!(w.beendet());
    assert!(!w.schritt());
    assert_eq!(w.zeit,TAG);
}
