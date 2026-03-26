# Use Cases

## When `service-impact` is a good fit

`service-impact` is for systems where service relationships already exist, but the relationship between a code change and required verification is still too fuzzy.

Typical cases:

- a monorepo with several deployable services
- multiple backend services sharing contracts or events
- frontend + API + worker stacks with downstream dependencies
- internal platform teams trying to cut CI cost without losing confidence

## Example workflow

1. A PR changes `billing-api/src/events/publisher.rs`.
2. CI collects changed paths.
3. `service-impact` reads the service registry.
4. It finds which provided capabilities were touched.
5. It maps those capabilities to downstream consumers.
6. It returns impacted services plus their verification hooks.
7. CI runs only the scoped checks.

## Why not just use path globs?

Path globs are useful, but they break down when:

- dependencies cross directories
- service ownership is not path-local
- one service provides multiple capabilities
- different downstream consumers need different checks

`service-impact` moves the logic into a typed manifest instead of spreading it across CI conditionals.
