//! Übersicht (Startbildschirm) und Spielanleitung.

use super::hilfe::{self, text};
use super::namen::name;
use super::stil::*;
use super::Bildschirm;
use crate::Player;
use eframe::egui::{self, RichText};
use serde_json::Value;

impl Player {
    pub(crate) fn uebersicht(&mut self, ui: &mut egui::Ui, v: &Value) {
        let r = self.regeln.clone();
        hilfe::kopf(ui, Bildschirm::Uebersicht, &r);
        ui.horizontal(|ui| {
            let volk = text(&v["volk"]).to_lowercase();
            self.motiv(ui, "portraits", &volk, v);
            ui.vertical(|ui| {
                ui.strong("Menschenmodus · jederzeit handeln");
                ui.label("Deine Befehle werden sofort geprüft. Bau, Forschung und Flüge brauchen Spielzeit.");
                ui.label("KI-Reiche entscheiden alle 15 Spielminuten. Während sie denken, kannst du weiter Befehle geben.");
                ui.weak("Die Weltuhr wartet auf laufende KI-Antworten; ihr teilt dieselben Regeln und denselben Spielstand.");
            });
        });
        // In den ersten Tagen offen, danach eingeklappt; „Ausblenden“ entfernt die Karte ganz.
        if !self.willkommen_aus {
            if i(&v["tag"]) <= 3 {
                if hilfe::willkommen(ui, v, &r) {
                    self.willkommen_aus = true;
                }
            } else {
                egui::CollapsingHeader::new(RichText::new("👋 Willkommen und erste Schritte").color(GOLD)).id_salt("willkommen").show(ui, |ui| {
                    if hilfe::willkommen(ui, v, &r) {
                        self.willkommen_aus = true;
                    }
                });
            }
        }
        ui.add_space(6.0);
        let planeten = v["planeten"].as_array().cloned().unwrap_or_default();
        let einwohner: i64 = planeten.iter().map(|p| i(&p["bevoelkerung"])).sum();
        kennzahlen(ui, &[
            (z(&v["punkte"]["gesamt"]), "Punkte", GOLD),
            (format!("{} / {}", z(&v["rang"]), z(&v["spielerzahl"])), "Rang", TEXT),
            (roemisch(i(&v["stufe"])).to_string(), "Zivilisationsstufe", TEXT),
            (zahl(einwohner), "Einwohner", TEXT),
            (format!("{} / {}", z(&v["kolonien"]["anzahl"]), z(&v["kolonien"]["erlaubt"])), "Kolonien", TEXT),
            (z(&v["credits"]), "Credits", TEXT),
            (format!("{} / {}", z(&v["flottenplaetze"]["belegt"]), z(&v["flottenplaetze"]["gesamt"])), "Flotten unterwegs", TEXT),
        ]);
        ui.add_space(6.0);
        titel(ui, "Was jetzt ansteht");
        let hinweise = hilfe::hinweise(v, &r);
        if hinweise.is_empty() {
            hinweis_karte(ui, GUT, |ui| {
                ui.label("Alles im Lot. Lass die Zeit laufen, baue weiter aus und schau dich in der Galaxie um.");
            });
        }
        for h in hinweise {
            hinweis_karte(ui, h.farbe, |ui| {
                ui.horizontal_wrapped(|ui| {
                    if ui.button(format!("{} {}", h.ziel.zeichen(), h.ziel.name())).on_hover_text("dorthin wechseln").clicked() {
                        self.screen = h.ziel;
                    }
                    ui.label(&h.text);
                });
            });
        }
        ui.add_space(6.0);
        titel(ui, "Deine Planeten");
        for (n, p) in planeten.iter().enumerate() {
            karte(ui, |ui| {
                ui.horizontal(|ui| {
                    let art = if p["heimat"] == true { "🏠 Heimatwelt" } else { "🌍 Kolonie" };
                    if ui.link(RichText::new(format!("{art} {}", text(&p["koord"]))).strong().size(17.0)).clicked() {
                        self.colony = n;
                        self.screen = Bildschirm::Kolonie;
                    }
                    ui.label(RichText::new(name(p["zone"].as_str().unwrap_or(""))).color(LEISE));
                });
                ui.horizontal_wrapped(|ui| {
                    let (e, ver) = (i(&p["energie"]["erzeugung"]), i(&p["energie"]["verbrauch"]));
                    ui.label(format!("👥 {}", z(&p["bevoelkerung"])));
                    ui.label(RichText::new(format!("⚡ {} / {}", zahl(e), zahl(ver))).color(if ver > e { SCHLECHT } else { GUT }));
                    let st = i(&p["stabilitaet"]);
                    ui.label(RichText::new(format!("Stabilität {st}")).color(if st < 30 { SCHLECHT } else if st < 60 { WARNUNG } else { GUT }));
                    match p["bauschleife"].as_array().and_then(|s| s.first()) {
                        Some(a) if a["wartet"] == true => {
                            ui.label(RichText::new(format!("🔨 {} wartet auf Güter", name(a["gebaeude"].as_str().unwrap_or("")))).color(WARNUNG));
                        }
                        Some(a) => {
                            ui.label(format!("🔨 {} Stufe {} fertig {}", name(a["gebaeude"].as_str().unwrap_or("")), z(&a["stufe"]), in_zeit(i(&a["rest_min"]))));
                        }
                        None => {
                            ui.label(RichText::new("🔨 Bauschleife leer").color(WARNUNG));
                        }
                    }
                });
            });
        }
        let ereignisse = v["ereignisse"].as_array().cloned().unwrap_or_default();
        if !ereignisse.is_empty() {
            ui.add_space(6.0);
            titel(ui, "Zuletzt geschehen");
            for e in ereignisse.iter().take(6) {
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new(text(&e["zeit"])).color(LEISE));
                    ui.label(super::namen::woerter(&text(&e["text"])));
                });
            }
            if ui.link("Alle Ereignisse ›").clicked() {
                self.screen = Bildschirm::Berichte;
                self.berichte_reiter = 0;
            }
        }
    }

    pub(crate) fn anleitung(&mut self, ui: &mut egui::Ui, v: &Value) {
        let r = self.regeln.clone();
        hilfe::kopf(ui, Bildschirm::Anleitung, &r);
        karte(ui, |ui| {
            ui.label(RichText::new("Das Spiel in fünf Sätzen").size(18.0).strong().color(GOLD));
            for s in [
                "Du regierst ein Reich in einer Galaxie mit vielen anderen; nach 365 Spieltagen gewinnt die höchste Punktzahl.",
                "Bevölkerung arbeitet in Minen, Kraftwerken und Werken; daraus baust du Gebäude, Forschung, Schiffe und Verteidigung.",
                "Fünf Zivilisationsstufen schalten nach und nach Neues frei: von der Gründung über Industrie, Orbit und Sternenflug bis zum Imperium.",
                "Kolonien, Handel, Verträge und Allianzen bringen dich weiter – Raub und Eroberung auch, aber andere können sich wehren.",
                "Die Zeit läuft in Fenstern von 15 Spielminuten; alle Reiche handeln in denselben Fenstern, auch die Modellreiche.",
            ] {
                ui.label(format!("•  {s}"));
            }
        });
        ui.add_space(6.0);
        if hilfe::willkommen(ui, v, &r) {
            self.willkommen_aus = true;
        }
        ui.add_space(6.0);
        titel(ui, "Alle Regeln");
        ui.horizontal(|ui| {
            ui.label("🔍");
            ui.add(egui::TextEdit::singleline(&mut self.anleitung_suche).hint_text("Stichwort, etwa Kolonie, Bunker, Tribut").desired_width(320.0));
            if !self.anleitung_suche.is_empty() && ui.small_button("✖").clicked() {
                self.anleitung_suche.clear();
            }
        });
        let suche = self.anleitung_suche.to_lowercase();
        let mut treffer = 0;
        if i(&v["kolonisationsversion"]) > 0 {
            egui::CollapsingHeader::new("Aktuelle Kolonisation, Eroberung und Versorgung")
                .default_open(true).show(ui, |ui| {
                    if let Ok(current) = self.session.query(&serde_json::json!({"typ":"regel", "stichwort":"kolonie"})) {
                        // Use the version selected by this save, not a separate hardcoded rules copy.
                        ui.label(current["text"].as_str().unwrap_or(""));
                    }
                });
        }
        for a in kern::regeltext::abschnitte(&r) {
            if i(&v["kolonisationsversion"]) > 0 && ["kolonisierung", "kolonisation", "eroberung"].contains(&a.stichwort) { continue; }
            if !suche.is_empty() && !a.titel.to_lowercase().contains(&suche) && !a.text.to_lowercase().contains(&suche) {
                continue;
            }
            treffer += 1;
            egui::CollapsingHeader::new(RichText::new(a.titel).size(17.0).strong())
                .id_salt(("regel", a.stichwort))
                .default_open(!suche.is_empty())
                .show(ui, |ui| {
                    for absatz in a.text.lines() {
                        if absatz.trim().is_empty() {
                            continue;
                        }
                        ui.label(absatz);
                    }
                });
        }
        if treffer == 0 {
            ui.label(RichText::new("Kein Abschnitt passt zu diesem Stichwort.").color(LEISE));
        }
        ui.add_space(6.0);
        titel(ui, "Zahlen im Detail");
        ui.label(RichText::new("Alle Werte des Regelwerks als Tabellen stehen in docs/REGELWERK.md im Projektordner.").color(LEISE));
    }
}
