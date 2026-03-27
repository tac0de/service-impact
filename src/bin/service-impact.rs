use anyhow::{Context, Result};
use serde::Deserialize;
use service_impact::{validate_registry, AnalysisMode, ImpactEngine, Registry};
use std::env;
use std::fs;
use std::io::{self, Read};
use std::process::Command;

#[derive(Debug, Deserialize)]
struct ValidateRequest {
    #[serde(default = "default_registry_path")]
    registry_path: String,
    #[serde(default)]
    fail_on_warnings: bool,
}

#[derive(Debug, Deserialize)]
struct ImpactRequest {
    #[serde(default = "default_registry_path")]
    registry_path: String,
    service_id: String,
    #[serde(default)]
    mode: AnalysisMode,
    #[serde(default)]
    changed_paths: Vec<String>,
    changed_paths_file: Option<String>,
    git_diff_range: Option<String>,
    git_diff_from: Option<String>,
    git_diff_to: Option<String>,
}

fn main() -> Result<()> {
    let command = env::args().nth(1).context("missing subcommand")?;
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let output = match command.as_str() {
        "validate" => {
            let request: ValidateRequest =
                serde_json::from_str(&input).context("stdin must be valid json")?;
            let registry = Registry::load(&request.registry_path)?;
            let report = validate_registry(&registry);
            if !report.valid || (request.fail_on_warnings && !report.warnings.is_empty()) {
                anyhow::bail!("{}", serde_json::to_string_pretty(&report)?);
            }
            serde_json::to_value(report)?
        }
        "impact" | "plan" => {
            let request: ImpactRequest =
                serde_json::from_str(&input).context("stdin must be valid json")?;
            let registry = Registry::load(&request.registry_path)?;
            let engine = ImpactEngine::from_registry(registry)?;
            let changed_paths = load_changed_paths(&request)?;
            match command.as_str() {
                "impact" => serde_json::to_value(engine.impacted_services_with_mode(
                    &request.service_id,
                    &changed_paths,
                    request.mode,
                )?)?,
                "plan" => serde_json::to_value(engine.verification_plan_with_mode(
                    &request.service_id,
                    &changed_paths,
                    request.mode,
                )?)?,
                _ => unreachable!(),
            }
        }
        other => anyhow::bail!("unsupported subcommand: {}", other),
    };
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn default_registry_path() -> String {
    "fixtures/sample/registry.json".to_string()
}

fn load_changed_paths(request: &ImpactRequest) -> Result<Vec<String>> {
    if !request.changed_paths.is_empty() {
        return Ok(request.changed_paths.clone());
    }
    if let Some(path) = &request.changed_paths_file {
        return Ok(fs::read_to_string(path)?
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_string)
            .collect());
    }
    if let Some(range) = &request.git_diff_range {
        return git_diff_paths(&[range.as_str()]);
    }
    if let (Some(from), Some(to)) = (&request.git_diff_from, &request.git_diff_to) {
        return git_diff_paths(&[from.as_str(), to.as_str()]);
    }
    Ok(Vec::new())
}

fn git_diff_paths(args: &[&str]) -> Result<Vec<String>> {
    let output = Command::new("git")
        .arg("diff")
        .arg("--name-only")
        .args(args)
        .output()
        .context("failed to run git diff --name-only")?;
    if !output.status.success() {
        anyhow::bail!(
            "git diff failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8(output.stdout)?
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect())
}
