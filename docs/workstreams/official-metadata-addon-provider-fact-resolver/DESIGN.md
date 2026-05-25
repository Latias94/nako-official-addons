# Official Metadata Addon Provider Fact Resolver

Status: Active
Last updated: 2026-05-25

## Why This Lane Exists

The mature provider model research concluded that `nako-metadata-scraper` has a
good provider extension base, but still ranks isolated provider candidates and
deduplicates only exact `(provider, provider_id)` pairs.

The next fearless refactor should add a sidecar-local resolver that clusters
provider facts by shared external IDs before final ranking. It should also turn
external ID handling into a provider capability catalog instead of only
top-level payload aliases.

## Target State

- Provider suggestions are resolved through an internal provider fact resolver.
- Candidates sharing exact external IDs can be clustered before final ranking.
- Final `/metadata` response shape remains backward compatible.
- Candidate evidence keeps provider provenance redaction-safe.
- Provider external ID capabilities describe aliases, emitted IDs, accepted
  lookup IDs, value kind, and validation rules.
- Existing provider tests continue to pass for fixture, TMDB, Bangumi,
  browser_worker, and Douban.

## License Guardrails

- Reference repositories under `repo-ref/` are research-only inputs.
- Do not copy reference source code, comments, tests, fixture data, file
  structure, or naming structure.
- Reimplement behaviour from first principles using local Nako domain types and
  local tests.
- If a design resembles a mature project concept, document it as an
  architecture pattern, not as copied implementation.
- Keep all new code authored inside this repository and covered by this
  repository's license expectations.

## Scope

- `crates/nako-metadata-scraper/src/engine`
- `crates/nako-metadata-scraper/src/providers/registry.rs`
- Provider catalog entries for TMDB, Bangumi, browser_worker, Douban, and
  fixture where capability descriptors are required.
- Tests for resolver clustering, evidence, query alias compatibility, and
  provider-specific ID validation.
- Workstream docs and README updates if public behaviour is clarified.

## Non-Goals

- Do not add new metadata providers.
- Do not change the public Addon Protocol response shape unless a backward
  compatible evidence field is explicitly justified.
- Do not implement Nako core refresh state, locked fields, local NFO, local
  artwork priority, or final field merge policy in this sidecar.
- Do not add persistent provider cache or rate-limit runtime in this lane.
- Do not run live provider smoke or release packaging gates in this lane.

## Architecture Direction

The sidecar keeps the current provider interface initially:

1. Providers continue returning `ProviderMetadataCandidate`.
2. The engine adapts those candidates into resolver facts.
3. The resolver builds clusters from exact provider IDs and shared external IDs.
4. Ranking produces final `MetadataCandidate` values from resolved clusters.
5. Evidence records source providers and merge reasons without exposing raw
   sensitive query values.

External ID capabilities should be executable metadata:

- query parsing uses alias descriptors;
- providers declare which IDs they emit and accept;
- validation rules such as positive numeric IDs are owned by descriptors;
- resolver uses emitted IDs to cluster candidates.

## Close Criteria

This lane can close when:

- resolver-backed orchestration preserves current output compatibility;
- shared external IDs cluster provider facts before final ranking;
- provider external ID capabilities replace ad hoc alias-only descriptors;
- provenance evidence is redaction-safe and tested;
- full `nako-metadata-scraper` package gate passes;
- no reference repository source is copied or vendored.
