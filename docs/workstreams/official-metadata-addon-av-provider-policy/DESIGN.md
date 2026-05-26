# Official Metadata Addon AV Provider Policy

Status: Active
Last updated: 2026-05-26

## Problem

The metadata scraper now recognizes AV numbers, routes FC2 and non-FC2 traffic,
and exposes provider/bulk diagnostics. The next maturity gap is policy: users
need clear AV scrape request docs, more provider coverage, and a configurable
way to choose which provider wins for each metadata field.

MDCx shows a mature shape for this problem: multiple sites are queried according
to route and source priority, fields are reduced independently, and batch
execution remains debuggable. This repo should adopt that behavior at the
architecture level without copying GPLv3 implementation details.

## Reference Boundary

`repo-ref/mdcx` remains reference-only. We may use high-level behaviors such as
field-specific source priority, appointed-source preference, route-specific
provider families, and batch diagnostics. We must not copy source, comments,
fixtures, selectors, regex tables, or project structure.

## Target Shape

- Document the current AV request/response contract, including `query.av`,
  `provider_execution`, bulk `resume_state`, failure counters, and direct IDs.
- Treat `addons/browser-worker` as the browser/Crawlee execution boundary.
  Rust providers request rendered pages; the worker owns browser execution,
  waits, session/proxy evolution, and future anti-bot mechanics.
- Add a provider-field policy layer that can choose source priority per field.
  The default should be conservative:
  - explicit provider ID candidates outrank inferred search candidates;
  - official or direct provider facts should be preferred for title, release
    date, studio, publisher, series, and runtime;
  - aggregation providers can fill missing overview, tags, actors, and artwork;
  - field choices must remain visible in redaction-safe evidence.
- Make policy configurable by request first, then promote stable defaults to
  manifest/config once the contract proves useful.
- Add at least one more AV provider tracer using original Rust code and
  synthetic rendered HTML tests.

This lane chose DMM as the first provider-breadth tracer because the MDCx
configuration treats DMM as a browser-backed official source with high priority
for title, outline, release/runtime, score, and artwork fields. The Rust
implementation keeps only that strategy signal: DMM is a disabled-by-default
official censored-release provider that still uses this repo's own parser,
mapper, route declarations, and rendered-page tests.

## Scope

- `crates/nako-metadata-scraper/README.md`
- `addons/metadata-scraper/README.md`
- `addons/browser-worker/README.md` when the Crawlee boundary needs contract
  clarification.
- `crates/nako-metadata-scraper/src/engine`: provider-field policy and evidence.
- `crates/nako-metadata-scraper/src/providers`: additional AV provider modules
  or shared rendered-page helpers.
- Workstream docs and validation gates.

## Non-Goals

- No copying MDCx source, fixtures, selectors, or tables.
- No local file renaming, NFO writing, artwork downloading, actor photo database,
  or watermarking.
- No live network gate as a required test.
- No browser-worker session/cookie redesign in this lane unless a new provider
  proves the current `/render` contract insufficient.

## Assumptions

| Assumption | Confidence | Notes |
| --- | --- | --- |
| The Crawlee browser-worker should own browser execution rather than Rust providers. | High | Current `/render` already matches this boundary. |
| Field policy should be request-configurable before global config. | Medium | This keeps the API testable without committing UI/config shape too early. |
| One additional AV provider tracer is enough to validate provider breadth mechanics. | Medium | More providers can follow once the policy seam is stable. |
| Field-source evidence must remain redaction-safe. | High | AV filenames and IDs can be sensitive. |

## Architecture Direction

The existing resolver already groups provider facts by shared external IDs, and
ranking already emits field-source evidence for the winning candidate. This lane
should deepen that model by reducing fields across a resolved cluster according
to a policy rather than blindly keeping the highest-ranked candidate patch.

The policy should not know site selectors. It should operate on provider IDs,
field names, candidate facts, and field presence. Providers remain responsible
for extracting facts; resolver/ranking remains responsible for candidate
identity and confidence.
