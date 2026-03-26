# Contributing

## Development

```bash
cargo fmt
cargo test
cargo run --bin replay-bench -- fixtures/sample/registry.json fixtures/replay/cases.json
```

## Scope

Contributions should keep the crate focused on:

- manifest-driven service topology
- changed-file impact analysis
- verification hook planning
- replayable benchmark output

Please avoid broadening the crate into a general CI orchestrator or policy engine.
