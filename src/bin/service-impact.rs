use anyhow::{Context, Result};
use serde_json::Value;
use service_impact::{ImpactEngine, Registry};
use std::env;
use std::io::{self, Read};

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
    let engine = ImpactEngine::from_registry(registry)?;
    let service_id = payload
        .get("service_id")
        .and_then(Value::as_str)
        .context("service_id is required")?;
    let changed_paths = payload
        .get("changed_paths")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let output = match command.as_str() {
        "impact" => serde_json::to_value(engine.impacted_services(service_id, &changed_paths)?)?,
        "plan" => serde_json::to_value(engine.verification_plan(service_id, &changed_paths)?)?,
        other => anyhow::bail!("unsupported subcommand: {}", other),
    };
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
