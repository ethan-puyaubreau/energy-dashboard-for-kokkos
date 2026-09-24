use kokkos_energy::engine::{analyze_trace, RegionMetrics, TraceAnalysis};
use kokkos_energy::parser::load_trace_dir;

/// Analyze a trace directory located under tests/fixtures.
fn analyze_fixture(name: &str) -> TraceAnalysis {
    let dir = format!("{}/tests/fixtures/{}", env!("CARGO_MANIFEST_DIR"), name);
    let trace = load_trace_dir(&dir).expect("fixture should load");
    analyze_trace(&trace)
}

/// Find the aggregated metrics of a region by name.
fn region<'a>(analysis: &'a TraceAnalysis, name: &str) -> &'a RegionMetrics {
    analysis
        .regions
        .iter()
        .find(|r| r.name == name)
        .unwrap_or_else(|| panic!("region {name} not found"))
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-6,
        "expected {expected}, got {actual}"
    );
}

#[test]
fn synthetic_run_energy_per_region() {
    let analysis = analyze_fixture("synthetic_run");

    assert_eq!(analysis.regions.len(), 3);
    assert_close(region(&analysis, "MatVec").inclusive_energy_joules, 600.0);
    assert_close(
        region(&analysis, "Reduction").inclusive_energy_joules,
        900.0,
    );

    let main_loop = region(&analysis, "MainLoop");
    assert_close(main_loop.inclusive_energy_joules, 1875.0);
    assert_close(main_loop.exclusive_energy_joules, 375.0);
}

#[test]
fn multi_domain_series_are_integrated_separately() {
    let analysis = analyze_fixture("multi_domain_run");

    // GPU holds 100 W and CPU holds 50 W during 2 seconds
    assert_close(region(&analysis, "Step").inclusive_energy_joules, 300.0);
    assert_eq!(analysis.devices.len(), 2);
    assert_close(analysis.devices[0].energy_joules, 200.0);
    assert_close(analysis.devices[1].energy_joules, 100.0);
}

#[test]
fn real_rtx3080ti_trace_is_consistent() {
    let analysis = analyze_fixture("real_rtx3080ti_trace");

    assert!(analysis.total_trace_energy_joules > 0.0);
    assert!(!analysis.regions.is_empty());
    let mut exclusive_sum = 0.0;
    for r in &analysis.regions {
        assert!(r.inclusive_energy_joules.is_finite() && r.inclusive_energy_joules >= 0.0);
        assert!(r.exclusive_energy_joules <= r.inclusive_energy_joules + 1e-6);
        exclusive_sum += r.exclusive_energy_joules;
    }

    // Every Joule is attributed exactly once, to an event or to idle
    let attributed = exclusive_sum + analysis.idle_energy_joules;
    assert!((attributed - analysis.total_trace_energy_joules).abs() < 1e-6);
}
