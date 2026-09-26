use energy_dashboard_for_kokkos::engine::{analyze_trace, RegionMetrics, TraceAnalysis};
use energy_dashboard_for_kokkos::model::Trace;
use energy_dashboard_for_kokkos::parser::load_trace_dir;
use energy_dashboard_for_kokkos::report::render_terminal_report;

/// Analyze a trace directory located under tests/fixtures.
fn analyze_fixture(name: &str) -> TraceAnalysis {
    analyze_trace(&load_fixture(name))
}

fn load_fixture(name: &str) -> Trace {
    let dir = format!("{}/tests/fixtures/{}", env!("CARGO_MANIFEST_DIR"), name);
    load_trace_dir(&dir).expect("fixture should load")
}

/// Find the table row of a block in a rendered terminal report.
fn report_row<'a>(report: &'a str, name: &str) -> &'a str {
    report
        .lines()
        .find(|line| line.starts_with(&format!("\u{2502} {name} ")))
        .unwrap_or_else(|| panic!("row {name} not found in:\n{report}"))
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

#[test]
fn h100_dbscan_run_matches_the_published_figure() {
    // One ArborX DBSCAN run from the SMC 2025 poster data, converted with
    // analysis/to_trace_v1.py from the poster repository. Power is interpolated at the
    // region boundaries, hence 771.8 J where the poster script reads 769.3 J.
    let analysis = analyze_fixture("h100_arborx_fdbscan");
    let dbscan = region(&analysis, "DBSCANCalculation");

    assert!((dbscan.total_duration_sec - 2.681).abs() < 1e-3);
    assert!((dbscan.inclusive_energy_joules - 771.83).abs() < 0.01);
}

#[test]
fn terminal_report_lists_regions_idle_and_total() {
    let trace = load_fixture("synthetic_run");
    let report = render_terminal_report(&trace, &analyze_trace(&trace));

    assert!(report.contains("App: synthetic_bench (Host: test-node, Backend: CUDA)"));
    for (name, cells) in [
        ("MainLoop", ["1875.00", "375.00", "20.0%"]),
        ("Reduction", ["900.00", "900.00", "48.0%"]),
        ("MatVec", ["600.00", "600.00", "32.0%"]),
        ("Idle (outside events)", ["0.000", "0.00", "0.0%"]),
        ("Total Trace", ["1875.00", "234.4", "100.0%"]),
    ] {
        let row = report_row(&report, name);
        for cell in cells {
            assert!(row.contains(cell), "{name} row lacks {cell}: {row}");
        }
    }
    assert!(report.contains("GPU 0: 1875.00 J, 234.4 W avg"));
    assert!(!report.contains("Note:"));
}

#[test]
fn terminal_report_without_metadata_or_with_short_events() {
    let mut trace = load_fixture("real_rtx3080ti_trace");
    let analysis = analyze_trace(&trace);
    trace.metadata = None;
    let report = render_terminal_report(&trace, &analysis);

    assert!(report.contains("energy-dashboard-for-kokkos report"));
    // Most kernels of the real run are shorter than the NVML refresh period.
    assert!(report.contains("Note: "));
    assert!(report.contains("their power is interpolated"));
}
