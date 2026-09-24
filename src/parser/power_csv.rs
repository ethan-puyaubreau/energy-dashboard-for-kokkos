//! Parser for `power_samples.csv`.

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
    for (index, result) in rdr.deserialize().enumerate() {
        let sample: PowerSample = result.with_context(|| {
            format!(
                "Malformed record in power samples file: {}",
                path.as_ref().display()
            )
        })?;

        // NaN or infinite values would silently poison every energy sum
        if !sample.power_watts.is_finite() {
            anyhow::bail!(
                "Non-finite value in power samples file {} at record {}",
                path.as_ref().display(),
                index + 1
            );
        }
        samples.push(sample);
    }

    Ok(samples)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_power_samples() {
        let data = "timestamp_ns,domain,device_id,power_watts\n100,GPU,0,250.5\n200,GPU,1,120.0\n";
        let mut rdr = csv::Reader::from_reader(data.as_bytes());
        let samples: Vec<PowerSample> = rdr.deserialize().map(|r| r.unwrap()).collect();
        assert_eq!(samples.len(), 2);
        assert_eq!(samples[0].domain, crate::model::DeviceDomain::Gpu);
        assert_eq!(samples[0].power_watts, 250.5);
        assert_eq!(samples[1].device_id, 1);
    }

    #[test]
    fn test_legacy_energy_column_is_ignored() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        std::io::Write::write_all(
            &mut file,
            b"timestamp_ns,domain,device_id,power_watts,energy_joules
100,GPU,0,250.5,
200,GPU,0,260.0,450.2
",
        )
        .unwrap();
        let samples = parse_power_csv(file.path()).unwrap();
        assert_eq!(samples.len(), 2);
        assert_eq!(samples[1].power_watts, 260.0);
    }

    #[test]
    fn test_reject_non_finite_power() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        std::io::Write::write_all(
            &mut file,
            b"timestamp_ns,domain,device_id,power_watts
100,GPU,0,NaN
",
        )
        .unwrap();
        assert!(parse_power_csv(file.path()).is_err());
    }
}
