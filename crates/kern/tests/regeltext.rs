//! Der Regeltext ist das, was die Modelle über das Spiel wissen. Wo er sich aus dem Regelwerk ableiten
//! lässt, wird hier geprüft, dass er nichts auslässt: Freischaltungen je Stufe und die Abschnitte,
//! deren Sätze von Hand formuliert sind und vom Kern abwichen (Spezifikation, Anhang A).

use kern::regeltext::regeltext;
use kern::typen::*;
use kern::Regelwerk;

const REGELN: &str = include_str!("../../../regeln/regelwerk.ron");

fn norm(s: &str) -> String {
    s.to_lowercase()
        .replace('ä', "ae")
        .replace('ö', "oe")
        .replace('ü', "ue")
        .replace('ß', "ss")
        .replace([' ', '-', '_'], "")
}

/// Jedes Gebäude und jede Verteidigungsanlage ab Stufe II steht im Freischalttext genau der Stufe,
/// ab der sie baubar ist. Fehlte das Raketensilo bei Stufe III, erfuhr kein Modell davon.
#[test]
fn freischalttext_nennt_alle_gebaeude_und_anlagen_ihrer_stufe() {
    let r = Regelwerk::laden(REGELN).unwrap();
    let mut namen: Vec<(String, u8)> = Gebaeude::ALLE
        .iter()
        .map(|g| (g.name().to_string(), r.geb(*g).ab_stufe))
        .collect();
    namen.extend(
        Einheit::ALLE
            .iter()
            .filter(|e| !e.ist_schiff())
            .map(|e| (e.name().to_string(), r.einh(*e).ab_stufe)),
    );
    let mut fehlt = Vec::new();
    for (name, ab) in namen {
        if ab < 2 {
            continue;
        }
        let text = norm(&r.stufen[ab as usize - 2].schaltet_frei);
        if !text.split(',').any(|t| t == norm(&name)) {
            fehlt.push(format!("{name} (ab Stufe {ab})"));
        }
    }
    assert!(fehlt.is_empty(), "nicht im Freischalttext: {fehlt:?}");
}

/// Die von Hand formulierten Sätze, die der Kern anders umsetzte, sagen jetzt, was er tut.
#[test]
fn regeltext_beschreibt_was_der_kern_tut() {
    let r = Regelwerk::laden(REGELN).unwrap();
    let alle = regeltext(&r, Rolle::Alle);
    // wertung.rs punkte_neu: nur der Bonus der aktuellen Stufe.
    assert!(alle.contains("es zählt nur die aktuelle Stufe"));
    assert!(!alle.contains("einmalig"));
    // flotte.rs, Ankunft, Markt und Tribut: Zuflüsse ohne Lagergrenze.
    assert!(alle.contains("kommen auch über die Grenze hinaus an"));
    // flotte.rs spionieren: Bestände in jedem Bericht.
    assert!(alle.contains("Jeder Bericht zeigt die Bestände"));
    // flotte.rs verband_beitreten: fremde Flotten nur aus derselben Allianz.
    assert!(alle.contains("Mitgliedern derselben Allianz"));
    // wirtschaft.rs bauen: in eine leere Schleife nur bezahlbar, dahinter wartet ein Auftrag. Der alte Satz
    // „ein nicht bezahlbarer Auftrag wartet“ ließ den Verwalter im echten Lauf hunderte Aufträge in leere
    // Schleifen schicken, die der Kern ablehnte.
    assert!(alle.contains("Ist die Schleife leer, beginnt ein neuer Auftrag sofort und muss jetzt bezahlbar sein"));
    let mut w = kern::Welt::neu(r.clone(), 3, 2).unwrap();
    let pid = w.spieler[0].heimat as usize;
    let k = w.planeten[pid].koord.to_string();
    let bauen = |g: &str| serde_json::json!({"typ": "bauen", "planet": k, "gebaeude": g});
    let vorrat = w.planeten[pid].bestand;
    w.planeten[pid].bestand = [0; GUETER];
    let (ok, text) = w.handeln(0, Rolle::Alle, &bauen("erzmine"));
    assert!(!ok && text.contains("fehlen"), "leere Schleife, nicht bezahlbar: {text}");
    w.planeten[pid].bestand = vorrat;
    assert!(w.handeln(0, Rolle::Alle, &bauen("erzmine")).0, "bezahlbar beginnt sofort");
    w.planeten[pid].bestand = [0; GUETER];
    let (ok, text) = w.handeln(0, Rolle::Alle, &bauen("kristallmine"));
    assert!(ok, "hinter einem laufenden Bau wartet ein nicht bezahlbarer Auftrag: {text}");
}
