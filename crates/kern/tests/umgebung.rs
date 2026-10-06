use kern::{umgebung::*, *};
use serde_json::json;
use std::collections::BTreeSet;
fn env(max: u64) -> Umgebung {
    Umgebung::neu(
        Regelwerk::laden(include_str!("../../../regeln/regelwerk.ron")).unwrap(),
        42,
        3,
        BTreeSet::from([0, 1]),
        max,
    )
    .unwrap()
}
#[test]
fn reset_and_trajectory_are_reproducible() {
    let mut a = env(20);
    let mut b = env(20);
    let original = a.zustand_hash();
    for _ in 0..5 {
        let ar = a.step(a.no_op()).unwrap();
        let br = b.step(b.no_op()).unwrap();
        assert_eq!(ar.rewards, br.rewards);
        assert_eq!(ar.beobachtungen, br.beobachtungen);
        assert_eq!(a.zustand_hash(), b.zustand_hash());
    }
    a.reset(42).unwrap();
    assert_eq!(a.zustand_hash(), original);
    a.reset(43).unwrap();
    assert_ne!(a.zustand_hash(), original);
}
#[test]
fn incomplete_or_malformed_joint_call_is_atomic() {
    let mut e = env(20);
    let before = e.zustand_hash();
    let mut a = e.no_op();
    a.remove(&1);
    assert!(e.step(a).is_err());
    assert_eq!(e.zustand_hash(), before);
    let mut a = e.no_op();
    a.get_mut(&0).unwrap().push(RollenAktionen {
        rolle: Rolle::Stratege,
        aktionen: vec![json!({"typ":"steuern","satz":0.12})],
    });
    a.get_mut(&1).unwrap().push(RollenAktionen {
        rolle: Rolle::Alle,
        aktionen: vec![],
    });
    assert!(e.step(a).is_err());
    assert_eq!(e.zustand_hash(), before);
    let mut a = e.no_op();
    a.get_mut(&1).unwrap().push(RollenAktionen {
        rolle: Rolle::Stratege,
        aktionen: vec![json!("bad")],
    });
    assert!(e.step(a).is_err());
    assert_eq!(e.zustand_hash(), before);
}
#[test]
fn truncation_needs_reset_and_rewards_are_exact_point_deltas() {
    let mut e = env(1);
    let before = e.beobachten();
    let step = e.step(e.no_op()).unwrap();
    assert!(!step.terminated);
    assert!(step.truncated);
    for sid in [0, 1] {
        assert_eq!(
            step.rewards[&sid],
            step.beobachtungen[&sid]["punkte"]["gesamt"]
                .as_i64()
                .unwrap()
                - before.beobachtungen[&sid]["punkte"]["gesamt"]
                    .as_i64()
                    .unwrap()
        );
    }
    assert!(e.step(e.no_op()).is_err());
    e.reset(42).unwrap();
    assert!(e.step(e.no_op()).is_ok());
}
#[test]
fn terminal_epoch_is_not_time_limit_truncation() {
    let mut rules = Regelwerk::laden(include_str!("../../../regeln/regelwerk.ron")).unwrap();
    rules.welt.epoche_tage = 1;
    let mut e = Umgebung::neu(rules, 42, 2, BTreeSet::from([0, 1]), 1000).unwrap();
    loop {
        let r = e.step(e.no_op()).unwrap();
        if r.terminated {
            assert!(!r.truncated);
            assert_eq!(r.info.sekunden, TAG);
            break;
        }
    }
}
#[test]
fn observations_contain_only_own_planet_details() {
    let e = env(20);
    let observation = e.beobachten();
    assert_eq!(observation.beobachtungen.len(), 2);
    for (&sid, view) in &observation.beobachtungen {
        let own = view["planeten"].as_array().unwrap();
        assert_eq!(own.len(), 1);
        let other = &observation.beobachtungen[&(1 - sid)]["planeten"][0]["koord"];
        assert_ne!(&own[0]["koord"], other);
        assert!(view.get("welt").is_none());
        assert!(view.get("systeme").is_none());
        assert_eq!(view["erkundet"].as_array().unwrap().len(), 0);
    }
}
#[test]
fn invalid_game_actions_report_without_bypassing_role_rights() {
    let mut e = env(20);
    let mut a = e.no_op();
    a.get_mut(&0).unwrap().push(RollenAktionen {
        rolle: Rolle::Diplomat,
        aktionen: vec![json!({"typ":"bauen","planet":"1:1:1","gebaeude":"erzmine"})],
    });
    let r = e.step(a).unwrap();
    assert_eq!(r.info.aktionsresultate[&0].len(), 1);
    assert!(!r.info.aktionsresultate[&0][0].angenommen);
    assert_eq!(r.info.sekunden, 15 * MINUTE);
}
