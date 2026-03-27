use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use service_impact::{
    validate_registry, AnalysisMode, ImpactEngine, ImpactReason, ImpactResult, ImpactedService,
    PlannedHook, Registry, ValidationReport, VerificationPlan,
};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Parser)]
#[command(name = "service-impact")]
#[command(about = "Rules-based impact calculation for multi-service repos")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Validate {
        #[arg(long, default_value = "fixtures/sample/registry.json")]
        registry: PathBuf,
        #[arg(long, default_value_t = false)]
        fail_on_warnings: bool,
        #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
        format: OutputFormat,
    },
    Impact {
        #[arg(long, default_value = "fixtures/sample/registry.json")]
        registry: PathBuf,
        #[arg(long)]
        service: String,
        #[arg(long, value_enum, default_value_t = ModeArg::Conservative)]
        mode: ModeArg,
        #[arg(long = "changed-path")]
        changed_paths: Vec<String>,
        #[arg(long)]
        changed_paths_file: Option<PathBuf>,
        #[arg(long)]
        git_diff_range: Option<String>,
        #[arg(long)]
        git_diff_from: Option<String>,
        #[arg(long)]
        git_diff_to: Option<String>,
        #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
        format: OutputFormat,
    },
    Plan {
        #[arg(long, default_value = "fixtures/sample/registry.json")]
        registry: PathBuf,
        #[arg(long)]
        service: String,
        #[arg(long, value_enum, default_value_t = ModeArg::Conservative)]
        mode: ModeArg,
        #[arg(long = "changed-path")]
        changed_paths: Vec<String>,
        #[arg(long)]
        changed_paths_file: Option<PathBuf>,
        #[arg(long)]
        git_diff_range: Option<String>,
        #[arg(long)]
        git_diff_from: Option<String>,
        #[arg(long)]
        git_diff_to: Option<String>,
        #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
        format: OutputFormat,
    },
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

#[derive(Clone, Copy, Debug, ValueEnum)]
enum OutputFormat {
    Human,
    Json,
}

#[derive(Debug)]
struct ChangedPathsInput {
    changed_paths: Vec<String>,
    changed_paths_file: Option<PathBuf>,
    git_diff_range: Option<String>,
    git_diff_from: Option<String>,
    git_diff_to: Option<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Validate {
            registry,
            fail_on_warnings,
            format,
        } => {
            let registry = Registry::load(&registry)?;
            let report = validate_registry(&registry);
            if !report.valid || (fail_on_warnings && !report.warnings.is_empty()) {
                print_validation(&report, format);
                anyhow::bail!("validation failed");
            }
            print_validation(&report, format);
        }
        Commands::Impact {
            registry,
            service,
            mode,
            changed_paths,
            changed_paths_file,
            git_diff_range,
            git_diff_from,
            git_diff_to,
            format,
        } => {
            let registry = Registry::load(&registry)?;
            let engine = ImpactEngine::from_registry(registry)?;
            let changed_paths = load_changed_paths(ChangedPathsInput {
                changed_paths,
                changed_paths_file,
                git_diff_range,
                git_diff_from,
                git_diff_to,
            })?;
            let result =
                engine.impacted_services_with_mode(&service, &changed_paths, mode.into())?;
            print_impact(&result, format);
        }
        Commands::Plan {
            registry,
            service,
            mode,
            changed_paths,
            changed_paths_file,
            git_diff_range,
            git_diff_from,
            git_diff_to,
            format,
        } => {
            let registry = Registry::load(&registry)?;
            let engine = ImpactEngine::from_registry(registry)?;
            let changed_paths = load_changed_paths(ChangedPathsInput {
                changed_paths,
                changed_paths_file,
                git_diff_range,
                git_diff_from,
                git_diff_to,
            })?;
            let result =
                engine.verification_plan_with_mode(&service, &changed_paths, mode.into())?;
            print_plan(&result, format);
        }
    }

    Ok(())
}

fn load_changed_paths(input: ChangedPathsInput) -> Result<Vec<String>> {
    if !input.changed_paths.is_empty() {
        return Ok(input.changed_paths);
    }
    if let Some(path) = input.changed_paths_file {
        return Ok(fs::read_to_string(path)?
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_string)
            .collect());
    }
    if let Some(range) = input.git_diff_range {
        return git_diff_paths(&[range.as_str()]);
    }
    if let (Some(from), Some(to)) = (input.git_diff_from, input.git_diff_to) {
        return git_diff_paths(&[from.as_str(), to.as_str()]);
    }
    Ok(Vec::new())
}

fn git_diff_paths(args: &[&str]) -> Result<Vec<String>> {
    let output = Command::new("git")
        .arg("diff")
        .arg("--name-only")
        .args(args)
        .output()?;
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

fn print_validation(report: &ValidationReport, format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(report).unwrap());
        }
        OutputFormat::Human => {
            println!("{}", report.summary);
            if !report.errors.is_empty() {
                println!();
                println!("Errors:");
                for issue in &report.errors {
                    println!("- [{}] {}: {}", issue.code, issue.service_id, issue.message);
                }
            }
            if !report.warnings.is_empty() {
                println!();
                println!("Warnings:");
                for issue in &report.warnings {
                    println!("- [{}] {}: {}", issue.code, issue.service_id, issue.message);
                }
            }
        }
    }
}

fn print_impact(result: &ImpactResult, format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(result).unwrap());
        }
        OutputFormat::Human => {
            println!("{}", result.summary.summary);
            println!("Source service: {}", result.service_id);
            if !result.changed_paths.is_empty() {
                println!("Changed paths:");
                for path in &result.changed_paths {
                    println!("- {}", path);
                }
            }
            if result.impacted_services.is_empty() {
                println!("No impacted services were found.");
            } else {
                println!("Impacted services:");
                for service in &result.impacted_services {
                    print_impacted_service(service);
                }
            }
        }
    }
}

fn print_plan(result: &VerificationPlan, format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(result).unwrap());
        }
        OutputFormat::Human => {
            println!("{}", result.summary);
            println!("Source service: {}", result.source_service);
            if !result.directly_impacted_services.is_empty() {
                println!("Impacted services:");
                for service in &result.directly_impacted_services {
                    println!("- {}", service);
                }
            }
            if result.hooks.is_empty() {
                println!("No hooks to run.");
            } else {
                println!("Planned hooks:");
                for hook in &result.hooks {
                    print_hook(hook);
                }
            }
        }
    }
}

fn print_impacted_service(service: &ImpactedService) {
    println!("- {}", service.service_id);
    for reason in &service.reasons {
        println!("  reason: {}", format_reason(reason));
    }
    for hook in &service.verification_hooks {
        println!("  hook: {} -> {}", hook.name, hook.command);
    }
}

fn print_hook(hook: &PlannedHook) {
    println!("- {} / {} -> {}", hook.service_id, hook.name, hook.command);
}

fn format_reason(reason: &ImpactReason) -> String {
    match (
        reason.reason_type.as_str(),
        reason.kind.as_deref(),
        reason.name.as_deref(),
    ) {
        ("consumes", Some(kind), Some(name)) => format!("consumes {} {}", kind, name),
        ("depends_on", _, _) => "declared depends_on edge".to_string(),
        _ => reason.reason_type.clone(),
    }
}
