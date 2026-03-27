use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisMode {
    Strict,
    Conservative,
}

impl Default for AnalysisMode {
    fn default() -> Self {
        Self::Conservative
    }
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
pub struct ImpactSummary {
    pub mode: AnalysisMode,
    pub impacted_service_count: usize,
    pub verification_hook_count: usize,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImpactResult {
    pub service_id: String,
    pub changed_paths: Vec<String>,
    pub active_provides: Vec<Provide>,
    pub impacted_services: Vec<ImpactedService>,
    pub summary: ImpactSummary,
}

#[derive(Debug, Clone, Serialize)]
pub struct VerificationPlan {
    pub mode: AnalysisMode,
    pub source_service: String,
    pub changed_paths: Vec<String>,
    pub directly_impacted_services: Vec<String>,
    pub hooks: Vec<PlannedHook>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlannedHook {
    pub service_id: String,
    pub name: String,
    pub trigger: String,
    pub command: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ValidationReport {
    pub valid: bool,
    pub errors: Vec<ValidationIssue>,
    pub warnings: Vec<ValidationIssue>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ValidationIssue {
    pub service_id: String,
    pub code: String,
    pub message: String,
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
    #[serde(default)]
    pub baseline_strategy: String,
    #[serde(default)]
    pub notes: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReplayCaseResult {
    pub id: String,
    pub mode: AnalysisMode,
    pub baseline_impacted_services: Vec<String>,
    pub predicted_impacted_services: Vec<String>,
    pub actual_impacted_services: Vec<String>,
    pub missed_impacted_services: Vec<String>,
    pub false_positive_services: Vec<String>,
    pub baseline_minutes: f64,
    pub predicted_minutes: f64,
    pub minutes_saved: f64,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReplaySummary {
    pub mode: AnalysisMode,
    pub corpus_size: usize,
    pub missed_impacted_services: usize,
    pub false_positive_services: usize,
    pub median_scope_reduction_percent: f64,
    pub median_ci_minutes_saved: f64,
    pub p50_analysis_latency_ms: f64,
    pub p95_analysis_latency_ms: f64,
    pub summary: String,
    pub cases: Vec<ReplayCaseResult>,
}
