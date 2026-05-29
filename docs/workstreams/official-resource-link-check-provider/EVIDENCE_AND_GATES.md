# Official Resource Link Check Provider - Evidence And Gates

Status: Closed
Last updated: 2026-05-28

## Smallest Current Repro

```bash
cargo nextest run -p nako-resource-search resource_link_check --no-fail-fast
```

## Gate Set

```bash
cargo nextest run -p nako-resource-search resource_link_check --no-fail-fast
cargo nextest run -p nako-resource-search manifest --no-fail-fast
cargo fmt --all -- --check
cargo check -p nako-resource-search --tests
git diff --check
```

## Evidence Anchors

- `crates/nako-resource-search/src/manifest.rs`
- `crates/nako-resource-search/src/routes.rs`
- `crates/nako-resource-search/src/routes/resource_protocol.rs`
- `crates/nako-resource-search/src/engine/orchestrator.rs`
- `addons/resource-search/manifest.example.json`
- `addons/resource-search/smoke.local.ps1`

## Run Log

| Date | Evidence | Result |
| --- | --- | --- |
| 2026-05-28 | `cargo nextest run -p nako-resource-search resource_link_check --no-fail-fast` | Pass. 2 route tests passed. |
| 2026-05-28 | `cargo nextest run -p nako-resource-search manifest --no-fail-fast` | Pass. 4 manifest tests passed. |
| 2026-05-28 | `cargo nextest run -p nako-resource-search link_check conservative_checker runtime_check_link manifest --no-fail-fast` | Pass. 12 route/protocol/checker/runtime/manifest tests passed. |
| 2026-05-28 | `cargo fmt --all -- --check` | Pass. |
| 2026-05-28 | `cargo check -p nako-resource-search --tests` | Pass. |
| 2026-05-28 | `git diff --check` | Pass with existing Windows LF-to-CRLF warning for `addons/resource-search/smoke.local.ps1`. |

## Residual Risks

- The conservative checker does not perform live site/API checks.
- Peer-to-peer checks are explicitly unsupported.
- Site-specific cloud providers remain follow-ons and must keep download,
  transfer, and password persistence out of this read-only contract.
