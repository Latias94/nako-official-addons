# TODO

## OCRA-010 - Open workstream

Status: complete
Owner: Codex
Dependencies: Nako ECAB-050 protocol selection

Scope:

- Create durable workstream docs.
- Fix scope to the official Chromecast renderer adapter sidecar.
- Keep host policy and renderer command envelope ownership in Nako.

Validation:

- Docs exist under this workstream path.

## OCRA-020 - Manifest, config, and packaging skeleton

Status: complete
Owner: Codex
Dependencies: OCRA-010

Scope:

- Add `nako-chromecast-renderer` workspace member.
- Add manifest/config modules driven by official catalog constants.
- Add Dockerfile, compose example, README, and local smoke script.

Validation:

- Manifest unit test validates the runtime manifest.
- Checked-in example manifest matches runtime container manifest.

## OCRA-030 - Renderer adapter resource boundary

Status: complete
Owner: Codex
Dependencies: OCRA-020

Scope:

- Implement `/renderer-adapter` for readiness, target discovery, and command
  dispatch.
- Validate addon resource envelopes before parsing renderer adapter payloads.
- Reject unsafe command transports without echoing sensitive values.

Validation:

- Resource route tests cover readiness, manual discovery, dispatch, and invalid
  envelope rejection.

## OCRA-040 - Chromecast command-plan mapping

Status: complete
Owner: Codex
Dependencies: OCRA-030

Scope:

- Map host command envelopes to a Chromecast command plan.
- Bind command plans to cast-safe HTTP(S) transport URLs only.
- Keep live LAN hardware control optional and outside default CI gates.

Validation:

- Unit tests cover play mapping, pause/resume/seek/stop/volume command mapping,
  wrong protocol rejection, and forbidden transport rejection.

## OCRA-050 - Verify, commit, and handoff

Status: complete
Owner: Codex
Dependencies: OCRA-040

Scope:

- Run package-focused validation.
- Update evidence and handoff.
- Commit only Chromecast sidecar and workstream files.

Validation:

- `cargo nextest run -p nako-chromecast-renderer --no-fail-fast`
- `cargo fmt -p nako-chromecast-renderer -- --check`
- `git diff --check -- <chromecast paths>`
