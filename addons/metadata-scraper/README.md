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

## Explicit metadata writeback

Ordinary `POST /metadata` calls remain suggestion-only. To ask the sidecar to
submit the top-ranked candidate patch as a Nako `metadata_write` Addon Side
Effect, the request payload must include an explicit `writeback` object:

```json
{
  "title": "The Matrix",
  "year": 1999,
  "language": "en-US",
  "writeback": {
    "library_id": "018f0000-0000-7000-8000-000000000003",
    "target": {
      "kind": "media_source",
      "id": "018f0000-0000-7000-8000-000000000005"
    },
    "idempotency_key": "metadata-write-demo-1"
  }
}
```

The sidecar submits writes only when all runtime gates are configured:

- `NAKO_METADATA_SCRAPER_SIDE_EFFECTS_ENABLED=true`
- `NAKO_METADATA_SCRAPER_NAKO_BASE_URL=<Nako base URL>`
- `NAKO_METADATA_SCRAPER_ADDON_TOKEN=<one-time-issued Addon Token value>`
- a Nako Addon Grant allows `metadata_write` for the target library

The sidecar first calls `/addon/v1/access-check`, then submits
`/addon/v1/side-effects` with a typed `AddonMetadataPatch` payload. Missing
runtime config, missing grants, empty candidates, and transport failures return
a redaction-safe `payload.writeback` summary instead of mutating the library.

Use the direct smoke to exercise the explicit request shape:

```powershell
pwsh -File addons/metadata-scraper/smoke.local.ps1 `
  -SidecarBaseUrl http://127.0.0.1:9100 `
  -RunWriteback `
  -MetadataWritebackLibraryId 018f0000-0000-7000-8000-000000000003 `
  -MetadataWritebackTargetKind media_source `
  -MetadataWritebackTargetId 018f0000-0000-7000-8000-000000000005
```

Without the runtime gates above, the smoke should report a skipped writeback
status such as `nako_runtime_disabled`.

## Explicit artwork writeback

Provider image facts are returned as typed artwork candidates in each metadata
candidate. Ordinary `POST /metadata` calls do not publish or select artwork.
To ask the sidecar to submit a matching artwork candidate as a Nako
`artwork_write` Addon Side Effect, include an explicit `artwork_writeback`
object:

```json
{
  "title": "The Matrix",
  "year": 1999,
  "language": "en-US",
  "artwork_writeback": {
    "library_id": "018f0000-0000-7000-8000-000000000003",
    "target": {
      "kind": "media_item",
      "id": "018f0000-0000-7000-8000-000000000004"
    },
    "idempotency_key": "artwork-write-demo-1",
    "kind": "poster"
  }
}
```

The runtime gates are the same as metadata writeback, except the Nako Addon
Grant must allow `artwork_write` for the target library. The target must be a
`media_item`; Nako owns remote image fetch, validation, cache, selected artwork,
and public image serving.

## Bulk metadata scrape task status

The manifest intentionally keeps `tasks: []` for now. Nako core can validate
Addon Task declarations and build routing plans, but the generic Addon Task
scheduler/invoker is still deferred. Bulk Metadata Scrape now has a dedicated
design follow-on at
`docs/workstreams/official-metadata-addon-bulk-task-design/`; this addon will
declare `bulk-metadata-scrape` only after Nako owns task execution, progress,
retry, and cancellation.

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
enabled by default, TMDB, Bangumi, and Douban are disabled by default, and
provider secrets should be supplied by the operator's secret manager or
environment policy.

## Current provider strategy

Users install one Addon: `nako-metadata-scraper`.

Providers are code modules inside the Addon, not separate user-visible Addons.
Today that means fixture by default and optional TMDB movie metadata when
`NAKO_METADATA_SCRAPER_PROVIDER_TMDB_ENABLED=true` and
`NAKO_METADATA_SCRAPER_TMDB_READ_ACCESS_TOKEN` is configured. TMDB currently
uses movie search plus bounded detail and external-ID enrichment for runtime,
tagline, genres, selected IDs, and typed poster/backdrop artwork candidates.
Set `NAKO_METADATA_SCRAPER_TMDB_PROXY_URL` when TMDB traffic must use an
operator-managed proxy.

Bangumi metadata is available when
`NAKO_METADATA_SCRAPER_PROVIDER_BANGUMI_ENABLED=true`. Public subject search
and detail enrichment work without a token. Set
`NAKO_METADATA_SCRAPER_BANGUMI_USER_AGENT` to a Bangumi-compliant developer/app
identifier and optionally set `NAKO_METADATA_SCRAPER_BANGUMI_ACCESS_TOKEN` for
authenticated visibility. The baseline maps subject title/original title,
summary, release date, platform, subject type, episode counts, ratings, tags,
typed poster artwork candidates, and the Bangumi subject ID into
provider-neutral facts.
Set `NAKO_METADATA_SCRAPER_BANGUMI_PROXY_URL` when Bangumi traffic must use an
operator-managed proxy.

The Addon Health Check diagnostics and `/ui/diagnostics` show whether TMDB and
Bangumi proxy policy is configured. They intentionally expose only boolean
policy state, not proxy URLs or credentials.

Douban metadata is available when
`NAKO_METADATA_SCRAPER_PROVIDER_DOUBAN_ENABLED=true` and the browser-worker
companion service is reachable through
`NAKO_METADATA_SCRAPER_BROWSER_WORKER_BASE_URL`. The worker provides the
generic rendered HTML contract (`POST /render`); Douban search/detail parsing,
field mapping, ranking facts, and artwork candidates stay inside the Rust
provider. This keeps Playwright/Crawlee out of the Rust sidecar without turning
the worker into a second metadata scraper.

The runtime then deduplicates exact duplicate candidates, caps the final list,
and applies a small generic bonus from the shared community score/vote-count
facts exposed by TMDB, Bangumi, and Douban. The protocol envelope does not
change.

Future provider breadth will come through the runtime seam, not by turning each
provider into its own addon. The browser-worker companion service now owns the
Playwright/Crawlee path for rendered-page extraction, while provider-specific
metadata interpretation remains in `nako-metadata-scraper`.
