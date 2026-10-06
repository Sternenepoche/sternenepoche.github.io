//! Berichte (Ereignisse, Kampfberichte, Spionageberichte) und die Rangliste.

use super::hilfe::{self, text};
use super::namen::{gut_zeichen, name};
use super::stil::*;
use super::Bildschirm;
use crate::Player;
use eframe::egui::{self, RichText};
use serde_json::Value;

const REITER: [&str; 3] = ["📰 Ereignisse", "⚔ Kampfberichte", "✈ Spionageberichte"];

fn ereignis_motiv(art: &str, text: &str) -> Option<&'static str> {
    let t = text.to_lowercase();
    match art {
        "bau" if t.contains("fertig") => Some("bau_fertig"),
        "forschung" if t.contains("fertig") => Some("forschung_fertig"),
        "fertigung" if t.contains("fertig") => Some("fertigung_fertig"),
        "lieferung" if t.contains("geliefert") || t.contains("angekommen") => Some("handel_geliefert"),
        "flotte" if t.contains("zurück") || t.contains("zurueck") => Some("flotte_rueckkehr"),
        "flotte" if t.contains("angekommen") || t.contains("ankunft") => Some("flotte_ankunft"),
        "angriff" => Some("angriff_warnung"),
        "beute" => Some("pluenderung"),
        "eroberung" if t.contains("erobert") => Some("eroberung"),
        "kolonie" if t.contains("gegründet") || t.contains("gegruendet") => Some("kolonie_gegruendet"),
        "stufe" if t.contains("erreicht") => Some("stufe_erreicht"),
        "unterhalt" if t.contains("desert") => Some("desertion"),
        "spionage" if t.contains("abgewehrt") || t.contains("entdeckt") => Some("spionage_abgewehrt"),
        "spionage" if t.contains("bericht") => Some("spionage_bericht"),
        "vertrag" if t.contains("gebrochen") => Some("vertrag_bruch"),
        "vertrag" if t.contains("bietet") => Some("vertrag_angebot"),
        "vertrag" if t.contains("beendet") || t.contains("abgelaufen") => Some("vertrag_ende"),
        _ => None,
    }
}

/// Spielminuten seit Beginn der Epoche aus einer Zeitangabe wie „Tag 4, 03:15“; unlesbar ergibt 0.
pub(crate) fn zeit_minuten(t: &str) -> i64 {
    let Some(rest) = t.strip_prefix("Tag ") else { return 0 };
    let (tag, uhr) = rest.split_once(", ").unwrap_or((rest, "00:00"));
    let (h, m) = uhr.split_once(':').unwrap_or(("0", "0"));
    (tag.trim().parse::<i64>().unwrap_or(1) - 1) * 1440 + h.parse::<i64>().unwrap_or(0) * 60 + m.parse::<i64>().unwrap_or(0)
}

/// Die erste Koordinate (Sektor:System:Position) in einem Text, etwa „Erzmine Stufe 4 auf 1:45:4 fertig“.
pub(crate) fn koordinate(t: &str) -> Option<String> {
    t.split(|c: char| !(c.is_ascii_digit() || c == ':'))
        .find(|w| {
            let teile: Vec<&str> = w.split(':').collect();
            teile.len() == 3 && teile.iter().all(|x| !x.is_empty())
        })
        .map(String::from)
}

/// Ungelesene Ereignisse: neuer als das, was im Bereich Berichte zuletzt angesehen wurde.
pub(crate) fn ungelesen(v: &Value, gelesen_bis: i64) -> usize {
    v["ereignisse"].as_array().map_or(0, |e| e.iter().filter(|x| zeit_minuten(x["zeit"].as_str().unwrap_or("")) > gelesen_bis).count())
}

fn liste(v: &Value) -> String {
    let teile: Vec<String> = v.as_object().into_iter().flatten().map(|(k, n)| format!("{} {}", z(n), name(k))).collect();
    if teile.is_empty() {
        "keine".into()
    } else {
        teile.join(", ")
    }
}

impl Player {
    pub(crate) fn berichte(&mut self, ui: &mut egui::Ui, v: &Value) {
        let r = self.regeln.clone();
        hilfe::kopf(ui, Bildschirm::Berichte, &r);
        let neu = ungelesen(v, self.ereignisse_gelesen);
        if neu > 0 && self.berichte_reiter != 0 {
            ui.label(RichText::new(format!("{neu} neue Ereignisse unter 📰 Ereignisse.")).color(GOLD));
        }
        ui.horizontal(|ui| {
            for (n, t) in REITER.iter().enumerate() {
                if ui.selectable_label(self.berichte_reiter == n, RichText::new(*t).size(16.5)).clicked() {
                    self.berichte_reiter = n;
                }
            }
        });
        ui.separator();
        match self.berichte_reiter {
            0 => {
                let e = v["ereignisse"].as_array().cloned().unwrap_or_default();
                if e.is_empty() {
                    ui.label(RichText::new("Noch ist nichts geschehen.").color(LEISE));
                }
                // Was neuer ist als der letzte Besuch, trägt „neu“; gelesen gilt es, sobald du den Bereich verlässt.
                self.ereignisse_angesehen = true;
                let mut sprung: Option<(String, String)> = None;
                egui::Grid::new("ereignisse").striped(true).num_columns(3).spacing([16.0, 4.0]).show(ui, |ui| {
                    for x in e {
                        let art = x["art"].as_str().unwrap_or("");
                        let farbe = match art {
                            "kampf" | "beute" | "angriff" | "blockade" | "abgelehnt" | "unterhalt" | "raketen" | "eroberung" => SCHLECHT,
                            "stufe" | "kolonie" | "bau" | "forschung" | "fertigung" | "lieferung" => GUT,
                            "vertrag" | "allianz" | "nachricht" | "geschenk" | "markt" => INFO,
                            _ => TEXT,
                        };
                        let zeit = text(&x["zeit"]);
                        let t = zeit_minuten(&zeit);
                        self.ereignisse_neuestes = self.ereignisse_neuestes.max(t);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(&zeit).color(LEISE));
                            if t > self.ereignisse_gelesen {
                                etikett(ui, "neu", GOLD);
                            }
                        });
                        ui.horizontal(|ui| {
                            if let Some(key) = ereignis_motiv(art, x["text"].as_str().unwrap_or("")) { self.motiv_klein(ui, "events", key, v); }
                            ui.label(RichText::new(super::namen::woerter(&text(&x["text"]))).color(farbe));
                        });
                        if ui.small_button("›").on_hover_text("Dorthin, wo es geschah").clicked() {
                            sprung = Some((art.to_string(), text(&x["text"])));
                        }
                        ui.end_row();
                    }
                });
                if let Some((art, t)) = sprung {
                    self.zu_ereignis(v, &art, &t);
                }
            }
            1 => {
                let k = v["kampfberichte"].as_array().cloned().unwrap_or_default();
                if k.is_empty() {
                    ui.label(RichText::new("Noch keine Kämpfe mit deiner Beteiligung.").color(LEISE));
                }
                for b in k {
                    let verteidigt = b["seite"] == "verteidigung";
                    let gewonnen = (b["sieger"] == "verteidiger") == verteidigt && b["sieger"] != "unentschieden";
                    karte(ui, |ui| {
                        self.motiv(ui, "events", if b["sieger"] == "unentschieden" { "kampf_unentschieden" } else if gewonnen { "kampf_sieg" } else { "kampf_niederlage" }, v);
                        ui.horizontal(|ui| {
                            etikett(ui, if b["sieger"] == "unentschieden" { "unentschieden" } else if gewonnen { "gewonnen" } else { "verloren" },
                                if b["sieger"] == "unentschieden" { WARNUNG } else if gewonnen { GUT } else { SCHLECHT });
                            ui.label(RichText::new(format!("{} bei {}", name(b["mission"].as_str().unwrap_or("")), text(&b["ort"]))).strong());
                            ui.label(RichText::new(format!("{} · {} Runden · du warst {}", text(&b["zeit"]), z(&b["runden"]), if verteidigt { "Verteidiger" } else { "Angreifer" })).color(LEISE));
                        });
                        let namen = |x: &Value| x.as_array().into_iter().flatten().map(text).collect::<Vec<_>>().join(", ");
                        ui.label(format!("Angreifer: {} – Verluste: {}", namen(&b["angreifer"]), liste(&b["verluste_angreifer"])));
                        ui.label(format!("Verteidiger: {} – Verluste: {}", namen(&b["verteidiger"]), liste(&b["verluste_verteidiger"])));
                        let beute: Vec<String> = b["beute"].as_object().into_iter().flatten().map(|(g, n)| format!("{} {}", gut_zeichen(g), z(n))).collect();
                        if !beute.is_empty() {
                            ui.label(RichText::new(format!("Beute: {}", beute.join("  "))).color(if verteidigt { SCHLECHT } else { GUT }));
                        }
                        if i(&b["truemmer"]["erz"]) + i(&b["truemmer"]["kristall"]) > 0 {
                            ui.label(RichText::new(format!("Trümmerfeld: {} Erz, {} Kristall – Recycler können es einsammeln.", z(&b["truemmer"]["erz"]), z(&b["truemmer"]["kristall"]))).small().color(LEISE));
                        }
                    });
                }
            }
            _ => {
                let b = v["berichte"].as_array().cloned().unwrap_or_default();
                if b.is_empty() {
                    ui.label(RichText::new("Noch keine Spionageberichte. Schicke Spionagesonden mit der Mission Spionage los.").color(LEISE));
                }
                for x in b {
                    karte(ui, |ui| {
                        self.motiv(ui, "events", "spionage_bericht", v);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(format!("{} – {}", text(&x["ziel"]), text(&x["besitzer"]))).strong());
                            let alter = i(&x["alter_stunden"]);
                            etikett(ui, &format!("{} alt", dauer_h(alter)), if alter > 24 { WARNUNG } else { LEISE });
                        });
                        let bestand: Vec<String> = x["bestand"].as_object().into_iter().flatten().map(|(g, n)| format!("{} {}", gut_zeichen(g), z(n))).collect();
                        ui.label(format!("Bestand: {}", if bestand.is_empty() { "leer".into() } else { bestand.join("  ") }));
                        for (feld, titel) in [("schiffe", "Schiffe"), ("verteidigung", "Verteidigung"), ("gebaeude", "Gebäude"), ("forschung", "Forschung")] {
                            if x[feld].is_null() {
                                ui.label(RichText::new(format!("{titel}: unbekannt (mehr Sonden oder bessere Spionagetechnik)")).small().color(LEISE));
                            } else {
                                ui.label(RichText::new(format!("{titel}: {}", liste(&x[feld]))).small());
                            }
                        }
                        if ui.small_button("Als Flottenziel übernehmen").clicked() {
                            self.target = text(&x["ziel"]);
                            self.flotten_vorschau = None;
                            self.screen = Bildschirm::Flotten;
                        }
                    });
                }
            }
        }
    }

    /// Springt vom Ereignis zu seinem Gegenstand: Bericht, Vertrag, Planet, Flotte, Stufe.
    fn zu_ereignis(&mut self, v: &Value, art: &str, t: &str) {
        let koord = koordinate(t);
        if let Some(n) = koord.as_ref().and_then(|k| v["planeten"].as_array().and_then(|a| a.iter().position(|p| p["koord"] == k.as_str()))) {
            self.colony = n;
        }
        match art {
            "kampf" | "beute" | "eroberung" => self.berichte_reiter = 1,
            "spionage" => self.berichte_reiter = 2,
            "vertrag" => (self.screen, self.diplo.reiter) = (Bildschirm::Diplomatie, 2),
            "allianz" => (self.screen, self.diplo.reiter) = (Bildschirm::Diplomatie, 3),
            "geschenk" => (self.screen, self.diplo.reiter) = (Bildschirm::Diplomatie, 5),
            "markt" | "lieferung" => self.screen = Bildschirm::Markt,
            "raketen" => (self.screen, self.werft_reiter) = (Bildschirm::Werft, 3),
            "fertigung" => self.screen = Bildschirm::Werft,
            "flotte" | "blockade" => self.screen = Bildschirm::Flotten,
            "stufe" | "unterhalt" => self.screen = Bildschirm::Regierung,
            "forschung" => self.screen = Bildschirm::Forschung,
            "bau" | "abgelehnt" => self.screen = Bildschirm::Gebaeude,
            "kolonie" => self.screen = Bildschirm::Kolonie,
            _ => {
                if let Some(k) = koord {
                    self.sector = k.split(':').next().and_then(|s| s.parse().ok()).unwrap_or(self.sector);
                    self.galaxie_auswahl = Some(k);
                    self.screen = Bildschirm::Galaxie;
                }
            }
        }
    }

    pub(crate) fn rangliste(&mut self, ui: &mut egui::Ui, v: &Value) {
        let r = self.regeln.clone();
        hilfe::kopf(ui, Bildschirm::Rangliste, &r);
        let ich = text(&v["name"]);
        let ki: Vec<String> = self.session.ki_info()["reiche"].as_array().into_iter().flatten().map(|x| text(&x["name"])).collect();
        let liste = v["rangliste"].as_array().cloned().unwrap_or_default();
        if let (Some(eigen), Some(erster)) = (liste.iter().find(|e| text(&e[1]) == ich), liste.first()) {
            let (punkte, rang) = (i(&eigen[2]), i(&eigen[0]));
            let satz = if rang == 1 {
                let zweiter = liste.get(1).map(|e| i(&e[2])).unwrap_or(0);
                (format!("Du führst – {} Punkte vor {}.", zahl(punkte - zweiter), text(&liste[1][1])), GUT)
            } else {
                let vor = liste.iter().find(|e| i(&e[0]) == rang - 1).map(|e| i(&e[2]) - punkte).unwrap_or(0);
                (format!("Platz {rang}: {} Punkte bis zum nächsten Platz, {} bis zur Spitze ({}).", zahl(vor), zahl(i(&erster[2]) - punkte), text(&erster[1])), TEXT)
            };
            karte(ui, |ui| {
                ui.label(RichText::new(satz.0).size(17.0).color(satz.1));
            });
            ui.add_space(6.0);
        }
        egui::Grid::new("rangliste").striped(true).num_columns(4).spacing([24.0, 4.0]).show(ui, |ui| {
            for h in ["Rang", "Reich", "Punkte", "Stufe"] {
                ui.label(RichText::new(h).strong().color(LEISE));
            }
            ui.end_row();
            for e in v["rangliste"].as_array().into_iter().flatten() {
                let n = text(&e[1]);
                let farbe = if n == ich { GOLD } else if ki.contains(&n) { INFO } else { TEXT };
                ui.label(RichText::new(z(&e[0])).color(farbe));
                let zusatz = if n == ich { "  (du)" } else if ki.contains(&n) { "  💻" } else { "" };
                ui.label(RichText::new(format!("{n}{zusatz}")).color(farbe).strong());
                ui.label(RichText::new(z(&e[2])).color(farbe));
                ui.label(RichText::new(roemisch(i(&e[3]))).color(farbe));
                ui.end_row();
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn zeit_und_koordinate_aus_ereignissen() {
        assert_eq!(zeit_minuten("Tag 1, 00:00"), 0);
        assert_eq!(zeit_minuten("Tag 63, 11:35"), 62 * 1440 + 11 * 60 + 35);
        assert!(zeit_minuten("Tag 10, 00:00") > zeit_minuten("Tag 9, 23:59"));
        assert_eq!(koordinate("Erzmine Stufe 4 auf 1:45:4 fertig").as_deref(), Some("1:45:4"));
        assert_eq!(koordinate("Tag 63, 11:35 Flotte 6844 ist zurück"), None);
        let v = json!({"ereignisse": [{"zeit": "Tag 2, 01:00"}, {"zeit": "Tag 1, 12:00"}, {"zeit": "Tag 1, 00:15"}]});
        assert_eq!(ungelesen(&v, -1), 3);
        assert_eq!(ungelesen(&v, zeit_minuten("Tag 1, 12:00")), 1);
    }
}
