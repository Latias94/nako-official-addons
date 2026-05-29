# Official Metadata Addon Provider Breadth and Localization

Status: Complete
Last updated: 2026-05-23

## Why This Lane Exists

The addon now has a stable provider network policy seam, but TMDB and Bangumi
still leave match quality on the table for localized libraries. The next
durable gain is to deepen provider-local alias coverage, localized title
fallbacks, and artwork selection without drifting into browser automation or
host-side task orchestration.

## Relevant Authority

- ADRs:
  - `docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
  - `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
- Existing docs:
  - `README.md`
  - `crates/nako-metadata-scraper/README.md`
  - `docs/workstreams/official-metadata-addon-provider-hardening/DESIGN.md`
  - `docs/workstreams/official-metadata-addon-result-quality/DESIGN.md`
  - `docs/workstreams/official-metadata-bangumi-provider-baseline/DESIGN.md`
- Reference repositories:
  - `F:/SourceCodes/Rust/repo-ref/nako-scraper/CheckTMDB/README.md`
  - `F:/SourceCodes/Rust/repo-ref/nako-scraper/tinyMediaManager/README.md`
  - `F:/SourceCodes/Rust/repo-ref/nako-scraper/mdcx/README.md`
  - `F:/SourceCodes/Rust/repo-ref/nako-scraper/javinizer-go/README.md`
- Related workstreams:
  - `docs/workstreams/official-metadata-addon-provider-hardening`
  - `docs/workstreams/official-metadata-addon-result-quality`
  - `docs/workstreams/official-metadata-bangumi-provider-baseline`

## Problem

- TMDB and Bangumi have enough data for baseline scraping, but local-language
  libraries still lose match quality when the primary surface title is not the
  best search key.
- Artwork candidates exist, but the selection surface can still be made more
  useful when providers expose richer variants and clearer priority.
- Reference tools show valuable capability patterns such as multi-source
  scraping, multi-language handling, and artwork-heavy media management, but
  their code and generated data are not to be copied.

## Target State

When this lane closes:

- TMDB and Bangumi expose richer provider-local alias and localized title
  coverage.
- Ranking can use those signals without leaking provider-specific logic into
  routes.
- Artwork candidate selection is more deliberate and easier to reason about.
- The addon remains modular and reference-driven without browser automation,
  Douban crawling, or hosts-file/DNS orchestration in this lane.

## In Scope

- `crates/nako-metadata-scraper/src/engine/ranking.rs`
- `crates/nako-metadata-scraper/src/engine/artwork.rs`
- `crates/nako-metadata-scraper/src/providers/tmdb.rs`
- `crates/nako-metadata-scraper/src/providers/bangumi.rs`
- `crates/nako-metadata-scraper/src/providers/registry.rs`
- `crates/nako-metadata-scraper/README.md`
- `addons/metadata-scraper/README.md`
- focused tests and smoke coverage for the above seams

## Out Of Scope

- `addons/browser-worker`
- Douban crawler/browser automation implementation
- Addon Task runtime on the Nako host
- Bulk scrape orchestration inside the addon
- CheckTMDB-style hosts/DNS automation or code copying from reference repos

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Provider-local alias and localized title signals can improve ranking without route changes. | High | `engine/ranking.rs` already owns ranking depth. | Ranking would need a new seam. |
| Artwork candidate selection should remain typed and provider-local. | High | `engine/artwork.rs` already owns typed artwork candidates. | Provider modules would leak selection policy. |
| Reference repos are useful for capability comparison, not implementation reuse. | High | Their licenses vary and the user asked for license-safe reference only. | The lane would need to revisit sourcing rules. |
| Browser automation and Douban should stay in separate lanes. | High | Existing workstreams and current scope boundaries already isolate them. | This lane would absorb unrelated risk. |

## Architecture Direction

Keep title/alias shaping in provider modules and ranking.
Keep artwork candidate shaping typed and provider-local.
Keep route handlers thin and redaction-safe.
Prefer small, independently testable provider-quality slices over broad
multi-source rewrites.

## Closeout Condition

This lane can close when:

- provider-local alias and localized title coverage is deeper,
- artwork selection is visibly better or more explicit,
- evidence gates pass,
- docs reflect the shipped behavior,
- and any remaining breadth is either deferred or split into a narrower lane.

## Closeout Summary

Closed on 2026-05-23.

- Artwork selection now chooses stronger provider candidates instead of relying on first-match
  order.
- TMDB and Bangumi provide richer alternate-title evidence to ranking.
- Shared title normalization supports provider search fallback when the raw title returns no
  candidates.
- Browser automation, Douban-specific crawling, host task orchestration, transliteration, and
  non-empty multi-search merging remain follow-on scope.
