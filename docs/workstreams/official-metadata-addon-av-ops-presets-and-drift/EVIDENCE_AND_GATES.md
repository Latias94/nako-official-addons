# Evidence And Gates

Last updated: 2026-05-26

## Required Gates

- `cargo nextest run -p nako-metadata-scraper av_provider_preset manifest --no-fail-fast`
- `cargo nextest run -p nako-metadata-scraper av_drift field_health --no-fail-fast`
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`
- `cargo fmt -p nako-metadata-scraper -- --check`
- `python -m json.tool docs/workstreams/official-metadata-addon-av-ops-presets-and-drift/WORKSTREAM.json`
- `git diff --check`

## Optional Manual Live Gate

The manual AV drift gate must be ignored and env-gated. It should be run only
when the operator has opted in and configured provider access:

```powershell
$env:NAKO_METADATA_SCRAPER_LIVE_PROVIDER_DRIFT = '1'
$env:NAKO_METADATA_SCRAPER_LIVE_AV_PROVIDER_DRIFT_CASES = 'provider=AV-NUMBER,...'
cargo test -p nako-metadata-scraper --test live_provider_drift -- --ignored av_live_provider_field_health_smoke
```

For browser-rendered providers, start the browser worker and configure
`NAKO_METADATA_SCRAPER_BROWSER_WORKER_BASE_URL` first. Proxy-sensitive
providers may also require `NAKO_METADATA_SCRAPER_BROWSER_WORKER_PROXY_POLICY`.

## Evidence Log

- PASS on 2026-05-26:
  `cargo nextest run -p nako-metadata-scraper av_provider_preset manifest --no-fail-fast`
  passed with 8 tests.
- PASS on 2026-05-26:
  `cargo nextest run -p nako-metadata-scraper av_drift field_health --no-fail-fast`
  passed with 3 tests.
- PASS on 2026-05-26:
  `cargo nextest run -p nako-metadata-scraper --no-fail-fast` passed with 227
  tests and 3 skipped ignored live tests.
- PASS on 2026-05-26:
  `cargo fmt -p nako-metadata-scraper -- --check` passed.
- PASS on 2026-05-26:
  `python -m json.tool docs/workstreams/official-metadata-addon-av-ops-presets-and-drift/WORKSTREAM.json`
  passed.
- PASS on 2026-05-26: `git diff --check` passed.
