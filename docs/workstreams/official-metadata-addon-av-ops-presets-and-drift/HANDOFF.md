# Official Metadata Addon AV Ops Presets And Drift - Handoff

Status: Complete
Last updated: 2026-05-26

## Current State

The previous AV parser-quality lane is complete. Current AV providers share
row-level structured label parsing where useful. The next gap is operational:
provider presets and manual drift checks for live provider field health.

## Active Task

- None.

## Completed

- AVOPS-010: workstream opened.
- AVOPS-020: `NAKO_METADATA_SCRAPER_AV_PROVIDER_PRESET`, manifest schema,
  example manifest, docs, and tests added.
- AVOPS-030: manual AV live drift field-health harness added with
  deterministic drift config and redaction-safety tests.
- AVOPS-040: full package, fmt, JSON, and diff hygiene gates passed.

## Next Task

- Recommended follow-up is Wave 4 provider breadth now that preset/drift
  operations are stable.

## Guardrails

- MDCx is reference-only. Do not copy code, selectors, regex tables, fixtures,
  comments, or config text.
- Do not store adult-site payload values in repository tests or docs.
- Live adult-provider checks must remain opt-in and ignored.

## Blockers

- None.
