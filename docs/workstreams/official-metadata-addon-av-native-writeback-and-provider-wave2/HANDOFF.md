# Official Metadata Addon AV Native Writeback And Provider Wave 2 - Handoff

Status: Active
Last updated: 2026-05-26

## Current State

The lane is active. The previous AV MDCx parity foundation is complete and
committed at `3c23732`. OMAV2-020 is complete in `../nako` at commit
`a0ad9a8`, adding native graph metadata writeback and full catalog projection.
OMAV2-030 is complete in official addons: selected AV facts now materialize into
native metadata patch graph fields.
OMAV2-040 is complete in official addons: bulk scrape now reports retry
classes, explicit provider suppression/cooldown state, and per-item suppressed
providers without adding hidden scheduler state to Nako.
`../nako` still has unrelated identity/access workstream changes that must not
be reverted or staged.

## Active Task

- Task ID: OMAV2-050
- Owner: codex
- Files: `crates/nako-metadata-scraper/src/providers`, `crates/nako-metadata-scraper/src/config.rs`, `crates/nako-metadata-scraper/src/manifest.rs`, `addons/metadata-scraper`
- Validation: `cargo nextest run -p nako-metadata-scraper config registry manifest av javlibrary --no-fail-fast`
- Status: READY
- Review: Confirm selectors/parsers are independently implemented from reference-only MDCx code.
- Evidence:

## Decisions Since Last Update

- Break compatibility for the metadata writeback payload.
- Prefer full catalog projection after addon metadata write apply.
- Add Nako ADR `0035-addon-native-metadata-writeback`.
- Materialize selected AV facts into native `AddonMetadataPatch` graph fields:
  credits, studios, collections, external IDs, and image references.
- Keep browser-worker as the browser/proxy/session/wait owner.
- Keep bulk provider suppression as explicit task input/output state; Nako owns
  task scheduling, retry, progress, and cancellation.
- Add provider wave 2 only after native writeback and bulk maturity are in
  place.

## Blockers

- None. The dirty `../nako` files are unrelated, but staging must be precise.

## Next Recommended Action

- Add provider wave 2 behind disabled-by-default config, starting with
  JavLibrary plus one high-value route-specific provider.
