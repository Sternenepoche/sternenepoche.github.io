use sternenepoche_agenten::labor::balance;

#[test]
fn bounded_v4_acceptance_has_measured_limits_and_reproducible_results() {
    let report=balance::report().unwrap();assert_eq!(report["labor_version"],4);assert_eq!(report["status"],"PASS_WITH_LIMITS","{report}");
    let maps=report["map_cases"].as_array().unwrap();assert_eq!(maps.len(),18);
    for map in maps {assert_eq!(map["passed"],true,"{map}");
        if [3,11].contains(&map["players"].as_u64().unwrap()) {let triple=&map["triples"][0];assert_eq!(triple["single_seat_equal_pressure"],false);assert_eq!(map["full_single_seat_territorial_fairness"]["passed"],false);}
    }
    assert_eq!(report["tactical_cases"].as_array().unwrap().len(),2);
    for tactical in report["tactical_cases"].as_array().unwrap(){assert_eq!(tactical["passed"],true,"{tactical}");}
    assert_eq!(balance::report().unwrap(),report);
}
