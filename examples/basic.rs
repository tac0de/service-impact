use anyhow::Result;
use service_impact::{ImpactEngine, Registry};

fn main() -> Result<()> {
    let registry = Registry::load("fixtures/sample/registry.json")?;
    let engine = ImpactEngine::from_registry(registry)?;
    let result = engine.impacted_services("billing-api", &["src/invoice/mod.rs"])?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
