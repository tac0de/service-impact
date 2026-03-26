use anyhow::Result;
use service_impact::{load_replay_cases, run_replay, ImpactEngine, Registry};
use std::env;

fn main() -> Result<()> {
    let args = env::args().collect::<Vec<_>>();
    let registry_path = args
        .get(1)
        .map(String::as_str)
        .unwrap_or("fixtures/sample/registry.json");
    let replay_path = args
        .get(2)
        .map(String::as_str)
        .unwrap_or("fixtures/replay/cases.json");
    let hook_cost_minutes = args
        .get(3)
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(2.75);
    let engine = ImpactEngine::from_registry(Registry::load(registry_path)?)?;
    let cases = load_replay_cases(replay_path)?;
    let summary = run_replay(&engine, &cases, hook_cost_minutes)?;
    println!("{}", serde_json::to_string_pretty(&summary)?);
    Ok(())
}
