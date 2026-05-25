# Official Metadata Addon Provider Fact Resolver - Handoff

Status: Active
Last updated: 2026-05-25

## Current State

This workstream implements the follow-on chosen by the mature provider model
research lane: a sidecar-local provider fact resolver plus external ID
capability catalog.

OMAPFR-040 is complete. Provider catalog entries now own executable external ID
capabilities, runtime query parsing uses those capabilities, legacy alias
exposure is derived from the catalog for compatibility, and resolver clustering
uses declared emitted external IDs when catalog data is available.

OMAPFR-050 is complete. The full `nako-metadata-scraper` package gate passed,
and README docs now describe capability-derived external ID aliases plus shared
external-ID candidate resolution without changing the protocol envelope.

## Next Task

Start OMAPFR-060:

- review the completed lane;
- rerun closeout validation from TODO;
- close the workstream or split explicit follow-ons.

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
OMAPFR-030 passed with `cargo nextest run -p nako-metadata-scraper resolver orchestration ranking --no-fail-fast`, `cargo fmt --all -- --check`, and `git diff --check`.
OMAPFR-040 passed with `cargo nextest run -p nako-metadata-scraper external_id tmdb bangumi browser_worker resolver --no-fail-fast`, `cargo fmt --all -- --check`, and `git diff --check`.
OMAPFR-050 passed with `cargo nextest run -p nako-metadata-scraper --no-fail-fast`, `cargo fmt --all -- --check`, and `git diff --check`.
