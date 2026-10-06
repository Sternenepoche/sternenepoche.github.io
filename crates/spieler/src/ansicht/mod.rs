//! Die Bildschirme der Spieler-Oberfläche. Jeder Bildschirm ist ein `impl Player`-Block in eigener Datei;
//! gemeinsame Bausteine stehen in `stil`, Namen in `namen`, Erklärungen und Hinweise in `hilfe`.

pub mod befehl;
pub mod bilder;
pub mod berichte;
pub mod diplomatie;
pub mod flotten;
pub mod forschung;
pub mod freischaltungen;
pub mod galaxie;
pub mod gebaeude;
pub mod hilfe;
pub mod kolonie;
pub mod markt;
pub mod namen;
pub mod regierung;
pub mod stil;
pub mod uebersicht;
pub mod werft;

/// Alle Bildschirme in der Reihenfolge der Navigation.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Bildschirm {
    Uebersicht,
    Kolonie,
    Gebaeude,
    Forschung,
    Werft,
    Flotten,
    Galaxie,
    Markt,
    Diplomatie,
    Regierung,
    Berichte,
    Rangliste,
    Anleitung,
    LiveKi,
    Befehle,
    Galerie,
}

impl Bildschirm {
    pub const ALLE: [Bildschirm; 16] = [
        Bildschirm::Uebersicht,
        Bildschirm::Kolonie,
        Bildschirm::Gebaeude,
        Bildschirm::Forschung,
        Bildschirm::Werft,
        Bildschirm::Flotten,
        Bildschirm::Galaxie,
        Bildschirm::Markt,
        Bildschirm::Diplomatie,
        Bildschirm::Regierung,
        Bildschirm::Berichte,
        Bildschirm::Rangliste,
        Bildschirm::Anleitung,
        Bildschirm::LiveKi,
        Bildschirm::Befehle,
        Bildschirm::Galerie,
    ];

    /// Name in der Navigation und für `--screen`.
    pub fn name(self) -> &'static str {
        match self {
            Bildschirm::Uebersicht => "Übersicht",
            Bildschirm::Kolonie => "Kolonie",
            Bildschirm::Gebaeude => "Gebäude",
            Bildschirm::Forschung => "Forschung",
            Bildschirm::Werft => "Werft",
            Bildschirm::Flotten => "Flotten",
            Bildschirm::Galaxie => "Galaxie",
            Bildschirm::Markt => "Markt",
            Bildschirm::Diplomatie => "Diplomatie",
            Bildschirm::Regierung => "Zivilisation",
            Bildschirm::Berichte => "Berichte",
            Bildschirm::Rangliste => "Rangliste",
            Bildschirm::Anleitung => "Spielanleitung",
            Bildschirm::LiveKi => "Live-KI",
            Bildschirm::Befehle => "Befehlszentrale",
            Bildschirm::Galerie => "Grafikbibliothek",
        }
    }

    pub fn zeichen(self) -> &'static str {
        match self {
            Bildschirm::Uebersicht => "🏰",
            Bildschirm::Kolonie => "🌍",
            Bildschirm::Gebaeude => "🏭",
            Bildschirm::Forschung => "🔬",
            Bildschirm::Werft => "🚀",
            Bildschirm::Flotten => "✈",
            Bildschirm::Galaxie => "🌌",
            Bildschirm::Markt => "💰",
            Bildschirm::Diplomatie => "🕊",
            Bildschirm::Regierung => "👑",
            Bildschirm::Berichte => "📜",
            Bildschirm::Rangliste => "🏆",
            Bildschirm::Anleitung => "📖",
            Bildschirm::LiveKi => "💻",
            Bildschirm::Befehle => "⌨",
            Bildschirm::Galerie => "🖼",
        }
    }

    /// Gruppe in der Navigation.
    pub fn gruppe(self) -> &'static str {
        match self {
            Bildschirm::Uebersicht | Bildschirm::Kolonie | Bildschirm::Gebaeude | Bildschirm::Forschung | Bildschirm::Werft => "Reich",
            Bildschirm::Flotten | Bildschirm::Galaxie | Bildschirm::Markt | Bildschirm::Diplomatie => "Galaxie",
            Bildschirm::Regierung | Bildschirm::Berichte | Bildschirm::Rangliste => "Regierung",
            Bildschirm::Anleitung | Bildschirm::LiveKi | Bildschirm::Befehle | Bildschirm::Galerie => "Hilfe und Werkzeuge",
        }
    }

    pub fn aus_name(name: &str) -> Option<Bildschirm> {
        let n = name.to_lowercase();
        Bildschirm::ALLE.into_iter().find(|b| {
            b.name().to_lowercase() == n
                || format!("{b:?}").to_lowercase() == n
                || (n == "uebersicht" && *b == Bildschirm::Uebersicht)
                || (n == "gebaeude" && *b == Bildschirm::Gebaeude)
        })
    }
}
