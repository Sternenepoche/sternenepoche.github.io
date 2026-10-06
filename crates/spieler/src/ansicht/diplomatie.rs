//! Diplomatie: alle Reiche, Nachrichten, Verträge, Allianz, öffentliches Register und Geschenke.

use super::hilfe::{self, text};
use super::namen::{gut_zeichen, name, vertrag_text};
use super::befehl;
use super::stil::*;
use super::Bildschirm;
use crate::Player;
use eframe::egui::{self, RichText};
use kern::{Gut, Vertragsart};
use serde_json::Value;

const REITER: [&str; 6] = ["🌐 Reiche", "✉ Nachrichten", "📜 Verträge", "🕊 Allianz", "📖 Register", "🎁 Geschenke"];

/// Eingaben der Diplomatie; bleiben beim Wechsel zwischen Reitern erhalten.
pub struct DiploForm {
    pub reiter: usize,
    pub empfaenger: std::collections::BTreeSet<String>,
    pub an_allianz: bool,
    pub nachricht: String,
    pub partner: String,
    pub art: String,
    pub kaution: f64,
    pub tribut_gut: String,
    pub tribut_menge: f64,
    pub tribut_tage: i64,
    pub allianz_name: String,
    pub einladen: String,
    pub geschenk_an: String,
    pub geschenk_credits: f64,
}

impl Default for DiploForm {
    fn default() -> Self {
        Self {
            reiter: 0,
            empfaenger: Default::default(),
            an_allianz: false,
            nachricht: String::new(),
            partner: String::new(),
            art: "nichtangriffspakt".into(),
            kaution: 0.0,
            tribut_gut: "credits".into(),
            tribut_menge: 100.0,
            tribut_tage: 7,
            allianz_name: String::new(),
            einladen: String::new(),
            geschenk_an: String::new(),
            geschenk_credits: 100.0,
        }
    }
}

impl Player {
    pub(crate) fn diplomatie(&mut self, ui: &mut egui::Ui, v: &Value) {
        let r = self.regeln.clone();
        hilfe::kopf(ui, Bildschirm::Diplomatie, &r);
        let ich = text(&v["name"]);
        let reiche: Vec<(i64, String, i64, i64)> = v["rangliste"].as_array().into_iter().flatten()
            .map(|e| (i(&e[0]), text(&e[1]), i(&e[2]), i(&e[3]))).filter(|e| e.1 != ich).collect();
        let neue = v["nachrichten"].as_array().map_or(0, |n| n.iter().filter(|x| x["neu"] == true).count());
        let angebote = v["vertraege"].as_array().map_or(0, |n| n.iter().filter(|x| x["status"] == "angebot an dich").count());
        ui.horizontal(|ui| {
            for (n, t) in REITER.iter().enumerate() {
                let zusatz = match n {
                    1 if neue > 0 => format!(" ({neue})"),
                    2 if angebote > 0 => format!(" ({angebote})"),
                    3 if v["einladungen"].as_array().is_some_and(|e| !e.is_empty()) => " (!)".into(),
                    _ => String::new(),
                };
                if ui.selectable_label(self.diplo.reiter == n, RichText::new(format!("{t}{zusatz}")).size(16.5)).clicked() {
                    self.diplo.reiter = n;
                }
            }
        });
        ui.separator();
        self.motiv(ui, "diplomacy", match self.diplo.reiter { 3 => "allianz", 2 | 5 => "kaution", _ => "botschaft" }, v);
        match self.diplo.reiter {
            0 => self.diplo_reiche(ui, v, &reiche),
            1 => self.diplo_nachrichten(ui, v, &reiche, &r),
            2 => self.diplo_vertraege(ui, v, &reiche, &r),
            3 => self.diplo_allianz(ui, v, &reiche, &r),
            4 => {
                ui.label(RichText::new("Jeder Vertrag, jede Kündigung und jeder Bruch steht hier für alle sichtbar. Neueste zuerst.").color(LEISE));
                egui::Grid::new("register").striped(true).num_columns(3).spacing([16.0, 4.0]).show(ui, |ui| {
                    for e in v["register"].as_array().into_iter().flatten() {
                        ui.label(RichText::new(text(&e["zeit"])).color(LEISE));
                        ui.label(format!("{}: {} und {}", name(e["art"].as_str().unwrap_or("")), text(&e["a"]), text(&e["b"])));
                        let vorgang = text(&e["vorgang"]);
                        ui.label(RichText::new(&vorgang).color(if vorgang.contains("gebrochen") { SCHLECHT } else { TEXT }));
                        ui.end_row();
                    }
                });
            }
            _ => self.diplo_geschenke(ui, v, &reiche),
        }
    }

    fn diplo_reiche(&mut self, ui: &mut egui::Ui, v: &Value, reiche: &[(i64, String, i64, i64)]) {
        let nachbarn: Vec<String> = v["nachbarn"].as_array().into_iter().flatten().map(|n| text(&n["spieler"])).collect();
        let vertraege = v["vertraege"].as_array().cloned().unwrap_or_default();
        let allianz: Vec<String> = v["allianz"]["mitglieder"].as_array().into_iter().flatten().map(text).collect();
        ui.label(RichText::new("Alle anderen Reiche nach Rang. Nachbarn sind die, deren Planeten nah an deinen liegen.").color(LEISE));
        egui::Grid::new("reiche").striped(true).num_columns(6).spacing([14.0, 4.0]).show(ui, |ui| {
            for h in ["Rang", "Reich", "Punkte", "Stufe", "Beziehung", ""] {
                ui.label(RichText::new(h).strong().color(LEISE));
            }
            ui.end_row();
            for (rang, n, punkte, stufe) in reiche {
                ui.label(rang.to_string());
                ui.label(RichText::new(n).strong());
                ui.label(zahl(*punkte));
                ui.label(roemisch(*stufe));
                let mut bez: Vec<String> = vertraege.iter().filter(|x| x["partner"] == n.as_str()).map(|x| format!("{} ({})", name(x["art"].as_str().unwrap_or("")), text(&x["status"]))).collect();
                if allianz.contains(n) {
                    bez.insert(0, "Allianz".into());
                }
                if nachbarn.contains(n) {
                    bez.insert(0, "Nachbar".into());
                }
                ui.label(RichText::new(bez.join(", ")).small().color(LEISE));
                ui.horizontal(|ui| {
                    if ui.small_button("✉").on_hover_text("Nachricht schreiben").clicked() {
                        self.diplo.empfaenger.clear();
                        self.diplo.empfaenger.insert(n.clone());
                        self.diplo.an_allianz = false;
                        self.diplo.reiter = 1;
                    }
                    if ui.small_button("📜").on_hover_text("Vertrag anbieten").clicked() {
                        self.diplo.partner = n.clone();
                        self.diplo.reiter = 2;
                    }
                    if ui.small_button("🎁").on_hover_text("Credits schenken").clicked() {
                        self.diplo.geschenk_an = n.clone();
                        self.diplo.reiter = 5;
                    }
                });
                ui.end_row();
            }
        });
    }

    fn diplo_nachrichten(&mut self, ui: &mut egui::Ui, v: &Value, reiche: &[(i64, String, i64, i64)], r: &kern::Regelwerk) {
        let max = r.agenten.nachricht_zeichen;
        karte(ui, |ui| {
            ui.label(RichText::new("Neue Nachricht").size(18.0).strong());
            ui.horizontal_wrapped(|ui| {
                ui.label("An:");
                if v["allianz"].is_object() {
                    ui.checkbox(&mut self.diplo.an_allianz, "meine Allianz");
                }
                let gewaehlt: Vec<String> = self.diplo.empfaenger.iter().cloned().collect();
                for e in gewaehlt {
                    if ui.button(format!("{e} ✖")).on_hover_text("entfernen").clicked() {
                        self.diplo.empfaenger.remove(&e);
                    }
                }
                egui::ComboBox::from_id_salt("empfaenger").selected_text("Reich hinzufügen …").show_ui(ui, |ui| {
                    for (_, n, _, _) in reiche {
                        if ui.selectable_label(self.diplo.empfaenger.contains(n), n).clicked() {
                            self.diplo.empfaenger.insert(n.clone());
                        }
                    }
                });
            });
            ui.add(egui::TextEdit::multiline(&mut self.diplo.nachricht).desired_rows(4).desired_width(f32::INFINITY).hint_text("Freier Text, etwa ein Angebot oder eine Warnung."));
            let laenge = self.diplo.nachricht.chars().count();
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("{laenge} von {max} Zeichen · höchstens {} Nachrichten je Spieltag", r.agenten.nachrichten_je_tag)).small().color(if laenge > max { SCHLECHT } else { LEISE }));
            });
            let ok = !self.diplo.nachricht.trim().is_empty() && laenge <= max && (self.diplo.an_allianz || !self.diplo.empfaenger.is_empty());
            if knopf(ui, "Senden", ok, "Text und mindestens einen Empfänger angeben.") {
                let an: Vec<String> = self.diplo.empfaenger.iter().cloned().collect();
                self.act(befehl::nachricht(&an, self.diplo.an_allianz, &self.diplo.nachricht));
                if self.status_ok {
                    self.diplo.nachricht.clear();
                }
            }
        });
        ui.add_space(6.0);
        titel(ui, "Posteingang und Gesendetes (letzte 7 Tage)");
        let nachrichten = v["nachrichten"].as_array().cloned().unwrap_or_default();
        if nachrichten.is_empty() {
            ui.label(RichText::new("Noch keine Nachrichten.").color(LEISE));
        }
        let ich = text(&v["name"]);
        for n in nachrichten {
            karte(ui, |ui| {
                ui.horizontal(|ui| {
                    if n["neu"] == true {
                        etikett(ui, "neu", GOLD);
                    }
                    let von = text(&n["von"]);
                    let an = if n["allianz"] == true { "Allianz".to_string() } else { n["an"].as_array().into_iter().flatten().map(text).collect::<Vec<_>>().join(", ") };
                    ui.label(RichText::new(if von == ich { format!("Du an {an}") } else { format!("{von} an {an}") }).strong());
                    ui.label(RichText::new(text(&n["zeit"])).small().color(LEISE));
                    if von != ich && ui.small_button("Antworten").clicked() {
                        self.diplo.empfaenger.clear();
                        self.diplo.empfaenger.insert(von);
                    }
                });
                ui.label(text(&n["text"]));
            });
        }
    }

    fn diplo_vertraege(&mut self, ui: &mut egui::Ui, v: &Value, reiche: &[(i64, String, i64, i64)], r: &kern::Regelwerk) {
        titel(ui, "Deine Verträge");
        let vertraege = v["vertraege"].as_array().cloned().unwrap_or_default();
        if vertraege.is_empty() {
            ui.label(RichText::new("Keine laufenden Verträge oder Angebote.").color(LEISE));
        }
        for vt in &vertraege {
            karte(ui, |ui| {
                let status = text(&vt["status"]);
                ui.horizontal(|ui| {
                    self.motiv(ui, "contracts", vt["art"].as_str().unwrap_or(""), v);
                    ui.label(RichText::new(format!("{} mit {}", name(vt["art"].as_str().unwrap_or("")), text(&vt["partner"]))).strong());
                    etikett(ui, &status, if status == "aktiv" { GUT } else if status.starts_with("angebot") { GOLD } else { LEISE });
                    if i(&vt["kaution"]) > 0 {
                        ui.label(RichText::new(format!("Kaution {} Credits je Seite", z(&vt["kaution"]))).small().color(LEISE));
                    }
                    if let Some(t) = vt["tribut"].as_object() {
                        ui.label(RichText::new(format!("{} zahlt {} {} je Tag", text(&t["zahler"]), z(&t["menge_je_tag"]), name(t["gut"].as_str().unwrap_or("")))).small());
                    }
                });
                ui.horizontal(|ui| {
                    let id = vt["vertrag"].clone();
                    if status == "angebot an dich" {
                        if hauptknopf(ui, "Annehmen", true) {
                            self.act(befehl::vertrag_annehmen(&id));
                        }
                        if ui.button("Ablehnen").clicked() {
                            self.act(befehl::vertrag_ablehnen(&id));
                        }
                    } else if status == "von dir angeboten" {
                        if ui.button("Angebot zurückziehen").clicked() {
                            self.act(befehl::vertrag_ablehnen(&id));
                        }
                    } else if status == "aktiv" && ui.button("Kündigen …").clicked() {
                        let tribut = vt["art"] == "tribut";
                        self.bestaetigen(
                            &format!("{} mit {} kündigen?{}", name(vt["art"].as_str().unwrap_or("")), text(&vt["partner"]),
                                if tribut { " Stellt der Zahler einen Tribut vorzeitig ein, ist das ein Bruch: die Kautionen gehen an den Partner." } else { " Es gilt die Kündigungsfrist." }),
                            befehl::vertrag_kuendigen(&id),
                        );
                    }
                });
            });
        }
        ui.add_space(6.0);
        karte(ui, |ui| {
            ui.label(RichText::new("Vertrag anbieten").size(18.0).strong());
            ui.horizontal(|ui| {
                ui.label("Partner");
                egui::ComboBox::from_id_salt("partner").selected_text(if self.diplo.partner.is_empty() { "wählen …".to_string() } else { self.diplo.partner.clone() }).show_ui(ui, |ui| {
                    for (_, n, _, _) in reiche {
                        ui.selectable_value(&mut self.diplo.partner, n.clone(), n);
                    }
                });
            });
            ui.horizontal_wrapped(|ui| {
                for a in Vertragsart::ALLE {
                    ui.selectable_value(&mut self.diplo.art, a.name().to_string(), name(a.name()));
                }
            });
            ui.label(RichText::new(vertrag_text(&self.diplo.art)).color(LEISE));
            ui.horizontal(|ui| {
                ui.label("Kaution");
                ui.add(egui::DragValue::new(&mut self.diplo.kaution).range(0.0..=1e9).speed(10.0).suffix(" Credits"));
                ui.label(RichText::new("beide Seiten hinterlegen sie; bei einem Bruch bekommt der Geschädigte beide").small().color(LEISE));
            });
            if self.diplo.art == "tribut" {
                ui.horizontal(|ui| {
                    ui.label("Du zahlst täglich");
                    ui.add(egui::DragValue::new(&mut self.diplo.tribut_menge).range(1.0..=1e9).speed(10.0));
                    egui::ComboBox::from_id_salt("tributgut").selected_text(name(&self.diplo.tribut_gut)).show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.diplo.tribut_gut, "credits".to_string(), "💰 Credits");
                        for g in Gut::ALLE {
                            ui.selectable_value(&mut self.diplo.tribut_gut, g.name().to_string(), format!("{} {}", gut_zeichen(g.name()), name(g.name())));
                        }
                    });
                    ui.label("für");
                    ui.add(egui::DragValue::new(&mut self.diplo.tribut_tage).range(1..=365).suffix(" Tage"));
                });
            }
            let credits = v["credits"].as_f64().unwrap_or(0.0);
            let ok = !self.diplo.partner.is_empty() && self.diplo.kaution <= credits;
            let grund = if self.diplo.partner.is_empty() { "Erst einen Partner wählen." } else { "Die Kaution übersteigt deine Credits." };
            if knopf(ui, "Angebot senden", ok, grund) {
                let tribut = self.diplo.art == "tribut";
                let gut = (self.diplo.tribut_gut != "credits").then_some(self.diplo.tribut_gut.as_str());
                let b = befehl::vertrag_anbieten(&self.diplo.partner, &self.diplo.art, self.diplo.kaution,
                    tribut.then_some((gut, self.diplo.tribut_menge, self.diplo.tribut_tage)));
                self.act(b);
            }
            ui.label(RichText::new(format!("Höchstens {} Verteidigungsbündnisse je Reich.", r.diplomatie.buendnisse_max)).small().color(LEISE));
        });
    }

    fn diplo_allianz(&mut self, ui: &mut egui::Ui, v: &Value, reiche: &[(i64, String, i64, i64)], r: &kern::Regelwerk) {
        match v["allianz"].as_object() {
            Some(a) => {
                karte(ui, |ui| {
                    ui.label(RichText::new(format!("🕊 Allianz {}", text(&a["name"]))).size(19.0).strong().color(GOLD));
                    let mitglieder: Vec<String> = a["mitglieder"].as_array().into_iter().flatten().map(text).collect();
                    ui.label(format!("{} von {} Mitgliedern: {}", mitglieder.len(), r.diplomatie.allianz_max, mitglieder.join(", ")));
                    ui.horizontal(|ui| {
                        egui::ComboBox::from_id_salt("einladen").selected_text(if self.diplo.einladen.is_empty() { "Reich einladen …".to_string() } else { self.diplo.einladen.clone() }).show_ui(ui, |ui| {
                            for (_, n, _, _) in reiche.iter().filter(|x| !mitglieder.contains(&x.1)) {
                                ui.selectable_value(&mut self.diplo.einladen, n.clone(), n);
                            }
                        });
                        if knopf(ui, "Einladen", !self.diplo.einladen.is_empty(), "Erst ein Reich wählen.") {
                            self.act(befehl::allianz_einladen(&self.diplo.einladen));
                        }
                    });
                    if ui.button("Allianz verlassen …").clicked() {
                        self.bestaetigen("Die Allianz verlassen? Ihr gemeinsamer Kanal und der Schutz als Verbündete enden.", befehl::allianz_verlassen());
                    }
                });
            }
            None => {
                karte(ui, |ui| {
                    ui.label(RichText::new("Du bist in keiner Allianz.").size(18.0));
                    ui.horizontal(|ui| {
                        ui.label("Name");
                        ui.add(egui::TextEdit::singleline(&mut self.diplo.allianz_name).desired_width(220.0).hint_text("höchstens 30 Zeichen"));
                        let ok = !self.diplo.allianz_name.trim().is_empty() && self.diplo.allianz_name.chars().count() <= 30;
                        if knopf(ui, "Allianz gründen", ok, "1 bis 30 Zeichen.") {
                            self.act(befehl::allianz_gruenden(self.diplo.allianz_name.trim()));
                        }
                    });
                });
            }
        }
        for e in v["einladungen"].as_array().into_iter().flatten() {
            hinweis_karte(ui, GOLD, |ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("Einladung in die Allianz {}", text(e)));
                    if ui.button("Beitreten").clicked() {
                        self.act(befehl::allianz_beitreten(&text(e)));
                    }
                });
            });
        }
    }

    fn diplo_geschenke(&mut self, ui: &mut egui::Ui, v: &Value, reiche: &[(i64, String, i64, i64)]) {
        karte(ui, |ui| {
            ui.label(RichText::new("🎁 Credits schenken").size(18.0).strong());
            ui.label(RichText::new("Güter verschenkst du mit einem Transport zu einem Planeten des anderen Reichs (✈ Flotten).").color(LEISE));
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_salt("geschenk").selected_text(if self.diplo.geschenk_an.is_empty() { "Reich wählen …".to_string() } else { self.diplo.geschenk_an.clone() }).show_ui(ui, |ui| {
                    for (_, n, _, _) in reiche {
                        ui.selectable_value(&mut self.diplo.geschenk_an, n.clone(), n);
                    }
                });
                ui.add(egui::DragValue::new(&mut self.diplo.geschenk_credits).range(1.0..=1e9).speed(10.0).suffix(" Credits"));
            });
            let credits = v["credits"].as_f64().unwrap_or(0.0);
            let ok = !self.diplo.geschenk_an.is_empty() && self.diplo.geschenk_credits <= credits;
            if knopf(ui, "Schenken", ok, "Empfänger wählen; der Betrag darf deine Credits nicht übersteigen.") {
                self.act(befehl::schenken(&self.diplo.geschenk_an, self.diplo.geschenk_credits));
            }
        });
    }
}
