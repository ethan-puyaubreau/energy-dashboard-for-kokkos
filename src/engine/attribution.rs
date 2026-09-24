//! Attribution of the measured energy to Kokkos regions and kernels.

use super::integrate::integrate_energy_joules;
use crate::model::{DeviceDomain, RegionCategory, Trace};
use std::cmp::Reverse;
use std::collections::HashMap;

/// Aggregated metrics for a family of events with the same name and category.
#[derive(Debug, Clone)]
pub struct RegionMetrics {
    /// Region or kernel label.
    pub name: String,
    /// Kokkos category shared by every event of the family.
    pub category: RegionCategory,
    /// Number of events in the family.
    pub call_count: usize,
    /// Summed wall time of every event, in seconds.
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
    /// Measured hardware domain.
    pub domain: DeviceDomain,
    /// Device index within the domain.
    pub device_id: u32,
    /// Energy over the whole trace window, in Joules.
    pub energy_joules: f64,
    /// Energy divided by the trace window duration, in Watts.
    pub avg_power_watts: f64,
}

/// Global trace analysis result.
#[derive(Debug, Clone, Default)]
pub struct TraceAnalysis {
    /// Duration of the whole measured window, events and samples included.
    pub total_trace_duration_sec: f64,
    /// Energy of the whole measured window, summed over every device.
    pub total_trace_energy_joules: f64,
    /// Total energy divided by the window duration.
    pub avg_trace_power_watts: f64,
    /// Time of the window not covered by any event.
    pub idle_duration_sec: f64,
    /// Energy of the window not attributed to any event.
    pub idle_energy_joules: f64,
    /// Breakdown of the total energy per power series.
    pub devices: Vec<DeviceMetrics>,
    /// Median interval between two samples of a series, if at least two samples exist.
    pub sampling_period_sec: Option<f64>,
    /// Shortest duration the power readings resolve: the sampling period, or the NVML
    /// refresh interval when that is longer.
    pub resolution_sec: Option<f64>,
    /// Fraction of events shorter than `resolution_sec`, in [0, 1].
    ///
    /// The power of such events is interpolated between readings, not measured.
    pub short_event_fraction: f64,
    /// Per-region metrics sorted by inclusive energy, highest first.
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

/// NVML refreshes its power reading about every 100 ms (Yang, Adamek and Armour, SC24), so events
/// shorter than that are not measured on their own, however fast the connector samples.
const NVML_REFRESH_NS: u64 = 100_000_000;

/// Median interval in nanoseconds between consecutive samples of the same series.
fn median_sampling_period_ns(trace: &Trace) -> Option<u64> {
    let mut intervals: Vec<u64> = trace
        .series
        .iter()
        .flat_map(|s| s.samples.windows(2))
        .map(|w| w[1].timestamp_ns - w[0].timestamp_ns)
        .collect();
    if intervals.is_empty() {
        return None;
    }

    let mid = intervals.len() / 2;
    Some(*intervals.select_nth_unstable(mid).1)
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

    let sampling_period_ns = median_sampling_period_ns(trace);
    // The NVML floor only applies when a GPU is measured; CPU domains keep the sampling period.
    let has_gpu = trace.series.iter().any(|s| s.domain == DeviceDomain::Gpu);
    let resolution_ns = sampling_period_ns.map(|period| {
        if has_gpu {
            period.max(NVML_REFRESH_NS)
        } else {
            period
        }
    });
    let short_events = resolution_ns.map_or(0, |resolution| {
        trace
            .events
            .iter()
            .filter(|e| e.duration_ns() < resolution)
            .count()
    });
    let short_event_fraction = if trace.events.is_empty() {
        0.0
    } else {
        short_events as f64 / trace.events.len() as f64
    };

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
        sampling_period_sec: sampling_period_ns.map(|ns| ns as f64 / 1_000_000_000.0),
        resolution_sec: resolution_ns.map(|ns| ns as f64 / 1_000_000_000.0),
        short_event_fraction,
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
    fn test_events_shorter_than_sampling_period_are_counted() {
        let short = Event {
            end_ns: 500_000_000,
            ..event(2, 0, "Short", 0, 0)
        };
        let events = vec![event(1, 0, "Long", 0, 2), short];
        let analysis = analyze_trace(&Trace::new(None, events, constant_power(2)));

        assert_eq!(analysis.sampling_period_sec, Some(1.0));
        assert_eq!(analysis.resolution_sec, Some(1.0));
        assert!((analysis.short_event_fraction - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_events_shorter_than_nvml_refresh_are_counted() {
        // Sampled every 20 ms: a 50 ms event spans two samples but is still shorter
        // than the ~100 ms NVML refresh, so its power is not measured on its own.
        let ms = 1_000_000;
        let samples = (0..=10)
            .map(|i| PowerSample {
                timestamp_ns: i * 20 * ms,
                domain: DeviceDomain::Gpu,
                device_id: 0,
                power_watts: 100.0,
                energy_joules: None,
            })
            .collect();
        let events = vec![Event {
            start_ns: 0,
            end_ns: 50 * ms,
            ..event(1, 0, "Kernel", 0, 0)
        }];
        let analysis = analyze_trace(&Trace::new(None, events, samples));

        assert_eq!(analysis.sampling_period_sec, Some(0.02));
        assert_eq!(analysis.resolution_sec, Some(0.1));
        assert!((analysis.short_event_fraction - 1.0).abs() < 1e-9);

        // Without a GPU series the NVML floor does not apply.
        let cpu = Trace::new(
            None,
            vec![Event {
                start_ns: 0,
                end_ns: 50 * ms,
                ..event(1, 0, "Kernel", 0, 0)
            }],
            (0..=10)
                .map(|i| PowerSample {
                    timestamp_ns: i * 20 * ms,
                    domain: DeviceDomain::CpuPkg,
                    device_id: 0,
                    power_watts: 50.0,
                    energy_joules: None,
                })
                .collect(),
        );
        let analysis = analyze_trace(&cpu);
        assert_eq!(analysis.resolution_sec, Some(0.02));
        assert_eq!(analysis.short_event_fraction, 0.0);
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
