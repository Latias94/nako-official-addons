# Official Media Extension Addons - Evidence And Gates

Status: Complete
Last updated: 2026-05-28

## Gates

| Gate | Command | Status | Notes |
| --- | --- | --- | --- |
| Workstream docs | Manual review | Pass | OMEA-010 opened the lane. |
| Subtitle package tests | `cargo nextest run -p nako-subtitle-provider --no-fail-fast` | Pass | 10 passed on 2026-05-28. |
| DLNA package tests | `cargo nextest run -p nako-dlna-renderer --no-fail-fast` | Pass | 16 passed on 2026-05-28. |
| Acquisition runner docs | Manual review; `git diff --check` | Pass | OMEA-040 records future action-addon contract only. |
| Nako catalog sync | `cargo nextest run -p nako-official-addon-catalog --no-fail-fast`; `cargo nextest run -p nako-server addon_source_catalog --no-fail-fast`; `cargo check -p nako-official-addon-catalog -p nako-server --tests`; `cargo fmt --all -- --check`; `git diff --check` in `../nako` | Pass | Nako commit `52da469d`; no `../nako/web` changes. |
| Package check | `cargo check -p nako-subtitle-provider --tests`; `cargo check -p nako-dlna-renderer --tests` | Pass | OMEA-020 and OMEA-030. |
| Final focused tests | `cargo nextest run -p nako-subtitle-provider -p nako-dlna-renderer --no-fail-fast` | Pass | 26 passed on 2026-05-28. |
| Rust format | `cargo fmt --all -- --check` | Pass | Final closeout gate passed. |
| Diff hygiene | `git diff --check` | Pass | Final closeout gate passed. |

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-28 | OMEA-010 | Workstream docs created for Subtitle Provider, DLNA Renderer, and External Acquisition Runner follow-on. | Pass |
| 2026-05-28 | OMEA-020 | `cargo nextest run -p nako-subtitle-provider --no-fail-fast`; `cargo check -p nako-subtitle-provider --tests`; `cargo fmt --all -- --check`; `git diff --check`. | Pass |
| 2026-05-28 | OMEA-030 | `cargo nextest run -p nako-dlna-renderer --no-fail-fast`; `cargo check -p nako-dlna-renderer --tests`; `cargo fmt --all -- --check`; `git diff --check`. | Pass |
| 2026-05-28 | OMEA-040 | README and FOLLOW_ON_CONTRACTS manual review; `git diff --check`. | Pass |
| 2026-05-28 | OMEA-050 | `../nako` commit `52da469d`; official catalog and server addon-source catalog gates. | Pass |
| 2026-05-28 | OMEA-060 | `cargo nextest run -p nako-subtitle-provider -p nako-dlna-renderer --no-fail-fast`; `cargo check -p nako-subtitle-provider -p nako-dlna-renderer --tests`; `cargo fmt --all -- --check`; `git diff --check`. | Pass |

## Review Notes

- Subtitle Provider must stay read-only until Nako host owns subtitle import or
  write policy.
- DLNA Renderer must not perform live SSDP discovery or UPnP control in the
  foundation task.
- External Acquisition Runner needs a separate host contract and scopes before
  implementation.
