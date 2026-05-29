# Official Metadata Addon Provider Architecture Deepening - Handoff

Status: Complete
Last updated: 2026-05-25

## Current State

The user approved doing all five architecture candidates with fearless
refactoring for provider/plugin extensibility:

1. Provider descriptor ownership.
2. Shared search-enrichment policy.
3. Browser-worker support Seam.
4. Typed provider outcomes.
5. Single provider assembly.

This workstream is the authoritative execution lane for that Goal.

## Completed

OMAPAD-020 completed:

- `ProviderCatalogEntry` now carries default enablement, enablement env var,
  provider config loader, and proxy health fact function.
- `Config` builds provider configs from the provider catalog instead of
  hard-coding `ProviderId::ALL` and every provider-specific config branch.
- Targeted provider/config/manifest tests, format check, and diff hygiene
  passed.

OMAPAD-030 completed:

- `ProviderRegistry` exposes one `ProviderAssembly` for ready adapters,
  diagnostics, and provider-owned network policy facts.
- Routes consume provider diagnostics instead of reading provider-specific
  proxy facts from `Config`.

OMAPAD-040 completed:

- `providers/search_policy.rs` owns the common direct-lookup, title-variant
  search, dedupe, ranking-budget, partial-search, and degraded-fallback
  orchestration.
- TMDB and Bangumi pass provider-local callbacks for HTTP search, raw response
  parsing, result IDs, enrichment, degraded candidate mapping, and provider
  notes.
- The shared policy is closure-driven so raw TMDB/Bangumi result types are not
  promoted into a public shared trait contract.

OMAPAD-050 completed:

- `engine/outcome.rs` defines typed `ProviderOutcome` facts and renders the
  existing public `provider_note` evidence text from one redaction-safe module.
- TMDB, Bangumi, Douban, Fixture, BrowserWorker, and shared search policy now
  emit outcomes instead of provider-local note prose.
- `ProviderCandidateFacts.provider_note` remains as a compatibility fallback
  for tests and non-migrated providers, but current built-in providers no
  longer use it for diagnostic prose.

OMAPAD-060 completed:

- `providers/rendered_page.rs` owns browser-worker render/extract request
  construction, endpoint joining, response parsing, and status validation.
- Douban now asks the rendered-page runtime for HTML and keeps only Douban
  search URL construction plus Douban HTML parsing/mapping locally.
- `browser_worker` remains a provider identity because it has explicit metadata
  semantics: when a request carries a browser-worker URL external id, it
  returns a rendered-page metadata candidate. It is still default-off and now
  uses the same support runtime instead of owning protocol plumbing.

OMAPAD-070 completed:

- Full metadata scraper package gate passed after the provider descriptor,
  single assembly, shared search policy, typed outcome, and rendered-page
  support refactors.
- `crates/nako-metadata-scraper/README.md` and
  `addons/metadata-scraper/README.md` already describe the public
  browser-worker and Douban rendered-page strategy; no config/doc behavior
  change was required.

## Next Task

No next task in this lane.

Closeout result:

- Review found no blocking workstream compliance or code-quality findings.
- Final package, format, JSON, and diff hygiene gates passed.
- No architecture follow-on was split; release publishing and live provider
  smoke remain out of scope.

## Risks

- Provider config refactoring can accidentally change manifest defaults or
  secret reference ordering.
- Search policy extraction can overreach and pull raw TMDB/Bangumi payload
  shapes across the Seam. Keep raw parsing provider-local.
- Removing or demoting `browser_worker` provider identity may require docs and
  manifest example updates.
- Typed provider outcomes should preserve public payload compatibility unless a
  schema change is intentionally documented.

## Validation Memory

OMAPAD-010 passed with `python -m json.tool docs/workstreams/official-metadata-addon-provider-architecture-deepening/WORKSTREAM.json > $null`.
OMAPAD-020 passed with `cargo fmt --all -- --check`, `cargo nextest run -p nako-metadata-scraper provider registry config addon_manifest --no-fail-fast`, and `git diff --check`.
OMAPAD-030 passed with `cargo fmt --all -- --check`, `cargo nextest run -p nako-metadata-scraper provider health_endpoint diagnostics --no-fail-fast`, and `git diff --check`.
OMAPAD-040 passed with `cargo fmt --all -- --check`, `cargo nextest run -p nako-metadata-scraper tmdb bangumi relevance partial degraded --no-fail-fast`, and `git diff --check`.
OMAPAD-050 passed with `cargo fmt --all -- --check`, `cargo nextest run -p nako-metadata-scraper provider_note redaction ranking tmdb bangumi douban --no-fail-fast`, and `git diff --check`.
OMAPAD-060 passed with `cargo fmt --all -- --check`, `cargo nextest run -p nako-metadata-scraper browser_worker douban --no-fail-fast`, and `git diff --check`.
OMAPAD-070 passed with `cargo fmt --all -- --check`, `cargo nextest run -p nako-metadata-scraper --no-fail-fast`, and `git diff --check`.
OMAPAD-080 passed with `cargo fmt --all -- --check`, `python -m json.tool docs/workstreams/official-metadata-addon-provider-architecture-deepening/WORKSTREAM.json`, `cargo nextest run -p nako-metadata-scraper --no-fail-fast`, and `git diff --check`.
