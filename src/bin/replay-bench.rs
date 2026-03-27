use anyhow::Result;
use clap::{Parser, ValueEnum};
use service_impact::{load_replay_cases, run_replay, AnalysisMode, ImpactEngine, Registry};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "replay-bench")]
#[command(about = "Replay historical impact cases against the current rules engine")]
struct Cli {
    #[arg(long, default_value = "fixtures/sample/registry.json")]
    registry: PathBuf,
    #[arg(long, default_value = "fixtures/replay/cases.json")]
    replay: PathBuf,
    #[arg(long, default_value_t = 2.75)]
    hook_cost_minutes: f64,
    #[arg(long, value_enum, default_value_t = ModeArg::Conservative)]
    mode: ModeArg,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum ModeArg {
    Strict,
    Conservative,
}

impl From<ModeArg> for AnalysisMode {
    fn from(value: ModeArg) -> Self {
        match value {
            ModeArg::Strict => AnalysisMode::Strict,
            ModeArg::Conservative => AnalysisMode::Conservative,
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let engine = ImpactEngine::from_registry(Registry::load(&cli.registry)?)?;
    let cases = load_replay_cases(&cli.replay)?;
    let summary = run_replay(&engine, &cases, cli.hook_cost_minutes, cli.mode.into())?;
    println!("{}", serde_json::to_string_pretty(&summary)?);
    Ok(())
}
