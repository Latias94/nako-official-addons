# Official Metadata Addon AV Field Policy Presets

Status: Complete
Last updated: 2026-05-27

## Why This Lane Exists

AV scraping now has broad provider coverage and request-level `provider_field_policy`, but the
default field fusion policy is still derived from coarse provider quality scores. Mature AV
scrapers use a more operator-oriented model: each metadata field can prefer a different source
order, such as title from ThePornDB/MGStage/DMM, actors from ThePornDB/JavBus/JavLibrary/JavDB,
and samples from JavBus-like community pages.

## Relevant Authority

- Related workstreams:
  - `docs/workstreams/official-metadata-addon-av-provider-policy/`
  - `docs/workstreams/official-metadata-addon-av-mdcx-parity/`
  - `docs/workstreams/official-metadata-addon-av-javbus-field-quality/`
- Reference-only upstream:
  - `repo-ref/mdcx/mdcx/config/v1.py`
- License guardrail:
  - `repo-ref/mdcx` is GPLv3/reference-only; do not copy source, comments, fixtures, selector
    implementations, regex tables, or structure.

## Problem

Operators can override field preference per request, but they cannot choose a durable field policy
preset through addon configuration or manifest settings.

## Target State

- A runtime default field policy can be selected by config.
- The default preset maps this project's supported provider IDs and field names explicitly.
- Request payload `provider_field_policy` remains the narrowest override.
- Manifest and README docs explain the supported presets and precedence.
- Tests prove config parsing, registry policy construction, and runtime field fusion.

## In Scope

- Add an AV field policy preset enum/config value.
- Add a default preset using supported providers only.
- Wire the selected preset into routes/runtime default policy.
- Update manifest schema and docs.
- Add focused tests for policy construction and fusion behavior.

## Out Of Scope

- Adding new AV providers.
- Solving JavBus age verification without a real cookie/session.
- Adding UI beyond manifest/config exposure.
- Copying reference implementation details.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Existing `ProviderFieldPolicy` is the correct ownership boundary for field source order. | High | `engine/fusion.rs` already consults the policy per fused field. | If wrong, this lane needs an ADR-level fusion redesign. |
| Reference field source order should be adapted, not copied literally. | High | Provider sets differ and `repo-ref/mdcx` is GPLv3/reference-only. | Literal parity would be legally and technically wrong. |
| Default behavior can break cleanly if it improves operator ergonomics. | Medium | User explicitly accepted breaking changes for clean architecture. | Tests and docs must be updated to make the new default explicit. |

## Architecture Direction

Keep provider enablement presets and field-fusion presets separate:

- `AvProviderPreset` decides which providers are enabled by default.
- A new field policy preset decides how facts from selected providers are merged.
- `ProviderRegistry` owns default policy construction because it already owns provider catalog
  metadata and provider IDs.
- `MetadataScrapeRuntime` continues accepting an already-built `ProviderFieldPolicy`; it should not
  parse env/config.

## Closeout Condition

This lane can close when:

- the selected default policy is configurable and documented,
- the default preset is covered by tests,
- runtime fusion uses the selected default unless the request overrides it,
- evidence gates pass,
- and follow-on provider expansion is explicitly deferred.

Closed on 2026-05-27 after shipping configurable AV field policy presets,
manifest/README exposure, and runtime fusion coverage.
