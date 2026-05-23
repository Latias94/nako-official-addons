# Official Metadata Addon Result Quality - TODO

Status: Complete
Last updated: 2026-05-23

Task IDs use the `OMRQ` prefix.

## M0 - Scope

- [x] OMRQ-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-metadata-addon-result-quality]
  Goal: Open the result-quality lane with a bounded scope for candidate
  normalization, ranking, and provider signal refinement.
  Validation: Workstream docs agree.
  Evidence: Workstream docs.
  Result: DONE 2026-05-23.
  Handoff: Continue with OMRQ-020 candidate shaping.

## M1 - Candidate Shaping

- [x] OMRQ-020 [owner=codex] [deps=OMRQ-010] [scope=crates/nako-metadata-scraper/src/engine]
  Goal: Deduplicate exact duplicate provider candidates and cap the final
  result set in the runtime while preserving deterministic ordering and
  redaction-safe evidence.
  Validation: `cargo nextest run -p nako-metadata-scraper ranking evidence --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: Candidate shaping must stay provider-neutral and must not leak raw
  provider payloads.
  Evidence: ranking and runtime tests.
  Result: DONE 2026-05-23.
  Handoff: Continue with OMRQ-030 provider signal quality.

## M2 - Provider Signal Quality

- [x] OMRQ-030 [owner=codex] [deps=OMRQ-020] [scope=crates/nako-metadata-scraper/src/providers]
  Goal: Improve the cheap, safe ranking signals exposed by TMDB and Bangumi
  without changing the protocol contract.
  Validation: targeted provider tests; `cargo nextest run -p nako-metadata-scraper --no-fail-fast`; `git diff --check`.
  Review: Keep the provider runtime generic and avoid adding provider-local
  ordering logic.
  Evidence: provider tests and runtime evidence.
  Result: DONE 2026-05-23.
  Handoff: Continue with OMRQ-040 docs and closeout.

## M3 - Docs And Closeout

- [x] OMRQ-040 [owner=planner] [deps=OMRQ-020,OMRQ-030] [scope=README.md,addons/metadata-scraper,docs/workstreams/official-metadata-addon-result-quality]
  Goal: Update docs to describe the result-quality behavior and close the lane.
  Validation: `cargo fmt --all -- --check`; `cargo nextest run --workspace --no-fail-fast`; `git diff --check`.
  Review: No Admin Web claims, no protocol contract drift.
  Evidence: EVIDENCE_AND_GATES.md and WORKSTREAM.json.
  Result: DONE 2026-05-23.
  Handoff: Prepare the next provider or crawler lane.
