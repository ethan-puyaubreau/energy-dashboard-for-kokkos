use crate::model::PowerSample;
use anyhow::{Context, Result};
use std::fs::File;
use std::path::Path;

/// Parse power_samples.csv into a vector of PowerSample structs.
pub fn parse_power_csv<P: AsRef<Path>>(path: P) -> Result<Vec<PowerSample>> {
    let file = File::open(path.as_ref()).with_context(|| {
        format!(
            "Failed to open power samples file: {}",
            path.as_ref().display()
        )
    })?;

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .from_reader(file);

    let mut samples = Vec::new();
    for result in rdr.deserialize() {
        let sample: PowerSample = result.with_context(|| {
            format!(
                "Malformed record in power samples file: {}",
                path.as_ref().display()
            )
        })?;
        samples.push(sample);
    }

    Ok(samples)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_power_samples() {
        let data = "timestamp_ns,domain,device_id,power_watts,energy_joules\n100,GPU,0,250.5,\n200,CPU_PKG,1,120.0,450.2\n";
        let mut rdr = csv::Reader::from_reader(data.as_bytes());
        let samples: Vec<PowerSample> = rdr.deserialize().map(|r| r.unwrap()).collect();
        assert_eq!(samples.len(), 2);
        assert_eq!(samples[0].domain, crate::model::DeviceDomain::Gpu);
        assert_eq!(samples[0].power_watts, 250.5);
        assert_eq!(samples[0].energy_joules, None);
        assert_eq!(samples[1].energy_joules, Some(450.2));
    }
}
