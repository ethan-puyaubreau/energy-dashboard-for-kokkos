use crate::model::PowerSample;

/// Integrate power over time using the composite trapezoidal rule.
///
/// Returns energy in Joules between `start_ns` and `end_ns`.
/// If hardware cumulative energy counters are present in the boundary samples,
/// the exact difference is returned.
pub fn integrate_energy_joules(samples: &[PowerSample], start_ns: u64, end_ns: u64) -> f64 {
    if samples.is_empty() || start_ns >= end_ns {
        return 0.0;
    }

    // Filter samples within the interval
    let window: Vec<&PowerSample> = samples
        .iter()
        .filter(|s| s.timestamp_ns >= start_ns && s.timestamp_ns <= end_ns)
        .collect();

    if window.is_empty() {
        // Approximate using nearest surrounding samples if available
        let before = samples.iter().filter(|s| s.timestamp_ns < start_ns).last();
        let after = samples.iter().find(|s| s.timestamp_ns > end_ns);

        let power = match (before, after) {
            (Some(b), Some(a)) => (b.power_watts + a.power_watts) / 2.0,
            (Some(b), None) => b.power_watts,
            (None, Some(a)) => a.power_watts,
            (None, None) => return 0.0,
        };
        let dt_sec = (end_ns - start_ns) as f64 / 1_000_000_000.0;
        return power * dt_sec;
    }

    // If hardware cumulative energy is provided on first and last sample, use it
    if let (Some(first), Some(last)) = (window.first(), window.last()) {
        if let (Some(e_start), Some(e_end)) = (first.energy_joules, last.energy_joules) {
            if e_end >= e_start {
                return e_end - e_start;
            }
        }
    }

    // Composite trapezoidal integration
    let mut total_joules = 0.0;

    // Boundary 1: from start_ns to first sample
    if let Some(first) = window.first() {
        if first.timestamp_ns > start_ns {
            let dt_sec = (first.timestamp_ns - start_ns) as f64 / 1_000_000_000.0;
            total_joules += first.power_watts * dt_sec;
        }
    }

    // Main window segments
    for i in 0..window.len().saturating_sub(1) {
        let s0 = window[i];
        let s1 = window[i + 1];
        let dt_sec = (s1.timestamp_ns - s0.timestamp_ns) as f64 / 1_000_000_000.0;
        let avg_power = (s0.power_watts + s1.power_watts) / 2.0;
        total_joules += avg_power * dt_sec;
    }

    // Boundary 2: from last sample to end_ns
    if let Some(last) = window.last() {
        if end_ns > last.timestamp_ns {
            let dt_sec = (end_ns - last.timestamp_ns) as f64 / 1_000_000_000.0;
            total_joules += last.power_watts * dt_sec;
        }
    }

    total_joules
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::DeviceDomain;

    #[test]
    fn test_constant_power_integration() {
        // 100 Watts during 2 seconds = 200 Joules
        let samples = vec![
            PowerSample {
                timestamp_ns: 1_000_000_000,
                domain: DeviceDomain::Gpu,
                device_id: 0,
                power_watts: 100.0,
                energy_joules: None,
            },
            PowerSample {
                timestamp_ns: 2_000_000_000,
                domain: DeviceDomain::Gpu,
                device_id: 0,
                power_watts: 100.0,
                energy_joules: None,
            },
            PowerSample {
                timestamp_ns: 3_000_000_000,
                domain: DeviceDomain::Gpu,
                device_id: 0,
                power_watts: 100.0,
                energy_joules: None,
            },
        ];

        let energy = integrate_energy_joules(&samples, 1_000_000_000, 3_000_000_000);
        assert!((energy - 200.0).abs() < 1e-6);
    }
}
