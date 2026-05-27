# Official Metadata Addon AV Jav321 Provider - Handoff

Status: Complete
Last updated: 2026-05-27

## Current State

Closed. Jav321 is implemented as a raw HTTP provider with bounded form POST
search, direct URL/ID lookup, parser tests, config/catalog/manifest wiring,
default field policy participation, docs, and live drift proof.

## Active Task

- Task ID: OMJ321-040
- Owner: codex
- Files: `crates/nako-metadata-scraper/src/providers/http_runtime.rs`, `crates/nako-metadata-scraper/src/providers/jav321.rs`, `crates/nako-metadata-scraper/src/config.rs`, `crates/nako-metadata-scraper/src/providers/registry.rs`, docs and manifest files
- Validation: `cargo nextest run -p nako-metadata-scraper --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`; JSON checks; `git diff --check`
- Status: DONE
- Review: Self-reviewed with targeted, package, and live gates.
- Evidence: `EVIDENCE_AND_GATES.md`

## Decisions Since Last Update

- Use provider HTTP runtime for form POST/text HTML because Jav321 search is not a browser-worker GET render flow.
- Treat reference project behavior as field contract guidance only.
- Default field policy follows the referenced title/outline priority shape: Jav321 participates in title and overview/text fallback order, while actors/artwork/facts keep existing safer defaults.
- Jav321 has its own `NAKO_METADATA_SCRAPER_JAV321_PROXY_URL`; it is reported as a boolean diagnostics key only.
- Live drift on `jav321=SNOS-212` through `http://127.0.0.1:10809` returned one candidate with required fields present.

## Blockers

- None.

## Follow-Ons

- Optional: add more live cases for pages known to expose runtime and series, because `SNOS-212` did not expose those fields in the live response.
- Optional: add future provider breadth for unsupported sources in the reference configuration such as AvSex, FreeJavBT, or 7MMTV.

## Next Recommended Action

- Continue provider breadth or field-policy granularity work in a new lane; this Jav321 lane is closed.
