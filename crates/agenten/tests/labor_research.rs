use std::{collections::BTreeMap,fs,path::PathBuf,time::{SystemTime,UNIX_EPOCH}};
use sternenepoche_agenten::labor::{config::Config,research};

fn directory()->PathBuf {PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../laeufe/labor-tests").join(format!("research-{}-{}",std::process::id(),SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()))}
#[test]
fn crossed_mock_controls_rotate_all_seats_and_never_become_wins() {
    let mut c=Config::v4(2);c.models.insert("second".into(),c.models.values().next().unwrap().clone());
    let first=c.players[0].clone();let mut split=first.clone();split.roles[0].purpose="A different internal organization".into();
    let variants=BTreeMap::from([("baseline".into(),first),("organized".into(),split)]);let aliases=c.models.keys().cloned().collect::<Vec<_>>();let root=directory();
    let report=research::factorial(&c,&variants,&aliases,&root,&[42],1,false).unwrap();
    assert_eq!(report["participants"],4);assert_eq!(report["runs"],4);assert_eq!(report["claims"]["wins"],false);assert_eq!(report["source_trace"].as_array().unwrap().len(),4);
    for setup in report["setups"].as_array().unwrap() {
        assert_eq!(setup["inference"],"mock_control");assert_eq!(setup["summary"]["independent_seed_families"],1);assert!(setup["summary"]["standard_error"].is_null());
        assert_eq!(setup["seed_families"][0]["seat_observations"],4);assert_eq!(setup["finished_real_valid_data"],false);assert!(setup["win_claim"].is_null());
        assert!(setup["audit_metrics"]["event_counts"]["call.completed"].as_u64().unwrap()>0);
    }
    assert_eq!(research::factorial(&c,&variants,&aliases,&root,&[42],1,false).unwrap(),report);
    // Every returned source is verified again; corrupted sources cannot be reported.
    let path=root.join(report["source_trace"][0]["path"].as_str().unwrap()).join("checkpoints/00000001.bin");let mut bytes=fs::read(&path).unwrap();bytes[0]^=1;fs::write(path,bytes).unwrap();
    assert!(research::factorial(&c,&variants,&aliases,&root,&[42],1,false).is_err());
}
#[test]
fn factorial_rejects_rights_confounds_unknown_models_and_duplicate_seeds_before_running() {
    let c=Config::v4(2);let h=c.players[0].clone();let mut other=h.clone();other.roles[0].capabilities.clear();
    let aliases=c.models.keys().cloned().collect::<Vec<_>>();let root=directory();
    assert!(research::factorial(&c,&BTreeMap::from([("a".into(),h.clone()),("b".into(),other)]),&aliases,&root,&[1],1,false).unwrap_err().contains("Capability"));assert!(!root.exists());
    let variants=BTreeMap::from([("a".into(),h.clone()),("b".into(),h)]);
    assert!(research::factorial(&c,&variants,&["missing".into()],&root,&[1],1,false).is_err());
    assert!(research::factorial(&c,&variants,&aliases,&root,&[1,1],1,false).is_err());assert!(!root.exists());
}
