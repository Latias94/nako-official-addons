# Official Metadata Addon Production Baseline — Evidence And Gates

Status: Complete
Last updated: 2026-05-23

## Gate Policy

Prefer narrow `cargo nextest` filters during iteration. Broaden to package and
workspace gates before task completion and closeout. Default gates must not
require live TMDB network access.

## Targeted Gates

Live smoke:

```text
pwsh -File addons/metadata-scraper/smoke.local.ps1 -SidecarBaseUrl <url>
```

Ranking/evidence:

```text
cargo nextest run -p nako-metadata-scraper ranking evidence --no-fail-fast
```

TMDB baseline:

```text
cargo nextest run -p nako-metadata-scraper tmdb --no-fail-fast
```

Package and closeout:

```text
cargo fmt --all -- --check
cargo nextest run -p nako-metadata-scraper --no-fail-fast
cargo nextest run --workspace --no-fail-fast
git diff --check
```

## Redaction Gates

Changed smoke, diagnostics, provider, ranking, evidence, or artifact responses
must not expose:

- TMDB bearer tokens or provider API keys;
- Addon Tokens;
- administrator bearer tokens;
- raw provider response bodies;
- raw diagnostic payloads;
- credential-bearing proxy URLs;
- Source Locators or local filesystem paths;
- copied reference repository fixture data.

## Evidence Ledger

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-23 | OMAPB-010 | Opened production baseline workstream after closing the architecture lane. | Pass |
| 2026-05-23 | OMAPB-020 | Direct sidecar smoke passed on `127.0.0.1:19101`; live Nako Admin-mediated smoke was not run because `127.0.0.1:3000` refused connections and `NAKO_ADMIN_TOKEN` was unset. | Pass with concerns |
| 2026-05-23 | OMAPB-030 | Added provider-neutral `engine::ranking`, provider candidate facts, runtime-owned confidence scoring, deterministic tie-break sorting, external-ID query parsing, and redaction-safe evidence serialization. | Pass |
| 2026-05-23 | OMAPB-040 | Expanded TMDB to bounded movie search, detail, and external-ID enrichment through the shared HTTP runtime with synthetic fake-transport tests. | Pass |
| 2026-05-23 | OMAPB-050 | Updated root README and addon README to describe TMDB baseline and ranking/evidence runtime truth without claiming future providers. | Pass |
| 2026-05-23 | OMAPB-060 | Closed the workstream after JSON, fmt, package nextest, workspace nextest, and diff-check gates passed. | Pass |

OMAPB-020 fresh command evidence:

```text
python -m json.tool docs\workstreams\official-metadata-addon-production-baseline\WORKSTREAM.json | Out-Null
```

Result: pass. Proves the new workstream metadata is valid JSON.

```text
Invoke-WebRequest http://127.0.0.1:3000/admin/v1/addons
```

Result: not available. The local TCP connection to `127.0.0.1:3000` was
refused.

```text
NAKO_ADMIN_TOKEN
```

Result: unset.

```text
cargo build -p nako-metadata-scraper
```

Result: pass. Proves the sidecar binary builds before direct smoke.

```text
pwsh -NoProfile -File addons\metadata-scraper\smoke.local.ps1 -SidecarBaseUrl http://127.0.0.1:19101
```

Result: pass against a temporary sidecar started from
`target\debug\nako-metadata-scraper.exe`. The script verified manifest
ID/protocol version, health status `ok`, enabled provider `fixture`, one
metadata candidate, and one artifact. The temporary process was stopped after
the run.

OMAPB-030 fresh command evidence:

```text
cargo nextest run -p nako-metadata-scraper ranking evidence --no-fail-fast
```

Result: pass. 5 tests run, 5 passed, 26 skipped by filter. Proves exact
title/year/external-ID/language scoring, mismatch penalties, redaction-safe
evidence serialization, external-ID query parsing, and runtime sorting over
ranked candidates.

```text
cargo fmt --all -- --check
```

Result: pass after formatting the new ranking module.

```text
cargo nextest run -p nako-metadata-scraper --no-fail-fast
```

Result: pass. 31 tests run, 31 passed. Proves the package suite still passes
after moving final confidence ownership from providers into the runtime.

OMAPB-040 fresh command evidence:

```text
cargo nextest run -p nako-metadata-scraper tmdb --no-fail-fast
```

Result: pass. 3 tests run, 3 passed, 28 skipped by filter. Proves enabled TMDB
construction, missing-token diagnostics, and TMDB search/detail/external-ID
mapping through fake transport and shared HTTP runtime.

```text
cargo fmt --all -- --check
```

Result: pass after formatting TMDB enrichment changes.

```text
cargo nextest run -p nako-metadata-scraper --no-fail-fast
```

Result: pass. 31 tests run, 31 passed. Proves the package suite still passes
after TMDB production baseline enrichment and docs updates.

OMAPB-060 closeout command evidence:

```text
python -m json.tool docs\workstreams\official-metadata-addon-production-baseline\WORKSTREAM.json | Out-Null
```

Result: pass. Proves workstream metadata is valid JSON.

```text
cargo fmt --all -- --check
```

Result: pass. Proves Rust formatting is clean at closeout.

```text
git diff --check
```

Result: pass with a Cargo.lock line-ending warning. Proves no whitespace
conflict markers or trailing whitespace in the current diff; Git warned that
`Cargo.lock` may be normalized from LF to CRLF when touched.

```text
cargo nextest run -p nako-metadata-scraper --no-fail-fast
```

Result: pass. 31 tests run, 31 passed.

```text
cargo nextest run --workspace --no-fail-fast
```

Result: pass. 31 tests run, 31 passed. The current workspace contains the
metadata scraper package, so this is equivalent to the package gate today.

Closeout follow-ons:

- Run full Nako Admin-mediated smoke with a local Nako server and admin token.
- Add Bangumi/Douban provider adapters.
- Add artwork and subtitle provider lanes.
- Add rename planning and NFO-compatible workflows.
- Add bulk scrape, provider scoring feedback, and ranking hardening.

## Live-Network Policy

Live TMDB calls are not default tests. If a live smoke is useful later, gate it
behind explicit environment variables and record it as optional evidence only.

Live Nako Admin smoke may be run only against an operator-started local Nako
server. The smoke script must not start, stop, install, update, or supervise
Nako.
