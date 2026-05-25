# Official Metadata Addon Mature Provider Model Research - Refactor Candidates

Status: Draft
Last updated: 2026-05-25

## Decision Summary

The next fearless refactor should not copy Jellyfin's full host-side provider
manager into `nako-metadata-scraper`. Jellyfin's strongest lesson is the
boundary: providers own external integration, while the host owns library
policy, refresh state, locked fields, local assets, and final writes.

For the official addon, the highest-leverage next lane is a sidecar-local
resolver and fact model. It can improve correctness without requiring Nako core
to adopt Jellyfin semantics immediately.

Recommended next implementation lane:

1. Add a resolver that clusters provider facts by external IDs before ranking.
2. Preserve current `/metadata` response shape while enriching evidence.
3. Follow with provider external-ID capabilities and provider network/cache
   policy once the resolver has a stable fact boundary.

## Current Architecture Snapshot

- Request entry: `routes.rs` builds a `ProviderRegistry`, collects external ID
  aliases, assembles providers, and creates `MetadataScrapeRuntime`.
- Provider contract: `MetadataProvider::suggest` returns
  `ProviderMetadataCandidate` values with a metadata patch, facts, outcomes, and
  artwork candidates.
- Provider registry: catalog entries carry provider ID, enablement env var,
  capabilities, secret references, top-level external ID aliases, network
  diagnostics, config loader, and builder.
- Search flow: provider-local enrichment first tries direct external ID lookup
  and then title-variant search through `search_policy`.
- Ranking flow: orchestration ranks isolated provider candidates, sorts them,
  exact-dedupes `(provider, provider_id)`, and truncates.
- Artwork flow: artwork is attached to metadata candidates and selected by
  metadata confidence plus image area.
- Writeback flow: explicit metadata/artwork writeback payloads go through Nako
  runtime access checks before side-effect submission.
- Bulk flow: bulk scrape is a bounded batch wrapper around the resource scrape
  path, not a library refresh state machine.

## Ranking Scale

- P0: Should be the next implementation workstream or a direct prerequisite.
- P1: Valuable after P0, or useful if the P0 lane naturally exposes the seam.
- P2: Correct direction, but should wait for Nako core protocol or product
  pressure.
- P3: Not recommended now.

## Ranked Candidates

| Rank | Candidate | Recommendation |
| --- | --- | --- |
| P0 | Provider fact resolver and cross-provider merge | Do next |
| P0 | External ID capability catalog | Do with or directly after resolver |
| P1 | Host policy context boundary | Design with Nako core before coding |
| P1 | Provider network/cache/rate policy | Implement after resolver if provider drift or limits become costly |
| P1 | Internal artwork source pipeline | Implement before expanding artwork kinds |
| P2 | Matching strategy objects | Fold into resolver once baseline merge exists |
| P2 | Refresh/local metadata boundary | Keep sidecar light; mostly a Nako core workstream |
| P3 | Full Jellyfin-style provider manager in the addon | Do not do |

## P0 - Provider Fact Resolver And Cross-Provider Merge

Problem:

- `orchestration::suggest_candidates` ranks provider candidates independently
  and deduplicates only exact `(provider, provider_id)` pairs.
- Mature systems merge search results when provider IDs overlap, so multiple
  providers can contribute facts to one real media candidate.

Proposed change:

- Add `engine::resolver`.
- Introduce an internal `ProviderFactSet` or `ResolvedCandidateCluster` shape
  that can carry:
  - provider source and provider ID;
  - normalized external IDs;
  - metadata patch proposal;
  - artwork proposals;
  - provider outcomes and redaction-safe notes.
- Cluster provider facts by exact provider ID and shared external IDs.
- Rank clusters, not isolated rows.
- Build final `MetadataCandidate` output from the strongest cluster while
  preserving provider provenance in evidence.

Affected modules:

- `crates/nako-metadata-scraper/src/engine/orchestration.rs`
- `crates/nako-metadata-scraper/src/engine/ranking.rs`
- `crates/nako-metadata-scraper/src/engine/outcome.rs`
- `crates/nako-metadata-scraper/src/engine/response.rs`
- Provider mappers only if the new fact type cannot initially wrap
  `ProviderMetadataCandidate`.

Risks:

- Merged output can hide which provider supplied which field.
- Cross-provider ID matching can over-merge bad data if an upstream emits a
  wrong external ID.

Risk controls:

- Keep raw provider facts in cluster evidence.
- Start with exact external ID equality only.
- Preserve current patch output shape until resolver behaviour is covered.

Suggested gates:

- Unit test: two provider candidates sharing IMDB ID become one ranked cluster.
- Unit test: candidates with conflicting provider IDs remain separate.
- Unit test: merged evidence remains redaction-safe.
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`
- `cargo fmt --all -- --check`

## P0 - External ID Capability Catalog

Problem:

- Provider catalog entries currently expose top-level aliases for parsing input
  fields, but not a structured description of IDs each provider accepts,
  translates, or emits.
- Direct lookup support is provider-local code, so the orchestrator cannot use
  provider capabilities to plan lookup or deduplication.

Proposed change:

- Extend provider descriptors with `ExternalIdCapability` records:
  - provider namespace;
  - value kind: numeric, URL, opaque string;
  - accepted for direct lookup;
  - emitted in facts;
  - translated through provider API;
  - top-level payload aliases.
- Let `MetadataQuery` parsing continue accepting aliases, but feed the resolver
  from capabilities rather than hard-coded assumptions.

Affected modules:

- `crates/nako-metadata-scraper/src/providers/registry.rs`
- `crates/nako-metadata-scraper/src/engine/query.rs`
- `crates/nako-metadata-scraper/src/providers/tmdb/search.rs`
- `crates/nako-metadata-scraper/src/providers/bangumi/search.rs`
- Provider catalog entries in `tmdb.rs`, `bangumi.rs`, `browser_worker.rs`,
  and later `douban.rs`.

Risks:

- A too-rich capability model can become stale documentation.

Risk controls:

- Keep the first model tiny and executable by tests.
- Use capabilities in query parsing or resolver tests immediately, so the
  descriptor is behaviour, not metadata decoration.

Suggested gates:

- Unit test: descriptor aliases parse top-level IDs exactly as today.
- Unit test: resolver can identify shared IDs from provider-emitted facts.
- Unit test: invalid numeric IDs are rejected by value kind rules.

## P1 - Host Policy Context Boundary

Problem:

- Mature systems use per-library provider order, disabled provider lists,
  refresh mode, locked fields, and local metadata priority.
- The sidecar should not own those decisions, but it currently has no structured
  request context for them beyond the payload and writeback request.

Proposed change:

- Design an optional request policy envelope with Nako core before coding:
  - media kind;
  - provider allow/deny list or provider order;
  - requested field groups;
  - requested artwork kinds;
  - refresh reason or mode;
  - local/locked-field summary if Nako core wants less noisy proposals.
- Sidecar may use this context to filter or annotate suggestions.
- Nako core remains authoritative for locked fields, local assets, field merge,
  and final apply semantics.

Affected modules:

- `crates/nako-metadata-scraper/src/engine/query.rs`
- `crates/nako-metadata-scraper/src/engine/runtime.rs`
- `crates/nako-metadata-scraper/src/engine/writeback.rs`
- Addon protocol types, if the policy becomes shared contract.

Risks:

- Protocol churn can couple the sidecar too tightly to host internals.

Risk controls:

- Make fields optional and forward-compatible.
- Treat sidecar filtering as advisory; Nako core still validates writes.

Suggested gates:

- Contract tests for unknown/missing policy fields.
- Unit test: provider order from policy changes suggestion order only when
  confidence ties or when explicitly configured.
- Unit test: writeback still submits a proposal and Nako core remains the final
  authority.

## P1 - Provider Network, Cache, And Rate Policy

Problem:

- `ProviderHttpRuntime` has timeout, retry, response size, backoff, and proxy
  support, but not provider-specific cache TTLs, rate-limit buckets, token
  refresh locking, or `Retry-After` handling.
- Jellyfin plugins keep these policies close to provider API clients.

Proposed change:

- Add provider-local policy inputs consumed by the shared runtime:
  - cache key and TTL for safe GET/detail calls;
  - throttle key and minimum delay;
  - whether to honor `Retry-After`;
  - per-operation attempt/timeout override.
- Keep provider quirks in provider config or client modules.

Affected modules:

- `crates/nako-metadata-scraper/src/providers/http_runtime.rs`
- `crates/nako-metadata-scraper/src/providers/tmdb/client.rs`
- `crates/nako-metadata-scraper/src/providers/bangumi/client.rs`
- `crates/nako-metadata-scraper/src/providers/rendered_page.rs`
- Provider configs in `config.rs` and provider modules.

Risks:

- Persistent cache invalidation can become a product feature by accident.

Risk controls:

- Start with in-memory or explicitly configured cache only.
- Do not cache authenticated/sensitive responses unless a provider explicitly
  marks them safe.

Suggested gates:

- Runtime unit test for `Retry-After` handling.
- Runtime unit test for cache hit/miss by provider operation key.
- Provider test showing rate/caching policy is passed without leaking secrets.

## P1 - Internal Artwork Source Pipeline

Problem:

- Artwork is currently attached to metadata candidates and inherits metadata
  confidence.
- Mature systems treat image providers as a separate pipeline with per-kind,
  local-first, and provider-order behaviour.

Proposed change:

- Introduce an internal `ArtworkSource` adapter or resolver stage.
- Keep response payload compatible while selecting artwork with:
  - artwork kind;
  - provider/source priority;
  - language;
  - dimensions;
  - local-first context supplied by Nako core when available.

Affected modules:

- `crates/nako-metadata-scraper/src/engine/artwork.rs`
- `crates/nako-metadata-scraper/src/engine/orchestration.rs`
- Provider mappers for TMDB, Bangumi, and Douban.

Risks:

- Splitting too early can add indirection before more artwork kinds exist.

Risk controls:

- Keep provider mapper outputs unchanged at first; adapt them into the artwork
  pipeline inside the engine.
- Only expose new protocol fields after a host use case needs them.

Suggested gates:

- Unit test: metadata ranking and artwork selection can disagree deliberately.
- Unit test: poster/backdrop selection honours kind and resolution.
- Regression test: existing artwork writeback payload shape is preserved.

## P2 - Matching Strategy Objects

Problem:

- `search_title_variants` handles raw, normalized, and qualifier-stripped title
  forms, and providers already perform direct ID lookup.
- Kodi scraper references show additional hard-earned behaviour: manual ID
  lookup, trailing article stripping, year fallback to adjacent years and no
  year, and confidence penalties for fallback routes.

Proposed change:

- Move title/year/direct-ID planning into a small `SearchStrategy` model.
- Let providers opt into strategy pieces they can support.
- Feed strategy outcomes into resolver evidence and confidence deltas.

Affected modules:

- `crates/nako-metadata-scraper/src/engine/title.rs`
- `crates/nako-metadata-scraper/src/providers/search_policy.rs`
- Provider enrichment modules for TMDB and Bangumi.

Risks:

- Matching strategy can become a bag of heuristics if not tied to evidence.

Risk controls:

- Every fallback should emit a score reason or provider outcome.
- Prefer strategy tests over provider-specific duplicated tests.

Suggested gates:

- Unit test: year fallback order is deterministic.
- Unit test: fallback candidates carry lower confidence than exact-year
  candidates.
- Unit test: strategy does not duplicate requests for equivalent title variants.

## P2 - Refresh And Local Metadata Boundary

Problem:

- Bulk scraping is a bounded batch planner, not a mature refresh state machine.
- Local metadata, NFO files, local artwork, locked fields, and refresh
  scheduling are host/library responsibilities in mature systems.

Proposed change:

- Keep `nako-metadata-scraper` as a deterministic suggestion sidecar.
- Add only enough request context for the sidecar to understand refresh intent.
- Open a separate Nako core workstream for local metadata, refresh modes,
  locked fields, and provider order.

Affected modules:

- Sidecar: `engine/bulk.rs`, `routes.rs`, `engine/query.rs`.
- Likely Nako core/protocol modules outside this crate.

Risks:

- Putting refresh state in the sidecar would duplicate host library semantics.

Risk controls:

- Treat sidecar bulk as stateless batch execution.
- Keep scheduling, retry, local-first, and replace-all decisions in Nako core.

Suggested gates:

- Sidecar test: refresh context is accepted and echoed in redaction-safe
  evidence only.
- Core/protocol tests should own locked-field and local metadata semantics.

## P3 - Full Jellyfin-Style Provider Manager In The Addon

Recommendation: do not implement.

Why:

- Jellyfin's provider manager is host-side library infrastructure. It owns item
  lifecycle, local metadata, merge policy, image storage, refresh scheduling,
  and library configuration.
- `nako-metadata-scraper` is an official addon sidecar. Copying the full model
  would over-couple addons to Nako core and make writeback authority ambiguous.

Safer alternative:

- Build the sidecar resolver and provider capability model.
- Design a separate Nako core policy/refresh workstream when host-side metadata
  semantics are ready.
