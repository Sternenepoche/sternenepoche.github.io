use serde_json::json;
use sternenepoche_agenten::labor::{
    self, broker::ToolCall, config::Config, gateway, memory::Memory,
};

#[test]
fn flat_queries_use_engine_validation_and_preserve_legacy_catalog() {
    let c = Config::v4(2);
    let (w, _) = labor::world::create(&c).unwrap();
    let role = c.execution_roles(0, &c.players[0]).remove(0);
    let memory = Memory::new(0, c.budget.memory_bytes, &c.players[0]).unwrap();
    let planet = w.planeten[w.spieler[0].heimat as usize].koord.to_string();
    let call = |name: &str, arguments| ToolCall {
        id: "probe".into(),
        name: name.into(),
        arguments,
    };
    let hash = w.hash();
    let flat = gateway::execute(
        &w,
        &memory,
        &c,
        &role,
        &call(
            "query_costs",
            json!({"product_kind":"gebaeude","product":"solarkraftwerk","planet":planet}),
        ),
    )
    .unwrap();
    let legacy = gateway::execute(
        &w,
        &memory,
        &c,
        &role,
        &call(
            "world_query",
            json!({"typ":"kosten","gebaeude":"solarkraftwerk","planet":planet}),
        ),
    )
    .unwrap();
    assert_eq!(flat.result, legacy.result);
    assert_eq!(w.hash(), hash);
    assert!(gateway::execute(
        &w,
        &memory,
        &c,
        &role,
        &call("query_costs", json!({"product":"solarkraftwerk"}))
    )
    .is_err());
    let old = Config::v3(2);
    assert!(gateway::execute(
        &w,
        &memory,
        &old,
        &role,
        &call("query_rules", json!({"stichwort":"Energie"}))
    )
    .is_err());
    assert!(!gateway::catalog(&old, &role)
        .iter()
        .any(|t| t["function"]["name"]
            .as_str()
            .unwrap()
            .starts_with("query_")));
}

#[test]
fn mocks_cannot_authorize_a_throughput_budget() {
    let mut c = Config::v4(2);
    c.days = 1;
    c.budget.calls_per_window = 2;
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../laeufe/labor-tests/capacity-{unique}"));
    labor::run(&c, &root, 1, false).unwrap();
    let sources = vec![root.to_string_lossy().into_owned()];
    let plan = labor::capacity::plan(&c, &sources).unwrap();
    assert_eq!(plan["status"], "insufficient_evidence");
    assert_eq!(plan["estimated_feasible"], false);
    let out = root.join("not-approved.json");
    assert!(labor::capacity::fit(&c, &sources, &out).is_err());
    assert!(!out.exists());
    c.models
        .values_mut()
        .for_each(|m| m.context = c.budget.output_tokens);
    assert!(labor::capacity::plan(&c, &sources).is_err());
}
