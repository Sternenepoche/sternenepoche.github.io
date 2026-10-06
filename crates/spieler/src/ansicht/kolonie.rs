//! Kolonie: der Planet mit Bild, Bevölkerung, Versorgung, Strom, Arbeitskräften, Stabilität und Lager.

use super::hilfe::{self, text};
use super::namen::{gut_zeichen, name, zone_text};
use super::stil::*;
use super::Bildschirm;
use crate::Player;
use eframe::egui::{self, RichText};
use kern::{Gut, Zone};
use serde_json::Value;

impl Player {
    /// Planetenbild aus dem Rust-Renderer; Gelände ist Dekoration, die Rohstoffschicht zeigt den echten Ertragsfaktor.
    pub(crate) fn planetenbild(&mut self, ui: &mut egui::Ui, p: &Value, groesse: f32) {
        let basis = format!("{}-{}", text(&p["koord"]), text(&p["zone"]));
        // Keep the procedural resource overlay; use generated planet art for the normal orbit view.
        let variant = basis.bytes().fold(0u32, |n, b| n.wrapping_add(b as u32)) % 2;
        let key = if p["nebel"] == true { "nebel_xeno" } else {
            match (p["zone"].as_str().unwrap_or(""), variant) {
                ("glut", 0) => "glut_basalt", ("glut", _) => "glut_wueste",
                ("frost", 0) => "frost_eis", ("frost", _) => "frost_tundra",
                ("leben", 0) => "leben_kontinent", ("leben", _) => "leben_ozean",
                _ => "asteroid",
            }
        };
        if self.overlay == "keine" && self.motiv_groesse(ui, "planets", key, p, egui::vec2(groesse, groesse)) { return; }
        // Die Engine liefert Faktoren in Promille, der Renderer erwartet 1.0 = Durchschnitt.
        let faktor = p["faktoren"][&self.overlay].as_f64().map(|f| f / 1000.0);
        let schluessel = format!("{basis}-{}-{:?}", self.overlay, faktor);
        if self.texture_key != schluessel {
            let seed = basis.bytes().fold(2166136261u32, |v, b| (v ^ b as u32).wrapping_mul(16777619));
            let optionen = inhalt::planet::PlanetOptions {
                seed,
                zone: Zone::aus_name(p["zone"].as_str().unwrap_or("leben")).unwrap_or(Zone::Leben),
                overlay: Gut::aus_name(&self.overlay).and_then(|resource| {
                    faktor.map(|factor| inhalt::planet::ResourceOverlay { resource, profile_known: true, factor })
                }),
                ..Default::default()
            };
            if let Ok(bild) = inhalt::planet::render_orbit(320, &optionen) {
                self.texture = Some(ui.ctx().load_texture(
                    "kolonie",
                    egui::ColorImage::from_rgba_unmultiplied([bild.width as usize, bild.height as usize], &bild.pixels),
                    egui::TextureOptions::LINEAR,
                ));
                self.texture_key = schluessel;
            }
        }
        if let Some(t) = &self.texture {
            ui.image((t.id(), egui::vec2(groesse, groesse)));
        }
    }

    pub(crate) fn kolonie(&mut self, ui: &mut egui::Ui, v: &Value, p: &Value) {
        let r = self.session.regeln().clone();
        hilfe::kopf(ui, Bildschirm::Kolonie, &r);
        if p.is_null() {
            ui.label("Du hast keinen Planeten mehr.");
            return;
        }
        let zone = p["zone"].as_str().unwrap_or("");
        karte(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    self.planetenbild(ui, p, 230.0);
                    self.motiv(ui, "surfaces", zone, v);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Rohstoffschicht").small().color(LEISE));
                        egui::ComboBox::from_id_salt("schicht")
                            .selected_text(if self.overlay == "keine" { "keine".to_string() } else { name(&self.overlay) })
                            .show_ui(ui, |ui| {
                                for o in ["keine", "erz", "kristall", "deuterium", "nahrung"] {
                                    let n = if o == "keine" { "keine".to_string() } else { name(o) };
                                    ui.selectable_value(&mut self.overlay, o.to_string(), n);
                                }
                            });
                    });
                });
                ui.vertical(|ui| {
                    let art = if p["heimat"] == true { "Heimatwelt" } else { "Kolonie" };
                    ui.label(RichText::new(format!("{art} {}", text(&p["koord"]))).size(22.0).strong().color(GOLD));
                    ui.label(format!("{} – {}", name(zone), zone_text(zone)));
                    if p["nebel"] == true {
                        ui.label(RichText::new("Liegt in einem Nebel: hier gewinnt ein Xenoextraktor Xenokristall.").color(INFO));
                    }
                    let (belegt, gesamt) = (i(&p["felder"]["belegt"]), i(&p["felder"]["gesamt"]).max(1));
                    ui.add_space(4.0);
                    ui.label(RichText::new("Bauplätze").strong());
                    self.motiv_klein(ui, "stats", "baufelder", v);
                    balken(ui, belegt as f32 / gesamt as f32, format!("{belegt} von {gesamt} Feldern belegt"), fuellfarbe(belegt as f32 / gesamt as f32))
                        .on_hover_text("Jede Gebäudestufe belegt ein Feld. Terraforming und der Orbitalring schaffen neue.");
                    ui.add_space(4.0);
                    ui.label(RichText::new("Ertragsfaktoren dieses Planeten").strong());
                    egui::Grid::new("faktoren").num_columns(2).spacing([16.0, 2.0]).show(ui, |ui| {
                        for (k, n) in [("erz", "Erz"), ("kristall", "Kristall"), ("deuterium", "Deuterium"), ("nahrung", "Nahrung"), ("solar", "Solarstrom")] {
                            let f = i(&p["faktoren"][k]) as f32 / 1000.0;
                            ui.label(n);
                            let farbe = if f >= 1.1 { GUT } else if f <= 0.9 { SCHLECHT } else { TEXT };
                            ui.label(RichText::new(format!("{} %", (f * 100.0).round())).color(farbe))
                                .on_hover_text("100 % ist der Durchschnitt; Minen und Kraftwerke liefern hier entsprechend mehr oder weniger.");
                            ui.end_row();
                        }
                    });
                });
            });
            if let Some(b) = p["blockade"].as_object() {
                hinweis_karte(ui, SCHLECHT, |ui| {
                    ui.label(format!("⛔ Blockiert von {} ({}): Transporte kehren um, Marktlieferungen warten.", text(&b["durch"]), name(b["art"].as_str().unwrap_or(""))));
                });
            }
        });
        ui.add_space(6.0);
        ui.columns(2, |spalten| {
            karte(&mut spalten[0], |ui| {
                ui.label(RichText::new("👥 Bevölkerung").size(18.0).strong());
                ui.horizontal(|ui| {
                    for key in ["bevoelkerung", "wohnraum", "stabilitaet"] { self.motiv_klein(ui, "stats", key, v); }
                });
                let (bev, wohn) = (i(&p["bevoelkerung"]), i(&p["wohnraum"]).max(1));
                balken(ui, bev as f32 / wohn as f32, format!("{} Einwohner von {} Wohnraum", zahl(bev), zahl(wohn)), INFO)
                    .on_hover_text("Die Bevölkerung wächst, solange Wohnraum frei ist, Nahrung reicht und die Stabilität hoch genug ist. Wohnblöcke schaffen Wohnraum.");
                let (st, ziel) = (i(&p["stabilitaet"]), i(&p["stabilitaet_ziel"]));
                let unruhen = r.stabilitaet.unruhen_unter as i64;
                let farbe = if st < unruhen { SCHLECHT } else if st < 60 { WARNUNG } else { GUT };
                balken(ui, st as f32 / 100.0, format!("Stabilität {st} (strebt zu {ziel})"), farbe)
                    .on_hover_text(format!("Stabilität macht Gebäude produktiver und lässt die Bevölkerung wachsen (voll ab 60). Unter {unruhen} beginnen keine neuen Bauaufträge. Sie nähert sich langsam dem Zielwert."));
                if v["volk"] != "syntheten" {
                    let n = i(&p["nahrung_deckung"]);
                    balken(ui, n as f32 / 100.0, format!("Nahrung zu {n} % gedeckt"), if n < 100 { SCHLECHT } else { GUT })
                        .on_hover_text("Unter 100 % schrumpft die Bevölkerung. Farmen liefern Nahrung.");
                }
                let k = i(&p["konsum_deckung"]);
                balken(ui, k as f32 / 100.0, format!("Konsumgüter zu {k} % gedeckt"), if k < 50 { WARNUNG } else { GUT })
                    .on_hover_text("Gedeckte Konsumgüter heben die Stabilität um bis zu 20 Punkte. Das Konsumgüterwerk stellt sie her.");
            });
            karte(&mut spalten[1], |ui| {
                ui.label(RichText::new("⚡ Strom und Arbeit").size(18.0).strong());
                ui.horizontal(|ui| {
                    self.motiv_klein(ui, "resources", "energie", v);
                    for key in ["arbeitskraefte", "fachkraefte", "garnison"] { self.motiv_klein(ui, "stats", key, v); }
                });
                let (e, ver) = (i(&p["energie"]["erzeugung"]), i(&p["energie"]["verbrauch"]));
                balken(ui, if e > 0 { (ver as f32 / e as f32).min(1.0) } else { 1.0 }, format!("Strom: {} erzeugt, {} gebraucht", zahl(e), zahl(ver)), if ver > e { SCHLECHT } else { GUT })
                    .on_hover_text("Fehlt Strom, arbeiten alle Anlagen nur anteilig. Solar- und Fusionskraftwerke erzeugen Strom.");
                let (av, ab) = (i(&p["arbeit"]["verfuegbar"]), i(&p["arbeit"]["bedarf"]));
                balken(ui, if av > 0 { (ab as f32 / av as f32).min(1.0) } else { 1.0 }, format!("Arbeitskräfte: {} gebraucht, {} da", zahl(ab), zahl(av)), if ab > av { WARNUNG } else { GUT })
                    .on_hover_text("60 Prozent der Einwohner arbeiten. Fehlen Arbeitskräfte, bekommen die Gebäude mit der höchsten Priorität sie zuerst.");
                let (fv, fb) = (i(&p["fachkraefte"]["verfuegbar"]), i(&p["fachkraefte"]["bedarf"]));
                balken(ui, if fv > 0 { (fb as f32 / fv as f32).min(1.0) } else if fb > 0 { 1.0 } else { 0.0 }, format!("Fachkräfte: {} gebraucht, {} da", zahl(fb), zahl(fv)), if fb > fv { WARNUNG } else { GUT })
                    .on_hover_text("Labor, Werke und Werften brauchen Fachkräfte. Die Akademie bildet sie aus.");
                ui.label(RichText::new(format!("🛡 Garnison {} · Bunker schützt {} je Rohstoff", z(&p["garnison"]), z(&p["bunkerschutz"]))).small().color(LEISE));
                if i(&p["garnison"]) > 0 { self.motiv(ui, "ground_units", "garnison", v); }
            });
        });
        ui.add_space(6.0);
        karte(ui, |ui| {
            ui.label(RichText::new("📦 Lager").size(18.0).strong());
            egui::Grid::new("lager").striped(true).num_columns(5).spacing([18.0, 4.0]).show(ui, |ui| {
                for h in ["Gut", "Bestand", "je Stunde", "Lager", "Füllstand"] {
                    ui.label(RichText::new(h).strong().color(LEISE));
                }
                ui.end_row();
                for g in Gut::ALLE {
                    let k = g.name();
                    let (b, rt, l) = (i(&p["bestand"][k]), i(&p["rate"][k]), i(&p["lager"][k]).max(1));
                    if b == 0 && rt == 0 && matches!(g, Gut::Xenokristall | Gut::Antriebskern | Gut::Habitatmodul) {
                        continue;
                    }
                    ui.horizontal(|ui| {
                        self.motiv_klein(ui, "resources", k, v);
                        ui.label(format!("{} {}", gut_zeichen(k), name(k)));
                    });
                    ui.label(zahl(b));
                    ui.label(RichText::new(rate(rt)).color(if rt > 0 { GUT } else if rt < 0 { SCHLECHT } else { LEISE }));
                    ui.label(zahl(l));
                    let anteil = b as f32 / l as f32;
                    let hinweis = match p["voll_in_stunden"][k].as_i64() {
                        Some(0) => "voll".to_string(),
                        Some(h) => format!("voll {}", in_zeit(h * 60)),
                        None if rt < 0 && b > 0 => format!("leer {}", in_zeit(b / -rt * 60)),
                        None => format!("{} %", (anteil * 100.0).round()),
                    };
                    ui.add_sized([180.0, 18.0], egui::ProgressBar::new(anteil.clamp(0.0, 1.0)).fill(fuellfarbe(anteil).linear_multiply(0.75)).text(RichText::new(hinweis).small()));
                    ui.end_row();
                }
            });
        });
        ui.add_space(6.0);
        karte(ui, |ui| {
            ui.label(RichText::new("🏭 Gebäude auf diesem Planeten").size(18.0).strong());
            let gebaut: Vec<(String, i64)> = p["gebaeude"].as_object().into_iter().flatten().map(|(k, n)| (k.clone(), i(n))).filter(|(_, n)| *n > 0).collect();
            ui.horizontal_wrapped(|ui| {
                for (k, n) in gebaut {
                    etikett(ui, &format!("{} {n}", name(&k)), INFO);
                }
            });
            if ui.link("Zum Ausbau ›").clicked() {
                self.screen = Bildschirm::Gebaeude;
            }
        });
        if p["schiffe"].as_object().is_some_and(|s| !s.is_empty()) || p["verteidigung"].as_object().is_some_and(|s| !s.is_empty()) {
            ui.add_space(6.0);
            karte(ui, |ui| {
                ui.label(RichText::new("🚀 Im Orbit und am Boden").size(18.0).strong());
                ui.horizontal_wrapped(|ui| {
                    for gruppe in ["schiffe", "verteidigung"] {
                        for (k, n) in p[gruppe].as_object().into_iter().flatten() {
                            etikett(ui, &format!("{} × {}", z(n), name(k)), if gruppe == "schiffe" { GOLD } else { GUT });
                        }
                    }
                });
            });
        }
    }
}
