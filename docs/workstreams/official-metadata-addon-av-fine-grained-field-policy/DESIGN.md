# Official Metadata Addon AV Fine-Grained Field Policy

Status: Complete
Last updated: 2026-05-27

## Why This Lane Exists

The addon already supports request-level `provider_field_policy` and configurable AV field policy
presets, but the default AV preset still groups many fields into coarse title/text/fact/community
orders. Mature AV scrapers tend to use a field-by-field source matrix because provider reliability
differs sharply between title, outline, tag, release, runtime, score, actor, artwork, trailer,
studio, publisher, series, and wanted count.

This lane converts the default AV policy from broad groups into an explicit field matrix adapted to
the providers this project actually supports.

## Reference Boundary

- Reference-only upstream: `repo-ref/mdcx/mdcx/config/v1.py`
- Use only the provider-order intent exposed by configuration names and values.
- Do not copy implementation code, selectors, regexes, comments, fixtures, or crawler structure.

## Target State

- Default AV field policy has distinct provider order constants for title, outline, actors, thumb,
  poster, extra fanart, trailer, tags, release date, runtime, score, director, series, studio,
  publisher, and wanted count.
- Unsupported reference providers are intentionally omitted from defaults instead of becoming dead
  configuration entries.
- Score-like facts participate in provider-field fusion and produce redaction-safe field source
  evidence.
- Request-level overrides can control score by either the internal field names or the user-facing
  `score` alias.
- Quality-score preset keeps descriptor-derived behavior and now includes score/vote facts.

## In Scope

- `ProviderRegistry` default AV provider priority matrix.
- `ProviderFieldPolicy` quality descriptor field coverage.
- `engine/fusion.rs` score/vote fact fusion and field-source evidence.
- Focused unit/runtime tests for default policy, request override, score alias, and quality preset.
- README/workstream docs that describe the field names operators can configure.

## Out Of Scope

- Adding new AV providers.
- Changing provider enablement presets.
- Browser-worker/Crawlee scraping behavior.
- Live website drift fixes beyond compile-time/runtime fake-provider coverage.

## Architecture Direction

Keep selection policy outside providers:

- Providers expose normalized facts.
- `ProviderRegistry` owns default provider priority because it already owns supported provider IDs.
- `ProviderFieldPolicy` remains the transport-neutral map from field to provider order.
- `fusion.rs` applies policy uniformly to patch fields, AV facts, artwork, and score-like facts.

This keeps the next provider wave simple: adding a provider should require mapper/parser work plus
one explicit choice about which fields it is trusted for by default.

## Closeout Condition

Closed on 2026-05-27 after shipping a field-by-field default AV provider matrix, score/vote fact
fusion, request aliases, docs, and passing verification gates.
