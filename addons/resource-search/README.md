# Nako Resource Search

Official resource search sidecar for Nako.

This addon proves the plugin-side boundary for external resource discovery:
provider aggregation, link classification, deterministic result fusion, and
safe diagnostics. It deliberately stays separate from metadata scraping.

## Protocol Status

This sidecar declares the first-class Nako `resource_search` addon resource at
`/resource-search`. It requires the read-only `acquisition_search_read` scope
and returns `nako.addon.resource_search.response.v1` payloads.

Resource search returns external acquisition candidates only. Link checking,
downloader execution, cloud-drive transfer, and password/code reference handling
remain separate follow-on contracts owned by Nako host policy.

## Safe Defaults

- Listens on `127.0.0.1:9130`.
- Fixture provider is enabled by default for local smoke and CI.
- The PanSou-compatible HTTP provider is disabled by default and only runs when
  explicitly enabled with a valid HTTP(S) base URL.
- Downloader or BitTorrent client invocation is not part of the search path.

## Run Locally

```powershell
cargo run -p nako-resource-search
pwsh -File addons/resource-search/smoke.local.ps1
```

Optional PanSou-compatible provider:

```powershell
$env:NAKO_RESOURCE_SEARCH_PANSOU_PROVIDER_ENABLED = '1'
$env:NAKO_RESOURCE_SEARCH_PANSOU_BASE_URL = 'http://127.0.0.1:8888'
$env:NAKO_RESOURCE_SEARCH_PANSOU_PLUGINS = 'jikepan,pansearch'
$env:NAKO_RESOURCE_SEARCH_PANSOU_CLOUD_TYPES = 'quark,aliyun,magnet'
cargo run -p nako-resource-search
```

Disable the fixture provider to verify degraded health:

```powershell
$env:NAKO_RESOURCE_SEARCH_FIXTURE_PROVIDER_ENABLED = 'false'
cargo run -p nako-resource-search
```
