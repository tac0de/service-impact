use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Registry {
    pub services: Vec<ServiceManifest>,
}

impl Registry {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let raw = fs::read_to_string(path.as_ref())
            .with_context(|| format!("failed to read {}", path.as_ref().display()))?;
        Ok(serde_json::from_str(&raw)?)
    }

    pub fn get(&self, service_id: &str) -> Option<&ServiceManifest> {
        self.services
            .iter()
            .find(|service| service.service_id == service_id)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServiceManifest {
    pub service_id: String,
    #[serde(default)]
    pub provides: Vec<Provide>,
    #[serde(default)]
    pub consumes: Vec<Consume>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub verification_hooks: Vec<VerificationHook>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Provide {
    pub kind: String,
    pub name: String,
    #[serde(default)]
    pub paths: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Consume {
    #[serde(default)]
    pub service_id: Option<String>,
    pub kind: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VerificationHook {
    pub name: String,
    pub trigger: String,
    pub command: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImpactReason {
    #[serde(rename = "type")]
    pub reason_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub via: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImpactedService {
    pub service_id: String,
    pub reasons: Vec<ImpactReason>,
    pub verification_hooks: Vec<VerificationHook>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImpactResult {
    pub service_id: String,
    pub changed_paths: Vec<String>,
    pub active_provides: Vec<Provide>,
    pub impacted_services: Vec<ImpactedService>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VerificationPlan {
    pub source_service: String,
    pub changed_paths: Vec<String>,
    pub directly_impacted_services: Vec<String>,
    pub hooks: Vec<PlannedHook>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlannedHook {
    pub service_id: String,
    pub name: String,
    pub trigger: String,
    pub command: String,
}

#[derive(Debug, Clone)]
pub struct ImpactEngine {
    services: BTreeMap<String, ServiceManifest>,
}

impl ImpactEngine {
    pub fn from_registry(registry: Registry) -> Result<Self> {
        let mut services = BTreeMap::new();
        for service in registry.services {
            if services
                .insert(service.service_id.clone(), service)
                .is_some()
            {
                return Err(anyhow!("duplicate service_id"));
            }
        }
        Ok(Self { services })
    }

    pub fn impacted_services(
        &self,
        service_id: &str,
        changed_paths: &[impl AsRef<str>],
    ) -> Result<ImpactResult> {
        let source = self
            .services
            .get(service_id)
            .ok_or_else(|| anyhow!("unknown service_id: {}", service_id))?;
        let changed_paths = changed_paths
            .iter()
            .map(|path| path.as_ref().to_string())
            .collect::<Vec<_>>();
        let active_provides = source
            .provides
            .iter()
            .filter(|provided| matches_paths(&provided.paths, &changed_paths))
            .cloned()
            .collect::<Vec<_>>();

        let mut impacted_services = Vec::new();
        for (other_id, other) in &self.services {
            if other_id == service_id {
                continue;
            }
            let mut reasons = Vec::new();
            if other.depends_on.iter().any(|item| item == service_id) {
                reasons.push(ImpactReason {
                    reason_type: "depends_on".to_string(),
                    kind: None,
                    name: None,
                    via: Some(service_id.to_string()),
                });
            }
            for consumed in &other.consumes {
                if let Some(target) = &consumed.service_id {
                    if target != service_id {
                        continue;
                    }
                }
                for provided in &active_provides {
                    if consumed.kind == provided.kind && consumed.name == provided.name {
                        reasons.push(ImpactReason {
                            reason_type: "consumes".to_string(),
                            kind: Some(consumed.kind.clone()),
                            name: Some(consumed.name.clone()),
                            via: Some(service_id.to_string()),
                        });
                    }
                }
            }
            if !reasons.is_empty() {
                impacted_services.push(ImpactedService {
                    service_id: other_id.clone(),
                    reasons,
                    verification_hooks: other.verification_hooks.clone(),
                });
            }
        }
        impacted_services.sort_by(|left, right| left.service_id.cmp(&right.service_id));
        Ok(ImpactResult {
            service_id: service_id.to_string(),
            changed_paths,
            active_provides,
            impacted_services,
        })
    }

    pub fn verification_plan(
        &self,
        service_id: &str,
        changed_paths: &[impl AsRef<str>],
    ) -> Result<VerificationPlan> {
        let impact = self.impacted_services(service_id, changed_paths)?;
        let hooks = impact
            .impacted_services
            .iter()
            .flat_map(|service| {
                service.verification_hooks.iter().map(|hook| PlannedHook {
                    service_id: service.service_id.clone(),
                    name: hook.name.clone(),
                    trigger: hook.trigger.clone(),
                    command: hook.command.clone(),
                })
            })
            .collect::<Vec<_>>();
        Ok(VerificationPlan {
            source_service: impact.service_id,
            changed_paths: impact.changed_paths,
            directly_impacted_services: impact
                .impacted_services
                .iter()
                .map(|service| service.service_id.clone())
                .collect(),
            hooks,
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReplayCase {
    pub id: String,
    pub source_service: String,
    pub changed_paths: Vec<String>,
    #[serde(default)]
    pub baseline_impacted_services: Vec<String>,
    #[serde(default)]
    pub actual_impacted_services: Vec<String>,
    #[serde(default)]
    pub baseline_minutes: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReplayCaseResult {
    pub id: String,
    pub baseline_impacted_services: Vec<String>,
    pub predicted_impacted_services: Vec<String>,
    pub actual_impacted_services: Vec<String>,
    pub missed_impacted_services: Vec<String>,
    pub false_positive_services: Vec<String>,
    pub baseline_minutes: f64,
    pub predicted_minutes: f64,
    pub minutes_saved: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReplaySummary {
    pub corpus_size: usize,
    pub missed_impacted_services: usize,
    pub false_positive_services: usize,
    pub median_scope_reduction_percent: f64,
    pub median_ci_minutes_saved: f64,
    pub p50_analysis_latency_ms: f64,
    pub p95_analysis_latency_ms: f64,
    pub cases: Vec<ReplayCaseResult>,
}

pub fn load_replay_cases(path: impl AsRef<Path>) -> Result<Vec<ReplayCase>> {
    let raw = fs::read_to_string(path.as_ref())
        .with_context(|| format!("failed to read {}", path.as_ref().display()))?;
    Ok(serde_json::from_str(&raw)?)
}

pub fn run_replay(
    engine: &ImpactEngine,
    cases: &[ReplayCase],
    hook_cost_minutes: f64,
) -> Result<ReplaySummary> {
    let mut latencies_ms = Vec::new();
    let mut results = Vec::new();

    for case in cases {
        let started = std::time::Instant::now();
        let prediction = engine.impacted_services(&case.source_service, &case.changed_paths)?;
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
            baseline_impacted_services: sorted_vec(baseline),
            predicted_impacted_services: sorted_vec(predicted.clone()),
            actual_impacted_services: sorted_vec(actual.clone()),
            missed_impacted_services: missed,
            false_positive_services: false_positives,
            baseline_minutes: case.baseline_minutes,
            predicted_minutes,
            minutes_saved: (case.baseline_minutes - predicted_minutes).max(0.0),
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
        cases: results,
    })
}

fn sorted_vec(values: BTreeSet<String>) -> Vec<String> {
    values.into_iter().collect()
}

fn matches_paths(prefixes: &[String], changed_paths: &[String]) -> bool {
    if prefixes.is_empty() || changed_paths.is_empty() {
        return true;
    }
    changed_paths.iter().any(|changed| {
        prefixes.iter().any(|prefix| {
            let normalized = prefix.trim_end_matches('/');
            changed == normalized || changed.starts_with(&format!("{}/", normalized))
        })
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    fn engine() -> ImpactEngine {
        let registry = Registry {
            services: vec![
                ServiceManifest {
                    service_id: "api".to_string(),
                    provides: vec![Provide {
                        kind: "http".to_string(),
                        name: "billing".to_string(),
                        paths: vec!["src/billing".to_string()],
                    }],
                    consumes: vec![],
                    depends_on: vec![],
                    verification_hooks: vec![],
                },
                ServiceManifest {
                    service_id: "worker".to_string(),
                    provides: vec![],
                    consumes: vec![Consume {
                        service_id: Some("api".to_string()),
                        kind: "http".to_string(),
                        name: "billing".to_string(),
                    }],
                    depends_on: vec!["api".to_string()],
                    verification_hooks: vec![VerificationHook {
                        name: "worker-smoke".to_string(),
                        trigger: "impact".to_string(),
                        command: "cargo test -p worker smoke".to_string(),
                    }],
                },
            ],
        };
        ImpactEngine::from_registry(registry).unwrap()
    }

    #[test]
    fn computes_impacted_services() {
        let result = engine()
            .impacted_services("api", &["src/billing/mod.rs"])
            .unwrap();
        assert_eq!(result.impacted_services.len(), 1);
        assert_eq!(result.impacted_services[0].service_id, "worker");
    }

    #[test]
    fn builds_verification_plan() {
        let plan = engine()
            .verification_plan("api", &["src/billing/mod.rs"])
            .unwrap();
        assert_eq!(plan.directly_impacted_services, vec!["worker"]);
        assert_eq!(plan.hooks.len(), 1);
    }
}
