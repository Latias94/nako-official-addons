# Nako Metadata Scraper Addon

Official metadata scraper Addon Sidecar for Nako.

Current release target: `v0.1.0-alpha.2`.

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
- `POST /tasks/bulk-metadata-scrape`
- `POST /events/library-scanned`
- `GET /ui/diagnostics`

## Local smoke

Start the sidecar first, then run the direct sidecar smoke:

```powershell
pwsh -File addons/metadata-scraper/smoke.local.ps1 `
  -SidecarBaseUrl http://127.0.0.1:9100
```

The direct smoke fetches `/manifest.json`, calls `/health`, and calls the
`metadata` resource with a fixture movie query. It also posts a safe
`library.scanned` event envelope to `/events/library-scanned`. It expects at
least one candidate and one generated artifact with the default fixture
provider.

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
  -RunResourceCall `
  -RunTaskPath
```

`-RunTaskPath` syncs routing plans, creates a Nako-owned direct Addon Task run
for `bulk-metadata-scrape`, waits for terminal success, and verifies the task
result contains the sidecar's bulk metadata scrape output schema.

Any option that requires Nako-owned behavior, including `-RunTaskPath`,
`-RunResourceCall`, `-Enable`, `-IssueAddonToken`, and `-RequireNako`, now
requires `-RegisterInNako`. This keeps release smoke gates from reporting a
sidecar-only pass after silently skipping Nako Admin paths.

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

For a release gate, assert the expected writeback outcome explicitly:

```powershell
pwsh -File addons/metadata-scraper/smoke.local.ps1 `
  -SidecarBaseUrl http://127.0.0.1:9100 `
  -RunWriteback `
  -ExpectedWritebackStatus skipped `
  -ExpectedWritebackSafeErrorCode nako_runtime_disabled `
  -MetadataWritebackLibraryId 018f0000-0000-7000-8000-000000000003 `
  -MetadataWritebackTargetKind media_source `
  -MetadataWritebackTargetId 018f0000-0000-7000-8000-000000000005
```

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

The manifest now declares `bulk-metadata-scrape` at
`/tasks/bulk-metadata-scrape`. Nako owns task execution, progress, retry, and
cancellation, while the addon sidecar owns the bounded batch planner and the
same explicit metadata payload shape used by `POST /metadata`.

Each task item is a metadata request payload and may include the explicit
`writeback` and `artwork_writeback` objects described above. The task response
returns a batch summary plus the per-item metadata payloads produced by the
existing runtime.

For AV libraries, items may provide `av_number`, `number`, `file_name`,
`filename`, `path`, `title`, or `name`. The sidecar normalizes the recognized
AV number into `payload.query.av`, including the route family such as `fc2` or
`censored`, and does not echo full local paths. Duplicate AV numbers without
side-effect requests can reuse a previous result and report
`reused_from_index`.

Task callers may pass the previous task output `resume_state` into a later task
payload. This lets the sidecar reuse safe duplicate AV-number results across
bounded batches while Nako keeps ownership of task scheduling, progress, retry,
and cancellation. Batch output also includes `summary.failed_items`,
`summary.failure_reasons`, and `summary.provider_execution` for redaction-safe
failure accounting.

## Library scanned event proof

The manifest declares one event subscription:

- `id`: `library-scanned`
- `event_kind`: `library.scanned`
- `path`: `/events/library-scanned`
- `required_scopes`: `webhook_event_read`

The handler is intentionally small. It validates the event envelope and returns
a redaction-safe ACK with payload keys, not payload values. Its purpose is to
prove that the official metadata sidecar can carry event-driven capabilities in
the same deployment unit as metadata resources and Addon Tasks. Full
notification bridges and provider fan-out remain separate future addons or
suite capabilities.

## Register in Nako Admin Web

1. Start this sidecar or generate the manifest from `/manifest.json`.
2. Paste the manifest JSON into Nako Admin Web Addon Onboarding.
3. Register as disabled.
4. Follow the generated Install Guide.
5. Run Addon Health Check.
6. Configure future token/grant flow and enable the Addon.

## Docker example

This alpha targets Nako Addon Protocol `0.1.0-alpha.1` and
`nako-addon-protocol` Rust crate version `0.1.0-alpha.2`.

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
  -t ghcr.io/latias94/nako-metadata-scraper:0.1.0-alpha.2 \
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
summary, release date, platform, subject type, NSFW/locked/series flags,
episode and collection counts, ratings, selected infobox facts, tags, typed
poster artwork candidates, and the Bangumi subject ID into provider-neutral
facts.
Set `NAKO_METADATA_SCRAPER_BANGUMI_PROXY_URL` when Bangumi traffic must use an
operator-managed proxy.

The Addon Health Check diagnostics and `/ui/diagnostics` show whether TMDB and
Bangumi proxy policy is configured. They intentionally expose only boolean
policy state, not proxy URLs or credentials.

Metadata requests may provide explicit `external_ids` or top-level aliases:
`tmdb_id`, `imdb_id`, `bangumi_id`, `browser_worker_url`, `javdb_id`, `dmm_id`,
`dmm_url`, `fc2_id`, and `av_number`. These aliases are derived from
provider-owned external ID capabilities.

Douban metadata is available when
`NAKO_METADATA_SCRAPER_PROVIDER_DOUBAN_ENABLED=true` and the browser-worker
companion service is reachable through
`NAKO_METADATA_SCRAPER_BROWSER_WORKER_BASE_URL`. The worker provides the
generic rendered HTML contract (`POST /render`); Douban search/detail parsing,
field mapping, ranking facts, and artwork candidates stay inside the Rust
provider. This keeps Playwright/Crawlee out of the Rust sidecar without turning
the worker into a second metadata scraper.

JavDB, DMM, and FC2 metadata are available when their providers are enabled and
the same browser-worker companion service is reachable. JavDB searches by
normalized non-FC2 AV numbers and supports explicit `javdb_id` direct lookup.
DMM is an official censored-release tracer that searches by normalized AV number
and supports explicit `dmm_id` or `dmm_url` direct lookup. FC2 handles FC2-number
direct article lookup and supports explicit `fc2_id` direct lookup. These AV
providers emit `av_number` external IDs so the resolver can merge compatible AV
facts across sources.

The runtime then resolves exact duplicate candidates and candidates that share
declared provider-emitted external IDs, caps the final list, and applies a
small generic bonus from the shared community score/vote-count facts exposed by
TMDB, Bangumi, and Douban. The protocol envelope does not change.

Each metadata response includes `provider_execution`, which records the
provider IDs selected, skipped by AV route, returned, empty, or failed with a
safe failure category. This is the single-scrape counterpart to the bulk
provider summary.

Requests may optionally include `provider_field_policy` for field-level source
priority inside an already-merged candidate cluster:

```json
{
  "av_number": "SSNI-644",
  "provider_field_policy": {
    "title": ["javdb"],
    "overview": ["dmm"],
    "tags": ["dmm"]
  }
}
```

The policy does not merge unrelated candidates by itself; it only chooses fields
after providers have emitted compatible external IDs such as the same
`av_number`. When no request policy is supplied, AV clusters use a conservative
built-in policy inspired by MDCx's field-priority behavior: DMM is preferred
before JavDB and FC2 for official title, overview, release/runtime, tags, and
poster/backdrop artwork when those providers emitted compatible facts. Passing
an explicit `provider_field_policy` object replaces that default for the
request.

Future provider breadth will come through the runtime seam, not by turning each
provider into its own addon. The browser-worker companion service now owns the
Playwright/Crawlee path for rendered-page extraction, while provider-specific
metadata interpretation remains in `nako-metadata-scraper`.
