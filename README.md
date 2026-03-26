# service-impact

> For multi-service repos that over-run CI because impact scope is fuzzy.

[![CI](https://github.com/tac0de/service-impact/actions/workflows/ci.yml/badge.svg)](https://github.com/tac0de/service-impact/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

`service-impact` is a standalone Rust library and CLI that turns service manifests plus changed paths into:

- impacted services
- verification hooks to run
- a smaller verification plan

Instead of:

```text
full test/build across every service
```

You get:

```text
changed files -> impacted services -> required checks only
```

Good fit:

- platform teams
- multi-service repos
- CI pipelines that run too much by default
- teams replacing brittle path-glob rules

Not trying to be:

- a CI orchestrator
- a build system
- a runtime dependency discovery engine

## Try It On Your Repo In 5 Minutes

1. Clone the repo and run the sample validator.

```bash
git clone https://github.com/tac0de/service-impact.git
cd service-impact
cargo test
echo '{"registry_path":"fixtures/sample/registry.json"}' | cargo run --bin service-impact -- validate
```

2. Point it at your own registry and changed paths.

```bash
git diff --name-only origin/main...HEAD > changed_paths.txt
echo '{
  "registry_path": "registry.json",
  "service_id": "api",
  "changed_paths_file": "changed_paths.txt",
  "mode": "conservative"
}' | cargo run --bin service-impact -- impact
```

3. If you want CI-style validation, fail on warnings too.

```bash
echo '{
  "registry_path": "registry.json",
  "fail_on_warnings": true
}' | cargo run --bin service-impact -- validate
```

More setup help:
- see the sections below in this README

## What This Actually Helps With

If you have multiple services, packages, or deployable units, you usually hit one of these problems:

- every change triggers too much CI
- impact scope lives in tribal knowledge
- path globs are too rough
- teams know dependencies exist, but cannot query them cleanly

`service-impact` helps you answer:

- "Which services are affected by this change?"
- "Which checks should run now?"
- "What is the smallest reasonable verification scope?"

In practice, it is useful for:

- monorepos with multiple apps or services
- polyrepos with a central manifest of linked services
- platform teams trying to reduce CI waste
- internal developer portals that need impact previews
- build/release tooling that wants deterministic verification plans

## Why It Matters

Teams often over-verify because their service boundaries live in tribal knowledge, CI YAML, or path globs.

`service-impact` makes those boundaries explicit and queryable.

It is a good fit for:

- multi-service repos
- linked workspaces
- platform teams maintaining service manifests
- CI pipelines trying to cut unnecessary verification without guessing

## What Makes It Different

| Capability | Path globs / ad hoc CI | `service-impact` |
| --- | --- | --- |
| Service dependency graph | Partial | Yes |
| Manifest-driven capabilities | No | Yes |
| Changed-file impact analysis | Rough | Yes |
| Verification hook planning | No | Yes |
| Replayable benchmark output | Rare | Yes |

## Install

Clone the repo:

```bash
git clone https://github.com/tac0de/service-impact.git
cd service-impact
```

Run the tests:

```bash
cargo test
```

Run the CLI without installing globally:

```bash
cargo run --bin service-impact -- impact
```

If you want the CLI binary on your machine:

```bash
cargo install --path .
```

If you want to use it as a library before crates.io publishing, use a git dependency:

```toml
[dependencies]
service-impact = { git = "https://github.com/tac0de/service-impact" }
```

## Reliability

`service-impact` is designed to become trustworthy, but it should not be oversold today.

What increases trust:

- manifests are explicit and reviewable
- outputs include impact reasons
- replay benchmarks are reproducible
- registry validation can catch broken metadata before analysis

What still limits trust:

- hidden runtime dependencies are not inferred automatically
- stale manifests create stale results
- current checked-in benchmark is still a sample corpus, not a large production replay set

The intended reliability path is:

1. validate manifests
2. replay against real project history
3. keep missed impacted services at or near zero
4. tune false positives with analysis mode

Current public evidence level:

- checked-in benchmark is still a sample corpus
- real-history replay export tooling is included
- the repo does not yet claim a large production replay set

## Quickstart

```rust
use anyhow::Result;
use service_impact::{ImpactEngine, Registry};

fn main() -> Result<()> {
    let registry = Registry::load("fixtures/sample/registry.json")?;
    let engine = ImpactEngine::from_registry(registry)?;
    let result = engine.impacted_services("billing-api", &["src/invoice/mod.rs"])?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
```

Run the example:

```bash
cargo run --example basic
```

Validate the sample registry:

```bash
echo '{"registry_path":"fixtures/sample/registry.json"}' | cargo run --bin service-impact -- validate
```

Fail CI when warnings should be treated as errors:

```bash
echo '{"registry_path":"fixtures/sample/registry.json","fail_on_warnings":true}' | cargo run --bin service-impact -- validate
```

## How It Works

You describe your system in a manifest:

- each service has a `service_id`
- a service can `provide` capabilities
- another service can `consume` those capabilities
- a service can declare `depends_on`
- each service can define `verification_hooks`

Then `service-impact` takes:

- a source service
- changed file paths
- the manifest registry

and computes:

- active provides touched by the change
- impacted downstream services
- why each service is impacted
- which verification hooks are worth running

You can choose an analysis mode:

- `strict`: only capability-consume matches
- `conservative`: capability-consume matches plus declared `depends_on`

`strict` reduces false positives.
`conservative` is safer when your capability metadata is still incomplete.

## CLI

Impact query:

```bash
echo '{
  "registry_path": "fixtures/sample/registry.json",
  "service_id": "billing-api",
  "changed_paths": ["src/http/router.rs"],
  "mode": "conservative"
}' | cargo run --bin service-impact -- impact
```

Verification plan:

```bash
echo '{
  "registry_path": "fixtures/sample/registry.json",
  "service_id": "billing-api",
  "changed_paths": ["src/events/publisher.rs"],
  "mode": "strict"
}' | cargo run --bin service-impact -- plan
```

Use changed paths from a file:

```bash
echo "src/http/router.rs" > changed_paths.txt
echo '{
  "registry_path": "fixtures/sample/registry.json",
  "service_id": "billing-api",
  "changed_paths_file": "changed_paths.txt"
}' | cargo run --bin service-impact -- impact
```

Use changed paths from git diff:

```bash
echo '{
  "registry_path": "fixtures/sample/registry.json",
  "service_id": "billing-api",
  "git_diff_range": "HEAD~1..HEAD"
}' | cargo run --bin service-impact -- impact
```

Example output fields:

- `active_provides`: capabilities touched by the changed paths
- `impacted_services`: downstream services that should be reconsidered
- `reasons`: why each service was selected
- `verification_hooks`: checks attached to those impacted services
- `summary`: quick human-readable result summary

## Library Usage

The main entry points are:

- `Registry::load(...)`
- `ImpactEngine::from_registry(...)`
- `ImpactEngine::impacted_services(...)`
- `ImpactEngine::verification_plan(...)`

Minimal example:

```rust
use anyhow::Result;
use service_impact::{ImpactEngine, Registry};

fn main() -> Result<()> {
    let registry = Registry::load("fixtures/sample/registry.json")?;
    let engine = ImpactEngine::from_registry(registry)?;

    let impact = engine.impacted_services(
        "billing-api",
        &["src/http/router.rs", "src/invoice/mod.rs"],
    )?;

    let plan = engine.verification_plan(
        "billing-api",
        &["src/http/router.rs", "src/invoice/mod.rs"],
    )?;

    println!("impacted services: {:?}", impact.impacted_services);
    println!("planned hooks: {:?}", plan.hooks);
    Ok(())
}
```

## Example Output

```json
{
  "service_id": "billing-api",
  "changed_paths": ["src/invoice/mod.rs"],
  "active_provides": [
    {
      "kind": "http",
      "name": "invoice",
      "paths": ["src/invoice", "src/http"]
    }
  ],
  "impacted_services": [
    {
      "service_id": "billing-web",
      "reasons": [
        {
          "type": "depends_on",
          "via": "billing-api"
        },
        {
          "type": "consumes",
          "kind": "http",
          "name": "invoice",
          "via": "billing-api"
        }
      ],
      "verification_hooks": [
        {
          "name": "web-e2e",
          "trigger": "impact",
          "command": "pnpm test:e2e"
        }
      ]
    }
  ]
}
```

## Replay Benchmark

This is the current A/B-style evidence path in the repo.

Baseline A:

- broad verification across many or all services
- rough path heuristics or default full validation

Candidate B:

- manifest-driven impact analysis with `service-impact`

Current checked-in benchmark is still a sample corpus intended to show the workflow, not a final production claim.

Replay the sample corpus:

```bash
cargo run --bin replay-bench -- fixtures/sample/registry.json fixtures/replay/cases.json
```

Current sample output:

- `0` missed impacted services
- `1` false positive service across the sample corpus
- `50.0%` median verification scope reduction
- `5.5` median CI minutes saved
- sub-millisecond analysis latency on the sample corpus

This benchmark is intended to be replayed against real change history. The sample fixture only shows the shape of the output.
See [fixtures/replay/summary.json](fixtures/replay/summary.json) for the checked-in sample summary.

Where to look:

- replay cases: [fixtures/replay/cases.json](fixtures/replay/cases.json)
- checked-in benchmark summary: [fixtures/replay/summary.json](fixtures/replay/summary.json)
- benchmark runner: [src/bin/replay-bench.rs](src/bin/replay-bench.rs)

What the A/B comparison is measuring:

- missed impacted services
- false positives
- median scope reduction
- median CI minutes saved
- analysis latency

Recommended next step for a real team:

1. export changed paths from 20-50 historical PRs
2. define your current baseline verification scope
3. mark actual impacted services
4. replay both `strict` and `conservative` mode
5. compare misses before promoting the tool into CI gating

Export a replay seed from actual git history:

```bash
cargo run --bin git-history-export -- /path/to/your/repo 20 > replay_seed.json
```

This exports real commits and changed paths, then leaves the impact labels for you to fill in.

If you want a real benchmark for your own repo, replace the sample replay corpus with:

- actual changed paths from past PRs or commits
- your current baseline verification scope
- the services that were actually impacted
- your real per-hook or per-service verification cost

## Use Cases

### 1. CI scope reduction

Instead of always running everything:

- read changed files from git or CI
- ask `service-impact` which services are affected
- run only the hooks attached to those services

### 1a. GitHub Actions style flow

```yaml
- name: Collect changed paths
  run: git diff --name-only origin/main...HEAD > changed_paths.txt

- name: Compute impact
  run: |
    echo '{
      "registry_path": "registry.json",
      "service_id": "billing-api",
      "changed_paths_file": "changed_paths.txt",
      "mode": "conservative"
    }' | cargo run --bin service-impact -- impact
```

### 2. PR impact preview

Show reviewers:

- which services a change may affect
- why they are included
- which checks should be watched closely

### 3. Platform architecture visibility

Use the manifest as a queryable map of:

- service boundaries
- provided capabilities
- downstream consumers
- verification ownership

### 4. Migration away from path-glob CI

If your current CI logic is mostly hand-written path matching, `service-impact` can become the typed layer that explains those relationships explicitly.

## Real Repo Quickstart

You do not need `trace-hub`.

Minimal steps for an existing repo:

1. create a registry file like [fixtures/sample/registry.json](fixtures/sample/registry.json)
2. define services, capabilities, and verification hooks
3. run `validate` on the registry
4. feed changed paths from git diff or CI
5. use the impact output to decide which checks to run

If you want to build a real replay corpus from your history:

1. export commit history with `git-history-export`
2. convert those commits into replay cases
3. fill actual impacted services from your own review or incident history
4. run `replay-bench`

Minimal registry sketch:

```json
{
  "services": [
    {
      "service_id": "api",
      "provides": [
        { "kind": "http", "name": "orders", "paths": ["services/api/src/orders"] }
      ],
      "consumes": [],
      "depends_on": [],
      "verification_hooks": [
        { "name": "api-test", "trigger": "impact", "command": "cargo test -p api" }
      ]
    },
    {
      "service_id": "worker",
      "provides": [],
      "consumes": [
        { "service_id": "api", "kind": "http", "name": "orders" }
      ],
      "depends_on": ["api"],
      "verification_hooks": [
        { "name": "worker-test", "trigger": "impact", "command": "cargo test -p worker" }
      ]
    }
  ]
}
```

## Manifest Format

```json
{
  "services": [
    {
      "service_id": "billing-api",
      "provides": [
        {
          "kind": "http",
          "name": "invoice",
          "paths": ["src/invoice"]
        }
      ],
      "consumes": [],
      "depends_on": [],
      "verification_hooks": [
        {
          "name": "api-unit",
          "trigger": "change",
          "command": "cargo test -p billing-api"
        }
      ]
    }
  ]
}
```

See the sample registry at [fixtures/sample/registry.json](fixtures/sample/registry.json).

## Correctness Boundary

`service-impact` is only as accurate as the manifests you maintain.

It does not:

- infer runtime dependencies automatically
- inspect source code for hidden edges
- mutate CI configuration
- decide release policy for you
- guarantee correctness without validated manifests and replay testing

## Good Fit

- explicit service manifests already exist
- teams want a deterministic answer for impact scope
- verification hooks can be attached to services

## Status

`0.1.0` is an initial public cut with:

- library API
- CLI
- replay benchmark harness
- sample fixtures and examples
- registry validation
- strict and conservative analysis modes
- git diff and changed-path file input support

## License

MIT
