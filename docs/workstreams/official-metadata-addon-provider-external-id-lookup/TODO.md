# Official Metadata Addon Provider External ID Lookup — TODO

Status: Complete
Last updated: 2026-05-24

## M0 — Scope And Direct Lookup Policy Freeze

- [x] OMPEIL-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-metadata-addon-provider-external-id-lookup]
  Goal: Freeze direct provider-ID lookup behavior, fallback policy, non-goals, and evidence anchors.
  Validation: DESIGN.md, TODO.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json agree.
  Evidence: docs/workstreams/official-metadata-addon-provider-external-id-lookup/DESIGN.md
  Handoff: Opened on 2026-05-24. First executable task is TMDB direct lookup.

## M1 — TMDB Direct Lookup

- [x] OMPEIL-020 [owner=codex] [deps=OMPEIL-010] [scope=crates/nako-metadata-scraper/src/providers/tmdb.rs]
  Goal: Use query `tmdb` external IDs for direct TMDB movie enrichment before fuzzy title search, while keeping invalid/failing direct lookups on the title-search fallback path.
  Validation: cargo nextest run -p nako-metadata-scraper tmdb ranking --no-fail-fast
  Review: review-workstream for scope and code quality before accepting completion.
  Evidence: crates/nako-metadata-scraper/src/providers/tmdb.rs
  Handoff: Done on 2026-05-24. Added direct `tmdb` movie lookup through `suggest`, invalid ID fallback, and failed direct lookup fallback tests. Gate passed.

## M2 — Bangumi Direct Lookup

- [x] OMPEIL-030 [owner=codex] [deps=OMPEIL-020] [scope=crates/nako-metadata-scraper/src/providers/bangumi.rs]
  Goal: Use query `bangumi` external IDs for direct Bangumi subject enrichment before fuzzy title search, while keeping invalid/failing direct lookups on the title-search fallback path.
  Validation: cargo nextest run -p nako-metadata-scraper bangumi ranking --no-fail-fast
  Review: review-workstream for scope and code quality before accepting completion.
  Evidence: crates/nako-metadata-scraper/src/providers/bangumi.rs
  Handoff: Done on 2026-05-24. Added direct `bangumi` subject lookup through `suggest`, invalid ID fallback, and failed direct lookup fallback tests. Gate passed.

## M3 — Closeout

- [x] OMPEIL-040 [owner=planner] [deps=OMPEIL-030] [scope=docs/workstreams/official-metadata-addon-provider-external-id-lookup]
  Goal: Close the lane or split remaining cross-provider ID lookup work into follow-ons.
  Validation: verify-rust-workstream records fresh final gate evidence.
  Review: review-workstream has no blocking findings.
  Evidence: EVIDENCE_AND_GATES.md, WORKSTREAM.json
  Handoff: Completed on 2026-05-24. Lane closed with targeted, package, workspace, format, and whitespace gates passing; cross-provider ID mapping remains follow-on scope.

## M4 — Query Payload Compatibility Addendum

- [x] OMPEIL-050 [owner=codex] [deps=OMPEIL-040] [scope=crates/nako-metadata-scraper/src/engine/mod.rs]
  Goal: Parse object-form `external_ids` values that contain string arrays and keep trying later same-provider IDs when an earlier valid direct lookup fails.
  Validation: cargo nextest run -p nako-metadata-scraper metadata_query_parses_external_id_object_arrays tmdb_provider_uses_later_valid_query_external_id_when_first_lookup_fails bangumi_provider_uses_later_valid_query_external_id_when_first_lookup_fails --no-fail-fast
  Review: keep public response shape unchanged, preserve existing fallback-to-search behavior after all direct IDs fail, and preserve the existing string and array-of-object payload forms.
  Evidence: crates/nako-metadata-scraper/src/engine/mod.rs, crates/nako-metadata-scraper/src/providers/tmdb.rs, crates/nako-metadata-scraper/src/providers/bangumi.rs
  Handoff: Done on 2026-05-24. Added object-value array parsing plus direct lookup retry across multiple same-provider IDs; package/workspace gates remain green.

## M5 — Direct Lookup Duplicate Request Addendum

- [x] OMPEIL-060 [owner=codex] [deps=OMPEIL-050] [scope=crates/nako-metadata-scraper/src/providers/tmdb.rs,crates/nako-metadata-scraper/src/providers/bangumi.rs]
  Goal: Deduplicate repeated provider-native query IDs before TMDB/Bangumi direct lookup to avoid duplicate external requests.
  Validation: cargo nextest run -p nako-metadata-scraper tmdb_provider_deduplicates_query_external_ids_before_direct_lookup bangumi_provider_deduplicates_query_external_ids_before_direct_lookup --no-fail-fast
  Review: preserve payload order, later-different-ID continuation, and fallback-to-search behavior.
  Evidence: crates/nako-metadata-scraper/src/providers/tmdb.rs, crates/nako-metadata-scraper/src/providers/bangumi.rs
  Handoff: Done on 2026-05-24. Provider-native direct lookup now keeps the first occurrence of each parsed ID and skips repeated duplicates.

## M6 — Query Payload Array Object Alias Addendum

- [x] OMPEIL-070 [owner=codex] [deps=OMPEIL-060] [scope=crates/nako-metadata-scraper/src/engine/mod.rs]
  Goal: Parse array-of-object `external_ids` values from common `value`, `id`, or `external_id` fields.
  Validation: cargo nextest run -p nako-metadata-scraper metadata_query_parses_external_id_array_object_value_aliases ranking_evidence_metadata_query_parses_external_ids metadata_query_parses_external_id_object_arrays --no-fail-fast
  Review: keep non-string values ignored and preserve existing object-form and `value` array-object behavior.
  Evidence: crates/nako-metadata-scraper/src/engine/mod.rs
  Handoff: Done on 2026-05-24. Array-of-object external IDs now accept common value field aliases without changing response shape.

## M7 — Query Payload External ID Trim Addendum

- [x] OMPEIL-080 [owner=codex] [deps=OMPEIL-070] [scope=crates/nako-metadata-scraper/src/engine/mod.rs]
  Goal: Trim parsed query external ID provider/value fields and skip entries whose provider or value becomes empty.
  Validation: cargo nextest run -p nako-metadata-scraper metadata_query_trims_external_ids_and_skips_empty_entries metadata_query_parses_external_id_array_object_value_aliases ranking_evidence_metadata_query_parses_external_ids metadata_query_parses_external_id_object_arrays --no-fail-fast
  Review: apply the same normalization across object, object-array, and array-of-object payload forms while keeping non-string values ignored.
  Evidence: crates/nako-metadata-scraper/src/engine/mod.rs
  Handoff: Done on 2026-05-24. Query external ID parsing now normalizes boundary whitespace before provider matching and direct lookup.

## M8 — Query Payload Top-Level External ID Alias Addendum

- [x] OMPEIL-090 [owner=codex] [deps=OMPEIL-080] [scope=crates/nako-metadata-scraper/src/engine/mod.rs]
  Goal: Parse common top-level `tmdb_id`, `imdb_id`, and `bangumi_id` payload aliases into query external IDs.
  Validation: cargo nextest run -p nako-metadata-scraper metadata_query_parses_top_level_external_id_aliases metadata_query_preserves_external_ids_before_top_level_aliases --no-fail-fast
  Review: preserve explicit `external_ids` ordering before top-level aliases, reuse trim/empty-entry behavior, and keep provider direct lookup/fallback behavior unchanged.
  Evidence: crates/nako-metadata-scraper/src/engine/mod.rs
  Handoff: Done on 2026-05-24. Top-level ID aliases now feed the same TMDB, Bangumi, and IMDb lookup paths as explicit `external_ids`.

## M9 — Query Payload Numeric External ID Addendum

- [x] OMPEIL-100 [owner=codex] [deps=OMPEIL-090] [scope=crates/nako-metadata-scraper/src/engine/mod.rs]
  Goal: Parse integer JSON values as query external IDs for top-level aliases and object-form `external_ids` payloads.
  Validation: cargo nextest run -p nako-metadata-scraper metadata_query_parses_numeric_top_level_external_id_aliases metadata_query_parses_numeric_external_id_values --no-fail-fast
  Review: accept integer JSON numbers only for unambiguous provider-keyed fields, keep array-of-object non-string values ignored, preserve string trim behavior, and keep TMDB/Bangumi direct lookup fallback behavior unchanged.
  Evidence: crates/nako-metadata-scraper/src/engine/mod.rs
  Handoff: Done on 2026-05-24. Numeric external ID payload values now feed the same query external ID path as strings.
