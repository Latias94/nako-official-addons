# Official Metadata Addon Provider External ID Lookup — Milestones

Status: Complete
Last updated: 2026-05-24

## M0 — Scope And Direct Lookup Policy Freeze

Exit criteria:

- Direct provider-ID behavior is explicit.
- Invalid-ID and direct-failure fallback behavior is explicit.
- Cross-provider ID mapping remains out of scope.

Primary evidence:

- `docs/workstreams/official-metadata-addon-provider-external-id-lookup/DESIGN.md`
- `docs/workstreams/official-metadata-addon-provider-external-id-lookup/TODO.md`

## M1 — TMDB Direct Lookup

Exit criteria:

- TMDB query external ID fetches detail without a search request.
- Invalid TMDB ID falls back to title search.
- Failed direct lookup can still fall back to title search.

Primary gates:

- `cargo nextest run -p nako-metadata-scraper tmdb ranking --no-fail-fast`

## M2 — Bangumi Direct Lookup

Exit criteria:

- Bangumi query external ID fetches detail without a search request.
- Invalid Bangumi ID falls back to title search.
- Failed direct lookup can still fall back to title search.

Primary gates:

- `cargo nextest run -p nako-metadata-scraper bangumi ranking --no-fail-fast`

## M3 — Closeout

Exit criteria:

- Fresh gate evidence is recorded.
- Package, workspace, format, and whitespace gates pass.
- Cross-provider ID mapping is deferred or split.

Status:

- Complete on 2026-05-24. The lane closed after targeted provider gates, package/workspace nextest,
  rustfmt check, and whitespace check passed. Cross-provider ID mapping is deferred.

## M4 — Query Payload Compatibility Addendum

Exit criteria:

- Object-form `external_ids` still supports string values.
- Object-form `external_ids` also accepts arrays of strings for repeated provider IDs.
- Array-of-object `external_ids` remains unchanged.
- TMDB/Bangumi continue to later same-provider IDs when an earlier valid direct lookup fails.
- TMDB/Bangumi still fall back to title search after all direct IDs fail.
- Package, workspace, format, and whitespace gates pass.

Status:

- Complete on 2026-05-24. The addendum lets real payloads reach same-provider later-valid-ID direct
  lookup paths and keeps trying later IDs after one direct lookup fails, without changing the public
  response shape.

## M5 — Direct Lookup Duplicate Request Addendum

Exit criteria:

- Repeated query `tmdb` IDs are requested at most once before trying later distinct IDs.
- Repeated query `bangumi` IDs are requested at most once before trying later distinct IDs.
- Fallback-to-search behavior after all distinct direct IDs fail is preserved.
- Package, workspace, format, and whitespace gates pass.

Status:

- Complete on 2026-05-24. Provider-native direct lookup now deduplicates repeated parsed IDs while
  preserving first-seen order.

## M6 — Query Payload Array Object Alias Addendum

Exit criteria:

- Array-of-object `external_ids` accepts `value`.
- Array-of-object `external_ids` accepts `id`.
- Array-of-object `external_ids` accepts `external_id`.
- Non-string values remain ignored.
- Package, workspace, format, and whitespace gates pass.

Status:

- Complete on 2026-05-24. Query external ID parsing now accepts common array-object value aliases
  while preserving existing payload forms.

## M7 — Query Payload External ID Trim Addendum

Exit criteria:

- Parsed external ID providers are trimmed.
- Parsed external ID values are trimmed.
- Empty providers or values are skipped.
- Object, object-array, and array-of-object payload forms use the same normalization.
- Package, workspace, format, and whitespace gates pass.

Status:

- Complete on 2026-05-24. Query external ID parsing now trims boundary whitespace and skips empty
  IDs before provider matching.
