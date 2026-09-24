use super::integrate::integrate_energy_joules;
use crate::model::{RegionCategory, Trace};
use std::collections::HashMap;

/// Aggregated metrics for a family of events with the same name and category.
#[derive(Debug, Clone)]
pub struct RegionMetrics {
    pub name: String,
    pub category: RegionCategory,
    pub call_count: usize,
    pub total_duration_sec: f64,
    pub total_energy_joules: f64,
    pub avg_power_watts: f64,
    pub energy_percentage: f64,
}

/// Global trace analysis result.
#[derive(Debug, Clone)]
pub struct TraceAnalysis {
    pub total_trace_duration_sec: f64,
    pub total_trace_energy_joules: f64,
    pub avg_trace_power_watts: f64,
    pub regions: Vec<RegionMetrics>,
}

/// Analyze a complete trace and compute energy attribution per region/kernel.
pub fn analyze_trace(trace: &Trace) -> TraceAnalysis {
    if trace.events.is_empty() {
        return TraceAnalysis {
            total_trace_duration_sec: 0.0,
            total_trace_energy_joules: 0.0,
            avg_trace_power_watts: 0.0,
            regions: Vec::new(),
        };
    }

    // Determine overall trace time bounds
    let min_start = trace.events.iter().map(|e| e.start_ns).min().unwrap_or(0);
    let max_end = trace.events.iter().map(|e| e.end_ns).max().unwrap_or(0);
    let total_trace_duration_sec = (max_end.saturating_sub(min_start)) as f64 / 1_000_000_000.0;
    let total_trace_energy_joules = integrate_energy_joules(&trace.samples, min_start, max_end);
    let avg_trace_power_watts = if total_trace_duration_sec > 0.0 {
        total_trace_energy_joules / total_trace_duration_sec
    } else {
        0.0
    };

    // Aggregate by (name, category)
    struct Agg {
        count: usize,
        duration_sec: f64,
        energy_joules: f64,
    }

    let mut map: HashMap<(String, RegionCategory), Agg> = HashMap::new();

    for event in &trace.events {
        let dur = event.duration_sec();
        let energy = integrate_energy_joules(&trace.samples, event.start_ns, event.end_ns);

        let entry = map
            .entry((event.name.clone(), event.category))
            .or_insert(Agg {
                count: 0,
                duration_sec: 0.0,
                energy_joules: 0.0,
            });

        entry.count += 1;
        entry.duration_sec += dur;
        entry.energy_joules += energy;
    }

    let mut regions = Vec::new();
    for ((name, category), agg) in map {
        let avg_power = if agg.duration_sec > 0.0 {
            agg.energy_joules / agg.duration_sec
        } else {
            0.0
        };

        let energy_pct = if total_trace_energy_joules > 0.0 {
            (agg.energy_joules / total_trace_energy_joules) * 100.0
        } else {
            0.0
        };

        regions.push(RegionMetrics {
            name,
            category,
            call_count: agg.count,
            total_duration_sec: agg.duration_sec,
            total_energy_joules: agg.energy_joules,
            avg_power_watts: avg_power,
            energy_percentage: energy_pct,
        });
    }

    // Sort by energy descending
    regions.sort_by(|a, b| b.total_energy_joules.total_cmp(&a.total_energy_joules));

    TraceAnalysis {
        total_trace_duration_sec,
        total_trace_energy_joules,
        avg_trace_power_watts,
        regions,
    }
}
