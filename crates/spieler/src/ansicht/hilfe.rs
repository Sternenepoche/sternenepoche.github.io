//! Erklärungen für Menschen: Einleitung und „So funktioniert es“ je Bildschirm, die passenden Regelabschnitte
//! im Wortlaut, die Hinweise „Was jetzt ansteht“ und die Spielanleitung.
//!
//! Die eigenen Texte erklären, die Regelabschnitte kommen unverändert aus dem Kern (`kern::regeltext`), damit
//! nichts auseinanderläuft. Hinweise sind Tatsachen aus der Sicht des Spielers mit einem Vorschlag, was hilft.

use super::namen::{name, volk_text};
use super::stil::{self, *};
use super::Bildschirm;
use eframe::egui::{self, RichText};
use kern::Regelwerk;
use serde_json::Value;

/// Ein Satz unter der Überschrift.
pub fn satz(b: Bildschirm) -> &'static str {
    match b {
        Bildschirm::Uebersicht => "Dein Reich auf einen Blick: was gut läuft, was fehlt und was als Nächstes ansteht.",
        Bildschirm::Kolonie => "Dein Planet: Bevölkerung, Versorgung, Strom und Lager.",
        Bildschirm::Gebaeude => "Hier wächst dein Reich: Minen fördern, Kraftwerke liefern Strom, Werke veredeln.",
        Bildschirm::Forschung => "Forschung macht dein Reich stärker und schaltet Neues frei.",
        Bildschirm::Werft => "Schiffe für Handel, Kolonien und Krieg, Verteidigung für deine Planeten.",
        Bildschirm::Flotten => "Schiffe auf die Reise schicken: Handel, Kolonien, Spionage und Angriff.",
        Bildschirm::Galaxie => "Die Karte: wer wo wohnt, wo Nebel und Asteroiden liegen und wo noch Platz ist.",
        Bildschirm::Markt => "Handel mit den anderen Reichen über ein gemeinsames Orderbuch.",
        Bildschirm::Diplomatie => "Nachrichten, Verträge und Allianzen mit den anderen Reichen.",
        Bildschirm::Regierung => "Der Weg von Stufe I zu Stufe V, deine Steuern und deine Punkte.",
        Bildschirm::Berichte => "Was in deinem Reich geschah: Ereignisse, Kämpfe und Spionageberichte.",
        Bildschirm::Rangliste => "Alle Reiche nach Punkten. Am Ende der Epoche gewinnt die höchste Punktzahl.",
        Bildschirm::Anleitung => "Die Regeln des Spiels, verständlich zusammengefasst und im Wortlaut.",
        Bildschirm::LiveKi => "Reiche, die Sprachmodelle regieren: starten, beobachten, ihre Entscheidungen nachlesen.",
        Bildschirm::Befehle => "Jeder Befehl der Engine als JSON, für Fortgeschrittene.",
        Bildschirm::Galerie => "Die geplanten Bilder des Spiels und ihr Stand.",
    }
}

/// Eigene Erklärung je Bildschirm; Absätze durch Leerzeilen getrennt.
pub fn erklaerung(b: Bildschirm) -> &'static str {
    match b {
        Bildschirm::Uebersicht => "Ziel: Nach 365 Spieltagen hat das Reich mit den meisten Punkten gewonnen. Punkte gibt es für das, was du aufgebaut hast: Gebäudestufen, Forschung, den Wert von Flotte und Verteidigung, Bevölkerung und deine Zivilisationsstufe. Gelagerte Güter und Credits zählen nicht, horten lohnt sich also nicht.\n\nDie Zeit läuft in Fenstern von 15 Spielminuten. Mit ▶ läuft sie von selbst, das Tempo legt fest, wie viele Spielminuten je Sekunde vergehen. Alle anderen Reiche handeln in denselben Fenstern wie du.\n\n„Was jetzt ansteht“ zeigt, was in deinem Reich gerade fehlt oder bereitliegt. Ein Klick bringt dich zum passenden Bereich.",
        Bildschirm::Kolonie => "Alles hängt an der Bevölkerung: Sie stellt die Arbeitskräfte für Minen und Werke, zahlt Steuern und zählt selbst als Punkte. Sie wächst, solange Wohnraum frei ist, die Versorgung mit Nahrung reicht und die Stabilität hoch genug ist.\n\nGebäude brauchen Strom. Fehlt Strom, arbeiten alle Anlagen nur noch anteilig. Behalte deshalb die Strombilanz im Blick.\n\nJedes Gut hat eine Lagergrenze. Ist sie erreicht, endet die Produktion dieses Guts; Lieferungen kommen trotzdem an.",
        Bildschirm::Gebaeude => "Pro Planet läuft ein Bau, bis zu fünf Aufträge warten in der Bauschleife. Bezahlt wird beim Baubeginn. Ist die Schleife leer, beginnt ein neuer Auftrag sofort und muss jetzt bezahlbar sein. Hinter anderen Aufträgen wartet er, bis genug Güter da sind.\n\nJede Stufe kostet mehr als die vorige (Kostenfaktor) und bringt mehr Ertrag, braucht aber auch mehr Strom und Arbeitskräfte. Rot markierte Kosten zeigen, was gerade fehlt.\n\nWelche Gebäude zuerst Arbeitskräfte bekommen, legst du mit den Prioritäten fest. Abreißen erstattet nichts.",
        Bildschirm::Forschung => "Labore erzeugen Forschungspunkte. Ein Projekt läuft gleichzeitig, bis zu drei warten. Jede Forschung kostet Güter und Forschungspunkte; die Dauer hängt davon ab, wie viele Punkte alle Labore zusammen je Stunde schaffen.\n\nManche Forschung verlangt eine bestimmte Laborstufe auf dem Planeten, auf dem geforscht wird, meist deiner Heimatwelt. Neue Forschungen schaltet erst die nächste Zivilisationsstufe frei.",
        Bildschirm::Werft => "Die Werft baut Schiffe, die Orbitalwerft Bauteile für Kolonieschiffe. Bezahlt wird bei der Bestellung, die Besatzung kommt aus der Bevölkerung. Schiffe kosten täglich Unterhalt in Credits, Verteidigungsanlagen nicht.\n\nVerteidigung schützt deine Planeten vor Plünderung; der Bunker schützt zusätzlich einen Teil der Güter. Das Raketensilo (ab Stufe III) baut Abfangraketen gegen Raketenangriffe und Interplanetarraketen gegen fremde Verteidigung.",
        Bildschirm::Flotten => "Flotten starten von einem Planeten mit Raumhafen. Die Flugzeit hängt von der Entfernung und vom langsamsten Schiff ab; langsamer fliegen spart Treibstoff (Deuterium), der für Hin- und Rückflug beim Start bezahlt wird. Wie viele Flotten gleichzeitig unterwegs sein dürfen, bestimmt die Computertechnik.\n\nDie Vorschau rechnet Flugzeit, Treibstoff und Laderaum aus, bevor du startest. Für Angriffe zeigt die Kampfsimulation die Siegchance, wenn ein Spionagebericht des Ziels vorliegt.\n\nAnfängerschutz: In den ersten Tagen kann dich niemand angreifen und du niemanden, ohne den eigenen Schutz zu verlieren.",
        Bildschirm::Galaxie => "Die Galaxie hat zwei Sektoren mit je 60 Systemen und 12 Plätzen. Flüge im eigenen Sektor dauern Stunden, in den anderen deutlich länger. Nebelsysteme sind die einzigen Orte, an denen ein Xenoextraktor Xenokristall gewinnt; in Asteroidengürteln bauen Bergbauschiffe ab.\n\nKlicke einen Platz an, um ihn als Ziel für eine Flotte zu übernehmen. Freie Plätze eignen sich für Kolonien; was dort zu holen ist, zeigt eine Spionagesonde.",
        Bildschirm::Markt => "Der Markt ist ein Orderbuch zwischen den Reichen, es gibt keinen Händler außerhalb. Eine Kauforder hinterlegt sofort die Credits, eine Verkaufsorder die Ware. Passen zwei Preise zusammen, wird gehandelt; die gekaufte Ware bringt eine Handelsflotte.\n\nJede Seite zahlt 2 Prozent Gebühr. Für Orders brauchst du einen Markt auf dem Planeten, je Marktstufe fünf offene Orders.",
        Bildschirm::Diplomatie => "Nachrichten gehen sofort an ein oder mehrere Reiche oder an deine Allianz, höchstens 20 je Spieltag. Verträge sind verbindlich und stehen öffentlich im Register: wer sie bricht, ist für alle sichtbar. Eine Kaution macht einen Vertrag glaubwürdig, beim Bruch bekommt der Geschädigte beide Kautionen.\n\nIn einer Allianz (bis zu 8 Reiche) gelten alle als verbündet und haben einen gemeinsamen Kanal. Gewertet wird trotzdem jedes Reich einzeln.",
        Bildschirm::Regierung => "Die Zivilisationsstufe ist dein großer Fortschritt: Jede Stufe schaltet neue Gebäude, Schiffe und Forschung frei und bringt Punkte. Für den Aufstieg müssen alle Bedingungen 48 Stunden ohne Unterbrechung erfüllt sein, dann kostet er einmal Güter von der Heimatwelt.\n\nDer Steuersatz bringt Credits für Unterhalt, Markt und Verträge. Bis 15 Prozent ist er frei, darüber kostet jeder Prozentpunkt einen Punkt Stabilität. Stabilität macht alle Gebäude produktiver und lässt die Bevölkerung wachsen (voll ab 60); unter 30 beginnen keine neuen Bauaufträge.\n\nTöpfe und Doktrin gelten für die vier Rollen der Modellreiche. Du regierst selbst und bezahlst direkt aus Lager und Credits.",
        Bildschirm::Berichte => "Ereignisse sind alles, was deinem Reich zustieß: fertige Bauten, Lieferungen, Angriffe, Verträge. Kampfberichte zeigen Ausgang, Verluste und Beute. Spionageberichte zeigen, was deine Sonden gesehen haben; je älter, desto unsicherer.",
        Bildschirm::Rangliste => "Die Rangliste ist öffentlich und wird laufend aktualisiert. Gezählt wird investierter Wert, nicht Reichtum im Lager.",
        Bildschirm::Anleitung => "",
        Bildschirm::LiveKi => "Hier spielst du gegen Sprachmodelle. Jedes KI-Reich hat vier Rollen: der Stratege setzt die Richtung, der Verwalter baut und forscht, der Feldherr führt die Flotten, der Diplomat verhandelt. Sie sehen dasselbe wie du, nutzen dieselben Aktionen und Regeln, und jede Entscheidung samt Begründung steht unten im Protokoll.\n\nAlle vier Rollen entscheiden alle 15 Spielminuten. Während die Modelle nachdenken, wartet die Weltuhr; deine Befehle werden sofort geprüft und ausgeführt. KI-Befehle werden anschließend erneut am aktuellen Spielstand geprüft. Die Antwortdauer hängt vom Anbieter ab. Das Budget begrenzt die Kosten: Ist es aufgebraucht, hält die Partie an. Mit der Attrappe probierst du alles ohne Modell und ohne Kosten aus.",
        Bildschirm::Befehle => "Für Fortgeschrittene: Hier schreibst du jeden Befehl als JSON, genau so, wie ihn die Modellreiche schicken. Jede Aktion hat auch einen eigenen Dialog in den Bereichen links; diese Konsole braucht man zum Spielen nicht.\n\nDarunter stehen die lesenden Abfragen der Engine, etwa Flugzeit oder Kampfsimulator.",
        Bildschirm::Galerie => "",
    }
}

/// Stichworte der Regelabschnitte (`kern::regeltext::abschnitte`), die zu einem Bildschirm gehören.
pub fn regel_stichworte(b: Bildschirm) -> &'static [&'static str] {
    match b {
        Bildschirm::Uebersicht => &["ziel", "zeit"],
        Bildschirm::Kolonie => &["bevoelkerung", "energie", "stabilitaet", "lager"],
        Bildschirm::Gebaeude => &["wirtschaft", "grossprojekte"],
        Bildschirm::Forschung => &["forschung"],
        Bildschirm::Werft => &["schiffe", "verteidigung", "raketen"],
        Bildschirm::Flotten => &["flug", "missionen", "kampf", "pluenderung", "schutz", "verband"],
        Bildschirm::Galaxie => &["galaxie", "spionage"],
        Bildschirm::Markt => &["markt"],
        Bildschirm::Diplomatie => &["diplomatie"],
        Bildschirm::Regierung => &["stufen", "steuern", "stabilitaet", "regierung"],
        Bildschirm::Berichte => &["kampf", "spionage", "blockade", "eroberung"],
        Bildschirm::Rangliste => &["ziel"],
        _ => &[],
    }
}

/// Die Regelabschnitte zu Stichworten, im Wortlaut des Kerns.
pub fn regeln(r: &Regelwerk, stichworte: &[&str]) -> Vec<(String, String)> {
    kern::regeltext::abschnitte(r)
        .into_iter()
        .filter(|a| stichworte.contains(&a.stichwort))
        .map(|a| (a.titel.to_string(), a.text))
        .collect()
}

/// Kopf eines Bildschirms: Überschrift, Satz und aufklappbare Erklärung mit Regeln.
pub fn kopf(ui: &mut egui::Ui, b: Bildschirm, r: &Regelwerk) {
    stil::kopf(ui, b.zeichen(), b.name(), satz(b));
    let text = erklaerung(b);
    if !text.is_empty() {
        stil::erklaerung(ui, b.name(), text, &regeln(r, regel_stichworte(b)));
    }
    ui.add_space(6.0);
}

/// Ein Hinweis: was los ist, und wohin ein Klick führt.
pub struct Hinweis {
    pub farbe: egui::Color32,
    pub text: String,
    pub ziel: Bildschirm,
}

/// Was jetzt ansteht, aus der Sicht des Spielers. Dringendes zuerst.
pub fn hinweise(v: &Value, r: &Regelwerk) -> Vec<Hinweis> {
    let unruhen = r.stabilitaet.unruhen_unter as i64;
    let mut h = Vec::new();
    let mut neu = |farbe, text: String, ziel| h.push(Hinweis { farbe, text, ziel });
    for a in v["angriffe"].as_array().into_iter().flatten() {
        neu(SCHLECHT, format!("⚔ Feindliche Flotte von {} ({} Schiffe, {}) erreicht {} {}.", text(&a["von"]), z(&a["schiffe"]),
            name(a["mission"].as_str().unwrap_or("")), text(&a["ziel"]), in_zeit(i(&a["in_min"]))), Bildschirm::Flotten);
    }
    for r in v["raketensalven"].as_array().into_iter().flatten() {
        neu(SCHLECHT, format!("🚀 {} Raketen von {} schlagen um {} auf {} ein.", z(&r["anzahl"]), text(&r["von"]), text(&r["ankunft"]), text(&r["ziel"])), Bildschirm::Werft);
    }
    let syntheten = v["volk"] == "syntheten";
    for p in v["planeten"].as_array().into_iter().flatten() {
        let k = text(&p["koord"]);
        let (e, ver) = (i(&p["energie"]["erzeugung"]), i(&p["energie"]["verbrauch"]));
        if e < ver {
            neu(SCHLECHT, format!("⚡ Auf {k} fehlt Strom ({} erzeugt, {} gebraucht): alle Anlagen arbeiten nur anteilig. Ein Solarkraftwerk hilft schnell.", zahl(e), zahl(ver)), Bildschirm::Gebaeude);
        }
        if !syntheten && i(&p["nahrung_deckung"]) < 100 {
            neu(SCHLECHT, format!("🌾 Auf {k} reicht die Nahrung nur zu {} Prozent: die Bevölkerung schrumpft. Baue eine Farm aus.", i(&p["nahrung_deckung"])), Bildschirm::Gebaeude);
        }
        if i(&p["stabilitaet"]) < unruhen {
            neu(SCHLECHT, format!("⚠ Unruhen auf {k} (Stabilität {}): neue Bauaufträge beginnen nicht. Konsumgüter, freier Wohnraum und Steuern bis 15 Prozent heben die Stabilität.", i(&p["stabilitaet"])), Bildschirm::Regierung);
        }
        if i(&p["arbeit"]["bedarf"]) > i(&p["arbeit"]["verfuegbar"]) {
            neu(WARNUNG, format!("👥 Auf {k} fehlen Arbeitskräfte ({} gebraucht, {} da). Mehr Wohnraum lässt die Bevölkerung wachsen.", z(&p["arbeit"]["bedarf"]), z(&p["arbeit"]["verfuegbar"])), Bildschirm::Gebaeude);
        }
        if p["bauschleife"].as_array().is_some_and(Vec::is_empty) {
            neu(INFO, format!("🔨 Die Bauschleife auf {k} ist leer. Jeder freie Moment ohne Bau ist verlorenes Wachstum."), Bildschirm::Gebaeude);
        } else if p["bauschleife"][0]["wartet"] == true {
            neu(WARNUNG, format!("🔨 Der nächste Bau auf {k} ({}) wartet auf Güter.", name(p["bauschleife"][0]["gebaeude"].as_str().unwrap_or(""))), Bildschirm::Gebaeude);
        }
        for (gut, h) in p["voll_in_stunden"].as_object().into_iter().flatten() {
            if i(h) < 6 {
                neu(WARNUNG, format!("📦 Das Lager für {} auf {k} ist in unter 6 Stunden voll. Lager ausbauen, verbrauchen oder verkaufen.", name(gut)), Bildschirm::Gebaeude);
            }
        }
        if let Some(b) = p["blockade"].as_object() {
            if b["art"] == "invasion" {
                neu(SCHLECHT, format!("⚔ {k} wird von {} belagert: Jeder Tag senkt das Stabilitätsziel um {} Punkte. Fällt die Stabilität unter {} und sind die Truppen stärker als die Garnison, ist die Kolonie verloren. Greif die Flotte an oder ruf Verbündete.",
                    text(&b["durch"]), r.stabilitaet.belagerung_je_tag as i64, r.stabilitaet.uebernahme_unter as i64), Bildschirm::Flotten);
            } else {
                neu(SCHLECHT, format!("⛔ {k} wird von {} blockiert: Transporte kehren um, Marktlieferungen warten. Ein Angriff auf die Flotte beendet die Blockade.", text(&b["durch"])), Bildschirm::Flotten);
            }
        }
    }
    for vt in v["vertraege"].as_array().into_iter().flatten() {
        if vt["status"] == "angebot an dich" {
            neu(INFO, format!("🕊 {} bietet dir einen {} an.", text(&vt["partner"]), name(vt["art"].as_str().unwrap_or(""))), Bildschirm::Diplomatie);
        }
    }
    for e in v["einladungen"].as_array().into_iter().flatten() {
        neu(INFO, format!("🕊 Einladung in die Allianz {}.", text(e)), Bildschirm::Diplomatie);
    }
    let neue = v["nachrichten"].as_array().map_or(0, |n| n.iter().filter(|x| x["neu"] == true).count());
    if neue > 0 {
        neu(INFO, format!("✉ {neue} neue Nachrichten."), Bildschirm::Diplomatie);
    }
    let f = &v["forschung"];
    if f["aktiv"].is_null() && f["moeglich"].as_array().is_some_and(|m| !m.is_empty()) {
        neu(INFO, "🔬 Es läuft keine Forschung.".into(), Bildschirm::Forschung);
    }
    if let Some(st) = v["naechste_stufe"].as_object() {
        let offen: Vec<String> = st["bedingungen"].as_array().into_iter().flatten()
            .filter(|b| b["erfuellt"] != true).map(|b| super::namen::woerter(&text(&b["text"]))).collect();
        if offen.is_empty() {
            if i(&st["erfuellt_seit_stunden"]) >= i(&st["haltezeit_stunden"]) {
                neu(GUT, format!("👑 Alle Bedingungen für {} sind erfüllt: Du kannst aufsteigen!", text(&st["name"])), Bildschirm::Regierung);
            } else {
                neu(GUT, format!("👑 Alle Bedingungen für {} sind erfüllt, noch {} bis zum Aufstieg.", text(&st["name"]),
                    dauer_h(i(&st["haltezeit_stunden"]) - i(&st["erfuellt_seit_stunden"]))), Bildschirm::Regierung);
            }
        } else {
            neu(LEISE, format!("👑 Für {} fehlt noch: {}.", text(&st["name"]), offen.join("; ")), Bildschirm::Regierung);
        }
    }
    h
}

pub fn text(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => "–".into(),
        x => x.to_string(),
    }
}

/// Willkommen und erste Schritte, mit den Zahlen der Stufe II aus dem Regelwerk.
pub fn willkommen(ui: &mut egui::Ui, v: &Value, r: &Regelwerk) -> bool {
    let mut schliessen = false;
    hinweis_karte(ui, GOLD, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("Willkommen, Herrscherin oder Herrscher von {}!", text(&v["name"]))).size(20.0).strong().color(GOLD));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Ausblenden").clicked() {
                    schliessen = true;
                }
            });
        });
        let volk = v["volk"].as_str().unwrap_or("");
        ui.label(format!("Dein Volk sind die {}. {}", name(volk), volk_text(volk)));
        ui.label(format!(
            "Ziel: In {} Spieltagen das Reich mit den meisten Punkten zu werden, gegen {} andere Reiche.",
            i(&v["epoche_tage"]),
            i(&v["spielerzahl"]) - 1
        ));
        ui.add_space(4.0);
        ui.label(RichText::new("Erste Schritte").strong());
        let s2 = r.stufen.first();
        let minen = s2.map(|s| s.gebaeude.iter().map(|(g, n)| format!("{} Stufe {n}", name(g.name()))).collect::<Vec<_>>().join(" und ")).unwrap_or_default();
        let schritte = [
            "Oben siehst du deine Rohstoffe: Bestand, Zuwachs je Stunde und wie voll das Lager ist. Fährst du mit der Maus darüber, erfährst du mehr.".to_string(),
            "Unter 🏭 Gebäude baust du aus. Sorge zuerst für Strom (Solarkraftwerk), dann für Minen und eine Farm. Die Bauschleife fasst fünf Aufträge, halte sie gefüllt.".into(),
            "Mit ▶ läuft die Zeit standardmäßig in Echtzeit. Du kannst jederzeit handeln, auch während KI-Reiche denken. Sie entscheiden alle 15 Spielminuten; während ihrer Antworten wartet die Weltuhr. Höheres Tempo und Zeitsprünge sind freiwillig.".into(),
            "Unter 🔬 Forschung startest du Projekte; dafür braucht deine Heimatwelt ein Labor.".into(),
            format!("Stufe II erreichst du mit {} Einwohnern und {minen}. Den Weg zeigt 👑 Zivilisation.", s2.map(|s| zahl(s.einwohner as i64)).unwrap_or_default()),
            "„Was jetzt ansteht“ in der Übersicht sagt dir jederzeit, wo es hakt. Speichern kannst du links unten.".into(),
        ];
        for (n, s) in schritte.iter().enumerate() {
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new(format!("{}.", n + 1)).color(GOLD).strong());
                ui.label(s);
            });
        }
    });
    schliessen
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Eine Belagerung (Invasion) warnt vor dem Verlust der Kolonie, eine Blockade sagt, was sie stört.
    #[test]
    fn belagerung_und_blockade_werden_unterschieden() {
        let r = spieler::Session::new(1, 4, 0).unwrap().regeln().clone();
        let planet = |art: &str| json!({"koord": "1:2:6", "blockade": {"durch": "Rabor", "art": art, "flotte": 7}});
        let v = json!({"planeten": [planet("invasion")], "stufe": 4});
        let t: Vec<String> = hinweise(&v, &r).into_iter().filter(|h| h.farbe == SCHLECHT).map(|h| h.text).collect();
        assert!(t.iter().any(|x| x.contains("belagert") && x.contains("Garnison")), "{t:?}");
        let v = json!({"planeten": [planet("blockade")], "stufe": 4});
        let t: Vec<String> = hinweise(&v, &r).into_iter().filter(|h| h.farbe == SCHLECHT).map(|h| h.text).collect();
        assert!(t.iter().any(|x| x.contains("blockiert") && !x.contains("belagert")), "{t:?}");
    }
}
