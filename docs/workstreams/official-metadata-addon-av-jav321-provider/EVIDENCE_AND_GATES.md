# Official Metadata Addon AV Jav321 Provider - Evidence And Gates

Status: Complete
Last updated: 2026-05-27

## Smallest Current Repro

```bash
cargo nextest run -p nako-metadata-scraper jav321 http_runtime --no-fail-fast
```

## Gate Set

```bash
cargo nextest run -p nako-metadata-scraper jav321 http_runtime --no-fail-fast
cargo nextest run -p nako-metadata-scraper config registry manifest jav321 --no-fail-fast
cargo nextest run -p nako-metadata-scraper --no-fail-fast
cargo fmt -p nako-metadata-scraper -- --check
python -m json.tool docs/workstreams/official-metadata-addon-av-jav321-provider/WORKSTREAM.json
git diff --check
```

## Evidence Anchors

- `crates/nako-metadata-scraper/src/providers/http_runtime.rs`
- `crates/nako-metadata-scraper/src/providers/jav321.rs`
- `crates/nako-metadata-scraper/src/config.rs`
- `crates/nako-metadata-scraper/src/providers/registry.rs`
- `addons/metadata-scraper/README.md`

## Fresh Evidence - 2026-05-27

- `cargo nextest run -p nako-metadata-scraper jav321 http_runtime --no-fail-fast`
  - Result: 14 passed, 241 skipped.
  - Covers raw text/form runtime, existing HTTP runtime behavior, and Jav321 provider tests.
- `cargo nextest run -p nako-metadata-scraper config registry manifest jav321 --no-fail-fast`
  - Result: 40 passed, 215 skipped.
  - Covers provider config, catalog, field policy, manifest, routes, and Jav321 provider tests.
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`
  - Result: 252 passed, 3 skipped.
  - Covers the full metadata scraper package.
- `cargo fmt -p nako-metadata-scraper -- --check`
  - Result: passed.
- `python -m json.tool docs/workstreams/official-metadata-addon-av-jav321-provider/WORKSTREAM.json`
  - Result: passed.
- `python -m json.tool addons/metadata-scraper/manifest.example.json`
  - Result: passed.
- `git diff --check`
  - Result: passed.
- `NAKO_METADATA_SCRAPER_LIVE_PROVIDER_DRIFT=1 NAKO_METADATA_SCRAPER_LIVE_AV_PROVIDER_DRIFT_CASES=jav321=SNOS-212 NAKO_METADATA_SCRAPER_JAV321_PROXY_URL=http://127.0.0.1:10809 cargo test -p nako-metadata-scraper --test live_provider_drift -- --ignored av_live_provider_field_health_smoke --nocapture`
  - Result: passed; 1 Jav321 candidate; required fields present.
  - Present fields: title, av_number, overview, release_date, tags, actors, all_actors, studio, publisher, maker, label, thumb_url, extrafanart_urls, artwork_candidates, external_ids, provider_outcomes.
  - Optional fields missing on the live page: runtime_minutes, genres, directors, series, wanted_count, trailer_url.

## Notes

- Parser tests assert fields, not merely candidate existence.
- Live proof passed through the configured local proxy. Optional missing fields are recorded as page-data gaps.
