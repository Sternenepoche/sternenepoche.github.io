//! Flottenkommando: Angriffe auf dich, eigene Flotten, Verbände und eine neue Flotte Schritt für Schritt
//! mit Vorschau von Flugzeit, Treibstoff und Laderaum, für Angriffe mit Kampfsimulation.

use super::hilfe::{self, text};
use super::namen::{gut_zeichen, mission_text, name};
use super::befehl;
use super::stil::*;
use super::Bildschirm;
use crate::Player;
use eframe::egui::{self, RichText};
use kern::{Einheit, Gut, Mission, SCHIFFE};
use serde_json::{json, Map, Value};

/// Missionen, die mit Ladung fliegen.
const MIT_LADUNG: [&str; 5] = ["transport", "stationieren", "kolonisieren", "kampfkolonisieren", "saven"];
const FEINDLICH: [&str; 5] = ["angriff", "blockade", "invasion", "bombardieren", "kampfkolonisieren"];

impl Player {
    pub(crate) fn flotten(&mut self, ui: &mut egui::Ui, v: &Value, p: &Value) {
        let r = self.regeln.clone();
        hilfe::kopf(ui, Bildschirm::Flotten, &r);
        for a in v["angriffe"].as_array().into_iter().flatten() {
            hinweis_karte(ui, SCHLECHT, |ui| {
                let ships = a["schiffe"].as_i64().map(|n|n.to_string()).unwrap_or_else(||"unbekannte Zahl".into());
                let from = a["von"].as_str().unwrap_or("Unbekannter Angreifer");
                ui.label(RichText::new(format!("⚔ {} greift {} an: {} Schiffe, {} – Ankunft {} ({}).", from, text(&a["ziel"]), ships,
                    name(a["mission"].as_str().unwrap_or("")), text(&a["ankunft"]), in_zeit(i(&a["in_min"])))).strong());
                if v["aufklaerungsregeln"] == true && ui.button("Mit einer Spionagesonde untersuchen").clicked() {
                    self.act(befehl::flotte_ausspaehen(p["koord"].as_str().unwrap_or(""), &a["flotte"], 1));
                }
                if let Some(report) = a.get("sondenbericht") {
                    ui.label(format!("Sondenbericht ({} Sekunden alt): {}",i(&report["alter_sekunden"]),
                        report["schiffe"].as_i64().map(|n|format!("{n} Schiffe")).unwrap_or_else(||"durch Abschirmung verdeckt".into())));
                    if let Some(types) = report["schiffstypen"].as_object() {
                        ui.label(types.iter().map(|(kind,count)|format!("{} × {}",z(count),name(kind))).collect::<Vec<_>>().join(", "));
                    }
                }
                ui.label("Verteidigung bauen, Güter in den Bunker-Schutz bringen (verbrauchen oder verschicken) oder Verbündete um Hilfe bitten.");
            });
        }
        if let Some(an) = v["anfaengerschutz_bis"].as_str() {
            hinweis_karte(ui, INFO, |ui| {
                ui.label(format!("🛡 Anfängerschutz bis {an}: Niemand kann dich angreifen. Greifst du selbst an, endet er sofort."));
            });
        }
        let (belegt, gesamt) = (i(&v["flottenplaetze"]["belegt"]), i(&v["flottenplaetze"]["gesamt"]));
        titel(ui, &format!("✈ Deine Flotten ({belegt} von {gesamt} Flottenplätzen)"));
        let flotten = v["flotten"].as_array().cloned().unwrap_or_default();
        if flotten.is_empty() {
            ui.label(RichText::new("Keine Flotte unterwegs.").color(LEISE));
        }
        let verbaende = v["verbaende"].as_array().cloned().unwrap_or_default();
        for f in &flotten {
            karte(ui, |ui| {
                let m = f["mission"].as_str().unwrap_or("");
                let zustand = match f["zustand"].as_str().unwrap_or("") {
                    "hinflug" => "auf dem Hinflug, Ankunft",
                    "im_orbit" => "im Orbit bis",
                    _ => "auf dem Rückflug, zurück",
                };
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("Flotte {} · {}", z(&f["flotte"]), name(m))).strong().color(if FEINDLICH.contains(&m) { SCHLECHT } else { GOLD }));
                    ui.label(format!("{} › {}", text(&f["start"]), text(&f["ziel"])));
                    ui.label(RichText::new(format!("{zustand} {}", text(&f["bis"]))).color(LEISE));
                    if f["verbandsfuehrung"] == true {
                        etikett(ui, "führt einen Verband", WARNUNG);
                    } else if !f["verband"].is_null() {
                        etikett(ui, &format!("im Verband von Flotte {}", z(&f["verband"])), WARNUNG);
                    }
                });
                let schiffe: Vec<String> = f["schiffe"].as_object().into_iter().flatten().map(|(k, n)| format!("{} × {}", z(n), name(k))).collect();
                let ladung: Vec<String> = f["ladung"].as_object().into_iter().flatten().map(|(k, n)| format!("{} {}", gut_zeichen(k), z(n))).collect();
                ui.label(RichText::new(schiffe.join(", ")).small());
                if !ladung.is_empty() {
                    ui.label(RichText::new(format!("Ladung: {}", ladung.join("  "))).small().color(LEISE));
                }
                ui.horizontal(|ui| {
                    if f["zustand"] != "rueckflug" && ui.button("Zurückrufen").on_hover_text("Die Flotte kehrt sofort um.").clicked() {
                        self.act(befehl::flotte_zurueckrufen(&f["flotte"]));
                    }
                    if m == "angriff" && f["zustand"] == "hinflug" && f["verband"].is_null()
                        && ui.button("Als Verband öffnen").on_hover_text("Andere Flotten deiner Allianz mit demselben Ziel können sich anschließen und gemeinsam ankommen.").clicked()
                    {
                        self.act(befehl::verband_oeffnen(&f["flotte"]));
                    }
                    if m == "angriff" && f["zustand"] == "hinflug" && f["verband"].is_null() {
                        for vb in verbaende.iter().filter(|vb| vb["ziel"] == f["ziel"] && vb["fuehrung"] != f["flotte"]) {
                            if ui.button(format!("Verband von {} beitreten", text(&vb["spieler"]))).clicked() {
                                self.act(befehl::verband_beitreten(&f["flotte"], &vb["fuehrung"]));
                            }
                        }
                    }
                });
            });
        }
        if !verbaende.is_empty() {
            ui.label(RichText::new(format!("Offene Verbände: {}", verbaende.iter().map(|vb| format!("Flotte {} von {} nach {} ({})", z(&vb["fuehrung"]), text(&vb["spieler"]), text(&vb["ziel"]), text(&vb["ankunft"]))).collect::<Vec<_>>().join("; "))).small().color(LEISE));
        }
        if !p.is_null() {
            ui.add_space(8.0);
            self.neue_flotte(ui, v, p);
        }
    }

    fn neue_flotte(&mut self, ui: &mut egui::Ui, v: &Value, p: &Value) {
        let k = text(&p["koord"]);
        titel(ui, &format!("✈ Neue Flotte von {k}"));
        if i(&p["gebaeude"]["raumhafen"]) == 0 {
            hinweis_karte(ui, WARNUNG, |ui| {
                ui.label("Dieser Planet hat keinen Raumhafen. Flotten starten nur von Planeten mit Raumhafen (ab Stufe III).");
            });
            return;
        }
        // 1. Ziel
        karte(ui, |ui| {
            ui.label(RichText::new("1. Ziel").strong().color(GOLD));
            ui.horizontal(|ui| {
                ui.label("Koordinate");
                ui.add(egui::TextEdit::singleline(&mut self.target).desired_width(110.0).hint_text("1:27:6"))
                    .on_hover_text("Sektor:System:Position. Position 0 ist der Asteroidengürtel eines Systems.");
                let mut wahl: Option<String> = None;
                egui::ComboBox::from_id_salt("zielwahl").selected_text("aus einer Liste wählen …").width(260.0).show_ui(ui, |ui| {
                    let mut gruppe = |ui: &mut egui::Ui, titel: &str, eintraege: Vec<(String, String)>| {
                        if eintraege.is_empty() {
                            return;
                        }
                        ui.label(RichText::new(titel).small().color(LEISE));
                        for (koord, beschreibung) in eintraege {
                            if ui.selectable_label(false, format!("{koord}  {beschreibung}")).clicked() {
                                wahl = Some(koord);
                            }
                        }
                        ui.separator();
                    };
                    gruppe(ui, "Eigene Planeten", v["planeten"].as_array().into_iter().flatten().filter(|x| x["koord"] != p["koord"])
                        .map(|x| (text(&x["koord"]), if x["heimat"] == true { "Heimatwelt".into() } else { "Kolonie".into() })).collect());
                    gruppe(ui, "Nachbarn", v["nachbarn"].as_array().into_iter().flatten().take(25)
                        .map(|x| (text(&x["koord"]), format!("{} · {} Punkte{}", text(&x["spieler"]), z(&x["punkte"]), if x["schutz"] == true { " · Anfängerschutz" } else { "" }))).collect());
                    gruppe(ui, "Erkundete freie Plätze", v["erkundet"].as_array().into_iter().flatten().filter(|x| x["frei"] == true)
                        .map(|x| (text(&x["koord"]), format!("{} Felder, {}", z(&x["felder"]), name(x["zone"].as_str().unwrap_or(""))))).collect());
                    gruppe(ui, "Trümmerfelder", v["truemmer"].as_array().into_iter().flatten()
                        .map(|x| (text(&x["koord"]), format!("{} Erz, {} Kristall", z(&x["erz"]), z(&x["kristall"])))).collect());
                });
                if let Some(w) = wahl {
                    self.target = w;
                    self.flotten_vorschau = None;
                }
            });
            if self.target.parse::<kern::Koord>().is_err() && !self.target.is_empty() {
                ui.label(RichText::new("Die Koordinate hat die Form Sektor:System:Position, etwa 1:27:6.").small().color(SCHLECHT));
            }
        });
        // 2. Mission
        karte(ui, |ui| {
            ui.label(RichText::new("2. Mission").strong().color(GOLD));
            ui.horizontal_wrapped(|ui| {
                for m in Mission::ALLE {
                    let n = m.name();
                    if n == "flotten_spionage" || (n == "saven" && v["aufklaerungsregeln"] != true) { continue; }
                    let modern = i(&v["kolonisationsversion"]) > 0;
                    if (modern && n == "invasion") || (!modern && ["bombardieren", "kampfkolonisieren"].contains(&n)) { continue; }
                    let farbe = if FEINDLICH.contains(&n) { SCHLECHT } else { TEXT };
                    if ui.selectable_label(self.mission == n, RichText::new(name(n)).color(farbe)).clicked() {
                        self.mission = n.to_string();
                        self.flotten_simulation = None;
                    }
                }
            });
            ui.label(RichText::new(mission_text(&self.mission)).color(LEISE));
            let mission = self.mission.clone();
            self.motiv(ui, "missions", &mission, v);
            if mission == "transport" && i(&v["kolonisationsversion"]) >= 2 {
                let targets: Vec<Value> = v["flotten"].as_array().into_iter().flatten()
                    .filter(|f| f["zustand"] == "im_orbit" && f["ziel"] == self.target).cloned().collect();
                if !targets.iter().any(|f| f["flotte"].as_u64() == self.versorgungsziel.map(u64::from)) { self.versorgungsziel = None; }
                ui.horizontal_wrapped(|ui| {
                    ui.selectable_value(&mut self.versorgungsziel, None, "Planet beliefern");
                    for f in targets {
                        if let Some(id) = f["flotte"].as_u64() {
                            ui.selectable_value(&mut self.versorgungsziel, Some(id as u32), format!("Eigene Orbitflotte {id} versorgen"));
                        }
                    }
                });
            }
            if ["kolonisieren", "kampfkolonisieren"].contains(&mission.as_str()) && i(&v["kolonisationsversion"]) > 0 {
                if let Ok(rules) = self.session.query(&json!({"typ":"regel", "stichwort":"kolonie"})) {
                    ui.label("Sonde zuerst; Kolonieschiff mit bewaffneter Eskorte und Startfracht losschicken. Die Fracht wird nach Ankunft selbst verbaut.");
                    kosten(ui, &rules["kolonie_startfracht"], &p["bestand"], 1);
                }
            }
            if ["halten", "abbau", "saven"].contains(&self.mission.as_str()) {
                ui.horizontal(|ui| {
                    ui.label("Aufenthalt am Ziel");
                    let max = if self.mission == "halten" { 168 } else if self.mission == "saven" {72} else { 48 };
                    let min = if self.mission == "saven" {0} else {1};
                    ui.add(egui::DragValue::new(&mut self.hold_hours).range(min..=max).suffix(" Stunden"));
                });
            }
        });
        // 3. Schiffe
        let mut gewaehlt = Map::new();
        karte(ui, |ui| {
            ui.label(RichText::new("3. Schiffe").strong().color(GOLD));
            let mut irgendwelche = false;
            egui::Grid::new("schiffswahl").num_columns(4).spacing([12.0, 4.0]).show(ui, |ui| {
                for e in Einheit::ALLE.into_iter().take(SCHIFFE) {
                    let vorhanden = i(&p["schiffe"][e.name()]);
                    self.ships[e.idx()] = self.ships[e.idx()].clamp(0, vorhanden);
                    if vorhanden == 0 {
                        continue;
                    }
                    irgendwelche = true;
                    ui.label(name(e.name()));
                    if ui.add(egui::DragValue::new(&mut self.ships[e.idx()]).range(0..=vorhanden)).changed() {
                        self.flotten_vorschau = None;
                    }
                    ui.label(RichText::new(format!("von {}", zahl(vorhanden))).color(LEISE));
                    if ui.small_button("alle").clicked() {
                        self.ships[e.idx()] = vorhanden;
                        self.flotten_vorschau = None;
                    }
                    ui.end_row();
                }
            });
            if !irgendwelche {
                ui.label(RichText::new("Auf diesem Planeten stehen keine Schiffe. Baue welche in der 🚀 Werft.").color(LEISE));
            }
        });
        for e in Einheit::ALLE.into_iter().take(SCHIFFE) {
            if self.ships[e.idx()] > 0 {
                gewaehlt.insert(e.name().into(), json!(self.ships[e.idx()]));
            }
        }
        // 4. Ladung und Tempo
        let mut ladung = Map::new();
        karte(ui, |ui| {
            ui.label(RichText::new("4. Ladung und Tempo").strong().color(GOLD));
            if MIT_LADUNG.contains(&self.mission.as_str()) {
                egui::Grid::new("ladung").num_columns(4).spacing([12.0, 4.0]).show(ui, |ui| {
                    for g in Gut::ALLE.into_iter().take(8) {
                        let da = i(&p["bestand"][g.name()]);
                        self.cargo[g.idx()] = self.cargo[g.idx()].clamp(0, da);
                        ui.label(format!("{} {}", gut_zeichen(g.name()), name(g.name())));
                        ui.add(egui::DragValue::new(&mut self.cargo[g.idx()]).range(0..=da).speed(10.0));
                        ui.label(RichText::new(format!("von {}", zahl(da))).color(LEISE));
                        ui.end_row();
                    }
                });
                if self.mission == "kolonisieren" {
                    ui.label(RichText::new("Tipp: Eine neue Kolonie hat anfangs nur, was das Schiff mitbringt. Ohne Nahrung hungert sie vom ersten Tag an.").small().color(WARNUNG));
                }
            } else {
                ui.label(RichText::new("Diese Mission fliegt ohne Ladung.").color(LEISE));
            }
            for g in Gut::ALLE.into_iter().take(8) {
                if self.cargo[g.idx()] > 0 && MIT_LADUNG.contains(&self.mission.as_str()) {
                    ladung.insert(g.name().into(), json!(self.cargo[g.idx()]));
                }
            }
            ui.horizontal(|ui| {
                ui.label("Tempo");
                if ui.add(egui::Slider::new(&mut self.flight_speed, 0.1..=1.0).step_by(0.1).custom_formatter(|x, _| format!("{} %", (x * 100.0).round()))).changed() {
                    self.flotten_vorschau = None;
                }
                ui.label(RichText::new("Langsamer fliegen spart Treibstoff.").small().color(LEISE));
            });
        });
        // 5. Vorschau und Start
        let ziel_ok = self.target.parse::<kern::Koord>().is_ok();
        let anfrage = json!({"typ": "flugzeit", "start": k, "ziel": self.target, "schiffe": gewaehlt, "geschwindigkeit": self.flight_speed});
        let schluessel = anfrage.to_string();
        if ziel_ok && !gewaehlt.is_empty() && self.flotten_vorschau.as_ref().is_none_or(|(s, _)| *s != schluessel) {
            let antwort = self.session.query(&anfrage);
            self.flotten_vorschau = Some((schluessel, antwort));
        }
        let ladung_summe: i64 = ladung.values().map(i).sum();
        karte(ui, |ui| {
            ui.label(RichText::new("5. Vorschau und Start").strong().color(GOLD));
            let mut bereit = ziel_ok && !gewaehlt.is_empty();
            match (&self.flotten_vorschau, ziel_ok && !gewaehlt.is_empty()) {
                (Some((_, Ok(vs))), true) => {
                    let kap = i(&vs["ladekapazitaet"]);
                    let treibstoff = i(&vs["treibstoff_je_strecke"]) * 2;
                    kennzahlen(ui, &[
                        (dauer_min(i(&vs["dauer_min"])), "Flugzeit je Strecke", INFO),
                        (zahl(treibstoff), "Deuterium hin und zurück", if treibstoff > i(&p["bestand"]["deuterium"]) { SCHLECHT } else { TEXT }),
                        (format!("{} / {}", zahl(ladung_summe), zahl(kap)), "Ladung / Laderaum", if ladung_summe > kap { SCHLECHT } else { TEXT }),
                    ]);
                    if ladung_summe > kap {
                        ui.label(RichText::new("Die Ladung passt nicht in die Schiffe.").color(SCHLECHT));
                        bereit = false;
                    }
                }
                (Some((_, Err(e))), true) => {
                    ui.label(RichText::new(format!("So geht es nicht: {e}")).color(SCHLECHT));
                }
                _ => {
                    ui.label(RichText::new("Wähle ein Ziel und mindestens ein Schiff, dann erscheint hier die Vorschau.").color(LEISE));
                }
            }
            if FEINDLICH.contains(&self.mission.as_str()) && bereit {
                if ui.button("Kampfsimulation").on_hover_text("Braucht einen Spionagebericht des Ziels mit Schiffen und Verteidigung.").clicked() {
                    self.flotten_simulation = Some(self.session.query(&json!({"typ": "kampfsimulator", "ziel": self.target, "schiffe": gewaehlt})));
                }
                match &self.flotten_simulation {
                    Some(Ok(s)) => {
                        let chance = i(&s["siegchance_prozent"]);
                        kennzahlen(ui, &[
                            (format!("{chance} %"), "Siegchance", if chance >= 70 { GUT } else if chance >= 40 { WARNUNG } else { SCHLECHT }),
                            (z(&s["eigene_verluste_wert"]), "eigene Verluste (Wert)", TEXT),
                            (z(&s["verluste_gegner_wert"]), "Verluste des Gegners", TEXT),
                        ]);
                        let beute: Vec<String> = s["beute_bei_sieg_ohne_ladegrenze"].as_object().into_iter().flatten().map(|(g, n)| format!("{} {}", gut_zeichen(g), z(n))).collect();
                        ui.label(RichText::new(format!("Beute bei Sieg (ohne Laderaumgrenze): {}", if beute.is_empty() { "nichts".into() } else { beute.join("  ") })).small());
                        ui.label(RichText::new(format!("Bericht {} Stunden alt, Technik des Gegners {}.", z(&s["bericht_alter_stunden"]), text(&s["technik_des_gegners"]))).small().color(LEISE));
                    }
                    Some(Err(e)) => {
                        ui.label(RichText::new(e).color(WARNUNG));
                    }
                    None => {}
                }
            }
            let aktion = if self.mission == "transport" && self.versorgungsziel.is_some() {
                befehl::flotte_versorgen(&k, &self.target, &json!(self.versorgungsziel.unwrap()), &gewaehlt, self.flight_speed, &ladung)
            } else {
                befehl::flotte_senden(&k, &self.target, &self.mission, &gewaehlt, self.flight_speed, &ladung, self.hold_hours)
            };
            ui.add_space(4.0);
            if hauptknopf(ui, &format!("🚀 Flotte starten: {}", name(&self.mission)), bereit) {
                if FEINDLICH.contains(&self.mission.as_str()) {
                    self.bestaetigen(
                        &format!("{} gegen {} starten? Das bricht Nichtangriffspakte mit dem Ziel und beendet deinen Anfängerschutz.", name(&self.mission), self.target),
                        aktion,
                    );
                } else {
                    self.act(aktion);
                }
                self.flotten_vorschau = None;
            }
        });
    }
}
