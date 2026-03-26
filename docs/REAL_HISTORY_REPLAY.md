# Real History Replay

## Goal

Build a believable replay corpus from actual git history instead of synthetic examples.

## Step 1: Export history

```bash
cargo run --bin git-history-export -- /path/to/your/repo 20 > replay_seed.json
```

This produces:

- real commit hashes
- commit subjects
- changed paths per commit
- a replay-case template you can fill in

## Step 2: Fill labels

For each exported commit, decide:

- source service
- baseline impacted services
- actual impacted services
- baseline verification minutes
- baseline strategy

This is the manual truth-labeling step.

## Step 3: Convert to replay cases

Take the `replay_case_template` entries and build a replay file shaped like:

- [`../fixtures/replay/cases.json`](../fixtures/replay/cases.json)

## Step 4: Run both modes

```bash
cargo run --bin replay-bench -- registry.json replay_cases.json 2.75 strict
cargo run --bin replay-bench -- registry.json replay_cases.json 2.75 conservative
```

## Step 5: Decide whether the tool is trustworthy enough

Before using `service-impact` as a gate:

- missed impacted services should be at or near zero
- false positives should be understandable
- scope reduction should be materially useful
- validation should be part of CI
