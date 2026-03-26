use anyhow::{Context, Result};
use serde_json::Value;
use service_impact::{validate_registry, AnalysisMode, ImpactEngine, Registry};
use std::env;
use std::fs;
use std::io::{self, Read};
use std::process::Command;

fn main() -> Result<()> {
    let command = env::args().nth(1).context("missing subcommand")?;
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let payload: Value = serde_json::from_str(&input).context("stdin must be valid json")?;
    let registry_path = payload
        .get("registry_path")
        .and_then(Value::as_str)
        .unwrap_or("fixtures/sample/registry.json");
    let registry = Registry::load(registry_path)?;
    let mode = parse_mode(payload.get("mode").and_then(Value::as_str))?;
    let output = match command.as_str() {
        "validate" => serde_json::to_value(validate_registry(&registry))?,
        "impact" | "plan" => {
            let engine = ImpactEngine::from_registry(registry)?;
            let service_id = payload
                .get("service_id")
                .and_then(Value::as_str)
                .context("service_id is required")?;
            let changed_paths = load_changed_paths(&payload)?;
            match command.as_str() {
                "impact" => serde_json::to_value(engine.impacted_services_with_mode(
                    service_id,
                    &changed_paths,
                    mode,
                )?)?,
                "plan" => serde_json::to_value(engine.verification_plan_with_mode(
                    service_id,
                    &changed_paths,
                    mode,
                )?)?,
                _ => unreachable!(),
            }
        }
        other => anyhow::bail!("unsupported subcommand: {}", other),
    };
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn parse_mode(value: Option<&str>) -> Result<AnalysisMode> {
    match value.unwrap_or("conservative") {
        "conservative" => Ok(AnalysisMode::Conservative),
        "strict" => Ok(AnalysisMode::Strict),
        other => anyhow::bail!("unsupported mode: {}", other),
    }
}

fn load_changed_paths(payload: &Value) -> Result<Vec<String>> {
    if let Some(items) = payload.get("changed_paths").and_then(Value::as_array) {
        return Ok(items
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect());
    }
    if let Some(path) = payload.get("changed_paths_file").and_then(Value::as_str) {
        return Ok(fs::read_to_string(path)?
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_string)
            .collect());
    }
    if let Some(range) = payload.get("git_diff_range").and_then(Value::as_str) {
        return git_diff_paths(&[range]);
    }
    let from = payload.get("git_diff_from").and_then(Value::as_str);
    let to = payload.get("git_diff_to").and_then(Value::as_str);
    if let (Some(from), Some(to)) = (from, to) {
        return git_diff_paths(&[from, to]);
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
