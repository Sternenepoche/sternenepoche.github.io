//! Markt: Preise aller Güter, eigene Orders und eine Order mit Vorschau von Kosten, Gebühr und Erlös.

use super::hilfe::{self, text};
use super::namen::{gut_zeichen, name};
use super::befehl;
use super::stil::*;
use super::Bildschirm;
use crate::Player;
use eframe::egui::{self, RichText};
use kern::{Gut, Volk};
use serde_json::Value;

impl Player {
    pub(crate) fn markt(&mut self, ui: &mut egui::Ui, v: &Value, p: &Value) {
        let r = self.regeln.clone();
        hilfe::kopf(ui, Bildschirm::Markt, &r);
        if p.is_null() {
            return;
        }
        let k = text(&p["koord"]);
        let stufe = i(&p["gebaeude"]["markt"]);
        let volk = Volk::aus_name(v["volk"].as_str().unwrap_or("")).unwrap_or(Volk::Aurelianer);
        let gebuehr = r.markt.gebuehr * r.volk(volk).marktgebuehr;
        let preise = &v["markt"]["preise"];
        karte(ui, |ui| {
            ui.label(RichText::new("📈 Preise im Orderbuch").size(18.0).strong());
            ui.label(RichText::new("„Verkauf ab“: die günstigste Verkaufsorder – dafür kannst du sofort kaufen. „Kauf bis“: das höchste Kaufgebot – dafür kannst du sofort verkaufen.").small().color(LEISE));
            egui::Grid::new("preise").striped(true).num_columns(4).spacing([24.0, 4.0]).show(ui, |ui| {
                for h in ["Gut", "Verkauf ab", "Kauf bis", "dein Bestand hier"] {
                    ui.label(RichText::new(h).strong().color(LEISE));
                }
                ui.end_row();
                for g in Gut::ALLE {
                    let n = g.name();
                    let eintrag = &preise[n];
                    let preis = |x: &Value| x.as_f64().map(|f| format!("{} Cr", komma(f, 3))).unwrap_or_else(|| "–".into());
                    ui.horizontal(|ui| {
                        self.motiv_klein(ui, "resources", n, v);
                        ui.label(format!("{} {}", gut_zeichen(n), name(n)));
                    });
                    ui.label(preis(&eintrag["verkauf_ab"]));
                    ui.label(preis(&eintrag["kauf_bis"]));
                    ui.label(z(&p["bestand"][n]));
                    ui.end_row();
                }
            });
            if preise.as_object().is_none_or(|m| m.is_empty()) {
                ui.label(RichText::new("Noch liegen keine Orders im Buch. Wer zuerst bietet, setzt den Preis.").color(LEISE));
            }
        });
        ui.add_space(6.0);
        let orders = v["markt"]["orders"].as_array().cloned().unwrap_or_default();
        karte(ui, |ui| {
            ui.label(RichText::new("📋 Deine offenen Orders").size(18.0).strong());
            if orders.is_empty() {
                ui.label(RichText::new("Keine. Eine Order bleibt im Buch, bis jemand zu deinem Preis handelt oder du sie stornierst.").color(LEISE));
            }
            for o in &orders {
                ui.horizontal(|ui| {
                    let kauf = o["seite"] == "kauf";
                    etikett(ui, if kauf { "Kauf" } else { "Verkauf" }, if kauf { INFO } else { GOLD });
                    ui.label(RichText::new(text(&o["planet"])).color(LEISE));
                    ui.label(format!("{} {} zu {} Credits", z(&o["menge"]), name(o["gut"].as_str().unwrap_or("")), komma(o["preis"].as_f64().unwrap_or(0.0), 3)));
                    if ui.small_button("Stornieren").on_hover_text("Hinterlegte Credits oder Ware kommen zurück.").clicked() {
                        self.act(befehl::markt_storno(&o["order"]));
                    }
                });
            }
        });
        ui.add_space(6.0);
        karte(ui, |ui| {
            ui.label(RichText::new(format!("📝 Neue Order auf {k}")).size(18.0).strong());
            if stufe == 0 {
                let ab = r.geb(kern::Gebaeude::Markt).ab_stufe as i64;
                if i(&v["stufe"]) < ab {
                    ui.label(RichText::new(format!("Handeln kannst du mit einem Markt. Den gibt es ab Zivilisationsstufe {}.", roemisch(ab))).color(WARNUNG));
                } else {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new("Auf diesem Planeten steht noch kein Markt. Jede Marktstufe erlaubt hier weitere offene Orders.").color(WARNUNG));
                        if ui.button("🏭 Markt bauen").clicked() {
                            self.screen = Bildschirm::Gebaeude;
                        }
                    });
                }
                return;
            }
            let hier = orders.iter().filter(|o| o["planet"] == k.as_str()).count();
            let grenze = stufe as usize * r.markt.orders_je_marktstufe;
            ui.label(RichText::new(format!("{hier} von {grenze} Orders auf diesem Planeten belegt (Markt Stufe {stufe}).")).color(if hier >= grenze { WARNUNG } else { LEISE }));
            ui.horizontal(|ui| {
                for (s, t) in [("kauf", "Kaufen"), ("verkauf", "Verkaufen")] {
                    if ui.selectable_label(self.order_side == s, RichText::new(t).size(16.0)).clicked() {
                        self.order_side = s.to_string();
                    }
                }
                egui::ComboBox::from_id_salt("marktgut").selected_text(format!("{} {}", gut_zeichen(&self.good), name(&self.good))).show_ui(ui, |ui| {
                    for g in Gut::ALLE {
                        ui.selectable_value(&mut self.good, g.name().to_string(), format!("{} {}", gut_zeichen(g.name()), name(g.name())));
                    }
                });
            });
            let kauf = self.order_side == "kauf";
            let vorschlag = preise[&self.good][if kauf { "verkauf_ab" } else { "kauf_bis" }].as_f64();
            ui.horizontal(|ui| {
                ui.label("Menge");
                ui.add(egui::DragValue::new(&mut self.market_quantity).range(1.0..=1e9).speed(10.0));
                ui.label("Preis je Einheit");
                ui.add(egui::DragValue::new(&mut self.price).range(0.001..=1e6).speed(0.01).max_decimals(3).suffix(" Cr"));
                if let Some(x) = vorschlag {
                    if ui.small_button(format!("Marktpreis {}", komma(x, 3))).on_hover_text("Übernimmt den Preis, zu dem sofort gehandelt wird.").clicked() {
                        self.price = x;
                    }
                }
            });
            let summe = self.market_quantity * self.price;
            let credits = v["credits"].as_f64().unwrap_or(0.0);
            let bestand = p["bestand"][&self.good].as_f64().unwrap_or(0.0);
            let (text_vorschau, ok) = if kauf {
                let gesamt = summe * (1.0 + gebuehr);
                (format!("Du hinterlegst {} Credits (davon {} Gebühr); du hast {}.", zahl(gesamt.ceil() as i64), zahl((summe * gebuehr).ceil() as i64), zahl(credits as i64)), gesamt <= credits)
            } else {
                (format!("Erlös nach Gebühr etwa {} Credits; du hinterlegst {} {} (vorrätig {}).", zahl((summe * (1.0 - gebuehr)) as i64), zahl(self.market_quantity as i64), name(&self.good), zahl(bestand as i64)), self.market_quantity <= bestand)
            };
            ui.label(RichText::new(text_vorschau).color(if ok { LEISE } else { SCHLECHT }));
            ui.label(RichText::new(format!("Gebühr {} Prozent je Seite; sie verschwindet aus dem Spiel. Gekaufte Ware bringt eine Handelsflotte.", komma(gebuehr * 100.0, 1))).small().color(LEISE));
            let beschriftung = if kauf { "Kauforder einstellen" } else { "Verkaufsorder einstellen" };
            if knopf(ui, beschriftung, ok, if kauf { "Nicht genug Credits." } else { "Nicht genug Ware auf diesem Planeten." }) {
                self.act(befehl::markt_order(&k, &self.good, &self.order_side, self.market_quantity, self.price));
            }
        });
    }
}
