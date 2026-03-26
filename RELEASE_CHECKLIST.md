# Release Checklist

1. Run `cargo fmt`
2. Run `cargo test`
3. Run `cargo run --bin replay-bench -- fixtures/sample/registry.json fixtures/replay/cases.json`
4. Update benchmark numbers in `README.md` if they changed
5. Verify `Cargo.toml` version and repository metadata
6. Tag and publish release
