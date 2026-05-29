# Official Metadata Addon AV Ops Presets And Drift

Status: Complete
Last updated: 2026-05-26

## Why This Lane Exists

The AV provider stack now has enough breadth that users need operational
choices, not only individual provider toggles. MDCx-style mature scraping is a
mix of source breadth, defaults that match common media classes, batch-friendly
behavior, proxy/cookie support, and drift monitoring. Nako already has provider
toggles, browser-worker rendering, proxy policy, field fusion, and bulk scrape;
the missing layer is an explicit provider preset and a redaction-safe way to
check whether live providers still return expected fields.

## Target State

When this lane closes:

- AV users can pick a named provider preset that enables a coherent set of AV
  providers while preserving explicit per-provider overrides;
- manifest/runtime docs expose the presets and the default behavior clearly;
- manual live drift checks can be run outside CI and report only provider IDs,
  field presence, and counts, never raw adult-site titles, URLs, or actor names;
- the drift harness covers both direct HTTP providers and browser-worker
  rendered AV providers through the existing provider seams;
- tests prove preset precedence and redaction-safe drift summaries.

## In Scope

- `crates/nako-metadata-scraper/src/config.rs`
- `crates/nako-metadata-scraper/src/manifest.rs`
- `crates/nako-metadata-scraper/tests/live_provider_drift.rs`
- README and metadata-scraper addon docs.
- This workstream's ledger and evidence.

## Out Of Scope

- Adding Wave 4 AV providers before preset/drift operations are stable.
- Copying MDCx source, selectors, regex tables, comments, fixtures, or
  configuration text. MDCx remains reference-only.
- Running live adult-site scraping in CI.
- Persisting adult payloads, screenshots, HTML, titles, actor lists, or source
  URLs in the repository.
- Reworking Nako core configuration UI.

## Architecture Direction

Put preset semantics in the configuration layer. Provider modules should not
know about presets; registry should keep catalog assembly and diagnostics. A
preset is a default enablement policy for AV provider IDs only. Explicit
provider environment variables remain the last-mile override so operators can
disable a fragile source or add a specialty source without inventing another
preset.

Drift checks should exercise the same `MetadataProvider::suggest` seam used by
runtime scraping. The test harness may use env-provided case lists and ignored
tests, but the report must summarize only field health:

- provider ID;
- whether at least one candidate returned;
- required/optional field names that are present or missing;
- artwork/trailer/external-id counts;
- error kind/message without source payload values.

## Preset Semantics

The initial preset vocabulary is intentionally small:

- `manual`: current behavior; providers use catalog defaults and explicit
  provider overrides only.
- `fast_safe`: a conservative default for common censored, FC2, amateur, and
  Prestige flows: `javdb`, `dmm`, `fc2`, `mgstage`, `prestige`.
- `official_only`: official-ish sources where available:
  `dmm`, `fc2`, `mgstage`, `prestige`, `caribbean`, `1pondo`, `10musume`.
- `community_first`: broad community plus official fallback:
  `javdb`, `javbus`, `javlibrary`, `dmm`, `fc2`, `fc2ppvdb`, `mgstage`,
  `prestige`.
- `fc2_enhanced`: FC2 direct plus FC2PPVDB fallback: `fc2`, `fc2ppvdb`.
- `uncensored_official`: official uncensored sites:
  `caribbean`, `1pondo`, `10musume`.

## Closeout Condition

This lane can close when:

- presets are implemented, documented, and tested;
- manifest example/schema exposes the preset control;
- live drift harness can run manually against env-provided AV cases and has a
  deterministic redaction-safety unit test;
- targeted and package validation pass;
- workstream docs and evidence are current.

## Closeout Result

Complete on 2026-05-26. AV provider presets are implemented in config,
manifest schema/example/docs expose the preset control, and explicit provider
enable env vars override preset defaults. Manual AV live drift now has an
ignored env-gated field-health harness that uses the provider registry and
prints only redaction-safe field names/counts. Targeted, package, fmt, JSON,
and diff hygiene gates passed.
