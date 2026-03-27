use crate::engine::ImpactEngine;
use crate::model::{AnalysisMode, ReplayCase, ReplayCaseResult, ReplaySummary};
use anyhow::{Context, Result};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

pub fn load_replay_cases(path: impl AsRef<Path>) -> Result<Vec<ReplayCase>> {
    let raw = fs::read_to_string(path.as_ref())
        .with_context(|| format!("failed to read {}", path.as_ref().display()))?;
    Ok(serde_json::from_str(&raw)?)
}

pub fn run_replay(
    engine: &ImpactEngine,
    cases: &[ReplayCase],
    hook_cost_minutes: f64,
    mode: AnalysisMode,
) -> Result<ReplaySummary> {
    let mut latencies_ms = Vec::new();
    let mut results = Vec::new();

    for case in cases {
        let started = std::time::Instant::now();
        let prediction =
            engine.impacted_services_with_mode(&case.source_service, &case.changed_paths, mode)?;
        latencies_ms.push(started.elapsed().as_secs_f64() * 1000.0);

        let predicted = prediction
            .impacted_services
            .iter()
            .map(|service| service.service_id.clone())
            .collect::<BTreeSet<_>>();
        let actual = case
            .actual_impacted_services
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        let baseline = case
            .baseline_impacted_services
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();

        let missed = actual.difference(&predicted).cloned().collect::<Vec<_>>();
        let false_positives = predicted.difference(&actual).cloned().collect::<Vec<_>>();
        let predicted_minutes = (predicted.len() as f64) * hook_cost_minutes;
        results.push(ReplayCaseResult {
            id: case.id.clone(),
            mode,
            baseline_impacted_services: sorted_vec(baseline),
            predicted_impacted_services: sorted_vec(predicted.clone()),
            actual_impacted_services: sorted_vec(actual.clone()),
            missed_impacted_services: missed,
            false_positive_services: false_positives,
            baseline_minutes: case.baseline_minutes,
            predicted_minutes,
            minutes_saved: (case.baseline_minutes - predicted_minutes).max(0.0),
            notes: case.notes.clone(),
        });
    }

    let mut scope_reduction = results
        .iter()
        .map(|result| {
            if result.baseline_impacted_services.is_empty() {
                0.0
            } else {
                100.0
                    * (1.0
                        - (result.predicted_impacted_services.len() as f64
                            / result.baseline_impacted_services.len() as f64))
            }
        })
        .collect::<Vec<_>>();
    let mut minutes_saved = results
        .iter()
        .map(|result| result.minutes_saved)
        .collect::<Vec<_>>();
    latencies_ms.sort_by(f64::total_cmp);
    scope_reduction.sort_by(f64::total_cmp);
    minutes_saved.sort_by(f64::total_cmp);

    Ok(ReplaySummary {
        mode,
        corpus_size: results.len(),
        missed_impacted_services: results
            .iter()
            .map(|result| result.missed_impacted_services.len())
            .sum(),
        false_positive_services: results
            .iter()
            .map(|result| result.false_positive_services.len())
            .sum(),
        median_scope_reduction_percent: median(&scope_reduction),
        median_ci_minutes_saved: median(&minutes_saved),
        p50_analysis_latency_ms: median(&latencies_ms),
        p95_analysis_latency_ms: percentile(&latencies_ms, 0.95),
        summary: format!(
            "Replay finished in {:?} mode with {} missed service(s), {} false positive service(s), and {:.1}% median scope reduction",
            mode,
            results
                .iter()
                .map(|result| result.missed_impacted_services.len())
                .sum::<usize>(),
            results
                .iter()
                .map(|result| result.false_positive_services.len())
                .sum::<usize>(),
            median(&scope_reduction)
        ),
        cases: results,
    })
}

fn sorted_vec(values: BTreeSet<String>) -> Vec<String> {
    values.into_iter().collect()
}

fn median(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let middle = values.len() / 2;
    if values.len() % 2 == 0 {
        (values[middle - 1] + values[middle]) / 2.0
    } else {
        values[middle]
    }
}

fn percentile(values: &[f64], ratio: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let index = ((values.len() - 1) as f64 * ratio).round() as usize;
    values[index]
}
