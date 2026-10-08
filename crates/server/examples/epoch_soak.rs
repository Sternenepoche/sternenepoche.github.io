//! Complete online epochs through the production tick, checkpoint and reset paths.
//! cargo run --release -p sternenepoche-server --example epoch_soak -- NEW_ROOT [DAYS=365] [EPOCHS=3] [SEED_OFFSET=0]
use kern::typen::*;
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
    time::Instant,
};
use sternenepoche_server::{password_hash, Game, BOT_COUNT, SEATS};

fn audit(g: &Game, frozen: &[(Vec<i64>, i64, i64)], dead: &mut BTreeMap<SpielerId, u32>) {
    let w = &g.world;
    w.ausscheiden.validieren(w).unwrap();
    assert!(g.storage_error.is_none());
    assert_eq!(w.belegung.len(), w.planeten.len());
    for (i, p) in w.planeten.iter().enumerate() {
        assert_eq!(p.id as usize, i);
        assert_eq!(w.belegung.get(&p.koord), Some(&p.id));
        assert!(w.spieler[p.besitzer as usize].planeten.contains(&p.id));
        assert!(
            w.bestand_jetzt(i).iter().all(|n| *n >= 0),
            "negative stock {}",
            p.koord
        );
        assert!(p.bevoelkerung >= 0 && p.einheiten.iter().all(|n| *n >= 0));
        assert!((0..=100_000).contains(&p.stabilitaet));
        if !w.spieler_aktiv(p.besitzer) {
            assert!(p.rate.iter().all(|n| *n == 0));
        }
    }
    for s in &w.spieler {
        let unique = s.planeten.iter().collect::<BTreeSet<_>>();
        assert_eq!(unique.len(), s.planeten.len());
        for pid in &s.planeten {
            assert_eq!(w.planeten[*pid as usize].besitzer, s.id);
        }
        let home = &w.planeten[s.heimat as usize];
        assert!(home.heimat);
        assert_eq!(home.besitzer, s.id, "original home changed owner");
        if w.ist_besiegt(s.id) {
            if let Some(actions) = dead.insert(s.id, s.statistik.aktionen) {
                assert_eq!(actions, s.statistik.aktionen, "defeated bot acted again");
            }
            assert!(s.forschung_aktiv.is_none());
        } else {
            assert!(!dead.contains_key(&s.id), "defeated empire revived");
        }
    }
    for (sid, (stock, population, credits)) in (BOT_COUNT..BOT_COUNT + SEATS).zip(frozen) {
        let s = &w.spieler[sid as usize];
        let p = &w.planeten[s.heimat as usize];
        assert!(!w.spieler_aktiv(sid));
        assert_eq!(&p.bestand.to_vec(), stock);
        assert_eq!(p.bevoelkerung, *population);
        assert_eq!(s.credits, *credits);
    }
    for (id, f) in &w.flotten {
        assert_eq!(id, &f.id);
        assert!(w.spieler_aktiv(f.besitzer), "inactive empire has fleet");
        assert!(f.schiffe.iter().all(|n| *n >= 0) && f.schiffe.iter().any(|n| *n > 0));
        assert!(f.ladung.iter().all(|n| *n >= 0) && f.siedler >= 0);
        assert!((f.start as usize) < w.planeten.len());
    }
}

fn summary(g: &Game) -> Value {
    let w = &g.world;
    json!({"seed":w.startwert,"days":w.zeit/TAG,"finished":w.beendet(),
        "actions":g.runtime.bot_actions,"rejected":g.runtime.bot_rejected,
        "active_bots":(0..BOT_COUNT).filter(|sid|w.spieler_aktiv(*sid)).count(),
        "stages":(1..=5).map(|n|w.spieler[..BOT_COUNT as usize].iter().filter(|s|s.stufe==n && w.spieler_aktiv(s.id)).count()).collect::<Vec<_>>(),
        "colonies":w.planeten.len()-(BOT_COUNT+SEATS) as usize,
        "combats":w.kampf_nr,"spy_resolutions":w.spionage_nr,"fleets":w.naechste_flotte,
        "defeated":w.ausgeschiedene(),"hash":w.hash(),"lobby":g.lobby(),
        "bots":w.spieler[..BOT_COUNT as usize].iter().map(|s|json!({"id":s.id,"name":s.name,"volk":s.volk,"strategy":g.runtime.bots[&s.id].typ.name(),"stage":s.stufe,"planets":s.planeten.len(),"status":w.reich_status(s.id),"statistics":s.statistik,"points":s.punkte,"credits":s.credits})).collect::<Vec<_>>()})
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let root = Path::new(
        args.get(1)
            .expect("Fresh, isolated output directory required"),
    );
    assert!(!root.exists(), "Refusing to touch an existing world");
    let days = args
        .get(2)
        .map(|x| x.parse::<i64>().unwrap())
        .unwrap_or(365);
    let epochs = args
        .get(3)
        .map(|x| x.parse::<usize>().unwrap())
        .unwrap_or(3);
    assert!((1..=365).contains(&days) && (1..=10).contains(&epochs));
    let seeds = [83, 42, 20261008, 517, 911, 17, 101, 239, 777, 2026];
    let offset = args
        .get(4)
        .map(|x| x.parse::<usize>().unwrap())
        .unwrap_or(0);
    assert!(offset < seeds.len());
    let started = Instant::now();
    let mut g = Game::open(&root.join("data"), seeds[offset]).unwrap();
    let account = g
        .register(
            &json!({"name":"EpochAuditSpectator"}),
            password_hash("isolated-epoch-test-password").unwrap(),
        )
        .unwrap();
    let old_token = account["token"].as_str().unwrap().to_owned();
    let mut results = Vec::new();
    for epoch in 0..epochs {
        let epoch_started = Instant::now();
        let frozen = (BOT_COUNT..BOT_COUNT + SEATS)
            .map(|sid| {
                let s = &g.world.spieler[sid as usize];
                let p = &g.world.planeten[s.heimat as usize];
                (p.bestand.to_vec(), p.bevoelkerung, s.credits)
            })
            .collect::<Vec<_>>();
        let mut dead = BTreeMap::new();
        let mut rejected = BTreeSet::new();
        let mut rejection_samples = Vec::new();
        let mut checkpoints = 0;
        for hour in 1..=days * 24 {
            g.advance(3600)
                .unwrap_or_else(|e| panic!("epoch {epoch}, hour {hour}: {e:?}"));
            assert_eq!(g.world.zeit, hour * STUNDE);
            let status = g.admin_status();
            assert!(status["invariant_errors"].as_array().unwrap().is_empty());
            for bot in status["monitor"]["bots"].as_array().unwrap() {
                for entry in bot["verlauf"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|v| v["ok"] == false)
                {
                    let key = format!("{}:{}:{}", bot["spieler"], entry["zeit"], entry["aktion"]);
                    if rejected.insert(key) {
                        rejection_samples.push(json!({"bot":bot["spieler"],"entry":entry}));
                    }
                }
            }
            if hour % 24 == 0 {
                audit(&g, &frozen, &mut dead);
            }
            if hour % (90 * 24) == 0 || hour == days * 24 {
                let hash = g.world.hash();
                let runtime = serde_json::to_value(&g.runtime).unwrap();
                drop(g);
                g = Game::open(&root.join("data"), 999).unwrap();
                assert_eq!(g.world.hash(), hash, "checkpoint world changed");
                assert_eq!(
                    serde_json::to_value(&g.runtime).unwrap(),
                    runtime,
                    "checkpoint runtime changed"
                );
                audit(&g, &frozen, &mut dead);
                checkpoints += 1;
            }
            if hour % (30 * 24) == 0 || hour == days * 24 {
                let mut progress = summary(&g);
                progress.as_object_mut().unwrap().remove("bots");
                progress.as_object_mut().unwrap().remove("lobby");
                progress["epoch"] = json!(epoch + 1);
                progress["elapsed_s"] = json!(epoch_started.elapsed().as_secs_f64());
                println!("{}", progress);
                fs::write(
                    root.join(format!("epoch-{}-day-{}.json", epoch + 1, hour / 24)),
                    serde_json::to_vec_pretty(&g.admin_status()).unwrap(),
                )
                .unwrap();
            }
        }
        let mut result = summary(&g);
        result["checkpoint_restarts"] = json!(checkpoints);
        result["daily_audits"] = json!(days);
        result["rejection_samples"] = json!(rejection_samples);
        result["elapsed_s"] = json!(epoch_started.elapsed().as_secs_f64());
        fs::write(
            root.join(format!("epoch-{}-summary.json", epoch + 1)),
            serde_json::to_vec_pretty(&result).unwrap(),
        )
        .unwrap();
        results.push(result);
        if days == 365 {
            let before = g.world.hash();
            g.advance(3600).unwrap();
            assert_eq!(g.world.hash(), before, "world moved after epoch end");
        }
        let old_id = g.runtime.world_id.clone();
        g.admin(&json!({"world_id":old_id,"action":"reset","confirm":format!("RESET {old_id}"),"seed":seeds[(offset+epoch+1)%seeds.len()]})).unwrap();
        assert_ne!(g.runtime.world_id, old_id);
        assert_eq!(g.world.zeit, 0);
        assert_eq!(g.runtime.bot_actions, 0);
        assert_eq!(g.lobby()["freie_plaetze"], SEATS);
        assert_eq!(g.runtime.admission_limit, 3);
        assert!(g.world.ausscheiden.besiegt.is_empty());
        assert!(g
            .account_by_name("EpochAuditSpectator")
            .unwrap()
            .sid
            .is_none());
        assert!(g.authenticate(&old_token).is_err());
        assert_eq!(
            g.admin(&json!({"world_id":old_id,"action":"settings","paused":true}))
                .unwrap_err()
                .0,
            409
        );
    }
    let report = json!({"epochs":results,"total_elapsed_s":started.elapsed().as_secs_f64(),"reset_checks":epochs,"invariant_errors":[]});
    fs::write(
        root.join("summary.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    println!(
        "{}",
        json!({"completed_epochs":epochs,"days_each":days,"elapsed_s":started.elapsed().as_secs_f64(),"report":root.join("summary.json")})
    );
}
