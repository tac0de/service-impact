# service-impact

> Compute which services and checks are actually affected by a code change.

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

## 1-Minute Quickstart

```bash
cargo add service-impact
```

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
