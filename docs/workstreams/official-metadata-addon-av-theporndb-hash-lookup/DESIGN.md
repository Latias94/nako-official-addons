# ThePornDB Scene Hash Lookup

## Problem

ThePornDB exposes scene hash lookup through `GET /scenes/hash/{hash}` with an
optional `type` query parameter (`OSHASH` or `PHASH`). The metadata scraper can
currently search ThePornDB by AV number/title and direct scene ID/URL, but it
cannot use file fingerprints for appointed-source correction.

## Target State

- Declare redaction-safe hash query aliases:
  - `file_oshash` -> provider `file_oshash`
  - `file_phash` -> provider `file_phash`
- Let ThePornDB perform direct scene lookup before AV/title search when either
  hash alias appears in `external_ids` or top-level payload fields.
- Call `/scenes/hash/{hash}?type=OSHASH` for `file_oshash`.
- Call `/scenes/hash/{hash}?type=PHASH` for `file_phash`.
- Map the returned `SceneResource` through the existing ThePornDB scene mapper.
- Emit returned hash records as external IDs when present.

## Architecture Direction

Use provider-owned `ProviderExternalIdCapability` for this slice. A dedicated
`MetadataQuery.file_hashes` value object is cleaner for multi-hash arrays, but
adding it now would force unrelated test churn across many direct
`MetadataQuery` struct literals. The capability approach keeps this vertical
slice small and still makes hash inputs visible to the runtime alias parser.

## Non-Goals

- No movie hash lookup yet; current ThePornDB mapping is scene-oriented.
- No local file hashing in this addon.
- No hash write/create calls to ThePornDB.
- No raw hash values in diagnostics or workstream evidence.

## References

- ThePornDB OpenAPI: `https://api.theporndb.net/specs`
- Prior workstream:
  `docs/workstreams/official-metadata-addon-av-theporndb-provider/`
