# Nako Metadata Scraper Addon

Official metadata scraper Addon Sidecar for Nako.

Current release target: `v0.1.0-alpha.1`.

Main Nako repository: <https://github.com/Latias94/nako>.
Official addons repository: <https://github.com/Latias94/nako-official-addons>.

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

## Docker example

This alpha targets Nako Addon Protocol `0.1.0-alpha.1` and
`nako-addon-protocol` Rust crate version `0.1.0-alpha.1`.

The addon manifest has separate version fields: `version` is this sidecar's
release version, while `protocol_version` is the Nako Addon Protocol wire
compatibility version used for registration, health checks, and resource calls.

The Dockerfile uses cargo-chef planner/cacher/builder stages so dependency
layers are reused when only provider or route code changes. `cargo chef cook`
and the final `cargo build` both run from `/src/nako-official-addons`, which is
required for the cached `target` directory to remain useful.

The Docker build uses this repository as its build context because
`nako-addon-protocol` is consumed from crates.io:

```bash
docker buildx build \
  -f addons/metadata-scraper/Dockerfile \
  -t ghcr.io/latias94/nako-metadata-scraper:0.1.0-alpha.1 \
  .
```

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

The runtime then deduplicates exact duplicate candidates, caps the final list,
and applies a small generic bonus from the shared community score/vote-count
facts exposed by TMDB and Bangumi. The protocol envelope does not change.

Future provider breadth will come through the runtime seam, not by turning each
provider into its own addon. Douban and any Playwright/crawler runtime are
explicitly deferred to a separate design lane.
