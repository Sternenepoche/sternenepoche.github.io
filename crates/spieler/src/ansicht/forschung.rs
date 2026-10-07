//! Forschung: laufendes Projekt, Warteschlange und alle Forschungen mit Wirkung, Kosten, Dauer und Laborbedarf.

use super::hilfe::{self, text};
use super::namen::{name, FORSCHUNG_GRUPPEN};
use super::befehl;
use super::stil::*;
use super::Bildschirm;
use crate::Player;
use eframe::egui::{self, RichText};
use kern::Forschung;
use serde_json::Value;

impl Player {
    pub(crate) fn forschung(&mut self, ui: &mut egui::Ui, v: &Value) {
        let r = self.regeln.clone();
        hilfe::kopf(ui, Bildschirm::Forschung, &r);
        self.freischaltungen(ui, v);
        let f = &v["forschung"];
        let heimat = v["planeten"].as_array().into_iter().flatten().find(|p| p["heimat"] == true).cloned().unwrap_or(Value::Null);
        let schlange: Vec<String> = f["schlange"].as_array().into_iter().flatten().map(text).collect();
        let max_schlange = r.wirtschaft.forschung_warteschlange;
        let schlange_voll = schlange.len() >= max_schlange && !f["aktiv"].is_null();
        karte(ui, |ui| {
            ui.horizontal(|ui| {
                kennzahl(ui, z(&f["punkte_je_stunde"]), "Forschungspunkte je Stunde", INFO)
                    .on_hover_text("Alle Labore aller Planeten zusammen. Mehr Laborstufen, Fachkräfte und das Forschungsarchiv erhöhen den Wert.");
                ui.vertical(|ui| {
                    match f["aktiv"].as_object() {
                        Some(a) => {
                            ui.label(RichText::new(format!("🔬 Läuft: {} Stufe {}", name(a["forschung"].as_str().unwrap_or("")), z(&a["stufe"]))).size(18.0).strong());
                            match a["rest_stunden"].as_i64() {
                                Some(h) => ui.label(format!("fertig etwa {}", in_zeit(h * 60))),
                                None => ui.label(RichText::new("Ohne Forschungspunkte wird sie nie fertig: baue ein Labor.").color(SCHLECHT)),
                            };
                        }
                        None => {
                            ui.label(RichText::new("Es läuft keine Forschung.").size(18.0).color(WARNUNG));
                        }
                    }
                    if !schlange.is_empty() {
                        ui.label(format!("Danach: {}", schlange.iter().map(|s| name(s)).collect::<Vec<_>>().join(", ")));
                    }
                    ui.label(RichText::new(format!("Warteschlange {} von {max_schlange}", schlange.len())).small().color(LEISE));
                });
            });
        });
        let stufe = i(&v["stufe"]) as u8;
        let moeglich: Vec<Value> = f["moeglich"].as_array().cloned().unwrap_or_default();
        for (zeichen, gruppe, liste) in FORSCHUNG_GRUPPEN {
            ui.add_space(6.0);
            egui::CollapsingHeader::new(RichText::new(format!("{zeichen}  {gruppe}")).size(18.0).strong())
                .id_salt(("forschungsgruppe", *gruppe))
                .default_open(true)
                .show(ui, |ui| {
                    for schluessel in *liste {
                        let Some(fo) = Forschung::aus_name(schluessel) else { continue };
                        let Some(regel) = r.forschung.get(&fo) else {continue;};
                        let jetzt = i(&f["stufen"][*schluessel]);
                        let eintrag = moeglich.iter().find(|m| m["forschung"] == *schluessel);
                        karte(ui, |ui| {
                            ui.horizontal(|ui| {
                                self.motiv(ui, "research", schluessel, v);
                                ui.label(RichText::new(name(schluessel)).size(17.0).strong().color(if eintrag.is_some() { TEXT } else { LEISE }));
                                if jetzt > 0 {
                                    etikett(ui, &format!("Stufe {jetzt}"), INFO);
                                }
                                if f["aktiv"]["forschung"] == *schluessel {
                                    etikett(ui, "läuft", GUT);
                                }
                                if schlange.iter().any(|s| s == schluessel) {
                                    etikett(ui, "in der Schlange", LEISE);
                                }
                                ui.label(RichText::new(&regel.wirkung).color(LEISE));
                            });
                            let Some(m) = eintrag else {
                                let grund = if regel.ab_stufe > stufe {
                                    format!("🔒 Ab Zivilisationsstufe {}", roemisch(regel.ab_stufe as i64))
                                } else {
                                    "Höchste Stufe erreicht.".into()
                                };
                                ui.label(RichText::new(grund).color(LEISE));
                                return;
                            };
                            egui::Grid::new(("forschung", *schluessel)).num_columns(2).spacing([12.0, 3.0]).show(ui, |ui| {
                                ui.label(RichText::new(format!("Stufe {}", z(&m["stufe"]))).color(LEISE));
                                kosten(ui, &m["kosten"], &heimat["bestand"], 1);
                                ui.end_row();
                                ui.label(RichText::new("Dauer").color(LEISE));
                                ui.label(m["dauer_stunden"].as_i64().map(dauer_h).unwrap_or_else(|| "ohne Labor nie".into()));
                                ui.end_row();
                                ui.label(RichText::new("Labor").color(LEISE));
                                let (labor, hat) = (i(&m["labor"]), i(&m["labor_heimat"]));
                                ui.label(RichText::new(format!("Stufe {labor} nötig, Heimatwelt hat {hat}")).color(if hat >= labor { GUT } else { SCHLECHT }));
                                ui.end_row();
                                if let Some(w) = wartezeit(&m["kosten"], &heimat, 1) {
                                    ui.label(RichText::new("Güter").color(LEISE));
                                    ui.label(RichText::new(format!("noch nicht beisammen: {w}")).color(WARNUNG));
                                    ui.end_row();
                                }
                            });
                            let (labor, hat) = (i(&m["labor"]), i(&m["labor_heimat"]));
                            let (aktiv, grund) = if hat < labor {
                                (false, format!("Die Heimatwelt braucht ein Labor der Stufe {labor}."))
                            } else if schlange_voll {
                                (false, "Die Forschungsschlange ist voll.".to_string())
                            } else {
                                (true, String::new())
                            };
                            let beschriftung = if f["aktiv"].is_null() { "Erforschen" } else { "In die Schlange" };
                            ui.horizontal(|ui| {
                                if knopf(ui, beschriftung, aktiv, &grund) {
                                    self.act(befehl::forschen(schluessel));
                                }
                                if !aktiv {
                                    ui.label(RichText::new(grund).small().color(LEISE));
                                }
                            });
                        });
                    }
                });
        }
    }
}
