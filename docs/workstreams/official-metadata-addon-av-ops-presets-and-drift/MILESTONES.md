# Milestones

## M1 - Workstream Open

Exit criteria:

- DESIGN/TODO/MILESTONES/EVIDENCE/HANDOFF/WORKSTREAM exist.
- Task ledger is split by vertical slices.
- MDCx reference-only guardrail is documented.

## M2 - Provider Presets

Exit criteria:

- `NAKO_METADATA_SCRAPER_AV_PROVIDER_PRESET` supports the documented enum.
- Explicit provider enable env vars override preset defaults.
- Manifest schema and example expose the setting.
- Targeted config/manifest tests pass.

## M3 - Manual Drift Harness

Exit criteria:

- Ignored live test can run AV provider cases from env.
- Redaction-safe summary helper has deterministic test coverage.
- README documents the command and required browser-worker/proxy notes.

## M4 - Closeout

Exit criteria:

- Full `nako-metadata-scraper` nextest gate passes.
- `cargo fmt` and diff hygiene pass.
- Workstream evidence is current and the lane is closed or follow-ups are split.
