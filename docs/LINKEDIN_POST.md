# LinkedIn Drafts

## Draft 1

I opened an early public Rust tool called `service-impact`.

It is for multi-service repos where CI tends to run too much because impact scope is fuzzy.

What it does:

- reads service manifests
- takes changed paths
- returns impacted services
- explains why they were selected
- suggests which verification hooks are worth running

The goal is simple:

run less verification without guessing

It is still early.
The public repo currently includes:

- standalone Rust library + CLI
- manifest validation
- strict / conservative analysis modes
- replay benchmark flow
- real-history replay export tooling

I am deliberately not calling it production-proven yet.
The checked-in benchmark is still a sample corpus, and the next step is replaying against larger real project history.

Repo:

https://github.com/tac0de/service-impact

## Draft 2

New public Rust project: `service-impact`

Problem:

- multi-service repos often over-run CI
- path-glob rules get brittle
- impact scope becomes hard to explain

Approach:

- define service relationships in a manifest
- map changed files to provided capabilities
- compute impacted services and verification hooks

What I like about this shape:

- standalone library / CLI
- explicit reasons in output
- validator for registry quality
- replayable evaluation path

Still early:

- current benchmark is a sample corpus
- real-history export is in place
- larger replay validation is next

https://github.com/tac0de/service-impact

## Short Version

Built a small Rust tool for multi-service repos: `service-impact`

It turns service manifests + changed files into:

- impacted services
- reasons
- verification hooks

Goal: reduce over-verification in CI without guessing.

Still early, but the standalone CLI/library, validator, and replay tooling are all public now.

https://github.com/tac0de/service-impact
