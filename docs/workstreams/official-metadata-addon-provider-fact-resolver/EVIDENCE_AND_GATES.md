# Official Metadata Addon Provider Fact Resolver - Evidence And Gates

Status: Active
Last updated: 2026-05-25

## Gate Plan

| Gate | Command | When |
| --- | --- | --- |
| Workstream JSON | `python -m json.tool docs/workstreams/official-metadata-addon-provider-fact-resolver/WORKSTREAM.json` | OMAPFR-010 and closeout |
| Resolver unit tests | `cargo nextest run -p nako-metadata-scraper resolver --no-fail-fast` | OMAPFR-020 and OMAPFR-030 |
| External ID tests | `cargo nextest run -p nako-metadata-scraper external_id tmdb bangumi browser_worker resolver --no-fail-fast` | OMAPFR-040 |
| Package gate | `cargo nextest run -p nako-metadata-scraper --no-fail-fast` | Integration and closeout |
| Format | `cargo fmt --all -- --check` | Before commits and closeout |
| Diff hygiene | `git diff --check` | Before commits and closeout |

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-25 | OMAPFR-010 scope | Workstream opened with explicit license guardrails: reference repositories are research-only, no copied source/comments/tests/fixtures/structure, implementation must be authored from local Nako domain model. Validated `WORKSTREAM.json`, ran `cargo fmt --all -- --check`, ran `git diff --check`, and confirmed `repo-ref/` remains ignored. | Pass |
| 2026-05-25 | OMAPFR-020 resolver model | Added local `engine::resolver` model that adapts `ProviderMetadataCandidate` values into resolver facts, clusters exact provider identities and shared external IDs, and emits redaction-safe cluster evidence without raw external ID values. Validation: `cargo nextest run -p nako-metadata-scraper resolver --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`. | Pass |
| 2026-05-25 | OMAPFR-030 resolver-backed orchestration | Routed `suggest_candidates` through resolver clustering before final ranking. Shared external IDs now collapse to one selected ranked candidate, exact provider identity dedupe remains covered, and the existing `/metadata` candidate response shape is preserved. Validation: `cargo nextest run -p nako-metadata-scraper resolver orchestration ranking --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`. | Pass |
| 2026-05-25 | OMAPFR-040 external ID capability catalog | Replaced provider-owned alias arrays with provider external ID capabilities for TMDB, Bangumi, browser worker, Douban, and fixture. Runtime parsing now consumes capabilities, the registry derives legacy aliases for compatibility, and resolver clustering filters shared IDs through declared `emits` capabilities when a catalog is provided. Validation: `cargo nextest run -p nako-metadata-scraper external_id tmdb bangumi browser_worker resolver --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`. | Pass |

## Notes

- Reference repositories under `repo-ref/` must remain ignored.
- Live provider smoke and release packaging are out of scope.
- Host-owned refresh, locked fields, local metadata, local artwork priority, and
  final merge policy are out of this sidecar lane.
