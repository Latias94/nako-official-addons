# Official Addons Architecture Boundary Hardening - Evidence And Gates

Status: Complete
Last updated: 2026-05-29

## Standing Gates

Prefer focused gates during each task. Broaden only after the touched boundary
is stable.

```powershell
git status --short --branch
cargo fmt --all -- --check
git diff --check
```

Cross-repo status gate:

```powershell
git -C ../nako status --short --branch
```

Manifest source-of-truth gates:

```powershell
cargo nextest run -p nako-resource-search manifest --no-fail-fast
cargo nextest run -p nako-subtitle-provider manifest --no-fail-fast
cargo nextest run -p nako-dlna-renderer manifest --no-fail-fast
cargo nextest run -p nako-official-addon-catalog resource_search subtitle dlna --no-fail-fast
```

Addon app service boundary gates:

```powershell
cargo nextest run -p nako-server addon --no-fail-fast
cargo check -p nako-server --tests
```

Provider HTTP operation policy gates:

```powershell
cargo nextest run -p nako-metadata-scraper http_runtime provider --no-fail-fast
cargo nextest run -p nako-metadata-scraper --no-fail-fast
```

Notification bridge route locality gates:

```powershell
cargo nextest run -p nako-notification-bridge routes diagnostics provider --no-fail-fast
```

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-29 | OAABH-010 | Opened lane after local architecture review identified catalog/runtime manifest duplication, Nako addon app-service boundary drift, provider HTTP operation-policy gaps, notification bridge route locality, and stale provider model research docs. | Pass |
| 2026-05-29 | OAABH-020 | `cargo fmt -p nako-resource-search -p nako-subtitle-provider -p nako-dlna-renderer -- --check` | Pass |
| 2026-05-29 | OAABH-020 | `cargo nextest run -p nako-resource-search -p nako-subtitle-provider -p nako-dlna-renderer manifest --no-fail-fast` | Pass: 10 tests passed, 68 skipped |
| 2026-05-29 | OAABH-020 | `cargo nextest run -p nako-official-addon-catalog resource_search subtitle dlna --no-fail-fast` in `../nako` | Pass: 6 tests passed |
| 2026-05-29 | OAABH-020 | `cargo nextest run -p nako-resource-search -p nako-subtitle-provider -p nako-dlna-renderer --no-fail-fast` | Pass: 78 tests passed |
| 2026-05-29 | OAABH-020 | `git diff --check` | Pass; Git emitted a Windows line-ending warning for `Cargo.lock` |
| 2026-05-29 | OAABH-030 | Split `../nako/crates/nako-server/src/app/addons.rs` into `catalog`, `surfaces`, `routing`, `diagnostics`, `resource_search`, and `subtitles` modules. Parent file reduced from 3644 to 926 lines. | Pass |
| 2026-05-29 | OAABH-030 | `cargo check -p nako-server --tests` in `../nako` | Pass |
| 2026-05-29 | OAABH-030 | `cargo nextest run -p nako-server addon --no-fail-fast` in `../nako` | Pass: 117 tests passed, 334 skipped |
| 2026-05-29 | OAABH-040 | Added explicit `ProviderHttpOperationPolicy` with retry-after, safe cache, and throttle intent; wired TMDB detail enrichment to declare authenticated safe-cache and provider-local throttle facts. | Pass |
| 2026-05-29 | OAABH-040 | `cargo fmt -p nako-metadata-scraper` | Pass |
| 2026-05-29 | OAABH-040 | `cargo check -p nako-metadata-scraper --tests` | Pass |
| 2026-05-29 | OAABH-040 | `cargo nextest run -p nako-metadata-scraper http_runtime tmdb --no-fail-fast` | Pass: 51 tests passed, 228 skipped |
| 2026-05-29 | OAABH-040 | `cargo nextest run -p nako-metadata-scraper http_runtime provider --no-fail-fast` | Pass: 187 tests passed, 92 skipped |
| 2026-05-29 | OAABH-040 | `cargo nextest run -p nako-metadata-scraper --no-fail-fast` | Pass: 276 tests passed, 3 skipped |
| 2026-05-29 | OAABH-040 | `cargo fmt -p nako-metadata-scraper -- --check` | Pass |
| 2026-05-29 | OAABH-050 | Split notification bridge provider fan-out into `provider_send` and diagnostics HTML rendering into `diagnostics`, leaving route handlers as HTTP adapters. | Pass |
| 2026-05-29 | OAABH-050 | `cargo fmt -p nako-notification-bridge` | Pass |
| 2026-05-29 | OAABH-050 | `cargo check -p nako-notification-bridge --tests` | Pass |
| 2026-05-29 | OAABH-050 | `cargo nextest run -p nako-notification-bridge routes diagnostics provider --no-fail-fast` | Pass: 36 tests passed, 10 skipped |
| 2026-05-29 | OAABH-050 | `cargo nextest run -p nako-notification-bridge --no-fail-fast` | Pass: 46 tests passed |
| 2026-05-29 | OAABH-050 | `cargo fmt -p nako-notification-bridge -- --check` | Pass |
| 2026-05-29 | OAABH-060 | Updated `official-metadata-addon-mature-provider-model-research` docs so completed resolver, external ID capability, provider field-policy fusion, and HTTP operation policy work is no longer described as future P0 work. | Pass |
| 2026-05-29 | OAABH-060 | `python -m json.tool docs/workstreams/official-metadata-addon-mature-provider-model-research/WORKSTREAM.json` | Pass |
| 2026-05-29 | OAABH-060 | `rg -n "Do next|open a follow-on|future P0|recommended scope|Should be the next implementation|deferred to a new provider fact resolver|follow-on resolver and external ID capability workstream deferred|highest-leverage next lane" docs/workstreams/official-metadata-addon-mature-provider-model-research` | Pass: no stale future-work matches |
| 2026-05-29 | OAABH-070 | `git status --short --branch` | Pass: expected lane changes in `nako-official-addons` |
| 2026-05-29 | OAABH-070 | `git -C ../nako status --short --branch` | Pass: expected server addon changes plus unrelated user web/workstream changes left untouched |
| 2026-05-29 | OAABH-070 | `python -m json.tool docs/workstreams/official-addons-architecture-boundary-hardening/WORKSTREAM.json`; `python -m json.tool docs/workstreams/official-metadata-addon-mature-provider-model-research/WORKSTREAM.json` | Pass |
| 2026-05-29 | OAABH-070 | `cargo fmt -p nako-resource-search -p nako-subtitle-provider -p nako-dlna-renderer -p nako-metadata-scraper -p nako-notification-bridge -- --check` | Pass |
| 2026-05-29 | OAABH-070 | `cargo nextest run -p nako-resource-search -p nako-subtitle-provider -p nako-dlna-renderer -p nako-metadata-scraper -p nako-notification-bridge --no-fail-fast` | Pass: 400 tests passed, 3 skipped |
| 2026-05-29 | OAABH-070 | `git diff --check` | Pass; Git emitted a Windows line-ending warning for `Cargo.lock` |
| 2026-05-29 | OAABH-070 | `rg -n '[ \t]+$'` over untracked notification bridge modules and workstream docs | Pass: no trailing whitespace matches |
| 2026-05-29 | OAABH-070 | `cargo fmt -p nako-server -- --check` in `../nako` | Pass |
| 2026-05-29 | OAABH-070 | `cargo check -p nako-server --tests` in `../nako` | Pass |
| 2026-05-29 | OAABH-070 | `cargo nextest run -p nako-server addon --no-fail-fast` in `../nako` | Pass: 117 tests passed, 334 skipped |
| 2026-05-29 | OAABH-070 | `git diff --check -- crates/nako-server/src/app/addons.rs crates/nako-server/src/app/addons/*.rs` equivalent explicit path list in `../nako` | Pass; Git emitted a Windows line-ending warning for `addons.rs` |
| 2026-05-29 | OAABH-070 | `rg -n '[ \t]+$'` over untracked `../nako` addon module files | Pass: no trailing whitespace matches |

## Known Constraints

- `nako-official-addons` is on `main`; current worktree was clean when this
  lane opened.
- `../nako` is on `main`, ahead of origin by one commit, and current worktree
  was clean when this lane opened.
- Do not use `git restore`, `git checkout`, `git reset`, or stash to remove
  changes that may belong to the user.
- Do not edit active `../nako/web` workstream files as part of this lane.
- External Acquisition Runner is intentionally split out of this lane.
