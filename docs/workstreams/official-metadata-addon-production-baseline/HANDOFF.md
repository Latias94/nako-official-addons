# Official Metadata Addon Production Baseline — Handoff

Status: Complete
Last updated: 2026-05-23

## Current State

The production baseline workstream is closed. OMAPB-010 through OMAPB-060 are
complete.

Current facts:

- Previous workstream
  `docs/workstreams/official-metadata-addon-fearless-refactor/` is complete.
- `ProviderRegistry`, `MetadataScrapeRuntime`, and `ProviderHttpRuntime` exist.
- TMDB currently supports bounded movie search only and is disabled by default.
- Direct sidecar smoke passed previously against a temporary local sidecar.
- Nako Admin-mediated smoke is scripted but has not been run with a live Nako
  server/admin token in this repository.

## Active Task

- Task ID: none
- Owner: planner
- Status: CLOSED
- Evidence: See EVIDENCE_AND_GATES.md.

Previous task:

- Task ID: OMAPB-060
- Status: DONE
- Evidence:
  - `python -m json.tool` passed for `WORKSTREAM.json`.
  - `cargo fmt --all -- --check` passed.
  - `git diff --check` passed with only a Cargo.lock line-ending warning.
  - `cargo nextest run -p nako-metadata-scraper --no-fail-fast` passed with
    31 tests.
  - `cargo nextest run --workspace --no-fail-fast` passed with 31 tests.

Previous implementation task:

- Task ID: OMAPB-050
- Status: DONE
- Evidence:
  - README/addon README updated for TMDB baseline and ranking/evidence truth.
  - `cargo nextest run -p nako-metadata-scraper --no-fail-fast` passed with
    31 tests after TMDB changes.

Earlier task:

- Task ID: OMAPB-040
- Status: DONE
- Evidence:
  - `cargo nextest run -p nako-metadata-scraper tmdb --no-fail-fast` passed.
  - TMDB fake transport test proves search/detail/external-ID calls and mapping.

Earlier task:

- Task ID: OMAPB-030
- Status: DONE
- Evidence:
  - `cargo nextest run -p nako-metadata-scraper ranking evidence --no-fail-fast`
    passed with 5 tests.
  - `cargo fmt --all -- --check` passed.
  - `cargo nextest run -p nako-metadata-scraper --no-fail-fast` passed with
    31 tests.

Earlier task:

- Task ID: OMAPB-020
- Status: DONE_WITH_CONCERNS
- Evidence:
  - Direct sidecar smoke passed against `127.0.0.1:19101`.
  - Live Nako Admin-mediated smoke was not run because `127.0.0.1:3000`
    refused connections and `NAKO_ADMIN_TOKEN` was unset.

## Decisions

- Execute smoke before TMDB breadth to prove the Nako boundary.
- Implement ranking/evidence before expanding TMDB so provider breadth uses a
  shared scoring policy.
- Keep all default TMDB tests synthetic and fake-transport based.
- Providers now report normalized candidate facts. `MetadataScrapeRuntime`
  owns final confidence scoring and deterministic sorting through
  `engine::ranking`.
- TMDB now enriches bounded movie search results with movie detail and external
  IDs before returning provider facts to the runtime.

## Residual Risks

- Live Nako Admin smoke still requires an operator-started local Nako server
  and administrator token. It is recorded as an external evidence gap, not a
  blocker for the completed ranking/evidence or TMDB baseline.

## Next Recommended Action

- Open the next workstream for one of:
  - live Nako Admin-mediated smoke evidence;
  - Bangumi/Douban provider adapters;
  - artwork/subtitle provider lanes;
  - rename planning and NFO-compatible workflows;
  - bulk scrape, provider scoring feedback, and ranking hardening.
