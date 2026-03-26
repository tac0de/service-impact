# Replay Benchmark

## Purpose

The replay benchmark is the current A/B-style validation path for `service-impact`.

It compares:

- baseline verification scope
- predicted verification scope from `service-impact`

The goal is to show whether scoped verification is smaller while still not missing truly impacted services.

## Files

- replay corpus: [`../fixtures/replay/cases.json`](../fixtures/replay/cases.json)
- checked-in summary: [`../fixtures/replay/summary.json`](../fixtures/replay/summary.json)
- runner: [`../src/bin/replay-bench.rs`](../src/bin/replay-bench.rs)

## Run it

```bash
cargo run --bin replay-bench -- fixtures/sample/registry.json fixtures/replay/cases.json
```

Export a real-history seed from another repo:

```bash
cargo run --bin git-history-export -- /path/to/your/repo 20 > replay_seed.json
```

## Metrics

- `missed_impacted_services`: services that should have been included but were not
- `false_positive_services`: services that were included but were not actually needed
- `median_scope_reduction_percent`: median reduction versus the baseline scope
- `median_ci_minutes_saved`: median estimated CI time saved
- `p50_analysis_latency_ms`: median analysis latency
- `p95_analysis_latency_ms`: tail analysis latency

## Using your own data

Replace the sample corpus with replay cases from your own history:

- source service
- changed paths
- baseline impacted services
- actual impacted services
- baseline minutes

This is the intended path for producing believable public numbers.

## Why the repo still ships a sample corpus

At the moment, the public repo does not yet include a 20-50 case production replay corpus.

That is intentional:

- a fake "real" corpus would be misleading
- a small sample corpus is better than invented claims
- `git-history-export` is the bridge toward a credible dataset
