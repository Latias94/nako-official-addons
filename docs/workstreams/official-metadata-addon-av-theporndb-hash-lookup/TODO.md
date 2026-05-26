# Task Ledger

Prefix: AVTPDBH

## Active

- None.

## Pending

- None.

## Completed

- [x] AVTPDBH-010 [owner=codex] [deps=-] [scope=docs/workstreams/official-metadata-addon-av-theporndb-hash-lookup]
  Goal: Open the hash lookup lane with ThePornDB API assumptions, query alias scope, and validation gates.
  Validation: `python -m json.tool docs/workstreams/official-metadata-addon-av-theporndb-hash-lookup/WORKSTREAM.json`
  Review: Hash values must remain out of diagnostics and evidence.
  Evidence: PASS, 2026-05-26.
  Handoff: Complete.

- [x] AVTPDBH-020 [owner=codex] [deps=AVTPDBH-010] [scope=crates/nako-metadata-scraper/src/providers/theporndb.rs,crates/nako-metadata-scraper/src/providers/registry.rs]
  Goal: Add file hash external-id capabilities and ThePornDB scene hash direct lookup with synthetic API tests.
  Validation: `cargo nextest run -p nako-metadata-scraper theporndb registry_exposes_provider_external_id_capabilities registry_derives_legacy_external_id_aliases_from_capabilities --no-fail-fast`
  Review: Hash direct lookup must run before AV/title search and include the correct `type` query parameter.
  Evidence: PASS, 2026-05-26.
  Handoff: Complete.

- [x] AVTPDBH-030 [owner=codex] [deps=AVTPDBH-020] [scope=README.md,addons/metadata-scraper/README.md,docs/workstreams/official-metadata-addon-av-theporndb-hash-lookup]
  Goal: Document hash aliases, record evidence, and close the workstream.
  Validation: `cargo nextest run -p nako-metadata-scraper --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`; `git diff --check`
  Review: Movie hash lookup must remain explicit follow-up.
  Evidence: PASS, 2026-05-26.
  Handoff: Complete.

## Follow-Up Candidates

- Add `MetadataQuery.file_hashes` for multi-hash arrays and provider-neutral hash routing.
- Add ThePornDB movie hash lookup once scene/movie intent is explicit.
