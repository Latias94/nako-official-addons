# Official Metadata Addon AV Native Writeback And Provider Wave 2 - Handoff

Status: Active
Last updated: 2026-05-26

## Current State

The lane is open. The previous AV MDCx parity foundation is complete and
committed at `3c23732`. Official addons are clean and `../nako` has unrelated
identity/access workstream changes that must not be reverted or staged.

## Active Task

- Task ID: OMAV2-020
- Owner: codex
- Files: `../nako/crates/nako-addon-protocol`, `../nako/crates/nako-addon-client`, `../nako/crates/nako-reference-addon`, `../nako/crates/nako-server/src/app/addons/metadata_write.rs`, `../nako/docs/adr`
- Validation: `cargo nextest run -p nako-addon-protocol -p nako-addon-client -p nako-reference-addon addon metadata_write --no-fail-fast`; `cargo nextest run -p nako-server addon_side_effect_metadata_write --no-fail-fast`; `cargo fmt -p nako-addon-protocol -p nako-addon-client -p nako-reference-addon -p nako-server -- --check`
- Status: IN_PROGRESS
- Review: Confirm no compatibility shim remains and graph fields are validated before apply.
- Evidence:

## Decisions Since Last Update

- Break compatibility for the metadata writeback payload.
- Prefer full catalog projection after addon metadata write apply.
- Keep browser-worker as the browser/proxy/session/wait owner.
- Add provider wave 2 only after native writeback and bulk maturity are in
  place.

## Blockers

- None. The dirty `../nako` files are unrelated, but staging must be precise.

## Next Recommended Action

- Implement the breaking Nako metadata writeback payload and focused tests in
  `../nako`, while avoiding unrelated identity/access workstream changes.
