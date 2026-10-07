//! Anzeigenamen mit Umlauten, Symbole und kurze Beschreibungen. Die Engine benutzt ASCII-Schlüssel
//! („konsumgueterwerk“), der Mensch liest „Konsumgüterwerk“.

/// Anzeigename für jeden Schlüssel der Engine; Unbekanntes wird lesbar gemacht.
pub fn name(schluessel: &str) -> String {
    let fest = match schluessel {
        // Güter
        "erz" => "Erz",
        "kristall" => "Kristall",
        "deuterium" => "Deuterium",
        "nahrung" => "Nahrung",
        "legierung" => "Legierung",
        "elektronik" => "Elektronik",
        "konsumgut" => "Konsumgüter",
        "xenokristall" => "Xenokristall",
        "antriebskern" => "Antriebskern",
        "habitatmodul" => "Habitatmodul",
        "credits" => "Credits",
        // Gebäude
        "erzmine" => "Erzmine",
        "kristallmine" => "Kristallmine",
        "deuteriumsynthesizer" => "Deuteriumsynthesizer",
        "farm" => "Farm",
        "solarkraftwerk" => "Solarkraftwerk",
        "fusionskraftwerk" => "Fusionskraftwerk",
        "giesserei" => "Gießerei",
        "elektronikwerk" => "Elektronikwerk",
        "konsumgueterwerk" => "Konsumgüterwerk",
        "xenoextraktor" => "Xenoextraktor",
        "lager" => "Lager",
        "bunker" => "Bunker",
        "wohnblock" => "Wohnblock",
        "akademie" => "Akademie",
        "markt" => "Markt",
        "verwaltungszentrum" => "Verwaltungszentrum",
        "labor" => "Labor",
        "bauhof" => "Bauhof",
        "nanofabrik" => "Nanofabrik",
        "raumhafen" => "Raumhafen",
        "werft" => "Werft",
        "orbitalwerft" => "Orbitalwerft",
        "sensorphalanx" => "Sensorphalanx",
        "geheimdienst" => "Geheimdienst",
        "kaserne" => "Kaserne",
        "raketensilo" => "Raketensilo",
        "orbitalring" => "Orbitalring",
        "forschungsarchiv" => "Forschungsarchiv",
        "versorgungsnetz" => "Versorgungsnetz",
        // Forschung
        "energietechnik" => "Energietechnik",
        "werkstoffkunde" => "Werkstoffkunde",
        "automatisierung" => "Automatisierung",
        "agrarwissenschaft" => "Agrarwissenschaft",
        "soziologie" => "Soziologie",
        "verbrennungsantrieb" => "Verbrennungsantrieb",
        "impulsantrieb" => "Impulsantrieb",
        "hyperraumantrieb" => "Hyperraumantrieb",
        "astrophysik" => "Astrophysik",
        "logistik" => "Logistik",
        "computertechnik" => "Computertechnik",
        "waffentechnik" => "Waffentechnik",
        "schildtechnik" => "Schildtechnik",
        "panzerung" => "Panzerung",
        "spionagetechnik" => "Spionagetechnik",
        "ueberwachungstechnik" => "Überwachungstechniken",
        "abschirmtechnik" => "Abschirmtechnologie",
        "xenomaterialkunde" => "Xenomaterialkunde",
        "terraforming" => "Terraforming",
        // Einheiten
        "spionagesonde" => "Spionagesonde",
        "kleiner_transporter" => "Kleiner Transporter",
        "grosser_transporter" => "Großer Transporter",
        "leichter_jaeger" => "Leichter Jäger",
        "bergbauschiff" => "Bergbauschiff",
        "kreuzer" => "Kreuzer",
        "recycler" => "Recycler",
        "kolonieschiff" => "Kolonieschiff",
        "truppentransporter" => "Truppentransporter",
        "schlachtschiff" => "Schlachtschiff",
        "bomber" => "Bomber",
        "zerstoerer" => "Zerstörer",
        "raketenwerfer" => "Raketenwerfer",
        "lasergeschuetz" => "Lasergeschütz",
        "ionengeschuetz" => "Ionengeschütz",
        "gausskanone" => "Gaußkanone",
        "plasmawerfer" => "Plasmawerfer",
        "planetenschild" => "Planetenschild",
        // Missionen
        "bombardieren" => "Bombardieren",
        "kampfkolonisieren" => "Kampfkolonisieren",
        "angriff" => "Angriff",
        "transport" => "Transport",
        "stationieren" => "Stationieren",
        "halten" => "Halten",
        "spionage" => "Spionage",
        "flotten_spionage" => "Flottenspionage",
        "system_erkunden" => "Sonnensystem erkunden",
        "saven" => "Saven / Leerfahrt",
        "kolonisieren" => "Kolonisieren",
        "recyceln" => "Recyceln",
        "abbau" => "Abbau",
        "blockade" => "Blockade",
        "invasion" => "Invasion",
        // Verträge
        "nichtangriffspakt" => "Nichtangriffspakt",
        "handelsabkommen" => "Handelsabkommen",
        "verteidigungsbuendnis" => "Verteidigungsbündnis",
        "tribut" => "Tribut",
        // Töpfe, Völker, Zonen, Rollen
        "wirtschaft" => "Wirtschaft",
        "militaer" => "Militär",
        "forschung" => "Forschung",
        "reserve" => "Reserve",
        "zivilisation" => "Zivilisation",
        "aurelianer" => "Aurelianer",
        "krath" => "Krath",
        "veyari" => "Veyari",
        "syntheten" => "Syntheten",
        "glut" => "Glutzone",
        "leben" => "Lebenszone",
        "frost" => "Frostzone",
        "stratege" => "Stratege",
        "verwalter" => "Verwalter",
        "feldherr" => "Feldherr",
        "diplomat" => "Diplomat",
        "abfang" => "Abfangrakete",
        "interplanetar" => "Interplanetarrakete",
        "kauf" => "Kauf",
        "verkauf" => "Verkauf",
        _ => "",
    };
    if !fest.is_empty() {
        return fest.to_string();
    }
    let mut s = schluessel.replace('_', " ");
    if let Some(c) = s.get_mut(0..1) {
        c.make_ascii_uppercase();
    }
    s
}

pub fn gut(schluessel: &str) -> String {
    name(schluessel)
}

/// Texte aus dem Kern für Menschen: Schlüsselwörter („werft“, „impulsantrieb“, „erz“) werden zu Namen, große Zahlen
/// bekommen Tausenderpunkte („32000“ → „32.000“). Satzzeichen am Wortrand bleiben stehen.
pub fn woerter(text: &str) -> String {
    use kern::typen::*;
    let bekannt = |w: &str| {
        Gut::aus_name(w).is_some() || Gebaeude::aus_name(w).is_some() || Forschung::aus_name(w).is_some()
            || Einheit::aus_name(w).is_some() || Mission::aus_name(w).is_some() || Vertragsart::aus_name(w).is_some()
    };
    let woerter: Vec<&str> = text.split(' ').collect();
    woerter
        .iter()
        .enumerate()
        .map(|(n, wort)| {
            // Nach „Flotte 12345“ steht eine Nummer, keine Menge: ohne Tausenderpunkt.
            let nummer = n > 0 && ["Flotte", "Flotten", "Vertrag", "Order", "Startwert"].contains(&woerter[n - 1]);
            let anfang = wort.find(|c: char| c.is_alphanumeric()).unwrap_or(wort.len());
            let ende = wort.rfind(|c: char| c.is_alphanumeric() || c == '_').map(|i| i + 1).unwrap_or(anfang);
            if anfang >= ende {
                return wort.to_string();
            }
            let (vor, kern_, nach) = (&wort[..anfang], &wort[anfang..ende], &wort[ende..]);
            if bekannt(kern_) {
                format!("{vor}{}{nach}", name(kern_))
            } else if !nummer && kern_.len() >= 4 && kern_.chars().all(|c| c.is_ascii_digit()) {
                format!("{vor}{}{nach}", super::stil::zahl(kern_.parse().unwrap_or(0)))
            } else {
                wort.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Symbol je Gut, damit Rohstoffe auf einen Blick unterscheidbar sind.
pub fn gut_zeichen(schluessel: &str) -> &'static str {
    match schluessel {
        "erz" => "⛏",
        "kristall" => "💎",
        "deuterium" => "💧",
        "nahrung" => "🌾",
        "legierung" => "🔩",
        "elektronik" => "💡",
        "konsumgut" => "📦",
        "xenokristall" => "✨",
        "antriebskern" => "🔥",
        "habitatmodul" => "🏠",
        "credits" => "💰",
        _ => "•",
    }
}

/// Was eine Mission tut und was sie verlangt, für die Auswahl im Flottenkommando.
pub fn mission_text(m: &str) -> &'static str {
    match m {
        "flotten_spionage" => "Eine physische Spionagesonde fliegt zu einer erkannten feindlichen Flotte. Abschirmung und eigene Geheimdienst- und Spionagetechnik bestimmen, welche Schiffstypen sichtbar werden. Der Bericht bleibt eine Aufnahme zum Sondenzeitpunkt.",
        "system_erkunden" => "Eine Sonde kartiert das Sonnensystem. Typ, Besitzer und Rohstoffe jedes Planeten bleiben bis zu dessen eigener Sondenmission unbekannt.",
        "saven" => "Leerfahrt mit Schiffen und Fracht: keine Entladung und kein Kampf am Ziel. Rückflug nach 0 bis 72 Stunden Wartezeit. Treibstoff für beide Strecken und ein Flottenplatz werden benötigt.",
        "bombardieren" => "Besiegt bewaffnete Verteidigung und senkt die Gebäudeintegrität auf höchstens 30 %. Gebäude behalten ihre Stufen, leisten aber weniger. Nur mit aktuellen Kolonisationsregeln.",
        "kampfkolonisieren" => "Benötigt ein Kolonieschiff und Orbitkontrolle. Übernahme erst nach ausgeschalteter Verteidigung, höchstens 30 % Gebäudeintegrität und zwei vollen Reaktionsfenstern (mindestens 30 Spielminuten). Ursprüngliche Heimatwelten sind geschützt.",
        "transport" => "Bringt Ladung zu einem Planeten und fliegt zurück. Bei einem fremden Reich zählt die Ladung als Geschenk.",
        "stationieren" => "Verlegt die Flotte dauerhaft zu einem eigenen Planeten, samt Ladung.",
        "kolonisieren" => "Ein Kolonieschiff gründet auf einem freien Platz eine Kolonie. In neuen Partien zuerst mit einer Sonde erkunden, eine bewaffnete Eskorte und die angezeigte Startfracht mitnehmen. Siedler kommen von der Heimat; Startfracht nach Ankunft selbst verbauen.",
        "spionage" => "Sonden erkunden einen Planeten. Je mehr Sonden und je besser die Spionagetechnik, desto mehr zeigt der Bericht. Das Ziel bemerkt den Versuch.",
        "angriff" => "Kampf gegen Schiffe und Verteidigung am Ziel. Gewinnt der Angreifer, plündert er einen Teil der ungeschützten Güter. Bricht Nichtangriffspakte und beendet den eigenen Anfängerschutz.",
        "halten" => "Die Flotte hilft einem Verbündeten, seinen Planeten zu verteidigen (1 bis 168 Stunden).",
        "recyceln" => "Recycler sammeln ein Trümmerfeld ein und bringen Erz und Kristall heim.",
        "abbau" => "Bergbauschiffe bauen in einem Asteroidengürtel (Position 0) 1 bis 48 Stunden Rohstoffe ab.",
        "blockade" => "Die Flotte hält den Orbit eines fremden Planeten: Transporte kehren um, Marktlieferungen warten. Ab Stufe IV.",
        "invasion" => "Truppentransporter belagern nach gewonnenem Orbitkampf eine fremde Kolonie, nie eine Heimatwelt. Fällt ihre Stabilität unter 15 und sind die Truppen stärker als die Garnison, gehört sie dir. Ab Stufe IV.",
        _ => "",
    }
}

pub fn vertrag_text(art: &str) -> &'static str {
    match art {
        "nichtangriffspakt" => "Beide versprechen, einander nicht anzugreifen. Ein Angriff ist ein Bruch und steht öffentlich im Register. Kündigung mit 48 Stunden Frist.",
        "handelsabkommen" => "Halbe Marktgebühr im Handel miteinander. Jederzeit kündbar.",
        "verteidigungsbuendnis" => "Flotten dürfen beim Partner die Mission Halten fliegen, Angriffswarnungen werden geteilt. Höchstens 3 je Reich, Kündigung mit 48 Stunden Frist.",
        "tribut" => "Du zahlst dem Partner eine Zeit lang täglich Credits oder ein Gut von Heimatwelt zu Heimatwelt. Wer vorzeitig einstellt oder nicht zahlen kann, bricht den Vertrag.",
        _ => "",
    }
}

pub fn topf_text(topf: &str) -> &'static str {
    match topf {
        "wirtschaft" => "bezahlt Gebäude und Marktkäufe",
        "militaer" => "bezahlt Schiffe, Verteidigung und Raketen",
        "forschung" => "bezahlt Forschung",
        "reserve" => "Notgroschen: das Militär darf bei einem Angriff darauf zugreifen",
        _ => "",
    }
}

pub fn zone_text(zone: &str) -> &'static str {
    match zone {
        "glut" => "nah an der Sonne: viel Solarstrom, wenig Deuterium und Nahrung",
        "leben" => "gemäßigt: viel Nahrung, ausgeglichene Erträge",
        "frost" => "fern der Sonne: viel Deuterium, wenig Solarstrom und Nahrung",
        _ => "",
    }
}

pub fn volk_text(volk: &str) -> &'static str {
    match volk {
        "aurelianer" => "Händlervolk: größere Laderäume, halbe Marktgebühr, mehr Steuereinnahmen, etwas schwächere Waffen.",
        "krath" => "Kriegervolk: stärkere Waffen, schnellere Werften und mehr Beute, aber langsamere Forschung.",
        "veyari" => "Naturvolk: wachsen schneller, mehr Nahrung, günstigere Kolonieschiffe, schwächere Panzerung.",
        "syntheten" => "Maschinenvolk: brauchen keine Nahrung, aber Strom für ihre Einwohner; erzeugen mehr Strom und forschen schneller, wachsen aber langsamer.",
        _ => "",
    }
}

/// Gruppen für die Gebäudeliste, in Spielreihenfolge.
pub const GEBAEUDE_GRUPPEN: &[(&str, &str, &[&str])] = &[
    ("⛏", "Rohstoffe", &["erzmine", "kristallmine", "deuteriumsynthesizer", "farm", "xenoextraktor"]),
    ("⚡", "Energie", &["solarkraftwerk", "fusionskraftwerk"]),
    ("🏭", "Verarbeitung", &["giesserei", "elektronikwerk", "konsumgueterwerk"]),
    ("👥", "Bevölkerung und Lager", &["wohnblock", "akademie", "lager", "bunker", "verwaltungszentrum"]),
    ("🔬", "Forschung und Bauwesen", &["labor", "bauhof", "nanofabrik"]),
    ("🚀", "Raumfahrt und Handel", &["raumhafen", "werft", "orbitalwerft", "markt", "sensorphalanx"]),
    ("⚔", "Militär und Aufklärung", &["kaserne", "raketensilo", "geheimdienst"]),
    ("🌟", "Großprojekte der Stufe V", &["orbitalring", "forschungsarchiv", "versorgungsnetz"]),
];

pub const FORSCHUNG_GRUPPEN: &[(&str, &str, &[&str])] = &[
    ("🏭", "Wirtschaft", &["energietechnik", "werkstoffkunde", "automatisierung", "agrarwissenschaft", "soziologie", "xenomaterialkunde", "terraforming"]),
    ("🚀", "Antriebe und Raumfahrt", &["verbrennungsantrieb", "impulsantrieb", "hyperraumantrieb", "astrophysik", "logistik", "computertechnik"]),
    ("⚔", "Militär und Aufklärung", &["waffentechnik", "schildtechnik", "panzerung", "spionagetechnik", "ueberwachungstechnik", "abschirmtechnik"]),
];

#[cfg(test)]
mod tests {
    use super::*;
    use kern::typen::*;

    /// Jeder Schlüssel der Engine hat einen echten Anzeigenamen, keinen bloß umformatierten.
    #[test]
    fn alles_hat_einen_anzeigenamen() {
        let mut schluessel: Vec<&str> = Vec::new();
        schluessel.extend(Gut::ALLE.iter().map(|x| x.name()));
        schluessel.extend(Gebaeude::ALLE.iter().map(|x| x.name()));
        schluessel.extend(Forschung::ALLE.iter().map(|x| x.name()));
        schluessel.extend(Einheit::ALLE.iter().map(|x| x.name()));
        schluessel.extend(Mission::ALLE.iter().map(|x| x.name()));
        schluessel.extend(Vertragsart::ALLE.iter().map(|x| x.name()));
        schluessel.extend(Topf::ALLE.iter().map(|x| x.name()));
        schluessel.extend(Volk::ALLE.iter().map(|x| x.name()));
        schluessel.extend(Zone::ALLE.iter().map(|x| x.name()));
        for s in schluessel {
            let n = name(s);
            assert!(n != s && !n.contains('_'), "{s} hat keinen Anzeigenamen");
        }
        for m in Mission::ALLE {
            assert!(!mission_text(m.name()).is_empty(), "{m}");
        }
        for v in Vertragsart::ALLE {
            assert!(!vertrag_text(v.name()).is_empty(), "{v}");
        }
        for g in Gut::ALLE {
            assert_ne!(gut_zeichen(g.name()), "•", "{g}");
        }
    }

    #[test]
    fn kerntexte_werden_lesbar() {
        assert_eq!(woerter("32000 Einwohner (jetzt 12286)"), "32.000 Einwohner (jetzt 12.286)");
        assert_eq!(woerter("werft Stufe 4 (jetzt 4)"), "Werft Stufe 4 (jetzt 4)");
        assert_eq!(woerter("+483 erz je Stunde"), "+483 Erz je Stunde");
        assert_eq!(woerter("grosser_transporter, kleiner_transporter."), "Großer Transporter, Kleiner Transporter.");
        assert_eq!(woerter("Stabilität der Heimatwelt ab 60"), "Stabilität der Heimatwelt ab 60");
        assert_eq!(woerter("Flotte 19363 (Transport) ist zurück mit 12000 Erz"), "Flotte 19363 (Transport) ist zurück mit 12.000 Erz");
    }

    /// Jedes Gebäude und jede Forschung steht in genau einer Gruppe der Listen.
    #[test]
    fn gruppen_decken_alles_ab() {
        let geb: Vec<&str> = GEBAEUDE_GRUPPEN.iter().flat_map(|g| g.2.iter().copied()).collect();
        for g in Gebaeude::ALLE {
            assert_eq!(geb.iter().filter(|x| **x == g.name()).count(), 1, "{g}");
        }
        let fo: Vec<&str> = FORSCHUNG_GRUPPEN.iter().flat_map(|g| g.2.iter().copied()).collect();
        for f in Forschung::ALLE {
            assert_eq!(fo.iter().filter(|x| **x == f.name()).count(), 1, "{f}");
        }
    }
}
