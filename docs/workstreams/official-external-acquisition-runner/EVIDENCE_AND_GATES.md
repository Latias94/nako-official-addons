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
cargo check -p nako-external-acquisition-runner --tests
cargo nextest run -p nako-external-acquisition-runner --no-fail-fast
cargo clippy -p nako-external-acquisition-runner --tests -- -D warnings
pwsh -File addons/external-acquisition-runner/smoke.local.ps1 -SidecarBaseUrl http://127.0.0.1:9160
```

Nako host dispatch gates:

```powershell
cargo fmt -p nako-server -- --check
cargo check -p nako-server --tests
cargo nextest run -p nako-server addon_external_acquisition_action --no-fail-fast
cargo nextest run -p nako-server addon_task_run --no-fail-fast
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
| 2026-05-29 | OEAR-030 | `cargo fmt -p nako-external-acquisition-runner -- --check` | Pass |
| 2026-05-29 | OEAR-030 | `cargo check -p nako-external-acquisition-runner --tests` | Pass |
| 2026-05-29 | OEAR-030 | `cargo nextest run -p nako-external-acquisition-runner --no-fail-fast` | Pass: 11 tests cover manifest, health, action envelope, idempotency, status, cancellation, and redaction |
| 2026-05-29 | OEAR-030 | `cargo clippy -p nako-external-acquisition-runner --tests -- -D warnings` | Pass |
| 2026-05-29 | OEAR-030 | `cargo build -p nako-external-acquisition-runner`; start debug sidecar on `127.0.0.1:19160`; `pwsh -File addons/external-acquisition-runner/smoke.local.ps1 -SidecarBaseUrl http://127.0.0.1:19160` | Pass |
| 2026-05-29 | OEAR-030 | `python -m json.tool docs/workstreams/official-external-acquisition-runner/WORKSTREAM.json` | Pass |
| 2026-05-29 | OEAR-030 | `git diff --check -- docs/workstreams/official-external-acquisition-runner` | Pass |
| 2026-05-29 | OEAR-030 | `git diff --check -- Cargo.toml Cargo.lock README.md crates/nako-external-acquisition-runner addons/external-acquisition-runner docs/workstreams/official-external-acquisition-runner` | Pass |
| 2026-05-29 | OEAR-040 | `cargo fmt -p nako-server -- --check` | Pass |
| 2026-05-29 | OEAR-040 | `cargo check -p nako-server --tests` | Pass |
| 2026-05-29 | OEAR-040 | `cargo nextest run -p nako-server addon_external_acquisition_action --no-fail-fast` | Pass: 4 tests cover direct dispatch, unsafe payload rejection, idempotency alignment, and runner rejection mapping |
| 2026-05-29 | OEAR-040 | `cargo nextest run -p nako-server addon_task_run --no-fail-fast` | Pass: 9 existing task runtime tests |
| 2026-05-29 | OEAR-040 | `cargo nextest run -p nako-server official_external_acquisition --no-fail-fast` | Pass |
| 2026-05-29 | OEAR-040 | `cargo nextest run -p nako-server admin_addon_source_catalog --no-fail-fast` | Pass |
| 2026-05-29 | OEAR-040 | `git diff --check -- crates/nako-server/src/app/addons.rs crates/nako-server/src/app/addons/task_runtime.rs crates/nako-server/src/app/addons/external_acquisition.rs crates/nako-server/src/http/tests/addons.rs` | Pass |
| 2026-05-29 | OEAR-050 | Reviewed official qBittorrent WebUI API, Transmission RPC spec, and aria2 manual; recorded adapter decision in `JOURNAL/2026-05-29-oear-050.md`. | Pass |
| 2026-05-29 | OEAR-050 | `python -m json.tool docs/workstreams/official-external-acquisition-runner/WORKSTREAM.json` | Pass |
| 2026-05-29 | OEAR-050 | `git diff --check -- docs/workstreams/official-external-acquisition-runner` | Pass |

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
- `cargo clippy -p nako-server --tests -- -D warnings` and `--no-deps`
  currently fail on pre-existing unrelated lint debt across `nako-server` and
  dependencies, so clippy is not used as an OEAR-040 completion gate.
