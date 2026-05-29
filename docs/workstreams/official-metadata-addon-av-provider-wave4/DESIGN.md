# Official Metadata Addon AV Provider Wave 4

Status: Complete
Last updated: 2026-05-26

## Why This Lane Exists

MDCx shows that mature AV scraping is not one provider with perfect parsing. It
is a batch-friendly source matrix: official sources where possible, community
sources for breadth, route-aware provider ordering, field-level preference, and
operational controls for proxy/cookie drift. Nako already has the provider
registry, field policy, presets, browser-worker rendering, proxy policy, and
redaction-safe drift harness. The next gap is provider breadth without
duplicating one-off rendered crawler code for every site.

## Reference Boundary

`repo-ref/mdcx` is reference-only. This lane uses it to identify high-level
coverage gaps and scraping strategy:

- MDCx includes no-token community/official-ish sources such as AirAV, AVSox,
  and XCity in addition to the providers Nako already has.
- MDCx separates website lists by media class and field type instead of trusting
  one provider for every field.
- MDCx treats providers as batch fallbacks, not as interactive single-lookups.

This lane does not copy MDCx implementation details, selectors, regex tables,
comments, fixtures, or file structure.

## Target State

When this lane closes:

- Nako has a reusable rendered-search AV provider base for search-page plus
  detail-page sources;
- the first Wave 4 providers are thin site definitions rather than copied
  provider runtimes;
- provider config, manifest schema/example, presets, field policy, registry
  diagnostics, external-id capabilities, and drift harness all know the new
  providers;
- tests prove provider routing, render contract, parsing, config loading, and
  field policy integration;
- docs explain why these providers exist and how proxy/browser-worker support
  applies.

## Provider Slice

Initial Wave 4 implementation targets no-token providers that fit the current
browser-worker rendered HTML architecture:

- `airav`: broad Asian AV catalog fallback used by MDCx for multiple AV routes.
- `avsox`: broad community fallback with AV-number search URL semantics.
- `xcity`: official-ish censored catalog source with compact-number search.

ThePornDB, Getchu, domestic providers, and form-post/search providers remain
follow-ups because they introduce token/hash workflows, different content
domains, or non-GET search mechanics.

## Architecture Direction

Add a shared `rendered_search_av` provider base:

- site definitions declare provider ID, URL external-id alias, env vars, base
  URL, search URL shape, supported routes, field-quality scores, capabilities,
  outcome, and tagline;
- runtime/rendering, direct URL lookup, search result filtering, detail parsing,
  candidate construction, artwork candidates, and config loading are shared;
- individual provider modules remain small and own only their public provider
  identity.

This keeps Wave 5 provider work cheap: adding another GET-search rendered
source should be a site definition plus focused fixture tests, not another
bespoke runtime.

## Closeout Condition

This lane can close when:

- AirAV, AVSox, and XCity are implemented and disabled by default;
- `community_first` includes the broad community fallbacks where appropriate;
- field-policy tests include the new provider qualities;
- manifest/example/docs and drift harness cover the new provider IDs;
- targeted and package validation pass;
- workstream evidence is current.

## Closeout Result

Complete on 2026-05-26. The lane added a reusable rendered-search AV provider
base plus AirAV, AVSox, and XCity thin providers. Config, provider presets,
manifest schema/example, field policy, diagnostics, external-id aliases, manual
drift support, README docs, and workstream evidence are current. Full package,
fmt, JSON, and diff hygiene gates passed.
