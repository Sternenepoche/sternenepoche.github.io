//! Aussehen und kleine Bausteine der Oberfläche: Farben, Karten, Kennzahlen, Balken, Zahlen- und Zeitformat.
//! Alle Bildschirme benutzen nur diese Bausteine, damit das Spiel überall gleich aussieht.

use eframe::egui::{self, Color32, RichText, Stroke};
use serde_json::Value;

pub const GOLD: Color32 = Color32::from_rgb(222, 183, 114);
pub const TEXT: Color32 = Color32::from_rgb(226, 231, 237);
pub const LEISE: Color32 = Color32::from_rgb(140, 160, 178);
pub const GUT: Color32 = Color32::from_rgb(126, 204, 152);
pub const SCHLECHT: Color32 = Color32::from_rgb(240, 124, 112);
pub const WARNUNG: Color32 = Color32::from_rgb(236, 190, 92);
pub const INFO: Color32 = Color32::from_rgb(118, 172, 232);
pub const FLAECHE: Color32 = Color32::from_rgb(12, 21, 31);
pub const KARTE: Color32 = Color32::from_rgb(19, 33, 47);
pub const KARTE_HELL: Color32 = Color32::from_rgb(26, 44, 62);
pub const RAND: Color32 = Color32::from_rgb(44, 66, 88);

/// Dunkles Weltraumthema mit goldenem Akzent; größere Schrift und Abstände für gute Lesbarkeit.
pub fn thema(ctx: &egui::Context) {
    let mut v = egui::Visuals::dark();
    v.panel_fill = FLAECHE;
    v.window_fill = Color32::from_rgb(17, 29, 42);
    v.extreme_bg_color = Color32::from_rgb(8, 14, 21);
    v.faint_bg_color = Color32::from_rgb(16, 28, 40);
    v.selection.bg_fill = Color32::from_rgb(52, 92, 122);
    v.selection.stroke = Stroke::new(1.0_f32, GOLD);
    v.hyperlink_color = INFO;
    v.override_text_color = Some(TEXT);
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, RAND);
    v.widgets.inactive.weak_bg_fill = Color32::from_rgb(30, 48, 66);
    v.widgets.inactive.bg_fill = Color32::from_rgb(30, 48, 66);
    v.widgets.hovered.weak_bg_fill = Color32::from_rgb(44, 70, 94);
    v.widgets.hovered.bg_fill = Color32::from_rgb(44, 70, 94);
    v.widgets.active.weak_bg_fill = Color32::from_rgb(60, 92, 120);
    v.window_corner_radius = egui::CornerRadius::same(10);
    ctx.set_visuals(v);
    ctx.style_mut(|s| {
        use egui::{FontId, TextStyle};
        s.text_styles.insert(TextStyle::Body, FontId::proportional(16.0));
        s.text_styles.insert(TextStyle::Button, FontId::proportional(15.5));
        s.text_styles.insert(TextStyle::Heading, FontId::proportional(25.0));
        s.text_styles.insert(TextStyle::Small, FontId::proportional(13.0));
        s.text_styles.insert(TextStyle::Monospace, FontId::monospace(14.0));
        s.spacing.item_spacing = egui::vec2(8.0, 6.0);
        s.spacing.button_padding = egui::vec2(10.0, 4.0);
        s.spacing.interact_size.y = 28.0;
    });
}

/// Zahl mit Tausenderpunkt: 1234567 → „1.234.567“.
pub fn zahl(n: i64) -> String {
    let s = n.unsigned_abs().to_string();
    let mut aus = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            aus.push('.');
        }
        aus.push(c);
    }
    if n < 0 {
        format!("−{aus}")
    } else {
        aus
    }
}

/// Zahl aus JSON, gerundet und mit Tausenderpunkt; fehlt sie, ein Strich.
pub fn z(v: &Value) -> String {
    v.as_f64().map(|x| zahl(x.round() as i64)).unwrap_or_else(|| "–".into())
}

pub fn i(v: &Value) -> i64 {
    v.as_f64().map(|x| x.round() as i64).unwrap_or(0)
}

/// Kommazahl mit Dezimalkomma.
pub fn komma(x: f64, stellen: usize) -> String {
    format!("{x:.stellen$}").replace('.', ",")
}

/// Mit Vorzeichen, für Raten: „+15“, „−3“.
pub fn rate(n: i64) -> String {
    if n > 0 {
        format!("+{}", zahl(n))
    } else {
        zahl(n)
    }
}

/// Dauer in Spielminuten, lesbar: „45 min“, „2 h 10 min“, „3 Tage 4 h“.
pub fn dauer_min(min: i64) -> String {
    let min = min.max(0);
    if min < 60 {
        format!("{min} min")
    } else if min < 24 * 60 {
        let (h, m) = (min / 60, min % 60);
        if m == 0 {
            format!("{h} h")
        } else {
            format!("{h} h {m} min")
        }
    } else {
        let (t, h) = (min / (24 * 60), min % (24 * 60) / 60);
        let tage = if t == 1 { "1 Tag".to_string() } else { format!("{t} Tage") };
        if h == 0 {
            tage
        } else {
            format!("{tage} {h} h")
        }
    }
}

/// Nach „in“ steht der Dativ: „in 6 Tagen 4 h“, „in 1 Tag“, „in 45 min“.
pub fn in_zeit(min: i64) -> String {
    format!("in {}", dauer_min(min).replace(" Tage", " Tagen"))
}

pub fn dauer_h(h: i64) -> String {
    dauer_min(h * 60)
}

/// Spielzeit aus Sekunden: „Tag 4, 03:15“.
pub fn spielzeit(sekunden: i64) -> String {
    format!("Tag {}, {:02}:{:02}", sekunden / 86_400 + 1, sekunden % 86_400 / 3600, sekunden % 3600 / 60)
}

pub fn roemisch(stufe: i64) -> &'static str {
    match stufe {
        1 => "I",
        2 => "II",
        3 => "III",
        4 => "IV",
        5 => "V",
        _ => "?",
    }
}

/// Eine Karte: Fläche mit Rand und Innenabstand, für zusammengehörige Inhalte.
pub fn karte<R>(ui: &mut egui::Ui, inhalt: impl FnOnce(&mut egui::Ui) -> R) -> R {
    egui::Frame::new()
        .fill(KARTE)
        .stroke(Stroke::new(1.0_f32, RAND))
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            inhalt(ui)
        })
        .inner
}

/// Karte mit farbigem linken Rand, etwa für Warnungen oder Tipps.
pub fn hinweis_karte<R>(ui: &mut egui::Ui, farbe: Color32, inhalt: impl FnOnce(&mut egui::Ui) -> R) -> R {
    egui::Frame::new()
        .fill(farbe.linear_multiply(0.12))
        .stroke(Stroke::new(1.0_f32, farbe.linear_multiply(0.6)))
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            inhalt(ui)
        })
        .inner
}

/// Hover-Fläche über einer ganzen Gruppe (Kachel, Etikett, Rohstoffanzeige), damit ihr Tooltip erscheint.
/// egui legt die Fläche einer Gruppe unter ihre Beschriftungen, und die (auswählbaren) Beschriftungen fangen den
/// Mauszeiger ab; eine danach angelegte Fläche liegt oben.
pub fn ueber(ui: &mut egui::Ui, gruppe: egui::Response) -> egui::Response {
    ui.interact(gruppe.rect, gruppe.id.with("ueber"), egui::Sense::hover())
}

/// Kennzahl-Kachel: großer Wert, kleine Beschriftung darunter.
pub fn kennzahl(ui: &mut egui::Ui, wert: impl Into<String>, beschriftung: &str, farbe: Color32) -> egui::Response {
    let r = egui::Frame::new()
        .fill(KARTE_HELL)
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
            ui.set_min_width(130.0);
            ui.set_max_width(130.0);
            ui.vertical(|ui| {
                ui.label(RichText::new(wert.into()).size(21.0).color(farbe).strong());
                ui.label(RichText::new(beschriftung).small().color(LEISE));
            });
        })
        .response;
    ueber(ui, r)
}

/// Mehrere Kennzahlen, auf so viele Zeilen verteilt, wie die Breite verlangt. Eine umbrechende Zeile reicht nicht:
/// egui kann Kacheln (Container) erst nach dem Zeichnen messen und würde sie über den Rand hinaus legen.
pub fn kennzahlen(ui: &mut egui::Ui, werte: &[(String, &str, Color32)]) {
    const BREITE: f32 = 156.0;
    let passen = ((ui.available_width() + 8.0) / (BREITE + 8.0)).floor().max(1.0) as usize;
    // Gleichmäßig verteilen: sieben Kacheln bei sechs Plätzen werden 4 + 3, nicht 6 + 1.
    let zeilen = werte.len().div_ceil(passen).max(1);
    let je_zeile = werte.len().div_ceil(zeilen).max(1);
    for zeile in werte.chunks(je_zeile) {
        ui.horizontal(|ui| {
            for (wert, beschriftung, farbe) in zeile {
                kennzahl(ui, wert.clone(), beschriftung, *farbe);
            }
        });
    }
}

/// Fortschrittsbalken mit Text im Balken; `anteil` 0..1.
pub fn balken(ui: &mut egui::Ui, anteil: f32, text: impl Into<String>, farbe: Color32) -> egui::Response {
    ui.add(
        egui::ProgressBar::new(anteil.clamp(0.0, 1.0))
            .text(RichText::new(text.into()).color(Color32::WHITE).small())
            .fill(farbe.linear_multiply(0.75))
            .desired_height(18.0),
    )
}

/// Farbe für einen Füllstand: grün, ab 85 % gelb, ab 98 % rot (Lager voll).
pub fn fuellfarbe(anteil: f32) -> Color32 {
    if anteil >= 0.98 {
        SCHLECHT
    } else if anteil >= 0.85 {
        WARNUNG
    } else {
        INFO
    }
}

/// Überschrift eines Bildschirms mit einem Satz, worum es geht.
pub fn kopf(ui: &mut egui::Ui, zeichen: &str, titel: &str, satz: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(zeichen).size(26.0));
        ui.label(RichText::new(titel).heading().color(GOLD));
    });
    ui.label(RichText::new(satz).color(LEISE));
    ui.add_space(4.0);
}

/// Zwischenüberschrift innerhalb eines Bildschirms.
pub fn titel(ui: &mut egui::Ui, text: &str) {
    ui.add_space(6.0);
    ui.label(RichText::new(text).size(19.0).strong().color(TEXT));
}

/// Aufklappbare Erklärung „So funktioniert es“ mit eigenem Text und den Regeln im Wortlaut.
pub fn erklaerung(ui: &mut egui::Ui, id: &str, text: &str, regeln: &[(String, String)]) {
    egui::CollapsingHeader::new(RichText::new("ℹ  So funktioniert es").color(INFO))
        .id_salt(("erklaerung", id))
        .show(ui, |ui| {
            for absatz in text.split("\n\n") {
                ui.label(absatz.trim());
                ui.add_space(2.0);
            }
            if !regeln.is_empty() {
                egui::CollapsingHeader::new(RichText::new("Regeln im Wortlaut").color(LEISE))
                    .id_salt(("regeln", id))
                    .show(ui, |ui| {
                        for (titel, text) in regeln {
                            ui.label(RichText::new(titel).strong());
                            ui.label(RichText::new(text).color(LEISE));
                            ui.add_space(4.0);
                        }
                    });
            }
        });
}

/// Kosten als farbige Liste: grün, was vorrätig ist, rot mit Fehlmenge, was fehlt.
/// Wann ein Planet `kosten` × `faktor` beisammen hat, aus Bestand, Zuwachs je Stunde und Lagergröße.
/// None heißt: alles schon da. Sonst ein Satzteil wie „Erz reicht in 11 h“ oder warum es nie reicht.
pub fn wartezeit(kosten: &Value, planet: &Value, faktor: i64) -> Option<String> {
    let mut minuten = 0i64;
    let mut engpass = String::new();
    for (gut, menge) in kosten.as_object()? {
        let (soll, ist) = (i(menge) * faktor, i(&planet["bestand"][gut]));
        if planet["bestand"].get(gut).is_none() || ist >= soll {
            continue;
        }
        let n = super::namen::gut(gut);
        let lager = &planet["lager"][gut];
        if lager.is_number() && i(lager) < soll {
            return Some(format!("das Lager fasst nur {} {n}, baue erst das Lager aus", zahl(i(lager))));
        }
        let rate = i(&planet["rate"][gut]);
        if rate <= 0 {
            return Some(format!("{n} wächst hier gerade nicht nach"));
        }
        let m = ((soll - ist) * 60 + rate - 1) / rate;
        if m > minuten {
            minuten = m;
            engpass = n;
        }
    }
    (minuten > 0).then(|| format!("{engpass} reicht {}", in_zeit(minuten)))
}

pub fn kosten(ui: &mut egui::Ui, kosten: &Value, bestand: &Value, faktor: i64) {
    let Some(m) = kosten.as_object() else {
        return;
    };
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 10.0;
        for (gut, menge) in m {
            let brauchen = i(menge) * faktor;
            if brauchen == 0 {
                continue;
            }
            let haben = i(&bestand[gut]);
            let (farbe, tip) = if haben >= brauchen {
                (GUT, format!("vorrätig: {}", zahl(haben)))
            } else {
                (SCHLECHT, format!("vorrätig: {}, es fehlen {}", zahl(haben), zahl(brauchen - haben)))
            };
            ui.label(RichText::new(format!("{} {}", super::namen::gut_zeichen(gut), zahl(brauchen))).color(farbe))
                .on_hover_text(format!("{}: {tip}", super::namen::gut(gut)));
        }
    });
}

/// Ob alle Kosten (mal `faktor`) aus dem Bestand bezahlbar sind.
pub fn bezahlbar(kosten: &Value, bestand: &Value, faktor: i64) -> bool {
    kosten.as_object().is_none_or(|m| m.iter().all(|(g, n)| i(&bestand[g]) >= i(n) * faktor))
}

/// Kleines Etikett mit Hintergrund, etwa „gesperrt“ oder „läuft“.
pub fn etikett(ui: &mut egui::Ui, text: &str, farbe: Color32) -> egui::Response {
    let r = egui::Frame::new()
        .fill(farbe.linear_multiply(0.18))
        .corner_radius(egui::CornerRadius::same(10))
        .inner_margin(egui::Margin::symmetric(8, 1))
        .show(ui, |ui| ui.label(RichText::new(text).small().color(farbe)))
        .response;
    ueber(ui, r)
}

/// Knopf, der nur bei Erfüllung aktiv ist; der Grund steht im Tooltip.
pub fn knopf(ui: &mut egui::Ui, text: &str, aktiv: bool, grund: &str) -> bool {
    let r = ui.add_enabled(aktiv, egui::Button::new(text));
    let r = if aktiv || grund.is_empty() { r } else { r.on_disabled_hover_text(grund) };
    r.clicked()
}

/// Hauptknopf in Gold für die wichtigste Handlung eines Bereichs.
pub fn hauptknopf(ui: &mut egui::Ui, text: &str, aktiv: bool) -> bool {
    ui.add_enabled(
        aktiv,
        egui::Button::new(RichText::new(text).color(Color32::from_rgb(20, 20, 20)).strong()).fill(GOLD),
    )
    .clicked()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wartezeit_aus_zuwachs_und_lager() {
        let planet = serde_json::json!({"bestand": {"erz": 100, "kristall": 500}, "rate": {"erz": 60, "kristall": 0}, "lager": {"erz": 1000, "kristall": 1000}});
        assert_eq!(wartezeit(&serde_json::json!({"erz": 50}), &planet, 1), None);
        assert_eq!(wartezeit(&serde_json::json!({"erz": 160}), &planet, 1).unwrap(), "Erz reicht in 1 h");
        assert_eq!(wartezeit(&serde_json::json!({"erz": 80}), &planet, 3).unwrap(), "Erz reicht in 2 h 20 min");
        assert!(wartezeit(&serde_json::json!({"erz": 5000}), &planet, 1).unwrap().contains("Lager fasst nur 1.000 Erz"));
        assert!(wartezeit(&serde_json::json!({"kristall": 600}), &planet, 1).unwrap().contains("wächst hier gerade nicht nach"));
    }

    #[test]
    fn deutsche_zahlen_und_zeiten() {
        assert_eq!(zahl(0), "0");
        assert_eq!(zahl(999), "999");
        assert_eq!(zahl(1000), "1.000");
        assert_eq!(zahl(1234567), "1.234.567");
        assert_eq!(zahl(-4500), "−4.500");
        assert_eq!(rate(15), "+15");
        assert_eq!(dauer_min(45), "45 min");
        assert_eq!(dauer_min(130), "2 h 10 min");
        assert_eq!(dauer_min(120), "2 h");
        assert_eq!(dauer_min(3 * 1440 + 240), "3 Tage 4 h");
        assert_eq!(dauer_min(1440), "1 Tag");
        assert_eq!(in_zeit(6 * 1440 + 14 * 60), "in 6 Tagen 14 h");
        assert_eq!(in_zeit(1440), "in 1 Tag");
        assert_eq!(in_zeit(45), "in 45 min");
        assert_eq!(spielzeit(3 * 86_400 + 3 * 3600 + 15 * 60), "Tag 4, 03:15");
        assert_eq!(komma(0.25, 2), "0,25");
    }
}
