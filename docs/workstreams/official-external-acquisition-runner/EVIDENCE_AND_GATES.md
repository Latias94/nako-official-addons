# Official External Acquisition Runner - Evidence And Gates

Status: Active
Last updated: 2026-05-29

## Standing Gates

```powershell
git status --short --branch
git -C ../nako status --short --branch
python -m json.tool docs/workstreams/official-external-acquisition-runner/WORKSTREAM.json
git diff --check -- docs/workstreams/official-external-acquisition-runner
```

Protocol and catalog gates:

```powershell
cargo nextest run -p nako-addon-protocol external_acquisition --no-fail-fast
cargo nextest run -p nako-addon-protocol --no-fail-fast
cargo nextest run -p nako-official-addon-catalog external_acquisition --no-fail-fast
cargo nextest run -p nako-official-addon-catalog --no-fail-fast
```

Runner sidecar gates, once the crate exists:

```powershell
cargo fmt -p nako-external-acquisition-runner -- --check
cargo nextest run -p nako-external-acquisition-runner --no-fail-fast
```

Nako host dispatch gates:

```powershell
cargo check -p nako-server --tests
cargo nextest run -p nako-server official_external_acquisition --no-fail-fast
cargo nextest run -p nako-server admin_addon_source_catalog --no-fail-fast
```

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-29 | OEAR-010 | Opened the workstream from ADR 0050 and official addon follow-on contracts. | Pass |
| 2026-05-29 | OEAR-010 | `python -m json.tool docs/workstreams/official-external-acquisition-runner/WORKSTREAM.json` | Pass |
| 2026-05-29 | OEAR-010 | `git diff --check -- docs/workstreams/official-external-acquisition-runner` | Pass |
| 2026-05-29 | OEAR-020 | `cargo fmt -p nako-addon-protocol -p nako-api -p nako-official-addon-catalog -p nako-server` | Pass |
| 2026-05-29 | OEAR-020 | `cargo fmt -p nako-addon-protocol -p nako-api -p nako-official-addon-catalog -p nako-server -- --check` | Pass |
| 2026-05-29 | OEAR-020 | `cargo nextest run -p nako-addon-protocol external_acquisition --no-fail-fast` | Pass |
| 2026-05-29 | OEAR-020 | `cargo nextest run -p nako-official-addon-catalog external_acquisition --no-fail-fast` | Pass |
| 2026-05-29 | OEAR-020 | `cargo nextest run -p nako-server official_external_acquisition --no-fail-fast` | Pass |
| 2026-05-29 | OEAR-020 | `cargo nextest run -p nako-server admin_addon_source_catalog_browses --no-fail-fast` | Pass |
| 2026-05-29 | OEAR-020 | `cargo nextest run -p nako-server admin_addon_source_catalog_resolves_external_acquisition_runner --no-fail-fast` | Pass |
| 2026-05-29 | OEAR-020 | `cargo nextest run -p nako-addon-protocol --no-fail-fast` | Pass |
| 2026-05-29 | OEAR-020 | `cargo nextest run -p nako-official-addon-catalog --no-fail-fast` | Pass |
| 2026-05-29 | OEAR-020 | `cargo check -p nako-server --tests` | Pass |
| 2026-05-29 | OEAR-020 | `cargo nextest run -p nako-server admin_addon_source_catalog --no-fail-fast` | Pass |
| 2026-05-29 | OEAR-020 | `git diff --check -- crates/nako-addon-protocol/src/lib.rs crates/nako-api/src/extension.rs crates/nako-api/src/admin_contract.rs crates/nako-official-addon-catalog/src/lib.rs crates/nako-server/src/app/addons/catalog.rs crates/nako-server/src/app/addons.rs crates/nako-server/src/http/tests/addons.rs` | Pass |
| 2026-05-29 | OEAR-020 | `python -m json.tool docs/workstreams/official-external-acquisition-runner/WORKSTREAM.json` | Pass |
| 2026-05-29 | OEAR-020 | `git diff --check -- docs/workstreams/official-external-acquisition-runner` | Pass |

## Known Constraints

- Do not stage, restore, or modify unrelated user changes if they appear while
  continuing this lane.
- `../nako/docs/workstreams/web-admin-acquisition-intake` is a related active
  web lane. This runner lane may depend on it for product UI but must not take
  over its route tasks.
- Current repo may still be ahead of origin depending on push state; verify
  before committing.
- No real runner adapter should be added before the fixture/no-op contract and
  host dispatch semantics pass focused gates.
