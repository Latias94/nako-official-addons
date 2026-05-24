# Official Addon Contract Smoke Provider Hardening - Evidence And Gates

Status: Complete
Last updated: 2026-05-24

## Standing Gates

```powershell
git status --short --branch
cargo fmt --all -- --check
git diff --check
```

## Task Envelope Gates

```powershell
cargo nextest run -p nako-metadata-scraper bulk task routes manifest --no-fail-fast
cargo nextest run -p nako-addon-protocol task envelope --no-fail-fast
```

## Smoke Gates

```powershell
$errors = $null; [System.Management.Automation.PSParser]::Tokenize((Get-Content -Raw -Path 'addons/metadata-scraper/smoke.local.ps1'), [ref]$errors) | Out-Null; if ($errors) { $errors; exit 1 }
$errors = $null; [System.Management.Automation.PSParser]::Tokenize((Get-Content -Raw -Path '../nako/scripts/official-addon-e2e-smoke.ps1'), [ref]$errors) | Out-Null; if ($errors) { $errors; exit 1 }
pwsh -File addons/metadata-scraper/smoke.local.ps1 -SidecarBaseUrl http://127.0.0.1:9100 -RunTaskPath
pwsh -NoProfile -ExecutionPolicy Bypass -File ../nako/scripts/official-addon-e2e-smoke.ps1
```

## Provider Descriptor Gates

```powershell
cargo nextest run -p nako-metadata-scraper provider registry manifest config --no-fail-fast
cargo nextest run -p nako-metadata-scraper --no-fail-fast
```

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-24 | OACSH-010 | Opened this lane after confirming ADR 0033 already owns Addon Protocol version/release separation. Selected task envelope unification, live smoke hardening, and provider descriptor boundary as next work. | Pass |
| 2026-05-24 | OACSH-020 | Removed metadata scraper local `AddonTaskRequest`/`AddonTaskResponse` mirrors from `engine/bulk.rs`; `bulk.rs` and `routes.rs` now use public `nako-addon-protocol` task envelope types. | Pass |
| 2026-05-24 | OACSH-020 | `cargo nextest run -p nako-metadata-scraper bulk task routes manifest --no-fail-fast` | Pass: 13 passed, 127 skipped |
| 2026-05-24 | OACSH-020 | `cargo fmt --all -- --check` | Pass |
| 2026-05-24 | OACSH-020 | `git diff --check -- crates/nako-metadata-scraper/src/engine/bulk.rs crates/nako-metadata-scraper/src/routes.rs docs/workstreams/official-addon-contract-smoke-provider-hardening` | Pass |
| 2026-05-24 | OACSH-030 | Hardened `smoke.local.ps1`: Nako-owned flags now require `-RegisterInNako`, manifest smoke asserts the task declaration/path, and writeback smoke can assert expected status/safe error code. Added `-PreflightOnly` to `../nako/scripts/official-addon-e2e-smoke.ps1`. | Pass |
| 2026-05-24 | OACSH-030 | PowerShell parser checks for `addons/metadata-scraper/smoke.local.ps1` and `../nako/scripts/official-addon-e2e-smoke.ps1` | Pass |
| 2026-05-24 | OACSH-030 | `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/official-addon-e2e-smoke.ps1 -AddonRepo F:\SourceCodes\Rust\nako-official-addons -AddonBinarySource workspace -PreflightOnly` in `../nako` | Pass |
| 2026-05-24 | OACSH-030 | `pwsh -NoProfile -ExecutionPolicy Bypass -File addons/metadata-scraper/smoke.local.ps1 -RunTaskPath` expected failure guard | Pass: failed before network calls with missing `-RegisterInNako` diagnostic |
| 2026-05-24 | OACSH-030 | `cargo nextest run -p nako-metadata-scraper bulk task routes manifest --no-fail-fast` | Pass: 13 passed, 127 skipped |
| 2026-05-24 | OACSH-030 | `cargo fmt --all -- --check`; path-scoped `git diff --check` in both repositories | Pass |
| 2026-05-24 | OACSH-040 | Provider modules now expose provider-owned catalog entries. `ProviderRegistry` composes those entries and exposes provider schema/secret-reference helpers consumed by manifest generation. | Pass |
| 2026-05-24 | OACSH-040 | `cargo nextest run -p nako-metadata-scraper provider registry manifest config --no-fail-fast` | Pass: 90 passed, 50 skipped |
| 2026-05-24 | OACSH-040 | `cargo nextest run -p nako-metadata-scraper --no-fail-fast` | Pass: 138 passed, 2 skipped |
| 2026-05-24 | OACSH-040 | `cargo fmt --all -- --check`; `git diff --check -- crates/nako-metadata-scraper/src/providers crates/nako-metadata-scraper/src/manifest.rs docs/workstreams/official-addon-contract-smoke-provider-hardening` | Pass |
| 2026-05-24 | OACSH-050 | Closed this workstream with evidence, handoff, and residual live-smoke/install-boundary notes. | Pass |

## Known Constraints

- `nako-official-addons` and `../nako` both have active dirty worktrees. Do not
  revert or format unrelated changes.
- This lane may touch `../nako` only for protocol helper tests or smoke script
  integration.
- Live smoke requires an actual Nako server/admin token and sidecar endpoint.
  If unavailable, record the exact blocker rather than weakening the assertion.
- Do not split the official metadata addon into multiple installed sidecars in
  this lane.
- OACSH-030 did not run full Docker/server live smoke in this session. The
  E2E script preflight and local smoke guards passed; live execution remains
  the release-time proof.
