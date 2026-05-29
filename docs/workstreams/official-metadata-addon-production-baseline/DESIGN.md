# Official Metadata Addon Production Baseline

Status: Complete
Last updated: 2026-05-23

## Why This Lane Exists

The previous architecture lane closed with a working official metadata Addon
Sidecar architecture: configuration and manifest truth are unified, provider
construction lives behind `ProviderRegistry`, metadata orchestration lives
behind `MetadataScrapeRuntime`, outbound provider calls share
`ProviderHttpRuntime`, and TMDB movie search exists as a disabled-by-default
proof.

The next product step is to prove the whole Nako integration and turn TMDB from
a search-only proof into a credible production baseline. Before adding broad
provider count, the addon also needs a stable ranking and evidence model so
future Bangumi, Douban, artwork, subtitle, rename, and bulk-scrape lanes do not
copy ad hoc confidence logic.

## Problem

1. Direct sidecar smoke is verified, but the full Nako Admin-mediated flow has
   not been run against a live local Nako server.
2. `CandidateEvidence` is too shallow for production ranking. It only records
   title/year booleans and a note.
3. Candidate confidence is provider-local today. TMDB and fixture assign scores
   independently, so adding more providers would harden arbitrary ordering.
4. TMDB currently uses only `/search/movie`. It does not fetch movie details,
   external IDs, runtime, tagline, richer genres, production facts, or image
   metadata artifacts.
5. The Addon Protocol metadata patch is intentionally small. Production TMDB
   baseline must respect that boundary and put non-patch facts into artifacts
   without inventing direct canonical writes.

## Target State

When this lane closes:

- A local operator has a repeatable Nako Admin-mediated smoke path with recorded
  evidence: register/reuse manifest, run health check, optionally enable, run
  metadata resource diagnostic, and verify redaction-safe responses.
- Candidate ranking is centralized in the metadata runtime or a dedicated
  ranking module, not spread across providers.
- Candidate evidence records the reasons for confidence in a provider-neutral,
  redaction-safe shape.
- TMDB provider baseline covers movie search plus selected detail enrichment
  through the shared HTTP runtime.
- TMDB output remains within Nako Addon boundaries: candidate `patch` contains
  supported metadata fields; richer provider facts are emitted as safe
  artifacts for future Nako intake/acceptance work.
- Default tests use synthetic fake HTTP responses only. No live TMDB network
  test runs unless explicitly gated by environment.

## In Scope

- Local Nako Admin-mediated smoke documentation, script hardening, and evidence.
- Provider-neutral ranking and evidence types.
- Runtime-owned candidate scoring/sorting policy.
- TMDB movie detail enrichment for:
  - runtime minutes;
  - tagline;
  - overview/title/original title/release date;
  - genres from detail response;
  - selected external IDs;
  - poster/backdrop metadata as safe artifacts or candidate evidence facts.
- Tests with synthetic TMDB search/detail/external-id responses.
- README and workstream evidence updates.

## Out Of Scope

- Live TMDB calls in default test gates.
- Bangumi, Douban, IMDb, TVDb, FanArt, subtitle, or artwork provider
  implementation in this lane.
- Addon Protocol schema expansion unless an existing protocol field is
  insufficient for baseline metadata suggestions.
- Direct canonical metadata writes or media-library mutation.
- Nako Admin Web UI changes.
- Addon Manager installation/process supervision.
- Copying code, fixtures, schemas, selectors, or generated artifacts from
  reference repositories.

## Architecture Direction

Preferred module direction:

1. `engine::ranking`
   - Provider-neutral scoring inputs and scoring policy.
   - Stable confidence range and tie-break behavior.
   - Evidence reasons that explain the score without leaking raw provider
     payloads.
2. `engine::candidate`
   - Candidate payload and artifacts should be shaped in one place.
   - Providers return normalized raw provider candidates with facts; runtime
     scores and emits final response shape.
3. `providers::tmdb`
   - Search for candidate IDs.
   - Fetch detail and external IDs only for bounded top search results.
   - Map provider responses into Nako-owned metadata fields and safe artifacts.
   - Keep HTTP policy entirely in `ProviderHttpRuntime`.
4. `addons/metadata-scraper/smoke.local.ps1`
   - Remains the local operator entrypoint.
   - Direct smoke is default; Nako Admin mutation remains explicit.

## Follow-On Split Rules

Split rather than expand this lane if:

- Addon Protocol needs new stable resource or patch fields;
- live Nako server setup needs changes in `../nako`;
- TMDB baseline requires broad series/episode support;
- artwork write/proposal needs a new product surface;
- provider scoring needs persisted Nako-side feedback or accepted/rejected
  mapping state.

## Closeout Condition

This lane can close when:

- workstream evidence records direct sidecar smoke and either live
  Nako-mediated smoke or a concrete external-blocker reason;
- ranking/evidence is centralized and covered by tests;
- TMDB baseline has synthetic tests proving search/detail/external-ID mapping
  and bounded HTTP calls;
- docs and examples describe the new provider behavior truthfully;
- `cargo fmt --all -- --check`, targeted nextest gates, workspace nextest, and
  `git diff --check` pass.

## Closeout Summary

Closed on 2026-05-23.

This lane completed the requested next phase:

- direct sidecar smoke was verified again; live Nako Admin-mediated smoke was
  attempted as a preflight but remained externally blocked by no local Nako
  server and no admin token;
- provider-neutral ranking/evidence now lives in `engine::ranking`;
- providers return normalized candidate facts instead of final scores;
- `MetadataScrapeRuntime` owns final confidence and deterministic sorting;
- TMDB baseline now enriches bounded movie search results with movie detail and
  external IDs through the shared HTTP runtime;
- docs describe the current TMDB baseline and do not claim future provider
  support.

Remaining follow-ons are live Nako Admin-mediated evidence, Bangumi/Douban
adapters, artwork/subtitle lanes, rename/NFO planning, and bulk
scrape/scoring hardening.
