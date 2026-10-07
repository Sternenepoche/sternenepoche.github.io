//! Gebäude: Bauschleife, alle Gebäude nach Gruppen mit Kosten, Wirkung und Bedarf der nächsten Stufe,
//! gesperrte mit ihrer Freischaltstufe, Abriss mit Rückfrage und die Prioritäten der Arbeitskräfte.

use super::hilfe::{self, text};
use super::namen::{name, GEBAEUDE_GRUPPEN};
use super::befehl;
use super::stil::*;
use super::Bildschirm;
use crate::Player;
use eframe::egui::{self, RichText};
use kern::Gebaeude;
use serde_json::Value;

impl Player {
    pub(crate) fn gebaeude(&mut self, ui: &mut egui::Ui, v: &Value, p: &Value) {
        let r = self.regeln.clone();
        hilfe::kopf(ui, Bildschirm::Gebaeude, &r);
        if p.is_null() {
            return;
        }
        let k = text(&p["koord"]);
        let schleife = p["bauschleife"].as_array().cloned().unwrap_or_default();
        let plaetze = i(&p["bauschleife_plaetze"]).max(1) as usize;
        let voll = schleife.len() >= plaetze;
        let (belegt, gesamt) = (i(&p["felder"]["belegt"]), i(&p["felder"]["gesamt"]));

        karte(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("🔨 Bauschleife auf {k}")).size(18.0).strong());
                ui.label(RichText::new(format!("{} von {plaetze} Plätzen · {belegt} von {gesamt} Feldern belegt", schleife.len())).color(LEISE));
            });
            if schleife.is_empty() {
                ui.label(RichText::new("Leer. Wähle unten ein Gebäude zum Ausbauen – ein freier Bauplatz ist verlorene Zeit.").color(WARNUNG));
            }
            for (n, a) in schleife.iter().enumerate() {
                ui.horizontal(|ui| {
                    let g = name(a["gebaeude"].as_str().unwrap_or(""));
                    if a["wartet"] == true {
                        etikett(ui, if n == 0 { "wartet auf Güter" } else { "in der Schlange" }, if n == 0 { WARNUNG } else { LEISE });
                        ui.label(format!("{g} Stufe {}", z(&a["stufe"])));
                    } else {
                        etikett(ui, "im Bau", GUT);
                        ui.label(format!("{g} Stufe {} – fertig {}", z(&a["stufe"]), in_zeit(i(&a["rest_min"]))));
                    }
                });
            }
            if schleife.iter().any(|a| a["wartet"] == true) && ui.button("Wartende Aufträge entfernen").on_hover_text("Entfernt alle noch nicht begonnenen Aufträge. Der laufende Bau bleibt.").clicked() {
                self.bestaetigen("Alle wartenden Bauaufträge auf diesem Planeten entfernen?", befehl::schleife_leeren(&k));
            }
        });
        ui.add_space(6.0);
        self.prioritaeten(ui, p, &k);

        let stufe = i(&v["stufe"]) as u8;
        let baubar: Vec<Value> = p["baubar"].as_array().cloned().unwrap_or_default();
        for (zeichen, gruppe, liste) in GEBAEUDE_GRUPPEN {
            ui.add_space(6.0);
            egui::CollapsingHeader::new(RichText::new(format!("{zeichen}  {gruppe}")).size(18.0).strong())
                .id_salt(("gruppe", *gruppe))
                .default_open(true)
                .show(ui, |ui| {
                    for schluessel in *liste {
                        let Some(g) = Gebaeude::aus_name(schluessel) else { continue };
                        let Some(regel) = r.gebaeude.get(&g) else {continue;};
                        let jetzt = i(&p["gebaeude"][*schluessel]);
                        let eintrag = baubar.iter().find(|b| b["gebaeude"] == *schluessel);
                        let mut abriss = false;
                        karte(ui, |ui| {
                            ui.horizontal(|ui| {
                                self.motiv(ui, "buildings", schluessel, v);
                                ui.label(RichText::new(name(schluessel)).size(17.0).strong().color(if eintrag.is_some() || jetzt > 0 { TEXT } else { LEISE }));
                                if jetzt > 0 {
                                    etikett(ui, &format!("Stufe {jetzt}"), INFO);
                                }
                                ui.label(RichText::new(&regel.wirkung).color(LEISE));
                                if jetzt > 0 {
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.small_button("Abreißen …").on_hover_text("Senkt das Gebäude um eine Stufe. Abriss erstattet nichts.").clicked() {
                                            abriss = true;
                                        }
                                    });
                                }
                            });
                            match eintrag {
                                Some(b) => self.gebaeude_ausbau(ui, p, b, &k, voll, belegt >= gesamt, schleife.is_empty()),
                                None if regel.ab_stufe > stufe => {
                                    ui.label(RichText::new(format!("🔒 Ab Zivilisationsstufe {}", roemisch(regel.ab_stufe as i64))).color(LEISE));
                                }
                                None if g == Gebaeude::Xenoextraktor && p["nebel"] != true => {
                                    ui.label(RichText::new("Nur auf Planeten in einem Nebelsystem.").color(LEISE));
                                }
                                None => {
                                    ui.label(RichText::new("Höchste Stufe erreicht oder schon gebaut.").color(LEISE));
                                }
                            }
                            if let Some(hp) = p["gebaeude_integritaet_prozent"][*schluessel].as_i64().filter(|hp| *hp < 100) {
                                ui.label(RichText::new(format!("Integrität {hp} % – Leistung vermindert")).color(WARNUNG));
                                let running = p["reparaturen"].as_array().into_iter().flatten().any(|x| x["gebaeude"] == *schluessel);
                                if running {
                                    ui.label("Reparatur läuft; sie teilt die Baustelle mit dem normalen Ausbau.");
                                } else if let Ok(quote) = self.session.query(&serde_json::json!({"typ":"kosten", "planet":k, "gebaeude":schluessel})) {
                                    let repair = &quote["reparatur"];
                                    if !repair.is_null() {
                                        kosten(ui, &repair["kosten"], &p["bestand"], 1);
                                        ui.label(format!("Reparaturdauer: {}", super::stil::in_zeit(i(&repair["dauer_sekunden"]) / 60)));
                                        if ui.add_enabled(schleife.is_empty() && bezahlbar(&repair["kosten"], &p["bestand"], 1), egui::Button::new("Reparieren")).clicked() {
                                            self.act(befehl::reparieren(&k, schluessel));
                                        }
                                    }
                                }
                            }
                            if abriss {
                                self.bestaetigen(
                                    &format!("{} auf {k} um eine Stufe abreißen? Das erstattet nichts und kostet Punkte.", name(schluessel)),
                                    befehl::abreissen(&k, schluessel),
                                );
                            }
                        });
                    }
                });
        }
    }

    fn gebaeude_ausbau(&mut self, ui: &mut egui::Ui, p: &Value, b: &Value, k: &str, schleife_voll: bool, felder_voll: bool, schleife_leer: bool) {
        let schluessel = b["gebaeude"].as_str().unwrap_or("");
        let fehlt: Vec<String> = b["fehlt"].as_array().into_iter().flatten().filter_map(|x| x.as_str().map(name)).collect();
        let braucht: Vec<String> = b["braucht"].as_array().into_iter().flatten().filter_map(|x| x.as_str()).map(|s| {
            let mut t = s.split(' ');
            format!("{} Stufe {}", name(t.next().unwrap_or("")), t.next().unwrap_or(""))
        }).collect();
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(format!("Stufe {}:", z(&b["stufe"]))).color(LEISE));
            kosten(ui, &b["kosten"], &p["bestand"], 1);
            ui.label(RichText::new(format!("·  ⏳ {}", dauer_min(i(&b["bauzeit_min"])))).color(LEISE)).on_hover_text("Bauzeit");
        });
        let bedarf: Vec<String> = [("strom_plus", "Strom"), ("arbeiter_plus", "Arbeitskräfte"), ("fachkraefte_plus", "Fachkräfte")]
            .iter()
            .filter(|(f, _)| i(&b[*f]) > 0)
            .map(|(f, n)| format!("+{} {n}", z(&b[*f])))
            .collect();
        ui.horizontal_wrapped(|ui| {
            if let Some(e) = b["ertrag"].as_object() {
                ui.label(RichText::new(format!("bringt +{} {}", z(&e["plus"]), super::namen::woerter(&text(&e["art"])))).color(GUT));
            }
            if !bedarf.is_empty() {
                ui.label(RichText::new(format!("braucht {}", bedarf.join(", "))).color(WARNUNG))
                    .on_hover_text("Was diese Stufe zusätzlich an Strom und Arbeitskräften verlangt.");
            }
            if !braucht.is_empty() {
                ui.label(RichText::new(format!("Voraussetzung: {}", braucht.join(", "))).color(SCHLECHT));
            }
            if let Some(f) = b["kostenfaktor"].as_f64() {
                ui.label(RichText::new(format!("jede weitere Stufe ×{} teurer", komma(f, 1))).small().color(LEISE));
            }
        });
        let (aktiv, grund) = if !braucht.is_empty() {
            (false, format!("Erst {} bauen.", braucht.join(", ")))
        } else if schleife_voll {
            (false, "Die Bauschleife ist voll.".to_string())
        } else if felder_voll {
            (false, "Alle Felder sind belegt.".to_string())
        } else if schleife_leer && !fehlt.is_empty() {
            let wann = wartezeit(&b["kosten"], p, 1).map(|w| format!(" – {w}")).unwrap_or_default();
            (false, format!("Es fehlt {}{wann}. Bei leerer Schleife beginnt ein Bau sofort, dafür muss alles da sein.", fehlt.join(", ")))
        } else {
            (true, String::new())
        };
        ui.horizontal(|ui| {
            if knopf(ui, &format!("Auf Stufe {} ausbauen", z(&b["stufe"])), aktiv, &grund) {
                self.act(befehl::bauen(k, schluessel));
            }
            if aktiv && !fehlt.is_empty() {
                let wann = wartezeit(&b["kosten"], p, 1).map(|w| format!(" ({w})")).unwrap_or_default();
                ui.label(RichText::new(format!("wartet in der Schlange, bis {} da ist{wann}", fehlt.join(", "))).small().color(WARNUNG));
            } else if !aktiv {
                ui.label(RichText::new(grund).small().color(LEISE));
            }
        });
    }

    /// Reihenfolge, in der Gebäude Arbeitskräfte bekommen: hoch und runter schieben, dann übernehmen.
    fn prioritaeten(&mut self, ui: &mut egui::Ui, p: &Value, k: &str) {
        let aktuell: Vec<String> = p["prioritaeten"].as_array().into_iter().flatten().filter_map(|x| x.as_str().map(String::from)).collect();
        egui::CollapsingHeader::new(RichText::new("👥 Prioritäten der Arbeitskräfte").strong())
            .id_salt("prioritaeten")
            .show(ui, |ui| {
                ui.label(RichText::new("Reichen die Arbeitskräfte nicht für alle Gebäude, bekommen die oberen sie zuerst. Ohne eigene Vorgabe gehen Farm und Kraftwerke vor.").color(LEISE));
                // Neu aufsetzen, wenn der Planet wechselt oder sich die Lage geändert hat (neues Gebäude, übernommene Liste).
                if self.prioritaeten_entwurf.as_ref().is_none_or(|(planet, basis, _)| planet != k || *basis != aktuell) {
                    self.prioritaeten_entwurf = Some((k.to_string(), aktuell.clone(), aktuell.clone()));
                }
                let liste = &mut self.prioritaeten_entwurf.as_mut().unwrap().2;
                let mut tausch = None;
                for (n, g) in liste.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!("{:>2}.", n + 1)).color(GOLD));
                        if ui.add_enabled(n > 0, egui::Button::new("⏶").small()).clicked() {
                            tausch = Some((n, n - 1));
                        }
                        if ui.add_enabled(n + 1 < liste.len(), egui::Button::new("⏷").small()).clicked() {
                            tausch = Some((n, n + 1));
                        }
                        ui.label(name(g));
                    });
                }
                if let Some((a, b)) = tausch {
                    liste.swap(a, b);
                }
                let geaendert = *liste != aktuell;
                let neu = liste.clone();
                ui.horizontal(|ui| {
                    if knopf(ui, "Reihenfolge übernehmen", geaendert, "Nichts geändert.") {
                        self.act(befehl::prioritaeten(k, &neu));
                        self.prioritaeten_entwurf = None;
                    }
                    if geaendert && ui.button("Zurücksetzen").clicked() {
                        self.prioritaeten_entwurf = None;
                    }
                });
            });
    }
}
