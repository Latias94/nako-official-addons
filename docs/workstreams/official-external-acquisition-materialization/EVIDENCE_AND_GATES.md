# Official External Acquisition Materialization - Evidence And Gates

Status: Active
Last updated: 2026-05-29

## Standing Gates

```powershell
git status --short --branch
git -C ../nako status --short --branch
python -m json.tool docs/workstreams/official-external-acquisition-materialization/WORKSTREAM.json
git diff --check -- docs/workstreams/official-external-acquisition-materialization
```

## Contract Gates

```powershell
cargo nextest run -p nako-addon-protocol external_acquisition --no-fail-fast
cargo nextest run -p nako-addon-protocol --no-fail-fast
```

Run focused `nako-api` tests when OEAM-020 adds API DTOs.

## Host Resolver Gates

```powershell
cargo fmt -p nako-server -- --check
cargo check -p nako-server --tests
cargo nextest run -p nako-server external_acquisition_materialization --no-fail-fast
cargo nextest run -p nako-server addon_external_acquisition_action --no-fail-fast
```

Run `cargo nextest run -p nako-server acquisition_intake --no-fail-fast` when
selected-link or intake-candidate storage behavior changes.

## Runner Gates

```powershell
cargo fmt -p nako-external-acquisition-runner -- --check
cargo check -p nako-external-acquisition-runner --tests
cargo nextest run -p nako-external-acquisition-runner materialization --no-fail-fast
cargo nextest run -p nako-external-acquisition-runner --no-fail-fast
cargo clippy -p nako-external-acquisition-runner --tests -- -D warnings
```

## Review Gate

Run `review-workstream` before accepting task completion. Record blocking
findings, missing gates, and residual risks in this file or in a linked journal
entry.

Fresh verification is required before marking a task, Codex goal, or lane
complete.

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-29 | OEAM-010 | Opened the workstream from OEAR closeout and ADR 0050 follow-on boundaries. | Pass |
| 2026-05-29 | OEAM-010 | `python -m json.tool docs/workstreams/official-external-acquisition-materialization/WORKSTREAM.json` | Pass |
| 2026-05-29 | OEAM-010 | `git diff --check -- docs/workstreams/official-external-acquisition-materialization` | Pass |

## Known Constraints

- Do not stage, restore, or modify unrelated user changes if they appear while
  continuing this lane.
- `../nako/docs/workstreams/web-admin-acquisition-intake` is related product UI
  work but remains out of scope for this lane.
- No production runner adapter should be added before materialization is defined
  and verified.
- `cargo clippy -p nako-server --tests -- -D warnings` is not a standing gate
  because OEAR recorded existing unrelated lint debt in `nako-server`.
