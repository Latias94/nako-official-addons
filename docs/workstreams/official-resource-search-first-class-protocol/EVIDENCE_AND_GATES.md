# Official Resource Search First-Class Protocol - Evidence And Gates

Status: Complete
Last updated: 2026-05-28

## Smallest Current Repro

```bash
cargo nextest run -p nako-resource-search resource_search --no-fail-fast
```

This proves the manifest, route, health diagnostics, and runtime protocol
adapter behavior for the resource-search sidecar.

## Gate Set

### Targeted Iteration Gate

```bash
cargo nextest run -p nako-resource-search resource_search --no-fail-fast
```

### Package Gate

```bash
cargo nextest run -p nako-resource-search --no-fail-fast
```

### Static Gates

```bash
cargo fmt --all -- --check
cargo check -p nako-resource-search --tests
git diff --check
```

### Review Gate

Review findings are recorded in this document before closeout. Blocking findings
must be fixed before `WORKSTREAM.json` can move to complete.

## Evidence Anchors

- `docs/workstreams/official-resource-search-first-class-protocol/DESIGN.md`
- `docs/workstreams/official-resource-search-first-class-protocol/TODO.md`
- `crates/nako-resource-search/src/manifest.rs`
- `crates/nako-resource-search/src/routes/resource_protocol.rs`
- `addons/resource-search/manifest.example.json`
- `addons/resource-search/smoke.local.ps1`

## Run Log

2026-05-28:

- `cargo nextest run -p nako-resource-search resource_search --no-fail-fast`
  passed: 6 tests run, 6 passed. This proves the manifest endpoint, route
  request/response behavior, invalid envelope handling, and first-class search
  response shape covered by the targeted filter.
- `cargo nextest run -p nako-resource-search --no-fail-fast` passed: 43 tests
  run, 43 passed. This proves package behavior including first-class protocol
  adapter tests, exact-link intent mapping, manifest equality, provider
  registry, fusion, PanSou mapping, and diagnostics.
- `cargo fmt --all -- --check` passed.
- `cargo check -p nako-resource-search --tests` passed.
- `git diff --check` passed for `nako-official-addons` with Windows
  line-ending warnings only.
- `git diff --check` passed for `../nako` ADR changes with Windows
  line-ending warnings only.
- `rg -n "automation alpha|alpha request|automation_run|future_protocol_resource|nako\.official\.resource-search\.alpha|AddonResource::Automation" crates/nako-resource-search addons/resource-search README.md`
  returned no matches.

## Review Notes

No blocking findings remain.

One important self-review finding was fixed before closeout: exact-link search
intent must prioritize the intent URL over a non-URL `query` field so the addon
domain does not infer the request as free text. The regression is covered by
`decode_search_request_uses_exact_link_intent_url_as_query`.
