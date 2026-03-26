use anyhow::{Context, Result};
use serde::Serialize;
use std::env;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Serialize)]
struct HistorySeed {
    repo_path: String,
    branch: String,
    commit_count: usize,
    commits: Vec<HistoryCommit>,
}

#[derive(Debug, Serialize)]
struct HistoryCommit {
    commit: String,
    subject: String,
    changed_paths: Vec<String>,
    replay_case_template: ReplayCaseTemplate,
}

#[derive(Debug, Serialize)]
struct ReplayCaseTemplate {
    id: String,
    source_service: String,
    changed_paths: Vec<String>,
    baseline_impacted_services: Vec<String>,
    actual_impacted_services: Vec<String>,
    baseline_minutes: f64,
    baseline_strategy: String,
    notes: String,
}

fn main() -> Result<()> {
    let args = env::args().collect::<Vec<_>>();
    let repo_path = args.get(1).map(String::as_str).unwrap_or(".");
    let max_count = args
        .get(2)
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(20);
    let branch = current_branch(repo_path)?;
    let commit_lines = git(
        repo_path,
        &[
            "log",
            "--format=%H\t%s",
            &format!("--max-count={}", max_count),
        ],
    )?;
    let mut commits = Vec::new();

    for line in commit_lines.lines() {
        let Some((commit, subject)) = line.split_once('\t') else {
            continue;
        };
        let changed_paths = git(
            repo_path,
            &["show", "--pretty=format:", "--name-only", commit],
        )?
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
        commits.push(HistoryCommit {
            commit: commit.to_string(),
            subject: subject.to_string(),
            replay_case_template: ReplayCaseTemplate {
                id: short_commit(commit),
                source_service: "fill-me".to_string(),
                changed_paths: changed_paths.clone(),
                baseline_impacted_services: Vec::new(),
                actual_impacted_services: Vec::new(),
                baseline_minutes: 0.0,
                baseline_strategy: "fill-me".to_string(),
                notes: format!("Fill labels for commit: {}", subject),
            },
            changed_paths,
        });
    }

    let seed = HistorySeed {
        repo_path: Path::new(repo_path).canonicalize()?.display().to_string(),
        branch,
        commit_count: commits.len(),
        commits,
    };

    println!("{}", serde_json::to_string_pretty(&seed)?);
    Ok(())
}

fn current_branch(repo_path: &str) -> Result<String> {
    Ok(git(repo_path, &["branch", "--show-current"])?
        .trim()
        .to_string())
}

fn git(repo_path: &str, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo_path)
        .output()
        .with_context(|| format!("failed to run git {}", args.join(" ")))?;
    if !output.status.success() {
        anyhow::bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8(output.stdout)?)
}

fn short_commit(commit: &str) -> String {
    commit.chars().take(8).collect()
}
