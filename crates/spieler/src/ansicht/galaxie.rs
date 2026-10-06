//! Galaxiekarte: ein Sektor als Raster aus Systemen (Spalten) und Plätzen (Zeilen). Daten kommen aus dem
//! öffentlichen Werkzeug `galaxie` der Engine; ein Klick wählt einen Platz als Ziel.

use super::hilfe::{self, text};
use super::namen::name;
use super::stil::*;
use super::Bildschirm;
use crate::Player;
use eframe::egui::{self, Color32, RichText, Sense, Stroke};
use serde_json::{json, Value};

const PLAETZE: u8 = 12;

impl Player {
    /// Systeme eines Sektors, einmal je Spieltag neu gelesen (das Werkzeug liefert höchstens 20 je Anfrage).
    fn sektor_daten(&mut self, sektor: u8, systeme: u8) -> Vec<Value> {
        let tag = self.session.seconds() / 86_400;
        if let Some((t, d)) = self.galaxie_cache.get(&sektor) {
            if *t == tag {
                return d.clone();
            }
        }
        let mut alle = Vec::new();
        let mut von = 1u8;
        while von <= systeme {
            let bis = von.saturating_add(19).min(systeme);
            if let Ok(a) = self.session.query(&json!({"typ": "galaxie", "sektor": sektor, "von": von, "bis": bis})) {
                alle.extend(a["systeme"].as_array().cloned().unwrap_or_default());
            }
            von = bis.saturating_add(1);
            if bis == systeme {
                break;
            }
        }
        self.galaxie_cache.insert(sektor, (tag, alle.clone()));
        alle
    }

    pub(crate) fn galaxie(&mut self, ui: &mut egui::Ui, v: &Value) {
        let r = self.regeln.clone();
        hilfe::kopf(ui, Bildschirm::Galaxie, &r);
        let sektoren = r.welt.sektoren;
        let systeme_je = r.welt.systeme_je_sektor;
        ui.horizontal(|ui| {
            for s in 1..=sektoren {
                if ui.selectable_label(self.sector == s, RichText::new(format!("Sektor {s}")).size(17.0)).clicked() {
                    self.sector = s;
                }
            }
            if ui.button("Neu einlesen").clicked() {
                self.galaxie_cache.clear();
            }
        });
        let daten = self.sektor_daten(self.sector, systeme_je);
        let eigene: Vec<String> = v["planeten"].as_array().into_iter().flatten().map(|p| text(&p["koord"])).collect();
        let max_punkte = daten.iter().flat_map(|s| s["belegt"].as_array().cloned().unwrap_or_default()).map(|b| i(&b["punkte"])).max().unwrap_or(1).max(1);
        let zelle = ((ui.available_width() - 40.0) / systeme_je as f32).clamp(10.0, 22.0);
        let (rect, antwort) = ui.allocate_exact_size(egui::vec2(40.0 + zelle * systeme_je as f32, 24.0 + zelle * PLAETZE as f32 + 18.0), Sense::click());
        let maler = ui.painter_at(rect);
        let ursprung = rect.min + egui::vec2(40.0, 20.0);
        let mut ueber: Option<(u8, u8)> = None;
        if let Some(pos) = antwort.hover_pos() {
            let d = pos - ursprung;
            if d.x >= 0.0 && d.y >= 0.0 {
                let (sys, platz) = ((d.x / zelle) as u8 + 1, (d.y / zelle) as u8 + 1);
                if sys <= systeme_je && platz <= PLAETZE {
                    ueber = Some((sys, platz));
                }
            }
        }
        for platz in [1u8, 6, 12] {
            maler.text(ursprung + egui::vec2(-8.0, (platz as f32 - 0.5) * zelle), egui::Align2::RIGHT_CENTER, platz.to_string(), egui::FontId::proportional(11.0), LEISE);
        }
        for s in &daten {
            let sys = i(&s["system"]) as u8;
            let x = ursprung.x + (sys as f32 - 1.0) * zelle;
            if sys % 10 == 0 {
                maler.text(egui::pos2(x + zelle / 2.0, rect.min.y + 9.0), egui::Align2::CENTER_CENTER, sys.to_string(), egui::FontId::proportional(11.0), LEISE);
            }
            if s["nebel"] == true {
                maler.rect_filled(egui::Rect::from_min_size(egui::pos2(x, ursprung.y), egui::vec2(zelle, zelle * PLAETZE as f32)), 2.0, Color32::from_rgba_unmultiplied(140, 90, 200, 40));
            }
            let unten = ursprung.y + zelle * PLAETZE as f32 + 9.0;
            if s["nebel"] == true {
                maler.text(egui::pos2(x + zelle / 2.0, unten), egui::Align2::CENTER_CENTER, "N", egui::FontId::proportional(11.0), Color32::from_rgb(180, 140, 230));
            } else if s["asteroidenguertel"] == true {
                maler.text(egui::pos2(x + zelle / 2.0, unten), egui::Align2::CENTER_CENTER, "A", egui::FontId::proportional(11.0), WARNUNG);
            }
            for platz in 1..=PLAETZE {
                let r2 = egui::Rect::from_min_size(egui::pos2(x + 1.0, ursprung.y + (platz as f32 - 1.0) * zelle + 1.0), egui::vec2(zelle - 2.0, zelle - 2.0));
                let koord = format!("{}:{sys}:{platz}", self.sector);
                let belegt = s["belegt"].as_array().and_then(|b| b.iter().find(|x| i(&x["position"]) == platz as i64));
                let farbe = match belegt {
                    Some(_) if eigene.contains(&koord) => GOLD,
                    Some(b) => {
                        let a = (i(&b["punkte"]) as f32 / max_punkte as f32).sqrt();
                        Color32::from_rgb((40.0 + 40.0 * a) as u8, (90.0 + 80.0 * a) as u8, (150.0 + 100.0 * a) as u8)
                    }
                    None => Color32::TRANSPARENT,
                };
                if belegt.is_some() {
                    maler.rect_filled(r2, 3.0, farbe);
                } else {
                    maler.rect_stroke(r2, 3.0, Stroke::new(1.0_f32, Color32::from_rgb(36, 54, 72)), egui::StrokeKind::Inside);
                }
                if self.galaxie_auswahl.as_deref() == Some(koord.as_str()) || ueber == Some((sys, platz)) {
                    maler.rect_stroke(r2.expand(1.0), 3.0, Stroke::new(2.0_f32, Color32::WHITE), egui::StrokeKind::Outside);
                }
            }
        }
        let sektor = self.sector;
        let beschreibung = |sys: u8, platz: u8| -> String {
            let koord = format!("{sektor}:{sys}:{platz}");
            let s = daten.iter().find(|s| i(&s["system"]) == sys as i64);
            let besitz = s.and_then(|s| s["belegt"].as_array().and_then(|b| b.iter().find(|x| i(&x["position"]) == platz as i64)).cloned());
            let mut t = match &besitz {
                Some(b) => format!("{koord}: {} ({} Punkte)", text(&b["spieler"]), z(&b["punkte"])),
                None => format!("{koord}: frei"),
            };
            if s.is_some_and(|s| s["nebel"] == true) {
                t.push_str(" · Nebel");
            }
            if s.is_some_and(|s| s["asteroidenguertel"] == true) {
                t.push_str(" · Asteroidengürtel");
            }
            t
        };
        if let Some((sys, platz)) = ueber {
            antwort.clone().on_hover_text(beschreibung(sys, platz));
            if antwort.clicked() {
                self.galaxie_auswahl = Some(format!("{sektor}:{sys}:{platz}"));
            }
        }
        ui.horizontal_wrapped(|ui| {
            etikett(ui, "dein Reich", GOLD);
            etikett(ui, "fremdes Reich (heller = mehr Punkte)", INFO);
            etikett(ui, "N Nebel", Color32::from_rgb(180, 140, 230));
            etikett(ui, "A Asteroidengürtel", WARNUNG);
        });
        let Some(auswahl) = self.galaxie_auswahl.clone() else {
            ui.label(RichText::new("Klicke einen Platz an, um ihn auszuwählen. Freie Plätze nahe deiner Heimat sind gute Kolonieziele: kurze Flüge, schnelle Hilfe im Ernstfall.").color(LEISE));
            return;
        };
        let eigen = eigene.contains(&auswahl);
        let mut teile = auswahl.split(':').map(|x| x.parse::<u8>().unwrap_or(0));
        let (_, sys, platz) = (teile.next().unwrap_or(0), teile.next().unwrap_or(0), teile.next().unwrap_or(0));
        let frei = !daten.iter().any(|s| i(&s["system"]) == sys as i64 && s["belegt"].as_array().is_some_and(|b| b.iter().any(|x| i(&x["position"]) == platz as i64)));
        karte(ui, |ui| {
            ui.label(RichText::new(beschreibung(sys, platz)).size(18.0).strong());
            if let Some(system) = daten.iter().find(|s| i(&s["system"]) == sys as i64) {
                ui.horizontal(|ui| {
                    if system["nebel"] == true { self.motiv(ui, "celestials", "nebelsystem", v); }
                    if system["asteroidenguertel"] == true { self.motiv(ui, "celestials", "asteroidenguertel", v); }
                });
            }
            if eigen {
                ui.label(RichText::new("Dein eigener Planet.").color(GOLD));
            }
            if let Some(e) = v["erkundet"].as_array().into_iter().flatten().find(|e| e["koord"] == auswahl) {
                ui.label(format!("Erkundet: {} Felder, {}, Erz {} %, Kristall {} %", z(&e["felder"]), name(e["zone"].as_str().unwrap_or("")),
                    i(&e["reich_erz"]) / 10, i(&e["reich_kristall"]) / 10));
            } else if frei {
                ui.label(RichText::new("Noch nicht erkundet: Eine Spionagesonde zeigt Felder, Zone und Rohstoffreichtum.").color(LEISE));
            }
            let mut zur_kolonie = false;
            ui.horizontal_wrapped(|ui| {
                if eigen {
                    zur_kolonie = ui.button("🌍 Kolonie ansehen").clicked();
                }
                let mut ziel = |ui: &mut egui::Ui, text: &str, mission: &str| {
                    if ui.button(text).clicked() {
                        self.target = auswahl.clone();
                        self.mission = mission.to_string();
                        self.flotten_vorschau = None;
                        self.screen = Bildschirm::Flotten;
                    }
                };
                if eigen {
                    ziel(ui, "📦 Waren hinbringen", "transport");
                    ziel(ui, "⚓ Schiffe stationieren", "stationieren");
                } else if frei {
                    ziel(ui, "✈ Spionieren", "spionage");
                    ziel(ui, "🏠 Kolonisieren", "kolonisieren");
                } else {
                    ziel(ui, "✈ Spionieren", "spionage");
                    ziel(ui, "📦 Transport", "transport");
                    ziel(ui, "⚔ Angreifen", "angriff");
                }
            });
            if zur_kolonie {
                if let Some(n) = v["planeten"].as_array().and_then(|a| a.iter().position(|p| p["koord"] == auswahl.as_str())) {
                    self.colony = n;
                }
                self.screen = Bildschirm::Kolonie;
            }
        });
    }
}
