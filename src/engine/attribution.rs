use super::integrate::integrate_energy_joules;
use crate::model::{DeviceDomain, RegionCategory, Trace};
use std::cmp::Reverse;
use std::collections::HashMap;

/// Aggregated metrics for a family of events with the same name and category.
#[derive(Debug, Clone)]
pub struct RegionMetrics {
    pub name: String,
    pub category: RegionCategory,
    pub call_count: usize,
    pub total_duration_sec: f64,
    /// Energy spent while the region was active, children included.
    pub inclusive_energy_joules: f64,
    /// Energy spent in the region itself, children excluded.
    pub exclusive_energy_joules: f64,
    /// Inclusive energy divided by duration.
    pub avg_power_watts: f64,
    /// Share of the total trace energy, based on exclusive energy.
    pub energy_percentage: f64,
}

/// Energy of a single power series over the whole trace window.
#[derive(Debug, Clone)]
pub struct DeviceMetrics {
    pub domain: DeviceDomain,
    pub device_id: u32,
    pub energy_joules: f64,
    pub avg_power_watts: f64,
}

/// Global trace analysis result.
#[derive(Debug, Clone)]
pub struct TraceAnalysis {
    /// Duration of the whole measured window, events and samples included.
    pub total_trace_duration_sec: f64,
    pub total_trace_energy_joules: f64,
    pub avg_trace_power_watts: f64,
    /// Time of the window not covered by any event.
    pub idle_duration_sec: f64,
    /// Energy of the window not attributed to any event.
    pub idle_energy_joules: f64,
    /// Breakdown of the total energy per power series.
    pub devices: Vec<DeviceMetrics>,
    pub regions: Vec<RegionMetrics>,
}

/// Energy in Joules summed over every power series between two timestamps.
fn energy_joules(trace: &Trace, start_ns: u64, end_ns: u64) -> f64 {
    trace
        .series
        .iter()
        .map(|s| integrate_energy_joules(&s.samples, start_ns, end_ns))
        .sum()
}

/// Compute the exclusive energy of each event with a timeline sweep.
///
/// The timeline is cut at every event boundary. The energy of each segment
/// goes to the innermost active events, split equally when several of them
/// overlap, so that no Joule is counted twice.
fn exclusive_energies(trace: &Trace, parents: &[Option<usize>]) -> Vec<f64> {
    let n = trace.events.len();

    // (timestamp, event index, is_start), starts first on equal timestamps
    let mut boundaries = Vec::with_capacity(2 * n);
    for (i, e) in trace.events.iter().enumerate() {
        boundaries.push((e.start_ns, i, true));
        boundaries.push((e.end_ns.max(e.start_ns), i, false));
    }
    boundaries.sort_unstable_by_key(|&(t, _, is_start)| (t, !is_start));

    let mut exclusive = vec![0.0; n];
    let mut active: Vec<usize> = Vec::new();
    let mut active_children = vec![0usize; n];

    let mut k = 0;
    while k < boundaries.len() {
        let t0 = boundaries[k].0;
        while k < boundaries.len() && boundaries[k].0 == t0 {
            let (_, i, is_start) = boundaries[k];
            if is_start {
                active.push(i);
                if let Some(p) = parents[i] {
                    active_children[p] += 1;
                }
            } else {
                if let Some(pos) = active.iter().position(|&a| a == i) {
                    active.swap_remove(pos);
                }
                if let Some(p) = parents[i] {
                    active_children[p] -= 1;
                }
            }
            k += 1;
        }

        let Some(&(t1, _, _)) = boundaries.get(k) else {
            break;
        };
        let innermost: Vec<usize> = active
            .iter()
            .copied()
            .filter(|&i| active_children[i] == 0)
            .collect();
        if innermost.is_empty() {
            continue;
        }

        let share = energy_joules(trace, t0, t1) / innermost.len() as f64;
        for i in innermost {
            exclusive[i] += share;
        }
    }

    exclusive
}

/// Total time in nanoseconds covered by at least one event.
///
/// `trace.events` is sorted by start timestamp.
fn covered_ns(trace: &Trace) -> u64 {
    let mut covered = 0;
    let mut current: Option<(u64, u64)> = None;
    for e in &trace.events {
        match current {
            Some((start, end)) if e.start_ns <= end => current = Some((start, end.max(e.end_ns))),
            _ => {
                if let Some((start, end)) = current {
                    covered += end - start;
                }
                current = Some((e.start_ns, e.end_ns.max(e.start_ns)));
            }
        }
    }
    covered + current.map_or(0, |(start, end)| end - start)
}

/// Analyze a complete trace and compute energy attribution per region/kernel.
pub fn analyze_trace(trace: &Trace) -> TraceAnalysis {
    // The window spans every event and every sample, so idle phases are measured too
    let (min_start, max_end) = trace.time_bounds().unwrap_or((0, 0));
    let total_trace_duration_sec = (max_end.saturating_sub(min_start)) as f64 / 1_000_000_000.0;
    let total_trace_energy_joules = energy_joules(trace, min_start, max_end);
    let avg_trace_power_watts = if total_trace_duration_sec > 0.0 {
        total_trace_energy_joules / total_trace_duration_sec
    } else {
        0.0
    };

    let devices = trace
        .series
        .iter()
        .map(|s| {
            let energy = integrate_energy_joules(&s.samples, min_start, max_end);
            DeviceMetrics {
                domain: s.domain,
                device_id: s.device_id,
                energy_joules: energy,
                avg_power_watts: if total_trace_duration_sec > 0.0 {
                    energy / total_trace_duration_sec
                } else {
                    0.0
                },
            }
        })
        .collect();

    // Inclusive energy is the exclusive energy summed over each subtree
    let parents = trace.event_parents();
    let exclusive = exclusive_energies(trace, &parents);
    let depths = trace.event_depths();
    let mut order: Vec<usize> = (0..trace.events.len()).collect();
    order.sort_unstable_by_key(|&i| Reverse(depths[i]));
    let mut inclusive = exclusive.clone();
    for i in order {
        if let Some(p) = parents[i] {
            inclusive[p] += inclusive[i];
        }
    }

    let exclusive_sum: f64 = exclusive.iter().sum();
    let idle_energy_joules = (total_trace_energy_joules - exclusive_sum).max(0.0);
    let idle_duration_sec =
        (max_end - min_start).saturating_sub(covered_ns(trace)) as f64 / 1_000_000_000.0;

    // Aggregate by (name, category)
    struct Agg {
        count: usize,
        duration_sec: f64,
        inclusive_joules: f64,
        exclusive_joules: f64,
    }

    let mut map: HashMap<(String, RegionCategory), Agg> = HashMap::new();

    for (i, event) in trace.events.iter().enumerate() {
        let entry = map
            .entry((event.name.clone(), event.category))
            .or_insert(Agg {
                count: 0,
                duration_sec: 0.0,
                inclusive_joules: 0.0,
                exclusive_joules: 0.0,
            });

        entry.count += 1;
        entry.duration_sec += event.duration_sec();
        entry.inclusive_joules += inclusive[i];
        entry.exclusive_joules += exclusive[i];
    }

    let mut regions = Vec::new();
    for ((name, category), agg) in map {
        let avg_power = if agg.duration_sec > 0.0 {
            agg.inclusive_joules / agg.duration_sec
        } else {
            0.0
        };

        let energy_pct = if total_trace_energy_joules > 0.0 {
            (agg.exclusive_joules / total_trace_energy_joules) * 100.0
        } else {
            0.0
        };

        regions.push(RegionMetrics {
            name,
            category,
            call_count: agg.count,
            total_duration_sec: agg.duration_sec,
            inclusive_energy_joules: agg.inclusive_joules,
            exclusive_energy_joules: agg.exclusive_joules,
            avg_power_watts: avg_power,
            energy_percentage: energy_pct,
        });
    }

    // Sort by inclusive energy descending
    regions.sort_by(|a, b| {
        b.inclusive_energy_joules
            .total_cmp(&a.inclusive_energy_joules)
    });

    TraceAnalysis {
        total_trace_duration_sec,
        total_trace_energy_joules,
        avg_trace_power_watts,
        idle_duration_sec,
        idle_energy_joules,
        devices,
        regions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{DeviceDomain, Event, PowerSample};

    /// Build an event spanning `start_sec` to `end_sec` seconds.
    fn event(id: u64, parent_id: u64, name: &str, start_sec: u64, end_sec: u64) -> Event {
        Event {
            id,
            parent_id,
            name: name.to_string(),
            category: RegionCategory::ParallelFor,
            start_ns: start_sec * 1_000_000_000,
            end_ns: end_sec * 1_000_000_000,
        }
    }

    /// Build a constant 100 W GPU power series from 0 to `end_sec` seconds.
    fn constant_power(end_sec: u64) -> Vec<PowerSample> {
        (0..=end_sec)
            .map(|t| PowerSample {
                timestamp_ns: t * 1_000_000_000,
                domain: DeviceDomain::Gpu,
                device_id: 0,
                power_watts: 100.0,
                energy_joules: None,
            })
            .collect()
    }

    fn find<'a>(analysis: &'a TraceAnalysis, name: &str) -> &'a RegionMetrics {
        analysis.regions.iter().find(|r| r.name == name).unwrap()
    }

    #[test]
    fn test_overlapping_siblings_share_energy() {
        // A and B overlap during 1 s, which is split between them
        let events = vec![event(1, 0, "A", 0, 2), event(2, 0, "B", 1, 3)];
        let analysis = analyze_trace(&Trace::new(None, events, constant_power(3)));

        assert!((find(&analysis, "A").exclusive_energy_joules - 150.0).abs() < 1e-6);
        assert!((find(&analysis, "B").exclusive_energy_joules - 150.0).abs() < 1e-6);
    }

    #[test]
    fn test_energy_outside_events_is_idle() {
        let events = vec![event(1, 0, "A", 1, 2)];
        let analysis = analyze_trace(&Trace::new(None, events, constant_power(4)));

        assert!((analysis.total_trace_energy_joules - 400.0).abs() < 1e-6);
        assert!((analysis.idle_energy_joules - 300.0).abs() < 1e-6);
        assert!((analysis.idle_duration_sec - 3.0).abs() < 1e-9);
    }

    #[test]
    fn test_parent_inclusive_contains_children() {
        let events = vec![event(1, 0, "Parent", 0, 4), event(2, 1, "Child", 1, 3)];
        let analysis = analyze_trace(&Trace::new(None, events, constant_power(4)));

        let parent = find(&analysis, "Parent");
        assert!((parent.inclusive_energy_joules - 400.0).abs() < 1e-6);
        assert!((parent.exclusive_energy_joules - 200.0).abs() < 1e-6);
        assert!((find(&analysis, "Child").inclusive_energy_joules - 200.0).abs() < 1e-6);
    }
}
