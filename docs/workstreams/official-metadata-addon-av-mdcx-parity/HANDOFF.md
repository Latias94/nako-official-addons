# Official Metadata Addon AV MDCx Parity - Handoff

Status: Complete
Last updated: 2026-05-26

## Current State

The lane is closed. OMAVM-010 through OMAVM-050 are complete. Structured AV
facts are response-side and field-policy-aware, browser-worker has a
proxy/wait/session-intent contract, and `javbus` is wired as the first
disabled-by-default MDCx-inspired fallback provider.

## Active Task

- Task ID: OMAVM-050
- Owner: codex
- Files: workstream docs, metadata scraper docs, browser-worker docs, verification outputs
- Validation: `cargo nextest run -p nako-metadata-scraper --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`; `npm --prefix addons/browser-worker test`; JSON validation; `git diff --check`
- Status: DONE
- Review: Confirmed all follow-ups are explicit and no reference-only source was copied.
- Evidence: 2026-05-26 closeout gates passed: 183 Rust tests passed with 2 skipped, 4 browser-worker tests passed, JSON checks and diff hygiene passed.

## Decisions Since Last Update

- Keep structured AV facts in addon response payloads in this lane.
- Treat Nako writeback persistence for credits/studios/collections as a separate
  protocol/server follow-up because `AddonMetadataPatch` is narrower than
  `CanonicalMetadata`.
- Put proxy/session/wait mechanics in browser-worker rather than duplicating
  browser behavior in every Rust provider.
- Add `javbus` before broader provider waves because it is a high-value broad
  fallback after DMM/JavDB/FC2.

## Blockers

- None.

## Next Recommended Action

- Commit the closed lane, then open a follow-up workstream for either Nako
  protocol persistence or additional AV provider waves.
