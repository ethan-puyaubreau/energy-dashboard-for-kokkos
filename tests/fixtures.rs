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
    assert_close(region(&analysis, "MatVec").total_energy_joules, 600.0);
    assert_close(region(&analysis, "Reduction").total_energy_joules, 900.0);
    assert_close(region(&analysis, "MainLoop").total_energy_joules, 1875.0);
}

#[test]
fn real_rtx3080ti_trace_is_consistent() {
    let analysis = analyze_fixture("real_rtx3080ti_trace");

    assert!(analysis.total_trace_energy_joules > 0.0);
    assert!(!analysis.regions.is_empty());
    for r in &analysis.regions {
        assert!(r.total_energy_joules.is_finite() && r.total_energy_joules >= 0.0);
        assert!(r.total_energy_joules <= analysis.total_trace_energy_joules + 1e-6);
    }
}
