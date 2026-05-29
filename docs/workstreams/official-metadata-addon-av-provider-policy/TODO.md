# Task Ledger

Prefix: OMAVP

## Completed

- [x] OMAVP-010 - Open AV provider policy workstream
  - Record scope, MDCx reference boundary, Crawlee/browser-worker ownership, and
    validation gates.
  - Validation: `python -m json.tool docs/workstreams/official-metadata-addon-av-provider-policy/WORKSTREAM.json`

- [x] OMAVP-020 - Document current AV request and batch contracts
  - Update metadata-scraper READMEs with AV fields, direct provider IDs,
    `provider_execution`, bulk `resume_state`, failure categories, and Crawlee
    worker boundary.
  - Validation: README diff review; `git diff --check`

## Completed

- [x] OMAVP-030 - Add request-configurable provider field policy
  - Add a policy input that can prefer providers per metadata/artwork field
    while preserving current default behavior when absent.
  - Default policy should prefer explicit/direct sources and allow aggregator
    fill for missing overview/tags/actors/artwork.
  - Validation: `cargo nextest run -p nako-metadata-scraper field_policy resolver ranking --no-fail-fast`

## Completed

- [x] OMAVP-040 - Add one more AV provider tracer
  - Add a disabled-by-default provider or provider skeleton for the next route
    family using rendered-page synthetic tests.
  - Implemented DMM as an official censored-release tracer with `dmm_id` /
    `dmm_url` direct lookup, rendered search/detail parsing, provider
    diagnostics, manifest schema, and synthetic browser-worker tests.
  - Validation:
    `cargo nextest run -p nako-metadata-scraper dmm --no-fail-fast`;
    `cargo nextest run -p nako-metadata-scraper config registry manifest --no-fail-fast`

## Completed

- [x] OMAVP-050 - Verify and close
  - Run package tests, modified-file formatting, JSON validation, and diff
    hygiene.
  - Validation: `cargo nextest run -p nako-metadata-scraper --no-fail-fast`;
    `cargo fmt -p nako-metadata-scraper -- --check`; `python -m json.tool
    docs/workstreams/official-metadata-addon-av-provider-policy/WORKSTREAM.json`;
    `python -m json.tool addons/metadata-scraper/manifest.example.json`;
    `git diff --check`

## Follow-Up Candidates

- Browser-worker session/wait/proxy contract expansion for Crawlee if a real
  provider needs more than `/render`.
- More AV providers after the field policy seam is stable, especially
  JavBus/JavLibrary-style aggregator fallback and FC2PPVDB-style FC2 fallback.
- Operator UI/config exposure for global provider-field policy defaults.
