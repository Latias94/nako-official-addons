# Official Metadata Addon AV Field Policy Presets - Milestones

Status: Complete
Last updated: 2026-05-27

## M0 - Scope And Evidence Freeze

Exit criteria:

- Problem and target state are explicit.
- Reference boundary is explicit.
- Related workstreams are linked.
- First proof target is chosen.

Primary evidence:

- `docs/workstreams/official-metadata-addon-av-field-policy-presets/DESIGN.md`
- `docs/workstreams/official-metadata-addon-av-field-policy-presets/TODO.md`

## M1 - Configurable Default Policy

Exit criteria:

- Config parses an AV field policy preset.
- Registry builds a supported-provider default field order.
- Quality-score policy remains available as a named preset.

Primary gates:

- `cargo nextest run -p nako-metadata-scraper config registry field_policy --no-fail-fast`

## M2 - Runtime Wiring And Public Contract

Exit criteria:

- Routes pass the configured default policy to runtime.
- Request-level `provider_field_policy` still overrides the configured default.
- Manifest and README docs expose the behavior.

Primary gates:

- `cargo nextest run -p nako-metadata-scraper runtime manifest routes --no-fail-fast`

## M3 - Verification And Closeout

Exit criteria:

- Gate set is recorded.
- Remaining work is completed, deferred, or split into a follow-on.
- `WORKSTREAM.json` status is updated.

Closed evidence:

- `cargo nextest run -p nako-metadata-scraper config registry field_policy --no-fail-fast`
- `cargo nextest run -p nako-metadata-scraper runtime manifest routes --no-fail-fast`
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`
- `cargo fmt -p nako-metadata-scraper -- --check`
- `python -m json.tool docs/workstreams/official-metadata-addon-av-field-policy-presets/WORKSTREAM.json`
- `python -m json.tool addons/metadata-scraper/manifest.example.json`
- `git diff --check`
