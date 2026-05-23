# Nako Metadata Scraper Addon

Official metadata scraper Addon Sidecar for Nako.

This repository is being refactored toward a future-facing metadata addon
architecture. The current runtime has one installable addon, a fixture adapter
for smoke tests, plus bounded TMDB movie and Bangumi subject baselines behind
the shared provider registry, ranking model, and HTTP runtime.

## Run locally

```bash
cargo run -p nako-metadata-scraper
```

Endpoints:

- `GET /manifest.json`
- `POST /health`
- `POST /metadata`
- `GET /ui/diagnostics`

## Local smoke

Start the sidecar first, then run the direct sidecar smoke:

```powershell
pwsh -File addons/metadata-scraper/smoke.local.ps1 `
  -SidecarBaseUrl http://127.0.0.1:9100
```

The direct smoke fetches `/manifest.json`, calls `/health`, and calls the
`metadata` resource with a fixture movie query. It expects at least one
candidate and one generated artifact with the default fixture provider.

When a local Nako server is already running, use the same script for an
Admin-mediated smoke. This path registers the manifest only when it is not
already registered, refreshes the manifest-granted metadata scopes when it
reuses an existing registration, runs Addon Health Check, and can optionally
enable the addon and call Nako's redaction-safe resource diagnostic:

```powershell
$env:NAKO_ADMIN_TOKEN = '<admin bearer token>'
pwsh -File addons/metadata-scraper/smoke.local.ps1 `
  -SidecarBaseUrl http://127.0.0.1:9100 `
  -NakoBaseUrl http://127.0.0.1:3000 `
  -RegisterInNako `
  -Enable `
  -RunResourceCall
```

Pass `-NoAdminAuth` only for an unauthenticated local development server. The
script never prints administrator bearer tokens, one-time Addon raw tokens, or
provider secrets.

## Register in Nako Admin Web

1. Start this sidecar or generate the manifest from `/manifest.json`.
2. Paste the manifest JSON into Nako Admin Web Addon Onboarding.
3. Register as disabled.
4. Follow the generated Install Guide.
5. Run Addon Health Check.
6. Configure future token/grant flow and enable the Addon.

## Docker example while the protocol crate is local

The current workspace depends on the local core checkout:

```text
../nako/crates/nako-addon-protocol
```

Therefore `compose.example.yml` expects this directory layout:

```text
<parent>/
  nako/
  nako-official-addons/
```

After the protocol crate is published, the Docker context can shrink back to
this repository only.

The Dockerfile and examples expose the runtime truth directly: fixture is
enabled by default, TMDB and Bangumi are disabled by default, and provider
secrets should be supplied by the operator's secret manager or environment
policy.

## Current provider strategy

Users install one Addon: `nako-metadata-scraper`.

Providers are code modules inside the Addon, not separate user-visible Addons.
Today that means fixture by default and optional TMDB movie metadata when
`NAKO_METADATA_SCRAPER_PROVIDER_TMDB_ENABLED=true` and
`NAKO_METADATA_SCRAPER_TMDB_READ_ACCESS_TOKEN` is configured. TMDB currently
uses movie search plus bounded detail and external-ID enrichment for runtime,
tagline, genres, selected IDs, and safe image-path metadata.

Bangumi metadata is available when
`NAKO_METADATA_SCRAPER_PROVIDER_BANGUMI_ENABLED=true`. Public subject search
and detail enrichment work without a token. Set
`NAKO_METADATA_SCRAPER_BANGUMI_USER_AGENT` to a Bangumi-compliant developer/app
identifier and optionally set `NAKO_METADATA_SCRAPER_BANGUMI_ACCESS_TOKEN` for
authenticated visibility. The baseline maps subject title/original title,
summary, release date, platform, subject type, episode counts, ratings, tags,
image URLs, and the Bangumi subject ID into provider-neutral facts.

Future provider breadth will come through the runtime seam, not by turning each
provider into its own addon. Douban and any Playwright/crawler runtime are
explicitly deferred to a separate design lane.
