//! Clickable, rule-derived technology graph, shared with the browser client.
use super::{namen::name, stil::*, Bildschirm};
use crate::Player;
use eframe::egui::{self, RichText, Stroke, Color32, Pos2};
use serde_json::Value;

impl Player {
    pub(crate) fn freischaltungen(&mut self, ui: &mut egui::Ui, v: &Value) {
        let graph=spieler::technologie::baum(&self.session);
        let nodes=graph["nodes"].as_array().unwrap();
        let cat_id=egui::Id::new("technology-category"); let focus_id=egui::Id::new("technology-focus");
        let mut category=ui.ctx().data_mut(|d|d.get_temp::<String>(cat_id)).unwrap_or("ships".into());
        let mut focus=ui.ctx().data_mut(|d|d.get_temp::<String>(focus_id)).unwrap_or("ships.spionagesonde".into());
        ui.heading("Technologiebaum");
        ui.label(RichText::new("Vom ersten Erkundungsflug zum interstellaren Imperium").color(LEISE));
        ui.horizontal(|ui| {
            for (key,title) in [("ships","Raumfahrt"),("research","Forschung"),("buildings","Infrastruktur"),("defenses","Verteidigung")] {
                ui.selectable_value(&mut category,key.into(),title);
            }
        });
        if let Some(n)=nodes.iter().find(|n|n["id"]==focus) {
            karte(ui,|ui| {
                ui.horizontal_top(|ui| {
                    self.motiv_groesse(ui,n["category"].as_str().unwrap(),n["key"].as_str().unwrap(),v,egui::vec2(115.,100.));
                    ui.vertical(|ui| {
                        ui.heading(name(n["key"].as_str().unwrap())); ui.label(n["effect"].as_str().unwrap());
                        ui.label(format!("Zivilisation {} · {} vorhanden / erforscht",roemisch(i(&n["stage"])),i(&n["level"])));
                        ui.horizontal_wrapped(|ui| {
                            for d in n["deps"].as_array().unwrap() {
                                let key=d["id"].as_str().unwrap().split('.').nth(1).unwrap();
                                if ui.button(RichText::new(format!("{} {} ({}/{})",name(key),i(&d["level"]),i(&d["have"]),i(&d["level"]))).color(if i(&d["have"])>=i(&d["level"]){GUT}else{WARNUNG})).clicked(){focus=d["id"].as_str().unwrap().into();category="buildings".into();}
                            }
                        });
                        if let Some(b)=n["bonus"].as_str(){ui.small(format!("Tempo verbessern: {} (keine Bauvoraussetzung)",name(b)));}
                        ui.horizontal_wrapped(|ui| {for (g,a) in n["quote"]["kosten"].as_object().into_iter().flatten(){ui.label(format!("{} {}",z(a),name(g)));}});
                        let allowed=n["accessible"]==true && n["affordable"]==true;
                        if ui.add_enabled(allowed,egui::Button::new(if category=="research"{"Erforschen / einreihen"}else{"Bauen / bestellen"})).clicked(){self.act(n["action"].clone());}
                        if !allowed {ui.small(if n["accessible"]!=true{"Voraussetzungen fehlen – klicke auf den benötigten Vorgänger."}else{"Es fehlen Rohstoffe. Die genauen Bestände stehen auf der Kolonie."});}
                        let next:Vec<_>=nodes.iter().filter(|x|x["deps"].as_array().unwrap().iter().any(|d|d["id"]==n["id"])).collect();
                        if !next.is_empty(){ui.horizontal_wrapped(|ui|{ui.label("Ermöglicht:");for x in next {if ui.small_button(name(x["key"].as_str().unwrap())).clicked(){focus=x["id"].as_str().unwrap().into();category=x["category"].as_str().unwrap().into();}}});}
                    });
                });
            });
        }
        egui::ScrollArea::horizontal().id_salt("technology-graph-scroll").show(ui,|ui| {
            let column=205.; let gap=18.; let card_h=94.;
            let groups:Vec<Vec<&Value>>=(1..=5).map(|tier|nodes.iter().filter(|n|n["category"]==category && i(&n["stage"])==tier).collect()).collect();
            let height=groups.iter().map(|g|g.len()).max().unwrap_or(1) as f32*card_h+65.;
            let (rect,_)=ui.allocate_exact_size(egui::vec2(5.*(column+gap),height),egui::Sense::hover());
            let mut positions=std::collections::BTreeMap::new();
            for (tier,group) in groups.iter().enumerate(){for (row,n) in group.iter().enumerate(){positions.insert(n["id"].as_str().unwrap(),Pos2::new(rect.left()+tier as f32*(column+gap),rect.top()+55.+row as f32*card_h));}}
            for (tier,group) in groups.iter().enumerate(){
                let x=rect.left()+tier as f32*(column+gap);
                ui.painter().text(egui::pos2(x,rect.top()+20.),egui::Align2::LEFT_CENTER,format!("{} · {}",roemisch(tier as i64+1),graph["stages"][tier].as_str().unwrap()),egui::FontId::proportional(17.),if (tier as i64)<i(&v["stufe"]){GOLD}else{LEISE});
                for n in group {
                    let at=positions[n["id"].as_str().unwrap()];
                    for d in n["deps"].as_array().unwrap(){if let Some(from)=positions.get(d["id"].as_str().unwrap()) {ui.painter().line_segment([*from+egui::vec2(column,35.),at+egui::vec2(0.,35.)],Stroke::new(1.3_f32,INFO));}}
                    let area=egui::Rect::from_min_size(at,egui::vec2(column,card_h-10.));
                    let color=if n["id"]==focus {GOLD}else if n["active"]==true {INFO}else if i(&n["level"])>0{GUT}else{LEISE};
                    ui.painter().rect(area,7.,Color32::from_rgb(15,27,42),Stroke::new(1.0_f32,color),egui::StrokeKind::Inside);
                    ui.scope_builder(egui::UiBuilder::new().max_rect(area.shrink(7.)),|ui|{
                        ui.horizontal(|ui|{
                            self.motiv_groesse(ui,n["category"].as_str().unwrap(),n["key"].as_str().unwrap(),v,egui::vec2(54.,60.));
                            ui.vertical(|ui|{
                                if ui.add(egui::Button::new(RichText::new(name(n["key"].as_str().unwrap())).color(color)).wrap()).clicked(){focus=n["id"].as_str().unwrap().into();}
                                ui.small(if n["active"]==true {"Forschung läuft"}else if n["accessible"]==true{"Zugang offen"}else{"Gesperrt"});
                            });
                        });
                    });
                }
            }
        });
        ui.ctx().data_mut(|d|{d.insert_temp(cat_id,category);d.insert_temp(focus_id,focus);});
        if ui.small_button("Aufstiegsbedingungen in der Regierung").clicked(){self.screen=Bildschirm::Regierung;}
        ui.separator();
    }
}
