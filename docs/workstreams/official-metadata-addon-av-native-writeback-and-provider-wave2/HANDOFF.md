# Official Metadata Addon AV Native Writeback And Provider Wave 2 - Handoff

Status: Active
Last updated: 2026-05-26

## Current State

The lane is active. The previous AV MDCx parity foundation is complete and
committed at `3c23732`. OMAV2-020 is complete in `../nako` at commit
`a0ad9a8`, adding native graph metadata writeback and full catalog projection.
OMAV2-030 is complete in official addons: selected AV facts now materialize into
native metadata patch graph fields.
`../nako` still has unrelated identity/access workstream changes that must not
be reverted or staged.

## Active Task

- Task ID: OMAV2-040
- Owner: codex
- Files: `crates/nako-metadata-scraper/src/engine/bulk.rs`, `addons/metadata-scraper/README.md`, `crates/nako-metadata-scraper/README.md`
- Validation: `cargo nextest run -p nako-metadata-scraper bulk --no-fail-fast`
- Status: IN_PROGRESS
- Review: Confirm bulk remains stateless from Nako's perspective and does not add a hidden scheduler.
- Evidence:

## Decisions Since Last Update

- Break compatibility for the metadata writeback payload.
- Prefer full catalog projection after addon metadata write apply.
- Add Nako ADR `0035-addon-native-metadata-writeback`.
- Materialize selected AV facts into native `AddonMetadataPatch` graph fields:
  credits, studios, collections, external IDs, and image references.
- Keep browser-worker as the browser/proxy/session/wait owner.
- Add provider wave 2 only after native writeback and bulk maturity are in
  place.

## Blockers

- None. The dirty `../nako` files are unrelated, but staging must be precise.

## Next Recommended Action

- Add bulk retry classes, provider suppression/cooldown hints, and resume-safe
  provider state.
