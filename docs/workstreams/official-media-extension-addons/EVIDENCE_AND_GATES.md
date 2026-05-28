# Official Media Extension Addons - Evidence And Gates

Status: Active
Last updated: 2026-05-28

## Gates

| Gate | Command | Status | Notes |
| --- | --- | --- | --- |
| Workstream docs | Manual review | Pass | OMEA-010 opened the lane. |
| Subtitle package tests | `cargo nextest run -p nako-subtitle-provider --no-fail-fast` | Pass | 10 passed on 2026-05-28. |
| DLNA package tests | `cargo nextest run -p nako-dlna-renderer --no-fail-fast` | Pending | OMEA-030 |
| Package check | `cargo check -p nako-subtitle-provider --tests` | Pass | OMEA-020. Combined check waits for OMEA-030. |
| Rust format | `cargo fmt --all -- --check` | Pass | OMEA-020. Final gate still required. |
| Diff hygiene | `git diff --check` | Pass | OMEA-020. Final gate still required. |

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-28 | OMEA-010 | Workstream docs created for Subtitle Provider, DLNA Renderer, and External Acquisition Runner follow-on. | Pass |
| 2026-05-28 | OMEA-020 | `cargo nextest run -p nako-subtitle-provider --no-fail-fast`; `cargo check -p nako-subtitle-provider --tests`; `cargo fmt --all -- --check`; `git diff --check`. | Pass |

## Review Notes

- Subtitle Provider must stay read-only until Nako host owns subtitle import or
  write policy.
- DLNA Renderer must not perform live SSDP discovery or UPnP control in the
  foundation task.
- External Acquisition Runner needs a separate host contract and scopes before
  implementation.
