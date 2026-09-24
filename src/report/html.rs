//! Standalone HTML dashboard export.

use anyhow::{Context, Result};
use serde_json::json;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::engine::TraceAnalysis;
use crate::model::Trace;
use crate::report::sampling_note;

/// Plotly bundle embedded so the report renders without network access.
const PLOTLY_JS: &str = include_str!("../../assets/plotly-2.35.2.min.js");

/// Escape text inserted into HTML markup.
fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Serialize a value to JSON that is safe to embed inside a script element.
///
/// A kernel name containing `</script>` would otherwise close the element early.
fn script_json<T: serde::Serialize>(value: &T) -> Result<String> {
    Ok(serde_json::to_string(value)?.replace("</", "<\\/"))
}

/// Generate a standalone interactive HTML report using embedded Plotly.js.
pub fn export_html_report<P: AsRef<Path>>(
    trace: &Trace,
    analysis: &TraceAnalysis,
    out_path: P,
) -> Result<()> {
    let out_path = out_path.as_ref();
    let mut file = File::create(out_path)
        .with_context(|| format!("Failed to create HTML report: {}", out_path.display()))?;

    // Determine baseline timestamp in seconds
    let min_ts_ns = trace.time_bounds().map_or(0, |(min, _)| min);

    // One Plotly line per power series
    let timeline_traces: Vec<_> = trace
        .series
        .iter()
        .map(|s| {
            let time_sec: Vec<f64> = s
                .samples
                .iter()
                .map(|p| (p.timestamp_ns.saturating_sub(min_ts_ns)) as f64 / 1_000_000_000.0)
                .collect();
            let power_watts: Vec<f64> = s.samples.iter().map(|p| p.power_watts).collect();
            json!({
                "x": time_sec,
                "y": power_watts,
                "mode": "lines",
                "name": format!("{} {}", s.domain, s.device_id),
                "line": { "width": 2 }
            })
        })
        .collect();

    // Prepare summary bar chart data
    let region_names: Vec<String> = analysis.regions.iter().map(|r| r.name.clone()).collect();
    let region_energies: Vec<f64> = analysis
        .regions
        .iter()
        .map(|r| r.exclusive_energy_joules)
        .collect();

    let app_name = trace
        .metadata
        .as_ref()
        .and_then(|m| m.app_name.as_deref())
        .unwrap_or("Kokkos Application");
    let hostname = trace
        .metadata
        .as_ref()
        .and_then(|m| m.hostname.as_deref())
        .unwrap_or("Unknown Node");

    let app_name = escape_html(app_name);
    let hostname = escape_html(hostname);
    let timeline_traces_json = script_json(&timeline_traces)?;
    let note_html = sampling_note(analysis)
        .map(|note| format!(r#"<div class="meta">Note: {note}</div>"#))
        .unwrap_or_default();
    let region_names_json = script_json(&region_names)?;
    let region_energies_json = script_json(&region_energies)?;

    let html_content = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>energy-dashboard-for-kokkos - {app_name}</title>
  <script>{plotly_js}</script>
  <style>
    body {{
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
      margin: 0;
      padding: 24px;
      background: #0f172a;
      color: #f8fafc;
    }}
    .header {{
      margin-bottom: 24px;
      border-bottom: 1px solid #334155;
      padding-bottom: 16px;
    }}
    h1 {{ margin: 0 0 8px 0; font-size: 24px; color: #38bdf8; }}
    .meta {{ color: #94a3b8; font-size: 14px; }}
    .card-grid {{
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
      gap: 16px;
      margin-bottom: 24px;
    }}
    .card {{
      background: #1e293b;
      padding: 16px;
      border-radius: 8px;
      border: 1px solid #334155;
    }}
    .card-title {{ font-size: 12px; color: #94a3b8; text-transform: uppercase; letter-spacing: 0.05em; }}
    .card-val {{ font-size: 24px; font-weight: bold; margin-top: 8px; color: #f1f5f9; }}
    .chart-box {{
      background: #1e293b;
      padding: 16px;
      border-radius: 8px;
      border: 1px solid #334155;
      margin-bottom: 24px;
    }}
  </style>
</head>
<body>
  <div class="header">
    <h1>energy-dashboard-for-kokkos report</h1>
    <div class="meta">Application: <strong>{app_name}</strong> | Host: <strong>{hostname}</strong></div>
    {note_html}
  </div>

  <div class="card-grid">
    <div class="card">
      <div class="card-title">Total Energy</div>
      <div class="card-val">{total_energy:.2} J</div>
    </div>
    <div class="card">
      <div class="card-title">Duration</div>
      <div class="card-val">{duration:.3} s</div>
    </div>
    <div class="card">
      <div class="card-title">Average Power</div>
      <div class="card-val">{avg_power:.1} W</div>
    </div>
    <div class="card">
      <div class="card-title">Analyzed Blocks</div>
      <div class="card-val">{num_regions}</div>
    </div>
  </div>

  <div class="chart-box">
    <div id="timelinePlot" style="height: 420px;"></div>
  </div>

  <div class="chart-box">
    <div id="barPlot" style="height: 380px;"></div>
  </div>

  <script>
    const timelineTraces = {timeline_traces_json};

    const timelineLayout = {{
      title: {{ text: 'Power Telemetry Timeline', font: {{ color: '#f8fafc' }} }},
      paper_bgcolor: 'transparent',
      plot_bgcolor: 'transparent',
      xaxis: {{ title: 'Time (s)', color: '#94a3b8', gridcolor: '#334155' }},
      yaxis: {{ title: 'Power (W)', color: '#94a3b8', gridcolor: '#334155' }},
      legend: {{ font: {{ color: '#94a3b8' }} }},
      margin: {{ t: 40, r: 20, l: 60, b: 40 }}
    }};

    Plotly.newPlot('timelinePlot', timelineTraces, timelineLayout, {{ responsive: true }});

    const barTrace = {{
      x: {region_names_json},
      y: {region_energies_json},
      type: 'bar',
      marker: {{ color: '#818cf8' }}
    }};

    const barLayout = {{
      title: {{ text: 'Exclusive Energy by Block (Joules)', font: {{ color: '#f8fafc' }} }},
      paper_bgcolor: 'transparent',
      plot_bgcolor: 'transparent',
      xaxis: {{ color: '#94a3b8', gridcolor: '#334155' }},
      yaxis: {{ title: 'Joules (J)', color: '#94a3b8', gridcolor: '#334155' }},
      margin: {{ t: 40, r: 20, l: 60, b: 80 }}
    }};

    Plotly.newPlot('barPlot', [barTrace], barLayout, {{ responsive: true }});
  </script>
</body>
</html>
"#,
        plotly_js = PLOTLY_JS,
        app_name = app_name,
        hostname = hostname,
        total_energy = analysis.total_trace_energy_joules,
        duration = analysis.total_trace_duration_sec,
        avg_power = analysis.avg_trace_power_watts,
        num_regions = analysis.regions.len(),
        timeline_traces_json = timeline_traces_json,
        note_html = note_html,
        region_names_json = region_names_json,
        region_energies_json = region_energies_json
    );

    file.write_all(html_content.as_bytes())?;
    file.flush()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::analyze_trace;
    use crate::model::{DeviceDomain, Event, Metadata, PowerSample, RegionCategory};

    #[test]
    fn test_untrusted_names_are_escaped() {
        let metadata = Metadata {
            spec_version: "1.0".to_string(),
            app_name: Some("<b>app</b>".to_string()),
            hostname: None,
            kokkos_backend: None,
            start_epoch_ns: None,
        };
        let event = Event {
            id: 1,
            parent_id: 0,
            name: "</script><b>kernel".to_string(),
            category: RegionCategory::ParallelFor,
            start_ns: 0,
            end_ns: 10,
        };
        let sample = PowerSample {
            timestamp_ns: 0,
            domain: DeviceDomain::Gpu,
            device_id: 0,
            power_watts: 100.0,
            energy_joules: None,
        };
        let trace = Trace::new(Some(metadata), vec![event], vec![sample]);
        let file = tempfile::NamedTempFile::new().unwrap();
        export_html_report(&trace, &analyze_trace(&trace), file.path()).unwrap();

        let html = std::fs::read_to_string(file.path()).unwrap();
        assert!(html.contains("&lt;b&gt;app&lt;/b&gt;"));
        assert!(!html.contains("<b>app"));
        // Only the Plotly bundle and the chart script close a script element
        assert_eq!(html.matches("</script>").count(), 2);
    }
}
