# Official Metadata Addon Fearless Refactor — Evidence And Gates

Status: Complete
Last updated: 2026-05-23

## Gate Policy

Use narrow gates first. Broaden only after a slice changes shared behavior.
Prefer `cargo nextest` for Rust tests.

## Smallest Current Repro

```text
cargo nextest run -p nako-metadata-scraper --no-fail-fast
```

The current skeleton test suite proves manifest validity, fixture provider
behavior, and the metadata route envelope.

## Targeted Iteration Gates

Configuration and manifest:

```text
cargo nextest run -p nako-metadata-scraper config manifest --no-fail-fast
```

Provider registry:

```text
cargo nextest run -p nako-metadata-scraper provider --no-fail-fast
```

Metadata scrape runtime:

```text
cargo nextest run -p nako-metadata-scraper metadata --no-fail-fast
```

Provider HTTP runtime:

```text
cargo nextest run -p nako-metadata-scraper http_runtime --no-fail-fast
```

First real provider proof:

```text
cargo nextest run -p nako-metadata-scraper tmdb --no-fail-fast
```

## Package And Closeout Gates

```text
cargo fmt --all -- --check
cargo nextest run -p nako-metadata-scraper --no-fail-fast
cargo nextest run --workspace --no-fail-fast
git diff --check
```

If a live Nako server is available, record manual smoke evidence for:

```text
cargo run -p nako-metadata-scraper
```

Then in `../nako`, register the manifest, run Addon Health Check, call the
metadata resource diagnostic, and confirm the response remains redaction-safe.

## Reference Repository Red Lines

Reference repositories under `F:/SourceCodes/Rust/repo-ref/nako-scraper/` are
allowed only for product capability mapping and architecture vocabulary:

- tinyMediaManager: Apache-2.0; high-level capability map is acceptable.
- MDCx: GPLv3 plus extra non-commercial terms; inspiration only.

Do not copy, port, translate, or derive:

- source code;
- schemas;
- tests;
- fixtures;
- provider mappings;
- generated files;
- comments;
- resource files;
- artwork;
- scraper expressions or selectors.

## Evidence Ledger

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-23 | OMAFR-010 | Created workstream docs after reading current addon code, Nako Addon workstreams, and reference repository README/license files. | Pass |
| 2026-05-23 | OMAFR-020 | Added provider-aware `Config`, generated Addon Manifest configuration schema from runtime config, filtered route providers by enablement, and updated README/example manifest/compose/systemd docs to advertise fixture-only runtime truth. | Pass |
| 2026-05-23 | OMAFR-030 | Added `providers::registry` with provider catalog construction, capability descriptors, ready/disabled/unavailable diagnostics, and route-level use of registry diagnostics. Removed `default_providers()`. | Pass |
| 2026-05-23 | OMAFR-040 | Added `MetadataScrapeRuntime`, moved request normalization and provider fan-out out of routes, and kept response/artifact shaping in one runtime seam. | Pass |
| 2026-05-23 | OMAFR-050 | Added `ProviderHttpRuntime` with fake-transport-testable timeout, retry, user-agent/proxy config, response-size budget, JSON parsing, and redaction-safe error classification. | Pass |
| 2026-05-23 | OMAFR-060 | Added bounded TMDB movie-search provider proof, manifest secret field behavior when enabled, registry unavailable diagnostics for missing token, and synthetic TMDB mapping tests. | Pass |
| 2026-05-23 | OMAFR-070 | Added `addons/metadata-scraper/smoke.local.ps1`, README local smoke commands, and a direct sidecar smoke run against a temporary sidecar on `127.0.0.1:19100`. Nako-mediated smoke is scripted but was not run because no local Nako server/admin token was available and `../nako` had unrelated dirty worktree changes. | Pass with concerns |
| 2026-05-23 | OMAFR-080 | Aligned root README, addon README, Dockerfile, systemd, compose, and example manifest with fixture-default/TMDB-disabled runtime truth. Added a test proving the checked-in example manifest matches runtime generation. | Pass |
| 2026-05-23 | OMAFR-090 | Closed the workstream after fresh `cargo fmt`, package nextest, workspace nextest, smoke script parse, workstream JSON validation, and `git diff --check` evidence. Split provider/product breadth into follow-ons. | Pass |

OMAFR-020 fresh command evidence:

```text
cargo nextest run -p nako-metadata-scraper config manifest --no-fail-fast
```

Result: pass. 8 tests run, 8 passed, 2 skipped by filter. Proves config
defaults/env overrides, manifest provider schema generation, manifest endpoint,
provider enablement filtering, and health provider diagnostics.

```text
cargo fmt --all -- --check
```

Result: pass. Proves Rust formatting is clean after the task.

```text
cargo nextest run -p nako-metadata-scraper --no-fail-fast
```

Result: pass. 10 tests run, 10 passed. Proves the full package suite still
passes after the configuration/manifest refactor.

```text
cargo nextest run --workspace --no-fail-fast
```

Result: pass. 10 tests run, 10 passed. The current workspace contains the
metadata scraper package, so this is equivalent to the package gate today.

```text
git diff --check
```

Result: pass. Proves no whitespace conflict markers or trailing whitespace in
the current diff.

OMAFR-050 fresh command evidence:

```text
cargo fmt --all -- --check
```

Result: pass. Proves Rust formatting is clean after adding the HTTP runtime.

```text
cargo nextest run -p nako-metadata-scraper http_runtime --no-fail-fast
```

Result: pass. 6 tests run, 6 passed, 17 skipped by filter. Proves retryable
HTTP status retry, non-retryable status behavior, transport retry, response
size budget, invalid JSON classification, POST body serialization, and runtime
policy propagation to the transport.

```text
cargo nextest run -p nako-metadata-scraper --no-fail-fast
```

Result: pass. 23 tests run, 23 passed. Proves the full package suite still
passes after adding the HTTP runtime and reqwest dependency.

```text
cargo nextest run --workspace --no-fail-fast
```

Result: pass. 23 tests run, 23 passed. The current workspace contains the
metadata scraper package, so this is equivalent to the package gate today.

```text
git diff --check
```

Result: pass with a Cargo.lock line-ending warning. Proves no whitespace
conflict markers or trailing whitespace in the current diff; Git warned that
`Cargo.lock` may be normalized from LF to CRLF when touched.

OMAFR-060 fresh command evidence:

```text
cargo nextest run -p nako-metadata-scraper tmdb --no-fail-fast
```

Result: pass. 3 tests run, 3 passed, 23 skipped by filter. Proves TMDB provider
mapping through fake transport, enabled TMDB construction with token, and
enabled-but-missing-token unavailable diagnostics.

```text
cargo fmt --all -- --check
```

Result: pass. Proves Rust formatting is clean after adding the TMDB provider.

```text
cargo nextest run -p nako-metadata-scraper --no-fail-fast
```

Result: pass. 26 tests run, 26 passed. Proves the full package suite still
passes after adding TMDB as a default-disabled real provider proof.

```text
cargo nextest run --workspace --no-fail-fast
```

Result: pass. 26 tests run, 26 passed. The current workspace contains the
metadata scraper package, so this is equivalent to the package gate today.

```text
git diff --check
```

Result: pass with a Cargo.lock line-ending warning. Proves no whitespace
conflict markers or trailing whitespace in the current diff; Git warned that
`Cargo.lock` may be normalized from LF to CRLF when touched.

OMAFR-040 fresh command evidence:

```text
cargo fmt --all -- --check
```

Result: pass. Proves Rust formatting is clean after the runtime seam move.

```text
cargo nextest run -p nako-metadata-scraper metadata --no-fail-fast
```

Result: pass. 4 tests run, 4 passed, 13 skipped by filter. Proves runtime
normalization, provider fan-out, response shaping, and route delegation.

```text
cargo nextest run -p nako-metadata-scraper --no-fail-fast
```

Result: pass. 17 tests run, 17 passed. Proves the full package suite still
passes after the runtime seam move.

```text
cargo nextest run --workspace --no-fail-fast
```

Result: pass. 17 tests run, 17 passed. The current workspace contains the
metadata scraper package, so this is equivalent to the package gate today.

```text
git diff --check
```

Result: pass. Proves no whitespace conflict markers or trailing whitespace in
the current diff.

OMAFR-030 fresh command evidence:

```text
cargo fmt --all -- --check
```

Result: pass. Proves Rust formatting is clean after the provider registry
split.

```text
cargo nextest run -p nako-metadata-scraper provider --no-fail-fast
```

Result: pass. 12 tests run, 12 passed, 3 skipped by filter. Proves registry
construction, disabled provider exclusion, redaction-safe diagnostics, and
unavailable provider reporting without constructing unavailable adapters.

```text
cargo nextest run -p nako-metadata-scraper --no-fail-fast
```

Result: pass. 15 tests run, 15 passed. Proves the full package suite still
passes after registry integration.

```text
cargo nextest run --workspace --no-fail-fast
```

Result: pass. 15 tests run, 15 passed. The current workspace contains the
metadata scraper package, so this is equivalent to the package gate today.

```text
git diff --check
```

Result: pass. Proves no whitespace conflict markers or trailing whitespace in
the current diff.

OMAFR-070 fresh command evidence:

```text
pwsh -NoProfile -Command '$null = [scriptblock]::Create((Get-Content -Raw -Path "addons\metadata-scraper\smoke.local.ps1")); "syntax-ok"'
```

Result: pass. Proves the local smoke script parses as PowerShell.

```text
cargo build -p nako-metadata-scraper
```

Result: pass. Proves the sidecar binary builds before live direct smoke.

```text
pwsh -NoProfile -File addons\metadata-scraper\smoke.local.ps1 -SidecarBaseUrl http://127.0.0.1:19100
```

Result: pass against a temporary sidecar started from
`target\debug\nako-metadata-scraper.exe` with
`NAKO_METADATA_SCRAPER_LISTEN_ADDR=127.0.0.1:19100` and
`NAKO_METADATA_SCRAPER_BASE_URL=http://127.0.0.1:19100`. The script verified
manifest ID/protocol version, health status `ok`, enabled provider `fixture`,
one metadata candidate, and one artifact. The temporary sidecar process was
stopped after the run.

Nako-mediated smoke:

Result: not run. The script supports `-RegisterInNako`, `-Enable`,
`-IssueAddonToken`, and `-RunResourceCall`, but no local Nako server/admin
token was available in this session. `../nako` also had unrelated dirty
worktree changes, so this task did not start or mutate the private core
checkout.

OMAFR-080 fresh command evidence:

```text
cargo nextest run -p nako-metadata-scraper manifest --no-fail-fast
```

Result: pass. 5 tests run, 5 passed, 22 skipped by filter. Proves manifest
validation, configuration-schema provider defaults, TMDB secret field behavior
when enabled, manifest endpoint validity, and checked-in
`addons/metadata-scraper/manifest.example.json` equality with the
runtime-generated manifest for the compose base URL.

OMAFR-090 closeout command evidence:

```text
cargo fmt --all -- --check
```

Result: pass. Proves Rust formatting is clean at closeout.

```text
python -m json.tool docs\workstreams\official-metadata-addon-fearless-refactor\WORKSTREAM.json | Out-Null
```

Result: pass. Proves closeout workstream JSON is parseable.

```text
pwsh -NoProfile -Command '$null = [scriptblock]::Create((Get-Content -Raw -Path "addons\metadata-scraper\smoke.local.ps1")); "syntax-ok"'
```

Result: pass. Proves the smoke script still parses at closeout.

```text
cargo nextest run -p nako-metadata-scraper --no-fail-fast
```

Result: pass. 27 tests run, 27 passed. Proves the full package suite passes
after configuration, manifest, registry, runtime, HTTP runtime, TMDB, smoke,
and docs/example cleanup changes.

```text
cargo nextest run --workspace --no-fail-fast
```

Result: pass. 27 tests run, 27 passed. The current workspace contains the
metadata scraper package, so this is equivalent to the package gate today.
Cargo briefly waited for a package-cache file lock, then completed normally.

```text
git diff --check
```

Result: pass with a Cargo.lock line-ending warning. Proves no whitespace
conflict markers or trailing whitespace in the current diff; Git warned that
`Cargo.lock` may be normalized from LF to CRLF when touched.

Closeout follow-ons:

- Run full Nako Admin-mediated local smoke when a local Nako server and admin
  token are available.
- Expand TMDB beyond bounded movie search.
- Add Bangumi and Douban providers behind the existing registry/runtime seams.
- Add artwork and subtitle provider lanes.
- Add rename planning and NFO-compatible sidecar workflows.
- Add bulk scrape, provider scoring, and ranking hardening.

## Redaction Gates

Any changed health, diagnostics, provider, or metadata response must not expose:

- provider API keys or bearer tokens;
- Addon Tokens;
- administrator bearer tokens;
- resolved Secret Reference values;
- proxy URLs containing credentials;
- local filesystem paths;
- Source Locators;
- raw provider response bodies unless a future explicit operator diagnostic
  owns that risk;
- copied reference repository fixture data.

## Review Gates

- Use `review-workstream` before accepting non-trivial implementation slices.
- Use `verify-rust-workstream` before marking the lane complete.
- Update `HANDOFF.md` after each completed task.
- Add `JOURNAL/YYYY-MM-DD-<task>.md` for implementation slices beyond docs.
