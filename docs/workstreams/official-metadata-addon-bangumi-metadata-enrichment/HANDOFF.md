# Official Metadata Addon Bangumi Metadata Enrichment — Handoff

Status: Complete
Last updated: 2026-05-26

## Current State

Workstream opened after recovering the previous provider fact resolver session.
Current repo is on `main` at `a7088dc`, clean before this lane started.

## Closed Task

- Task ID: OMBME-020
- Owner: codex
- Files:
  - `crates/nako-metadata-scraper/src/providers/bangumi.rs`
  - `crates/nako-metadata-scraper/src/providers/bangumi/parser.rs`
  - `crates/nako-metadata-scraper/src/providers/bangumi/mapper.rs`
- Validation: `cargo nextest run -p nako-metadata-scraper bangumi --no-fail-fast`
- Status: DONE
- Review: final closeout review found no blocking findings
- Evidence: `docs/workstreams/official-metadata-addon-bangumi-metadata-enrichment/EVIDENCE_AND_GATES.md`

## Decisions Since Last Update

- Reference repositories live under gitignored `repo-ref/`.
- `jellyfin-plugin-bangumi` is GPL-2.0 and must only inform behavior-level
  comparison.
- Official Bangumi API/server schema is the authority for field semantics.
- Protocol expansion is out of scope; use existing metadata patch fields and
  provider-prefixed tags.
- Concrete homepage URLs are not written to `patch.tags`; the mapper only emits
  a `bangumi_official_site` fact tag.

## Blockers

- None.

## Next Recommended Action

- Ask for commit confirmation if the user wants these changes committed.
