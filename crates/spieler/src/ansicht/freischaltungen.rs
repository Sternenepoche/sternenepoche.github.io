//! Unlock map derived from the current engine rules; no invented technology edges.
use super::{namen::name, stil::*};
use crate::Player;
use eframe::egui::{self, RichText};
use kern::{Einheit, Forschung, Gebaeude};
use serde_json::Value;

const STAGES: [&str; 5] = ["gruendung", "industrie", "orbit", "sternenflug", "imperium"];

impl Player {
    pub(crate) fn freischaltungen(&mut self, ui: &mut egui::Ui, v: &Value) {
        let r = self.regeln.clone();
        let stage = i(&v["stufe"]);
        let home = v["planeten"].as_array().into_iter().flatten().find(|p| p["heimat"] == true).cloned().unwrap_or(Value::Null);
        egui::CollapsingHeader::new(RichText::new("Freischaltübersicht · von der Gründung zum Imperium").size(18.0).strong())
            .id_salt("unlock-map").default_open(true).show(ui, |ui| {
            ui.label("Technologien nach Zivilisationsstufe. Klappe Aufstiegsbedingungen und Wirkungen auf; Kosten und Forschungsaufträge stehen darunter.");
            ui.small("Forschungszugang: Zivilisationsstufe und Labor auf der Heimatwelt. Gebäude und Schiffe haben eigene Voraussetzungen.");
            let column_width=((ui.available_width()-120.0)/5.0).clamp(156.0,220.0);
            egui::ScrollArea::horizontal().id_salt("unlock-map-scroll").show(ui, |ui| {
                ui.spacing_mut().item_spacing.x=6.0;
                ui.horizontal_top(|ui| {
                    for (index, key) in STAGES.iter().take(r.stufen.len()+1).enumerate() {
                        let tier = index as u8 + 1;
                        if index > 0 {
                            let (rect, _)=ui.allocate_exact_size(egui::vec2(14.0,75.0),egui::Sense::hover());
                            ui.painter().arrow(rect.center()-egui::vec2(6.0,0.0),egui::vec2(12.0,0.0),egui::Stroke::new(1.5_f32,LEISE));
                        }
                        ui.vertical(|ui| {
                            ui.set_width(column_width);
                            let title = if index == 0 { "I · Gründung".into() } else { r.stufen[index-1].name.clone() };
                            let color = if stage == tier as i64 { GOLD } else if stage > tier as i64 { GUT } else { LEISE };
                            self.motiv_groesse(ui,"progression",key,v,egui::vec2(column_width-10.0,65.0));
                            ui.label(RichText::new(title).strong().color(color));
                            ui.small(if stage >= tier as i64 { "Stufe erreicht" } else { "Stufe noch gesperrt" });
                            if index > 0 {
                                let s = &r.stufen[index-1];
                                egui::CollapsingHeader::new("Aufstieg").id_salt(("tier-needs",tier)).show(ui,|ui| {
                                    ui.label(format!("{} Einwohner",s.einwohner));
                                    for (b,n) in &s.gebaeude { ui.label(format!("{} {n}",name(b.name()))); }
                                    for (f,n) in &s.forschung { ui.label(format!("{} {n}",name(f.name()))); }
                                    if s.versorgung_plus { ui.label("Nahrung und Energie im Plus"); }
                                    if s.stabilitaet > 0. { ui.label(format!("Stabilität mindestens {}",s.stabilitaet)); }
                                    if s.konsum_deckung > 0. { ui.label(format!("Konsumdeckung mindestens {:.0} %",s.konsum_deckung*100.)); }
                                    if s.kolonien > 0 { ui.label(format!("{} Kolonien",s.kolonien)); }
                                    ui.small("Haltezeit, Aufstiegskosten und aktuelle Erfüllung: Regierung.");
                                    if ui.small_button("Zur Regierung").clicked() { self.screen=super::Bildschirm::Regierung; }
                                });
                            }
                            ui.separator();
                            if Forschung::ALLE.into_iter().any(|f|r.forsch(f).ab_stufe==tier) { ui.strong("Forschung"); }
                            else {ui.small("Keine neuen Technologien");}
                            for f in Forschung::ALLE.into_iter().filter(|f|r.forsch(*f).ab_stufe==tier) {
                                let rule=r.forsch(f);
                                let level=i(&v["forschung"]["stufen"][f.name()]);
                                let accessible=stage>=tier as i64 && i(&home["gebaeude"]["labor"])>=rule.labor as i64;
                                egui::Frame::group(ui.style()).show(ui,|ui| {
                                    ui.set_min_width(column_width-18.0);
                                    ui.horizontal(|ui| {
                                        self.motiv_groesse(ui,"research",f.name(),v,egui::vec2(40.0,40.0));
                                        ui.vertical(|ui| {
                                            ui.set_width(column_width-72.0);
                                            ui.small(format!("Labor {} nötig",rule.labor));
                                            ui.small(format!("Erforscht: {level}"));
                                        });
                                    });
                                    ui.label(RichText::new(name(f.name())).size(14.0).strong());
                                    ui.label(RichText::new(if accessible {"Zugang offen"} else {"Voraussetzung fehlt"}).small().color(if accessible {GUT}else{LEISE})).on_hover_text(&rule.wirkung);
                                    egui::CollapsingHeader::new("Wirkung").id_salt(("effect",f.name())).show(ui,|ui| {ui.label(&rule.wirkung);});
                                });
                            }
                            let buildings=Gebaeude::ALLE.into_iter().filter(|b|r.geb(*b).ab_stufe==tier).collect::<Vec<_>>();
                            egui::CollapsingHeader::new(format!("Gebäude ({})",buildings.len())).id_salt(("unlock-buildings",tier)).show(ui,|ui| {
                                for b in buildings {
                                    self.motiv_klein(ui,"buildings",b.name(),v);
                                    ui.strong(name(b.name()));
                                    for (need,level) in &r.geb(b).braucht {ui.small(format!("Benötigt: {} {level}",name(need.name())));}
                                    ui.label(&r.geb(b).wirkung);ui.separator();
                                }
                            });
                            let units=Einheit::ALLE.into_iter().filter(|e|r.einh(*e).ab_stufe==tier).collect::<Vec<_>>();
                            egui::CollapsingHeader::new(format!("Einheiten ({})",units.len())).id_salt(("unlock-units",tier)).show(ui,|ui| {
                                for e in units {
                                    let rule=r.einh(e);
                                    self.motiv_klein(ui,if e.ist_schiff(){"ships"}else{"defenses"},e.name(),v);
                                    ui.strong(name(e.name()));ui.small(format!("Werft {}",rule.werft));
                                    if let Some(drive)=rule.antrieb {ui.small(format!("Antriebsbonus: {}",name(drive.name())));}
                                    ui.separator();
                                }
                            });
                        });
                    }
                });
            });
        });
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn all_technologies_belong_to_an_existing_unlock_stage() {
        let r=kern::Regelwerk::laden(include_str!("../../../../regeln/regelwerk.ron")).unwrap();
        for f in kern::Forschung::ALLE {assert!((1..=super::STAGES.len() as u8).contains(&r.forsch(f).ab_stufe));}
        assert_eq!(r.stufen.len()+1,super::STAGES.len());
    }
}
