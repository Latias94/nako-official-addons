# Official Metadata Addon Provider Fact Resolver - Handoff

Status: Active
Last updated: 2026-05-25

## Current State

This workstream implements the follow-on chosen by the mature provider model
research lane: a sidecar-local provider fact resolver plus external ID
capability catalog.

## Next Task

Start OMAPFR-030:

- route `suggest_candidates` through resolver clustering;
- preserve `/metadata` response shape and deterministic ordering;
- remove the temporary resolver `dead_code` allow once orchestration consumes it.

## License Guardrails

- Reference repositories under `repo-ref/` are research-only.
- Do not copy reference source, comments, tests, fixtures, file structure, or
  naming structure.
- Implement from local Nako domain requirements and existing local code.

## Risks

- Over-merging provider facts can hide bad upstream IDs.
- Resolver evidence can leak too much if raw query or provider values are
  serialized carelessly.
- Host-owned policy must not move into this sidecar lane.

## Validation Memory

OMAPFR-010 passed with `python -m json.tool docs/workstreams/official-metadata-addon-provider-fact-resolver/WORKSTREAM.json`, `cargo fmt --all -- --check`, `git diff --check`, and ignored `repo-ref/` status confirmation.
OMAPFR-020 passed with `cargo nextest run -p nako-metadata-scraper resolver --no-fail-fast`, `cargo fmt --all -- --check`, and `git diff --check`.
