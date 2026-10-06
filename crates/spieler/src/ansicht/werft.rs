//! Werft: Schiffe, Verteidigung, Bauteile und Raketen, je mit Rolle, Kampfwerten, Kosten für die gewählte Menge
//! und Voraussetzungen; dazu die laufende Fertigung.

use super::hilfe::{self, text};
use super::namen::name;
use super::befehl;
use super::stil::*;
use super::Bildschirm;
use crate::Player;
use eframe::egui::{self, RichText};
use kern::{Einheit, Gut};
use serde_json::Value;

const REITER: [&str; 4] = ["🚀 Schiffe", "🛡 Verteidigung", "🔧 Bauteile", "🎯 Raketen"];

impl Player {
    pub(crate) fn werft(&mut self, ui: &mut egui::Ui, v: &Value, p: &Value) {
        let r = self.regeln.clone();
        hilfe::kopf(ui, Bildschirm::Werft, &r);
        if p.is_null() {
            return;
        }
        let k = text(&p["koord"]);
        let fertigung = p["fertigung"].as_array().cloned().unwrap_or_default();
        karte(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("⚙ Fertigung auf {k}")).size(18.0).strong());
                ui.label(RichText::new(format!("Werft Stufe {} · Orbitalwerft Stufe {}", i(&p["gebaeude"]["werft"]), i(&p["gebaeude"]["orbitalwerft"]))).color(LEISE));
            });
            if fertigung.is_empty() {
                ui.label(RichText::new("Nichts in Arbeit.").color(LEISE));
            }
            for f in &fertigung {
                ui.label(format!("{} × {} – nächstes fertig {} ({})", z(&f["rest"]), name(f["produkt"].as_str().unwrap_or("")),
                    in_zeit(i(&f["naechstes_in_min"])), if f["schleife"] == "werft" { "Werft" } else { "Orbitalwerft" }));
            }
        });
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            for (n, t) in REITER.iter().enumerate() {
                if ui.selectable_label(self.werft_reiter == n, RichText::new(*t).size(17.0)).clicked() {
                    self.werft_reiter = n;
                }
            }
        });
        ui.separator();
        match self.werft_reiter {
            0 | 1 => {
                let schiffe = self.werft_reiter == 0;
                let freigeschaltet: Vec<Value> = v["einheiten_kosten"].as_array().cloned().unwrap_or_default();
                for e in Einheit::ALLE.into_iter().filter(|e| e.ist_schiff() == schiffe) {
                    let schluessel = e.name();
                    let regel = r.einh(e);
                    let eintrag = freigeschaltet.iter().find(|x| x["einheit"] == schluessel).cloned();
                    let bestand = i(&p[if schiffe { "schiffe" } else { "verteidigung" }][schluessel]);
                    karte(ui, |ui| {
                        ui.horizontal(|ui| {
                            self.motiv(ui, if schiffe { "ships" } else { "defenses" }, schluessel, v);
                            ui.label(RichText::new(name(schluessel)).size(17.0).strong().color(if eintrag.is_some() { TEXT } else { LEISE }));
                            if bestand > 0 {
                                etikett(ui, &format!("{} vorhanden", zahl(bestand)), INFO);
                            }
                            ui.label(RichText::new(&regel.wirkung).color(LEISE));
                        });
                        let mut werte = vec![format!("⚔ Angriff {}", zahl(regel.angriff)), format!("🛡 Schild {}", zahl(regel.schild)), format!("❤ Struktur {}", zahl(regel.struktur))];
                        if schiffe {
                            werte.push(format!("📦 Ladung {}", zahl(regel.ladung)));
                            werte.push(format!("⏩ Tempo {}", zahl(regel.tempo)));
                            if regel.besatzung > 0 {
                                werte.push(format!("👥 Besatzung {}", zahl(regel.besatzung)));
                            }
                        }
                        if !regel.schnellfeuer.is_empty() {
                            werte.push(format!("Schnellfeuer gegen {}", regel.schnellfeuer.keys().map(|x| name(x.name())).collect::<Vec<_>>().join(", ")));
                        }
                        ui.label(RichText::new(werte.join("   ")).small().color(LEISE));
                        let Some(eintrag) = eintrag else {
                            ui.label(RichText::new(format!("🔒 Ab Zivilisationsstufe {}", roemisch(regel.ab_stufe as i64))).color(LEISE));
                            return;
                        };
                        let fehlend: Vec<String> = eintrag["braucht"].as_array().into_iter().flatten().filter_map(|b| b.as_str()).filter_map(|b| {
                            let mut t = b.split(' ');
                            let (g, n) = (t.next()?, t.next()?.parse::<i64>().ok()?);
                            (i(&p["gebaeude"][g]) < n).then(|| format!("{} Stufe {n}", name(g)))
                        }).collect();
                        let werft = i(&eintrag["werft"]);
                        let mut fehlt_alles = fehlend.clone();
                        if schiffe && i(&p["gebaeude"]["werft"]) < werft {
                            fehlt_alles.insert(0, format!("Werft Stufe {werft}"));
                        }
                        let anzahl = self.werft_anzahl.entry(schluessel.to_string()).or_insert(1);
                        ui.horizontal(|ui| {
                            ui.label("Menge");
                            ui.add(egui::DragValue::new(anzahl).range(1..=10_000).speed(0.2));
                            ui.label(RichText::new("kostet").color(LEISE));
                            kosten(ui, &eintrag["kosten"], &p["bestand"], *anzahl);
                        });
                        let n = *anzahl;
                        let bezahlbar_ = bezahlbar(&eintrag["kosten"], &p["bestand"], n);
                        let (aktiv, grund) = if !fehlt_alles.is_empty() {
                            (false, format!("Es fehlt: {}.", fehlt_alles.join(", ")))
                        } else if !bezahlbar_ {
                            let wann = wartezeit(&eintrag["kosten"], p, n).map(|w| format!(" ({w})")).unwrap_or_default();
                            (false, format!("Nicht genug Güter auf diesem Planeten{wann}; bezahlt wird bei der Bestellung."))
                        } else {
                            (true, String::new())
                        };
                        ui.horizontal(|ui| {
                            if knopf(ui, &format!("{} × {} bestellen", zahl(n), name(schluessel)), aktiv, &grund) {
                                self.act(befehl::fertigen_einheit(&k, schluessel, n));
                            }
                            if !aktiv {
                                ui.label(RichText::new(grund).small().color(LEISE));
                            }
                        });
                    });
                }
            }
            2 => {
                hinweis_karte(ui, INFO, |ui| {
                    ui.label(format!("Bauteile fertigt die Orbitalwerft (ab Stufe IV). Ein Kolonieschiff braucht {} und trägt Siedler von der Heimatwelt; jedes Habitatmodul wird {} Wohnraum der neuen Kolonie.",
                        "Antriebskerne und Habitatmodule", zahl(r.wirtschaft.habitat_wohnraum)));
                });
                for g in [Gut::Antriebskern, Gut::Habitatmodul] {
                    let schluessel = g.name();
                    let preis = r.wirtschaft.bauteile.get(&g).map(|p| serde_json::to_value(p).unwrap_or(Value::Null)).unwrap_or(Value::Null);
                    karte(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(name(schluessel)).size(17.0).strong());
                            etikett(ui, &format!("{} im Lager", z(&p["bestand"][schluessel])), INFO);
                        });
                        let anzahl = self.werft_anzahl.entry(schluessel.to_string()).or_insert(1);
                        ui.horizontal(|ui| {
                            ui.label("Menge");
                            ui.add(egui::DragValue::new(anzahl).range(1..=10_000).speed(0.2));
                            ui.label(RichText::new("kostet").color(LEISE));
                            kosten(ui, &preis, &p["bestand"], *anzahl);
                        });
                        let n = *anzahl;
                        let orbitalwerft = i(&p["gebaeude"]["orbitalwerft"]) > 0;
                        let (aktiv, grund) = if !orbitalwerft {
                            (false, "Dafür braucht der Planet eine Orbitalwerft.")
                        } else if !bezahlbar(&preis, &p["bestand"], n) {
                            (false, "Nicht genug Güter auf diesem Planeten.")
                        } else {
                            (true, "")
                        };
                        ui.horizontal(|ui| {
                            if knopf(ui, &format!("{} × {} fertigen", zahl(n), name(schluessel)), aktiv, grund) {
                                self.act(befehl::fertigen_bauteil(&k, schluessel, n));
                            }
                            if !aktiv {
                                ui.label(RichText::new(grund).small().color(LEISE));
                            }
                        });
                    });
                }
            }
            _ => self.raketen(ui, v, p, &k, &r),
        }
    }

    fn raketen(&mut self, ui: &mut egui::Ui, v: &Value, p: &Value, k: &str, r: &kern::Regelwerk) {
        let silo = i(&p["gebaeude"]["raketensilo"]);
        let rk = &p["raketen"];
        karte(ui, |ui| {
            ui.label(RichText::new("🎯 Raketensilo").size(18.0).strong());
            ui.horizontal(|ui| {
                for (key, label) in [("abfang", "Abfangrakete"), ("interplanetar", "Interplanetarrakete")] {
                    ui.vertical(|ui| {
                        self.motiv_groesse(ui, "missiles", key, v, egui::vec2(150.0, 110.0));
                        ui.label(label);
                    });
                }
            });
            if silo == 0 {
                ui.label(RichText::new(format!("Dieser Planet hat kein Raketensilo. Es ist ab Stufe {} baubar.", roemisch(r.geb(kern::Gebaeude::Raketensilo).ab_stufe as i64))).color(LEISE));
                return;
            }
            let kap = i(&rk["kapazitaet"]).max(1);
            balken(ui, i(&rk["belegt_mit_bau"]) as f32 / kap as f32, format!("{} von {} Plätzen belegt (mit Aufträgen)", z(&rk["belegt_mit_bau"]), zahl(kap)), INFO);
            ui.label(format!("{} Abfangraketen · {} Interplanetarraketen", z(&rk["abfang"]), z(&rk["interplanetar"])));
            for b in p["raketenbau"].as_array().into_iter().flatten() {
                ui.label(RichText::new(format!("im Bau: {} × {} bis {}", z(&b["anzahl"]), name(b["art"].as_str().unwrap_or("")), text(&b["fertig"]))).small().color(LEISE));
            }
        });
        if silo == 0 {
            return;
        }
        ui.add_space(6.0);
        karte(ui, |ui| {
            ui.label(RichText::new("Raketen bauen").strong());
            ui.label(RichText::new("Abfangraketen vernichten anfliegende Raketen automatisch. Interplanetarraketen zerstören Verteidigungsanlagen eines Ziels im selben Sektor.").color(LEISE));
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_salt("raketenart").selected_text(name(&self.rocket_type)).show_ui(ui, |ui| {
                    for a in ["abfang", "interplanetar"] {
                        ui.selectable_value(&mut self.rocket_type, a.to_string(), name(a));
                    }
                });
                ui.label("Menge");
                ui.add(egui::DragValue::new(&mut self.raketen_anzahl).range(1..=1_000).speed(0.2));
            });
            let art = if self.rocket_type == "abfang" { 0 } else { 1 };
            let preis = serde_json::to_value(&r.zusatz.raketen_kosten[art]).unwrap_or(Value::Null);
            kosten(ui, &preis, &p["bestand"], self.raketen_anzahl);
            if ui.button("Raketen bauen").clicked() {
                self.act(befehl::raketen_bauen(k, &self.rocket_type, self.raketen_anzahl));
            }
        });
        ui.add_space(6.0);
        karte(ui, |ui| {
            ui.label(RichText::new("Interplanetarraketen starten").strong());
            ui.label(RichText::new(format!("Reichweite {} Systeme je Silostufe im eigenen Sektor. Ein Start beendet deinen Anfängerschutz und bricht Nichtangriffspakte mit dem Ziel.", r.zusatz.raketen_reichweite_je_silo)).color(LEISE));
            ui.horizontal(|ui| {
                ui.label("Ziel");
                ui.add(egui::TextEdit::singleline(&mut self.target).desired_width(110.0).hint_text("1:27:6"));
                ui.label("Anzahl");
                ui.add(egui::DragValue::new(&mut self.raketen_anzahl).range(1..=1_000).speed(0.2));
                ui.label("gegen");
                egui::ComboBox::from_id_salt("zieltyp").selected_text(name(&self.rocket_target)).show_ui(ui, |ui| {
                    for e in Einheit::ALLE.into_iter().filter(|e| !e.ist_schiff()) {
                        ui.selectable_value(&mut self.rocket_target, e.name().to_string(), name(e.name()));
                    }
                });
            });
            if ui.button("Raketen starten …").clicked() {
                self.bestaetigen(
                    &format!("{} Interplanetarraketen auf {} feuern? Das ist ein Angriff.", self.raketen_anzahl, self.target),
                    befehl::raketen_starten(k, &self.target, self.raketen_anzahl, &self.rocket_target),
                );
            }
        });
    }
}
