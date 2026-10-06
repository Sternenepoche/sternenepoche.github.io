//! Zivilisation und Regierung: Stufenleiter, Bedingungen der nächsten Stufe mit Haltezeit, Aufstieg, Steuersatz
//! mit Vorschau, Punkte, Kolonien und Töpfe.

use super::hilfe::{self, text};
use super::namen::{name, topf_text};
use super::befehl;
use super::stil::*;
use super::Bildschirm;
use crate::Player;
use eframe::egui::{self, RichText};
use kern::Volk;
use serde_json::Value;

impl Player {
    pub(crate) fn regierung(&mut self, ui: &mut egui::Ui, v: &Value) {
        let r = self.regeln.clone();
        hilfe::kopf(ui, Bildschirm::Regierung, &r);
        let stufe = i(&v["stufe"]);
        let faction = v["volk"].as_str().unwrap_or("");
        self.motiv_groesse(ui, "civilizations", faction, v, egui::vec2(380.0, 150.0));
        karte(ui, |ui| {
            ui.label(RichText::new("👑 Zivilisationsstufen").size(18.0).strong());
            ui.horizontal_wrapped(|ui| {
                for n in 1..=5 {
                    let key = ["gruendung", "industrie", "orbit", "sternenflug", "imperium"][n as usize - 1];
                    self.motiv(ui, "progression", key, v);
                    let farbe = if n < stufe { GUT } else if n == stufe { GOLD } else { LEISE };
                    let titel = if n == 1 { "I Gründung".to_string() } else { r.stufen.get(n as usize - 2).map(|s| s.name.clone()).unwrap_or_default() };
                    let antwort = etikett(ui, &titel, farbe);
                    if n >= 2 {
                        if let Some(s) = r.stufen.get(n as usize - 2) {
                            antwort.on_hover_text(format!("Schaltet frei: {}", s.schaltet_frei));
                        }
                    }
                    if n < 5 {
                        ui.label(RichText::new("➡").color(LEISE));
                    }
                }
            });
        });
        ui.add_space(6.0);
        match v["naechste_stufe"].as_object() {
            None => {
                hinweis_karte(ui, GOLD, |ui| {
                    ui.label(RichText::new("Du hast die höchste Stufe erreicht: Imperium. Jetzt zählen Ausbau, Forschung und die Großprojekte.").size(17.0));
                });
            }
            Some(st) => {
                let heimat = v["planeten"].as_array().into_iter().flatten().find(|p| p["heimat"] == true).cloned().unwrap_or(Value::Null);
                karte(ui, |ui| {
                    ui.label(RichText::new(format!("Nächste Stufe: {}", text(&st["name"]))).size(18.0).strong().color(GOLD));
                    if let Some(s) = r.stufen.get(stufe as usize - 1) {
                        ui.label(RichText::new(format!("Schaltet frei: {}", s.schaltet_frei)).color(LEISE));
                    }
                    ui.add_space(4.0);
                    let mut alle = true;
                    for b in st["bedingungen"].as_array().into_iter().flatten() {
                        let ok = b["erfuellt"] == true;
                        alle &= ok;
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(if ok { "✔" } else { "✖" }).color(if ok { GUT } else { SCHLECHT }));
                            ui.label(super::namen::woerter(&text(&b["text"])));
                        });
                    }
                    let (seit, halte) = (i(&st["erfuellt_seit_stunden"]), i(&st["haltezeit_stunden"]).max(1));
                    ui.add_space(4.0);
                    if alle {
                        balken(ui, seit as f32 / halte as f32, format!("Alle Bedingungen erfüllt seit {seit} von {halte} Stunden"), GUT);
                    } else {
                        ui.label(RichText::new(format!("Sind alle Bedingungen erfüllt, müssen sie {halte} Stunden ohne Unterbrechung halten.")).color(LEISE));
                    }
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Kosten des Aufstiegs (von der Heimatwelt):").color(LEISE));
                        kosten(ui, &st["kosten"], &heimat["bestand"], 1);
                    });
                    let bereit = alle && seit >= halte;
                    let grund = if !alle { "Noch sind nicht alle Bedingungen erfüllt." } else if seit < halte { "Die Bedingungen müssen noch länger halten." } else { "" };
                    if bereit && !bezahlbar(&st["kosten"], &heimat["bestand"], 1) {
                        let wann = wartezeit(&st["kosten"], &heimat, 1).map(|w| format!(" ({w})")).unwrap_or_default();
                        ui.label(RichText::new(format!("Die Heimatwelt hat noch nicht alle Güter für den Aufstieg{wann}.")).color(WARNUNG));
                    }
                    let mut klick = false;
                    ui.horizontal(|ui| {
                        klick = hauptknopf(ui, &format!("👑 Aufsteigen zu {}", text(&st["name"])), bereit);
                        if !bereit {
                            ui.label(RichText::new(grund).small().color(LEISE));
                        }
                    });
                    if klick {
                        self.act(befehl::stufenaufstieg());
                    }
                });
            }
        }
        ui.add_space(6.0);
        ui.columns(2, |sp| {
            karte(&mut sp[0], |ui| {
                ui.label(RichText::new("💰 Steuern").size(18.0).strong());
                let aktuell = i(&v["steuersatz"]);
                if self.steuer_entwurf.is_none_or(|(basis, _)| basis != aktuell) {
                    self.steuer_entwurf = Some((aktuell, aktuell));
                }
                let (_, mut satz) = self.steuer_entwurf.unwrap();
                ui.spacing_mut().slider_width = 260.0;
                ui.add(egui::Slider::new(&mut satz, 0..=50).suffix(" %"));
                self.steuer_entwurf = Some((aktuell, satz));
                let einwohner: i64 = v["planeten"].as_array().into_iter().flatten().map(|p| i(&p["bevoelkerung"])).sum();
                let volk = Volk::aus_name(v["volk"].as_str().unwrap_or("")).unwrap_or(Volk::Aurelianer);
                let je_stunde = einwohner as f64 * satz as f64 / 100.0 * r.wirtschaft.steuer_je_einwohner_stunde * r.volk(volk).steuer;
                ui.label(format!("Einnahmen etwa {} Credits je Stunde ({} je Tag)", zahl(je_stunde.round() as i64), zahl((je_stunde * 24.0).round() as i64)));
                let frei = r.stabilitaet.steuer_frei_bis as i64;
                let malus = ((satz - frei).max(0) as f64 * r.stabilitaet.steuer_malus_je_punkt).round() as i64;
                ui.label(RichText::new(if malus > 0 { format!("Kostet {malus} Punkte Stabilitätsziel (frei bis {frei} %).") } else { format!("Bis {frei} % kostet der Steuersatz keine Stabilität.") })
                    .color(if malus > 0 { WARNUNG } else { GUT }));
                ui.label(RichText::new(format!("Credits jetzt: {}", z(&v["credits"]))).color(LEISE));
                if knopf(ui, "Steuersatz übernehmen", satz != aktuell, "Unverändert.") {
                    self.act(befehl::steuersatz(satz));
                }
            });
            karte(&mut sp[1], |ui| {
                ui.label(RichText::new("🏆 Punkte").size(18.0).strong());
                let pk = &v["punkte"];
                ui.label(RichText::new(format!("{} Punkte · Rang {} von {}", z(&pk["gesamt"]), z(&v["rang"]), z(&v["spielerzahl"]))).size(17.0).color(GOLD));
                let gesamt = i(&pk["gesamt"]).max(1) as f32;
                for (k, erklaerung) in [
                    ("wirtschaft", "Wert aller Gebäudestufen"),
                    ("forschung", "Wert aller erforschten Stufen"),
                    ("militaer", "aktueller Wert von Flotte und Verteidigung"),
                    ("zivilisation", "Bevölkerung und Stufenbonus"),
                ] {
                    balken(ui, i(&pk[k]) as f32 / gesamt, format!("{}: {}", name(k), z(&pk[k])), INFO).on_hover_text(erklaerung);
                }
                ui.label(RichText::new(format!("1 Punkt = {} Werteinheiten investierter Güter; je {} Einwohner 1 Punkt.", zahl(r.wertung.einheit), zahl(r.wertung.einwohner_je_punkt))).small().color(LEISE));
            });
        });
        ui.add_space(6.0);
        ui.columns(2, |sp| {
            karte(&mut sp[0], |ui| {
                ui.label(RichText::new("🌍 Kolonien").size(18.0).strong());
                let ko = &v["kolonien"];
                ui.label(format!("{} von {} erlaubten Kolonien", z(&ko["anzahl"]), z(&ko["erlaubt"])));
                ui.label(RichText::new(format!("Verwaltungsgrenze {}: jede Kolonie darüber kostet 5 Punkte Stabilität. Astrophysik erlaubt mehr Kolonien; je zwei Stufen Verwaltungszentrum heben die Grenze um eins.", z(&ko["verwaltungsgrenze"]))).small().color(LEISE));
                if let Some(s) = v["anfaengerschutz_bis"].as_str() {
                    ui.label(RichText::new(format!("🛡 Anfängerschutz bis {s}")).color(INFO));
                }
            });
            karte(&mut sp[1], |ui| {
                ui.label(RichText::new("🏦 Töpfe").size(18.0).strong());
                ui.label(RichText::new("Die Töpfe teilen das Einkommen der vier Rollen eines Modellreichs auf. Du regierst selbst und bezahlst direkt aus Lager und Credits; die Töpfe sind hier nur zur Information.").small().color(LEISE));
                for (t, w) in v["toepfe"].as_object().into_iter().flatten() {
                    ui.label(format!("{} – {} % des Einkommens, Guthaben {} ({})", name(t), z(&w["anteil"]), z(&w["guthaben"]), topf_text(t)));
                }
            });
        });
    }
}
