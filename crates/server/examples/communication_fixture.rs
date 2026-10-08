//! Isolated UI fixture. Never accepts an existing directory or touches a running world.
use kern::{Gebaeude, Gut, Rolle, M};
use serde_json::json;
use sternenepoche_server::{password_hash, Game};
fn main() {
    let root = std::path::PathBuf::from(std::env::args().nth(1).expect("new fixture directory"));
    assert!(!root.exists(), "Fixture must be new");
    let mut g = Game::open(&root, 812).unwrap();
    g.runtime.bots_enabled = false;
    let mut accounts = vec![];
    for name in ["Aster", "Mira", "Vega"] {
        let login = g
            .register(
                &json!({"name":name}),
                password_hash("local-fixture-only-2026").unwrap(),
            )
            .unwrap();
        let a = g.authenticate(login["token"].as_str().unwrap()).unwrap();
        g.claim(&a, &json!({"mode":"mensch","volk":"aurelianer"}))
            .unwrap();
        accounts.push(login);
    }
    for sid in 30..33u16 {
        let pid = g.world.spieler[sid as usize].heimat as usize;
        g.world.planeten[pid].gebaeude[Gebaeude::Markt.idx()] = 1;
        g.world.planeten[pid].bestand[Gut::Erz.idx()] = 20000 * M;
    }
    let actions = [
        (30, json!({"typ":"allianz_gruenden","name":"Sternenbund"})),
        (30, json!({"typ":"allianz_einladen","spieler":"Mira"})),
        (
            31,
            json!({"typ":"allianz_beitreten","allianz":"Sternenbund"}),
        ),
        (32, json!({"typ":"allianz_gruenden","name":"Orion-Kontor"})),
        (
            31,
            json!({"typ":"brief_senden","kanal":"privat","an":"Aster","betreff":"Versorgung der Außenposten","text":"Hallo Aster, ich bin Mira. Ich kann Erz für unseren nächsten Ausbau liefern. Welche Welt braucht zuerst Unterstützung?","antwort_auf":null}),
        ),
        (
            30,
            json!({"typ":"brief_senden","kanal":"privat","an":"Mira","betreff":"Re: Versorgung der Außenposten","text":"Danke Mira. Ich trage den Bedarf im Allianzbereich ein und prüfe die Lieferzeit.","antwort_auf":1}),
        ),
        (
            31,
            json!({"typ":"brief_senden","kanal":"allianz","an":"","betreff":"Allianzchat","text":"Willkommen im Sternenbund. Bitte meldet freie Waren und kritische Versorgung hier.","antwort_auf":null}),
        ),
        (
            30,
            json!({"typ":"brief_senden","kanal":"allianz","an":"","betreff":"Allianzchat","text":"Der interne Markt ist offen. Hilferufe stehen getrennt unterhalb der Handelsübersicht.","antwort_auf":null}),
        ),
        (
            32,
            json!({"typ":"brief_senden","kanal":"diplomatie","an":"Sternenbund","betreff":"Sichere Handelswege","text":"Das Orion-Kontor schlägt ein Handelsabkommen und einen Nichtangriffspakt zwischen unseren Allianzen vor.","antwort_auf":null}),
        ),
        (
            32,
            json!({"typ":"allianz_anfrage","allianz":"Sternenbund","art":"nichtangriffspakt","text":"Wir möchten langfristige Sicherheit für beide Allianzen vereinbaren."}),
        ),
        (
            30,
            json!({"typ":"diplomatie_entscheiden","anfrage":1,"annehmen":true}),
        ),
    ];
    for (sid, a) in actions {
        let result = g.world.handeln(sid, Rolle::Alle, &a);
        assert!(result.0, "{result:?}");
    }
    let k = g.world.planeten[g.world.spieler[31].heimat as usize].koord;
    g.world
        .intern_anbieten(31, k, Gut::Erz, 500 * M, M)
        .unwrap();
    let home = g.world.planeten[g.world.spieler[30].heimat as usize].koord;
    g.world
        .allianz_hilfe(
            30,
            home,
            "Vorbereitung: Bitte verfügbare Transporter für Versorgung melden.",
        )
        .unwrap();
    g.world.allianz_hilfe_status(31, 1, false).unwrap();
    g.advance(0).unwrap();
    std::fs::write(
        root.join("fixture.json"),
        serde_json::to_vec(&json!({"accounts":accounts,"world_id":g.runtime.world_id,"home":home}))
            .unwrap(),
    )
    .unwrap();
}
