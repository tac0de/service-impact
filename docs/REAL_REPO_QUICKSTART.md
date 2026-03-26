# Real Repo Quickstart

## Goal

Use `service-impact` in an existing repository without any `trace-hub` dependency.

## Step 1: Create a registry

Create a JSON file that declares:

- services
- provided capabilities
- consumed capabilities
- direct dependencies
- verification hooks

See the sample registry:

- [`../fixtures/sample/registry.json`](../fixtures/sample/registry.json)

## Step 2: Validate the registry

```bash
echo '{"registry_path":"registry.json"}' | cargo run --bin service-impact -- validate
```

This catches obvious metadata issues such as:

- duplicate service ids
- unknown dependencies
- consume targets that do not exist
- empty verification hook commands

## Step 3: Feed changed paths

From a file:

```bash
git diff --name-only origin/main...HEAD > changed_paths.txt
```

Then:

```bash
echo '{
  "registry_path": "registry.json",
  "service_id": "api",
  "changed_paths_file": "changed_paths.txt",
  "mode": "conservative"
}' | cargo run --bin service-impact -- impact
```

## Step 4: Use the result

The result tells you:

- which services are impacted
- why they were included
- which hooks are attached

That output can drive:

- CI scope reduction
- PR impact comments
- architecture dashboards

## Step 5: Build trust before gating CI

Before using it as a hard gate:

1. collect historical PRs or commits
2. build a replay corpus
3. compare `strict` and `conservative` modes
4. measure misses and false positives

Use:

```bash
cargo run --bin replay-bench -- registry.json replay_cases.json 2.75 conservative
```
