# service-impact

> Compute which services and checks are actually affected by a code change.

[![CI](https://github.com/tac0de/service-impact/actions/workflows/ci.yml/badge.svg)](https://github.com/tac0de/service-impact/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

`service-impact` turns service manifests plus changed paths into:

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

`service-impact` is on GitHub now. If you want to try it immediately:

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

## CLI

Impact query:

```bash
echo '{
  "registry_path": "fixtures/sample/registry.json",
  "service_id": "billing-api",
  "changed_paths": ["src/http/router.rs"]
}' | cargo run --bin service-impact -- impact
```

Verification plan:

```bash
echo '{
  "registry_path": "fixtures/sample/registry.json",
  "service_id": "billing-api",
  "changed_paths": ["src/events/publisher.rs"]
}' | cargo run --bin service-impact -- plan
```

Example output fields:

- `active_provides`: capabilities touched by the changed paths
- `impacted_services`: downstream services that should be reconsidered
- `reasons`: why each service was selected
- `verification_hooks`: checks attached to those impacted services

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

## Good Fit

- explicit service manifests already exist
- teams want a deterministic answer for impact scope
- verification hooks can be attached to services

## Not Trying To Be

- a full CI orchestrator
- a build system
- a runtime service catalog
- a policy engine

## Status

`0.1.0` is an initial public cut with:

- library API
- CLI
- replay benchmark harness
- sample fixtures and examples

## License

MIT
