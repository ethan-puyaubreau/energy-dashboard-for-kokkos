//! Numerical integration of sampled power over time.

use crate::model::PowerSample;

/// Number of nanoseconds in one second.
const NS_PER_SEC: f64 = 1_000_000_000.0;

/// Linearly interpolate the power at `t_ns`.
///
/// Outside the sampled range the nearest sample is held constant.
/// `samples` must be sorted by timestamp and non-empty.
fn power_at(samples: &[PowerSample], t_ns: u64) -> f64 {
    let i = samples.partition_point(|s| s.timestamp_ns <= t_ns);
    if i == 0 {
        return samples[0].power_watts;
    }
    if i == samples.len() {
        return samples[i - 1].power_watts;
    }

    let (s0, s1) = (&samples[i - 1], &samples[i]);
    let frac = (t_ns - s0.timestamp_ns) as f64 / (s1.timestamp_ns - s0.timestamp_ns) as f64;
    s0.power_watts + frac * (s1.power_watts - s0.power_watts)
}

/// Linearly interpolate the cumulative energy counter at `t_ns`.
///
/// Returns `None` when `t_ns` is outside the sampled range or when a
/// bracketing sample carries no counter value.
fn energy_at(samples: &[PowerSample], t_ns: u64) -> Option<f64> {
    let i = samples.partition_point(|s| s.timestamp_ns <= t_ns);
    let s0 = &samples[i.checked_sub(1)?];
    if s0.timestamp_ns == t_ns {
        return s0.energy_joules;
    }

    let s1 = samples.get(i)?;
    let (e0, e1) = (s0.energy_joules?, s1.energy_joules?);
    let frac = (t_ns - s0.timestamp_ns) as f64 / (s1.timestamp_ns - s0.timestamp_ns) as f64;
    Some(e0 + frac * (e1 - e0))
}

/// Energy in Joules of a linear power segment between two points.
fn trapezoid(t0_ns: u64, p0: f64, t1_ns: u64, p1: f64) -> f64 {
    (p0 + p1) / 2.0 * (t1_ns - t0_ns) as f64 / NS_PER_SEC
}

/// Integrate power over time using the composite trapezoidal rule.
///
/// Power is linearly interpolated at `start_ns` and `end_ns`, so intervals
/// shorter than the sampling period still get a time-accurate estimate.
/// `samples` must be sorted by timestamp.
///
/// Returns energy in Joules between `start_ns` and `end_ns`.
/// If a hardware cumulative energy counter brackets both endpoints, the
/// interpolated counter difference is returned instead. A decreasing counter
/// (reset or wraparound) falls back to power integration.
pub fn integrate_energy_joules(samples: &[PowerSample], start_ns: u64, end_ns: u64) -> f64 {
    if samples.is_empty() || start_ns >= end_ns {
        return 0.0;
    }

    // Prefer the hardware cumulative counter when it brackets both endpoints
    if let (Some(e_start), Some(e_end)) = (energy_at(samples, start_ns), energy_at(samples, end_ns))
    {
        if e_end >= e_start {
            return e_end - e_start;
        }
    }

    // Samples strictly inside the interval, bounded by interpolated endpoints
    let lo = samples.partition_point(|s| s.timestamp_ns <= start_ns);
    let hi = samples.partition_point(|s| s.timestamp_ns < end_ns);

    let mut total_joules = 0.0;
    let mut t0 = start_ns;
    let mut p0 = power_at(samples, start_ns);
    for s in &samples[lo..hi] {
        total_joules += trapezoid(t0, p0, s.timestamp_ns, s.power_watts);
        t0 = s.timestamp_ns;
        p0 = s.power_watts;
    }
    total_joules + trapezoid(t0, p0, end_ns, power_at(samples, end_ns))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::DeviceDomain;

    /// Build a GPU power sample at `t_sec` seconds.
    fn gpu(t_sec: f64, power_watts: f64) -> PowerSample {
        PowerSample {
            timestamp_ns: (t_sec * NS_PER_SEC) as u64,
            domain: DeviceDomain::Gpu,
            device_id: 0,
            power_watts,
            energy_joules: None,
        }
    }

    /// Integrate between two times expressed in seconds.
    fn integrate(samples: &[PowerSample], start_sec: f64, end_sec: f64) -> f64 {
        integrate_energy_joules(
            samples,
            (start_sec * NS_PER_SEC) as u64,
            (end_sec * NS_PER_SEC) as u64,
        )
    }

    #[test]
    fn test_constant_power_integration() {
        // 100 Watts during 2 seconds = 200 Joules
        let samples = vec![gpu(1.0, 100.0), gpu(2.0, 100.0), gpu(3.0, 100.0)];
        assert!((integrate(&samples, 1.0, 3.0) - 200.0).abs() < 1e-6);
    }

    #[test]
    fn test_boundary_is_interpolated() {
        // Power ramps from 100 W to 300 W, reaching 200 W at t = 1 s
        let samples = vec![gpu(0.0, 100.0), gpu(2.0, 300.0)];
        assert!((integrate(&samples, 0.0, 1.0) - 150.0).abs() < 1e-6);
    }

    #[test]
    fn test_interval_between_samples() {
        // Power is 150 W at 0.5 s and 160 W at 0.6 s
        let samples = vec![gpu(0.0, 100.0), gpu(2.0, 300.0)];
        assert!((integrate(&samples, 0.5, 0.6) - 15.5).abs() < 1e-6);
    }

    /// Attach a cumulative energy counter value to a sample.
    fn with_counter(mut sample: PowerSample, energy_joules: f64) -> PowerSample {
        sample.energy_joules = Some(energy_joules);
        sample
    }

    #[test]
    fn test_counter_is_interpolated_at_boundaries() {
        let samples = vec![
            with_counter(gpu(0.0, 0.0), 0.0),
            with_counter(gpu(1.0, 0.0), 100.0),
            with_counter(gpu(2.0, 0.0), 300.0),
        ];
        assert!((integrate(&samples, 0.5, 1.5) - 150.0).abs() < 1e-6);
    }

    #[test]
    fn test_partial_counter_falls_back_to_power() {
        let samples = vec![gpu(0.0, 100.0), with_counter(gpu(1.0, 100.0), 500.0)];
        assert!((integrate(&samples, 0.0, 1.0) - 100.0).abs() < 1e-6);
    }

    #[test]
    fn test_outside_sampled_range_holds_nearest() {
        let samples = vec![gpu(1.0, 100.0), gpu(2.0, 100.0)];
        assert!((integrate(&samples, 0.0, 3.0) - 300.0).abs() < 1e-6);
    }
}
