# Official Metadata Addon Host Policy Adapter - Evidence and Gates

Status: Complete
Last updated: 2026-05-26

## Evidence

- `MetadataWritebackRequest` uses `serde(deny_unknown_fields)`.
- Added tests in `writeback.rs` to reject `refresh_mode` in `writeback`.
- Native patch materialization still only deduplicates and enriches provider
  facts into `AddonMetadataPatch`.

## Gates Run

```text
cargo nextest run -p nako-metadata-scraper metadata_writeback --no-fail-fast
cargo nextest run -p nako-metadata-scraper side_effect --no-fail-fast
cargo nextest run -p nako-metadata-scraper metadata_target_validation --no-fail-fast
cargo fmt --all -- --check
git diff --check
```
