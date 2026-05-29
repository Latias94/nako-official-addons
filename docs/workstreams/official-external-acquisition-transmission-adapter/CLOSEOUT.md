# Official External Acquisition Transmission Adapter - Closeout

Date: 2026-05-29

Status: Complete

## Result

The official external acquisition runner now has a production Transmission
adapter path behind the host-owned materialization boundary.

Delivered:

- opt-in Transmission profile configuration;
- redaction-safe manifest/config schema and diagnostics;
- official Nako catalog schema for the Transmission profile and optional
  `transmission_password` secret field;
- typed Transmission RPC client with session-id retry;
- fake RPC coverage for add, duplicate, status, start, and stop;
- Transmission enqueue through Nako materialization;
- `runner_job_ref = transmission:<hash_string>`;
- status, pause, resume, and cancel controls from runner job refs only;
- route-level Transmission task coverage with fake materialization/RPC;
- fixture-only local smoke remains green by default.

## Final Evidence

Passed during the lane:

```bash
cargo nextest run -p nako-external-acquisition-runner config manifest diagnostics --no-fail-fast
cargo nextest run -p nako-official-addon-catalog external_acquisition_runner_default_manifest_matches_official_catalog_facts --no-fail-fast
cargo nextest run -p nako-external-acquisition-runner transmission --no-fail-fast
cargo nextest run -p nako-external-acquisition-runner transmission enqueue materialization --no-fail-fast
cargo nextest run -p nako-external-acquisition-runner transmission status cancel pause resume --no-fail-fast
cargo nextest run -p nako-external-acquisition-runner --no-fail-fast
cargo fmt -p nako-external-acquisition-runner -- --check
cargo fmt -p nako-official-addon-catalog -- --check
cargo clippy -p nako-external-acquisition-runner --tests -- -D warnings
pwsh -File addons/external-acquisition-runner/smoke.local.ps1 -SidecarBaseUrl http://127.0.0.1:19160
python -m json.tool docs/workstreams/official-external-acquisition-transmission-adapter/WORKSTREAM.json
git diff --check -- docs/workstreams/official-external-acquisition-transmission-adapter
```

## Review Result

No blocking findings.

Workstream compliance:

- OETA-010 through OETA-070 are complete.
- Transmission consumes OEAM materialization and does not reopen raw browser
  action input.
- Fixture remains the default local smoke path.
- Live Transmission daemon smoke is optional and not a CI gate.

Code quality:

- RPC transport is behind a fake-testable boundary.
- Runner profile behavior is explicit: fixture and Transmission are separate
  profile paths.
- Status/control operations use `transmission:<hash_string>` and do not call
  materialization.
- Public responses expose safe facts only.

## Residual Risks

- No live Transmission daemon smoke was run. The lane intentionally uses fake
  RPC tests for deterministic CI coverage.
- Cancel stops the torrent but does not remove torrent metadata or downloaded
  data.
- Password-bearing material is rejected until an adapter-safe password/code
  policy exists.
- Only Transmission is implemented; qBittorrent, aria2, generic HTTP, Usenet,
  and cloud-drive flows remain separate lanes.

## Follow-Ons

- Optional live Transmission smoke harness with a disposable daemon.
- Transmission remove/delete policy if product requirements need destructive
  cancellation.
- Password/code handling policy for adapter-safe material that requires access
  codes.
- qBittorrent as the next torrent adapter after the common profile boundary
  proves stable.
