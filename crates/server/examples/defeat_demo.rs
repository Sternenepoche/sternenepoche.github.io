//! Creates a NEW isolated proof world. Never accepts an existing directory.
use kern::*;
use serde_json::json;
use sternenepoche_server::{password_hash, Game};
fn main() {
    let path = std::path::PathBuf::from(
        std::env::args()
            .nth(1)
            .expect("Neues QA-Datenverzeichnis angeben"),
    );
    assert!(
        !path.exists(),
        "Eine vorhandene Welt darf nicht verändert werden"
    );
    let mut g = Game::open(&path, 83).unwrap();
    for (name, volk) in [("UiNotstand", "veyari"), ("UiRettung", "aurelianer")] {
        let v = g
            .register(
                &json!({"name":name}),
                password_hash("nur-ein-test-passwort").unwrap(),
            )
            .unwrap();
        let a = g.authenticate(v["token"].as_str().unwrap()).unwrap();
        let claimed = g
            .claim(&a, &json!({"mode":"gemischt","volk":volk}))
            .unwrap();
        let sid = claimed["spieler"].as_u64().unwrap() as usize;
        let pid = g.world.spieler[sid].heimat as usize;
        g.planets_for_demo(pid, name == "UiNotstand");
    }
    g.world.ausscheiden_pruefen();
    let wid = g.runtime.world_id.clone();
    g.admin(&json!({"world_id":wid,"action":"settings","paused":true,"tempo":60,"versorgung_stunden":1,"stillstand_stunden":1,"maintenance":"Separate UI-Prüfwelt: absichtlich vorbereiteter Notstand. Keine öffentliche Spielwelt."})).unwrap();
    println!("Separate QA-Welt gespeichert. Konten: UiNotstand und UiRettung; Testpasswort: nur-ein-test-passwort.");
}
trait DemoSetup {
    fn planets_for_demo(&mut self, pid: usize, stranded: bool);
}
impl DemoSetup for Game {
    fn planets_for_demo(&mut self, pid: usize, stranded: bool) {
        let sid = self.world.planeten[pid].besitzer;
        self.world.planeten[pid].gebaeude[Gebaeude::Farm.idx()] = 0;
        self.world.planeten[pid].bestand[Gut::Nahrung.idx()] = 0;
        if stranded {
            self.world.planeten[pid].gebaeude.fill(0);
            self.world.planeten[pid].bestand.fill(0);
            self.world.spieler[sid as usize].credits = 0;
        }
        self.world.raten_neu(pid);
    }
}
