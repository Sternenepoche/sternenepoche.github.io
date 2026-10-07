//! Alle Befehle, die die Oberfläche an den Kern schickt, an einer Stelle. Der Test unten prüft jeden gegen
//! `kern::aktion::Aktion`: lesbar, kein Feld geht verloren, der Typ ist für den Menschen erlaubt. Ein Tippfehler in
//! einem Feldnamen fiele sonst erst beim Klick auf.

use serde_json::{json, Map, Value};

pub fn bauen(planet: &str, gebaeude: &str) -> Value {
    json!({"typ": "bauen", "planet": planet, "gebaeude": gebaeude})
}
pub fn reparieren(planet: &str, gebaeude: &str) -> Value {
    json!({"typ": "reparieren", "planet": planet, "gebaeude": gebaeude})
}
pub fn flotte_versorgen(start: &str, ziel: &str, flotte: &Value, schiffe: &Map<String, Value>, geschwindigkeit: f64, ladung: &Map<String, Value>) -> Value {
    json!({"typ": "flotte_versorgen", "start": start, "ziel": ziel, "versorgungsflotte": flotte,
        "schiffe": schiffe, "geschwindigkeit": geschwindigkeit, "ladung": ladung})
}
pub fn abreissen(planet: &str, gebaeude: &str) -> Value {
    json!({"typ": "abreissen", "planet": planet, "gebaeude": gebaeude})
}
pub fn schleife_leeren(planet: &str) -> Value {
    json!({"typ": "schleife_leeren", "planet": planet})
}
pub fn prioritaeten(planet: &str, reihenfolge: &[String]) -> Value {
    json!({"typ": "prioritaeten", "planet": planet, "reihenfolge": reihenfolge})
}
/// Geforscht wird auf der Heimatwelt (`planet` null).
pub fn forschen(forschung: &str) -> Value {
    json!({"typ": "forschen", "forschung": forschung, "planet": Value::Null})
}
pub fn fertigen_einheit(planet: &str, einheit: &str, anzahl: i64) -> Value {
    json!({"typ": "fertigen", "planet": planet, "einheit": einheit, "anzahl": anzahl})
}
pub fn fertigen_bauteil(planet: &str, bauteil: &str, anzahl: i64) -> Value {
    json!({"typ": "fertigen", "planet": planet, "bauteil": bauteil, "anzahl": anzahl})
}
pub fn raketen_bauen(planet: &str, art: &str, anzahl: i64) -> Value {
    json!({"typ": "raketen_bauen", "planet": planet, "art": art, "anzahl": anzahl})
}
pub fn raketen_starten(start: &str, ziel: &str, anzahl: i64, zieltyp: &str) -> Value {
    json!({"typ": "raketen_starten", "start": start, "ziel": ziel, "anzahl": anzahl, "zieltyp": zieltyp})
}
pub fn steuersatz(prozent: i64) -> Value {
    json!({"typ": "steuersatz", "prozent": prozent})
}
pub fn stufenaufstieg() -> Value {
    json!({"typ": "stufenaufstieg"})
}
pub fn markt_order(planet: &str, gut: &str, seite: &str, menge: f64, preis: f64) -> Value {
    json!({"typ": "markt_order", "planet": planet, "gut": gut, "seite": seite, "menge": menge, "preis": preis})
}
pub fn markt_storno(order: &Value) -> Value {
    json!({"typ": "markt_storno", "order": order})
}
#[allow(clippy::too_many_arguments)]
pub fn flotte_senden(start: &str, ziel: &str, mission: &str, schiffe: &Map<String, Value>, geschwindigkeit: f64, ladung: &Map<String, Value>, halten: i64) -> Value {
    json!({"typ": "flotte_senden", "start": start, "ziel": ziel, "mission": mission, "schiffe": schiffe,
        "geschwindigkeit": geschwindigkeit, "ladung": ladung, "haltedauer_stunden": halten})
}
pub fn flotte_zurueckrufen(flotte: &Value) -> Value {
    json!({"typ": "flotte_zurueckrufen", "flotte": flotte})
}
pub fn flotte_ausspaehen(start: &str, flotte: &Value, sonden: i64) -> Value {
    json!({"typ":"flotte_ausspaehen","start":start,"flotte":flotte,"sonden":sonden,"geschwindigkeit":1.0})
}
pub fn verband_oeffnen(flotte: &Value) -> Value {
    json!({"typ": "verband_oeffnen", "flotte": flotte})
}
pub fn verband_beitreten(flotte: &Value, fuehrung: &Value) -> Value {
    json!({"typ": "verband_beitreten", "flotte": flotte, "fuehrung": fuehrung})
}
pub fn nachricht(an: &[String], allianz: bool, text: &str) -> Value {
    json!({"typ": "nachricht", "an": an, "allianz": allianz, "text": text})
}
/// `tribut`: Gut (None = Credits), Menge je Tag, Tage. Ohne Tribut zahlt niemand.
pub fn vertrag_anbieten(partner: &str, art: &str, kaution: f64, tribut: Option<(Option<&str>, f64, i64)>) -> Value {
    let (gut, menge, tage) = tribut.unwrap_or((None, 0.0, 0));
    json!({"typ": "vertrag_anbieten", "partner": partner, "art": art, "kaution": kaution,
        "tribut_gut": gut, "tribut_menge": menge, "tribut_tage": tage})
}
pub fn vertrag_annehmen(vertrag: &Value) -> Value {
    json!({"typ": "vertrag_annehmen", "vertrag": vertrag})
}
pub fn vertrag_ablehnen(vertrag: &Value) -> Value {
    json!({"typ": "vertrag_ablehnen", "vertrag": vertrag})
}
pub fn vertrag_kuendigen(vertrag: &Value) -> Value {
    json!({"typ": "vertrag_kuendigen", "vertrag": vertrag})
}
pub fn allianz_gruenden(name: &str) -> Value {
    json!({"typ": "allianz_gruenden", "name": name})
}
pub fn allianz_einladen(spieler: &str) -> Value {
    json!({"typ": "allianz_einladen", "spieler": spieler})
}
pub fn allianz_beitreten(allianz: &str) -> Value {
    json!({"typ": "allianz_beitreten", "allianz": allianz})
}
pub fn allianz_verlassen() -> Value {
    json!({"typ": "allianz_verlassen"})
}
pub fn schenken(an: &str, credits: f64) -> Value {
    json!({"typ": "schenken", "an": an, "credits": credits})
}

#[cfg(test)]
mod tests {
    use super::*;
    use kern::aktion::{erlaubte_typen, Aktion};
    use kern::Rolle;

    fn alle() -> Vec<Value> {
        let mut schiffe = Map::new();
        schiffe.insert("kleiner_transporter".into(), json!(2));
        let mut ladung = Map::new();
        ladung.insert("erz".into(), json!(500));
        let id = json!(7);
        vec![
            bauen("1:2:6", "solarkraftwerk"),
            reparieren("1:2:6", "erzmine"),
            flotte_versorgen("1:2:6", "1:4:6", &id, &schiffe, 1.0, &ladung),
            abreissen("1:2:6", "erzmine"),
            schleife_leeren("1:2:6"),
            prioritaeten("1:2:6", &["farm".into(), "erzmine".into()]),
            forschen("energietechnik"),
            fertigen_einheit("1:2:6", "kreuzer", 3),
            fertigen_bauteil("1:2:6", "habitatmodul", 2),
            raketen_bauen("1:2:6", "abfang", 5),
            raketen_starten("1:2:6", "1:4:6", 2, "raketenwerfer"),
            steuersatz(20),
            stufenaufstieg(),
            markt_order("1:2:6", "erz", "verkauf", 1000.0, 0.06),
            markt_storno(&id),
            flotte_senden("1:2:6", "1:4:6", "transport", &schiffe, 0.7, &ladung, 0),
            flotte_zurueckrufen(&id),
            flotte_ausspaehen("1:2:6", &id, 1),
            verband_oeffnen(&id),
            verband_beitreten(&id, &json!(9)),
            nachricht(&["Rabor".into()], false, "Frieden?"),
            vertrag_anbieten("Rabor", "nichtangriffspakt", 500.0, None),
            vertrag_anbieten("Rabor", "tribut", 0.0, Some((Some("erz"), 100.0, 7))),
            vertrag_anbieten("Rabor", "tribut", 0.0, Some((None, 100.0, 7))),
            vertrag_annehmen(&id),
            vertrag_ablehnen(&id),
            vertrag_kuendigen(&id),
            allianz_gruenden("Sternenbund"),
            allianz_einladen("Rabor"),
            allianz_beitreten("Sternenbund"),
            allianz_verlassen(),
            schenken("Rabor", 300.0),
        ]
    }

    /// Jeder Befehl der Oberfläche ist eine gültige Aktion des Kerns, ohne dass ein Feld verloren geht.
    #[test]
    fn jeder_befehl_versteht_der_kern() {
        let erlaubt = erlaubte_typen(Rolle::Alle);
        for b in alle() {
            let a: Aktion = serde_json::from_value(b.clone()).unwrap_or_else(|e| panic!("{b}: {e}"));
            assert!(erlaubt.contains(&b["typ"].as_str().unwrap()), "{b}");
            let zurueck = serde_json::to_value(&a).unwrap();
            for (feld, wert) in b.as_object().unwrap() {
                if feld != "typ" && !wert.is_null() {
                    assert!(zurueck.get(feld).is_some(), "{}: Feld {feld} kennt der Kern nicht", b["typ"]);
                }
            }
        }
    }

    /// Bis auf Doktrin und Meldung (nur für die Rollen der Modellreiche) hat jede Aktion einen Befehl in der Oberfläche.
    #[test]
    fn jede_aktion_ist_bedienbar() {
        let vorhanden: Vec<String> = alle().iter().map(|b| b["typ"].as_str().unwrap().to_string()).collect();
        for t in erlaubte_typen(Rolle::Alle) {
            if t == "doktrin" || t == "meldung" {
                continue;
            }
            assert!(vorhanden.iter().any(|v| v == t), "{t} ist in der Oberfläche nicht bedienbar");
        }
    }
}
