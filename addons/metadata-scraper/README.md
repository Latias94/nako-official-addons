# Nako Metadata Scraper Addon

Official metadata scraper Addon Sidecar for Nako.

Current release target: `v0.1.0-alpha.2`.

Main Nako repository: <https://github.com/Latias94/nako>.
Official addons repository: <https://github.com/Latias94/nako-official-addons>.

This repository is being refactored toward a future-facing metadata addon
architecture. The current runtime has one installable addon, a fixture adapter
for smoke tests, plus bounded TMDB movie/TV, Bangumi subject, and AniList anime
providers behind the shared provider registry, ranking model, and HTTP runtime.

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
`/addon/v1/side-effects` with a typed `AddonMetadataPatch` payload. The patch is
the native metadata graph shape: beyond scalar title fields, it can carry
ratings, image references, credits, collections, studios, and external IDs.
Missing runtime config, missing grants, empty candidates, and transport failures
return a redaction-safe `payload.writeback` summary instead of mutating the
library. The target must be a `media_source`; other target kinds are skipped
with `invalid_metadata_target_kind` before runtime access checks.

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
and public image serving. Other target kinds are skipped with
`invalid_artwork_target_kind` before runtime access checks.

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
failure accounting. Reusable resume entries carry their typed
`safe_failure_reason` and `suppressed_provider_ids`, so later batches do not
need to infer retry accounting from rendered item payload JSON.

Bulk requests may also include a `provider_policy` object:

```json
{
  "provider_policy": {
    "suppress_after_failures": 2,
    "cooldown_items": 3,
    "max_selected_providers_per_item": 4,
    "max_reusable_items": 128,
    "max_provider_states": 64
  }
}
```

The default policy suppresses a provider after repeated retryable failures and
records the cooldown in `resume_state.provider_states`. Callers that submit a
later batch should pass that `resume_state` back in; the sidecar does not keep a
hidden scheduler or background provider memory. Bulk output includes
`summary.suppressed_items`, `summary.retry_classes`, provider-level suppressed
and retry-class counts, `summary.budget_exhausted_items`, provider-level budget
counts, the applied top-level `provider_policy`, and per-item
`suppressed_provider_ids`. `max_reusable_items` bounds the duplicate-AV resume
cache and `max_provider_states` bounds persisted cooldown state. Retry classes
are redaction-safe: `timeout`, `rate_limited`, and `provider_error` are
retryable; `auth_or_forbidden` requires operator action; `not_found` and
`parse_error` are permanent for accounting.

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

The Docker build uses this repository as its primary build context and the
main Nako repository as a named BuildKit context because the alpha workspace
still uses local `../nako` SDK path dependencies:

```bash
docker buildx build \
  --build-context nako=../nako \
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
Today that means fixture by default and optional TMDB movie/TV metadata when
`NAKO_METADATA_SCRAPER_PROVIDER_TMDB_ENABLED=true` and
`NAKO_METADATA_SCRAPER_TMDB_READ_ACCESS_TOKEN` is configured. TMDB uses movie
and TV-series search plus bounded detail and external-ID enrichment for runtime,
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

AniList anime metadata is available when
`NAKO_METADATA_SCRAPER_PROVIDER_ANILIST_ENABLED=true`. Public GraphQL search and
detail lookup work without a token; optionally set
`NAKO_METADATA_SCRAPER_ANILIST_ACCESS_TOKEN` for authenticated requests.
AniList maps title variants, description, release date/year, episodes, runtime,
genres, high-confidence tags, studios, score, poster/backdrop artwork, AniList
IDs, MAL IDs, and canonical AniList URLs. Set
`NAKO_METADATA_SCRAPER_ANILIST_PROXY_URL` when AniList traffic must use an
operator-managed proxy.

The Addon Health Check diagnostics and `/ui/diagnostics` show whether TMDB,
Bangumi, AniList, Jav321, Prestige, ThePornDB, and browser-render proxy/session
policy is configured. They intentionally expose only boolean policy state, not
proxy URLs, credentials, or session key values.
Browser-rendered providers use the companion browser worker for proxying; set
`NAKO_BROWSER_WORKER_PROXY_URL` or `NAKO_BROWSER_WORKER_PROXY_LIST` on that
worker. The Rust sidecar can require, bypass, or default that worker proxy via
`NAKO_METADATA_SCRAPER_BROWSER_WORKER_PROXY_POLICY`, and can also pass
`NAKO_METADATA_SCRAPER_BROWSER_WORKER_WAIT_STATE`,
`NAKO_METADATA_SCRAPER_BROWSER_WORKER_WAIT_SELECTOR`,
`NAKO_METADATA_SCRAPER_BROWSER_WORKER_WAIT_TIMEOUT_MS`, and
`NAKO_METADATA_SCRAPER_BROWSER_WORKER_SESSION_KEY` as typed render intent
defaults. Browser-worker render intents can also carry provider-owned headers
and bounded page actions; these are used for site-specific operational gates
without moving metadata parsing into the worker. Browser-worker failures can
include redaction-safe `failure_kind` values such as `operator_action` or
`selector_timeout`; the sidecar maps these into provider execution failure
classes without exposing URLs, selectors, cookies, or proxy values.

AV provider presets can enable coherent provider groups without setting every
provider toggle individually. Set `NAKO_METADATA_SCRAPER_AV_PROVIDER_PRESET` to
one of:

- `manual`: catalog defaults only; this preserves the default fixture-only
  local smoke behavior.
- `fast_safe`: `javdb`, `dmm`, `fc2`, `mgstage`, `prestige`, and
  `theporndb`.
- `official_only`: `dmm`, `fc2`, `mgstage`, `prestige`, `caribbean`,
  `1pondo`, and `10musume`.
- `community_first`: `javdb`, `jav321`, `javbus`, `javlibrary`, `airav`,
  `avsox`, `dmm`, `xcity`, `fc2`, `fc2ppvdb`, `mgstage`, `prestige`, and
  `theporndb`.
- `fc2_enhanced`: `fc2`, `fc2ppvdb`, `airav`, and `avsox`.
- `uncensored_official`: `caribbean`, `1pondo`, and `10musume`.

The preset is only the default AV enablement policy. Any explicit
`NAKO_METADATA_SCRAPER_PROVIDER_*_ENABLED` value wins over the preset, so an
operator can disable one unstable site or add a specialty provider while
keeping the named base strategy.

AV field fusion also has a sidecar-wide preset. Set
`NAKO_METADATA_SCRAPER_AV_FIELD_POLICY_PRESET` to one of:

- `default`: uses the field source order adapted to this project: title from
  ThePornDB/MGStage/DMM/JavBus/Jav321/JavLibrary; outline from
  ThePornDB/DMM/Jav321; actors from ThePornDB/JavBus/JavLibrary/JavDB;
  thumbnail and poster art from ThePornDB/JavBus; extra fanart, tags,
  release/runtime, directors, series, studio, publisher, maker, and label from
  JavBus-first supported sources; score from Jav321/JavLibrary/JavDB; wanted
  count from JavLibrary/JavDB; trailers from MGStage/DMM.
- `quality_scores`: the previous descriptor-derived policy based on provider
  official/community/artwork/trailer quality scores.
- `none`: base candidate fields only unless the request supplies
  `provider_field_policy`.

Request payload `provider_field_policy` remains the narrowest override and
replaces the configured preset for that one scrape.

Manual AV provider drift checks are opt-in and ignored by default. They report
only provider IDs, field names, missing-field lists, and counts; raw titles,
actors, source URLs, artwork URLs, provider IDs, and AV numbers are not printed.
Configure one or more cases as `provider=AV-NUMBER`, start the browser worker
for rendered providers, and run:

```powershell
$env:NAKO_METADATA_SCRAPER_LIVE_PROVIDER_DRIFT = '1'
$env:NAKO_METADATA_SCRAPER_LIVE_AV_PROVIDER_DRIFT_CASES = 'javdb=SSNI-644;jav321=SSNI-644;fc2=FC2-1723984'
cargo test -p nako-metadata-scraper --test live_provider_drift -- --ignored av_live_provider_field_health_smoke
```

The harness enables only the AV providers named in the case list, then calls
the same provider registry and `MetadataProvider::suggest` seam used by runtime
scraping. Browser-rendered providers inherit
`NAKO_METADATA_SCRAPER_BROWSER_WORKER_*` render settings and proxy policy.
Jav321 uses the Rust HTTP runtime directly; set
`NAKO_METADATA_SCRAPER_JAV321_PROXY_URL` when it must use an operator-managed
proxy.

For the lower-level Browser Worker render drift harness, metadata-scraper can
generate provider-owned live case JSON for enabled rendered providers:

```powershell
$env:NAKO_METADATA_SCRAPER_PROVIDER_DOUBAN_ENABLED = 'true'
$env:NAKO_METADATA_SCRAPER_PROVIDER_JAVBUS_ENABLED = 'true'
$env:NAKO_METADATA_SCRAPER_PROVIDER_JAVLIBRARY_ENABLED = 'true'
$env:NAKO_METADATA_SCRAPER_RENDER_DRIFT_SAMPLE_AV_NUMBER = 'SSNI-644'
$cases = cargo run -q -p nako-metadata-scraper -- render-drift-cases

$env:NAKO_BROWSER_WORKER_LIVE_RENDER_DRIFT = '1'
$env:NAKO_BROWSER_WORKER_LIVE_RENDER_DRIFT_CASES = $cases
npm --prefix addons/browser-worker run live:render-drift
```

This avoids hand-writing provider URLs, selectors, and JavBus age-gate actions.
The generated cases currently cover Douban search, DMM search, JavBus detail,
JavLibrary search, XCity search, AirAV search, AVSox search, MGStage detail,
JavDB search, FC2 detail, FC2PPVDB detail, Caribbean detail, 1Pondo detail,
and 10Musume detail. Safe render defaults such as `proxy_policy` are emitted;
session keys, cookies, and header values are not.

Metadata requests may provide explicit `external_ids` or top-level aliases:
`tmdb_id`, `tmdb_tv_id`, `imdb_id`, `bangumi_id`, `bgm_id`, `anilist_id`,
`mal_id`, `browser_worker_url`, `browser_worker_recipe_url`, `javdb_id`,
`dmm_id`, `dmm_url`, `xcity_id`, `xcity_url`, `fc2_id`, `fc2ppvdb_id`,
`fc2ppvdb_url`,
`caribbean_id`, `caribbean_url`, `1pondo_id`, `1pondo_url`, `10musume_id`,
`10musume_url`, `jav321_id`, `jav321_url`, `javbus_id`, `javbus_url`,
`javlibrary_id`, `javlibrary_url`, `airav_id`, `airav_url`, `avsox_id`,
`avsox_url`, `mgstage_id`, `mgstage_url`, `prestige_id`, `prestige_url`, `theporndb_id`,
`theporndb_url`, `file_oshash`, `file_phash`, and `av_number`. These aliases
are derived from provider-owned external ID capabilities.

Douban metadata is available when
`NAKO_METADATA_SCRAPER_PROVIDER_DOUBAN_ENABLED=true` and the browser-worker
companion service is reachable through
`NAKO_METADATA_SCRAPER_BROWSER_WORKER_BASE_URL`. The worker provides the
generic rendered HTML contract (`POST /render`); Douban search/detail parsing,
field mapping, ranking facts, and artwork candidates stay inside the Rust
provider. This keeps Playwright/Crawlee out of the Rust sidecar without turning
the worker into a second metadata scraper.

JavDB, DMM, FC2, FC2PPVDB, Caribbean, 1Pondo, 10Musume, Jav321, JavBus,
JavLibrary, MGStage, Prestige, and ThePornDB metadata are available when their
providers are enabled.
JavDB, DMM, FC2, FC2PPVDB, Caribbean, 1Pondo, 10Musume, JavBus, JavLibrary, and
MGStage use the browser-worker companion service for rendered HTML. Jav321 posts
form search requests and parses raw HTML in the Rust sidecar. Prestige
uses the official JSON API and can use
`NAKO_METADATA_SCRAPER_PRESTIGE_PROXY_URL` directly from the Rust sidecar.
ThePornDB uses the ThePornDB JSON API, requires
`NAKO_METADATA_SCRAPER_THEPORNDB_API_TOKEN`, and can use
`NAKO_METADATA_SCRAPER_THEPORNDB_PROXY_URL` directly from the Rust sidecar.
JavDB searches by normalized non-FC2 AV numbers and supports explicit
`javdb_id` direct lookup. DMM is an official censored-release tracer that
searches by normalized AV number and supports explicit `dmm_id` or `dmm_url`
direct lookup. FC2 handles FC2-number direct article lookup and supports
explicit `fc2_id` direct lookup. FC2PPVDB is an FC2 long-tail fallback that
uses deterministic article URLs and supports `fc2ppvdb_id` or `fc2ppvdb_url`
direct lookup. Caribbean, 1Pondo, and 10Musume are official uncensored sources
for date-style IDs such as `010116-001`, `010116_001`, and `010116_01`; they
support `caribbean_id`/`caribbean_url`, `1pondo_id`/`1pondo_url`, and
`10musume_id`/`10musume_url` direct lookup. Jav321 is a raw-HTML community
fallback for normalized censored, uncensored, and amateur numbers; it posts
`sn=<AV number>` to `/search`, parses the returned detail HTML for title,
outline, score, actors, release date, runtime, studio/publisher, series, tags,
thumbnail/poster, and extra fanart, and supports `jav321_id` or `jav321_url`
direct lookup. JavBus is a broad
disabled-by-default AV fallback for normalized censored and uncensored numbers
and supports explicit `javbus_id` or
`javbus_url` direct lookup. JavBus first attempts a direct detail URL, then
falls back to search, and rejects age-verification pages instead of returning
them as metadata candidates. Set `NAKO_METADATA_SCRAPER_JAVBUS_COOKIE` when
JavBus requires an operator-provided age/region cookie. JavLibrary contributes
community facts such as
actors, score, and wanted count, and supports `javlibrary_id` or
`javlibrary_url` direct lookup. MGStage is a route-specific official source for
amateur/MGS numbers such as `300MIUM-382`, and supports `mgstage_id` or
`mgstage_url` direct lookup. Prestige is a censored-route official source and
supports `prestige_id` or `prestige_url` direct lookup. ThePornDB supports
scene search and explicit `theporndb_id` or `theporndb_url` scene detail lookup,
direct scene hash lookup through `file_oshash` or `file_phash`, and maps
performers, directors, site/network, tags, poster/backdrop artwork, trailer URL,
rating, runtime, and source links. These AV providers emit `av_number` external
IDs so the resolver can merge compatible AV facts across sources.

ThePornDB hash lookup accepts either top-level fields or `external_ids`, for
example `{"external_ids": {"file_oshash": "d7dae9cd888c5984"}}`. It calls
`/scenes/hash/{hash}` before ID, AV-number, or title search. Movie hash lookup
is kept separate until the query contract can distinguish scene and movie
intent.

ThePornDB environment knobs:

- `NAKO_METADATA_SCRAPER_PROVIDER_THEPORNDB_ENABLED=true`
- `NAKO_METADATA_SCRAPER_THEPORNDB_API_TOKEN=<ThePornDB bearer token>`
- `NAKO_METADATA_SCRAPER_THEPORNDB_API_BASE_URL=https://api.theporndb.net`
- `NAKO_METADATA_SCRAPER_THEPORNDB_PUBLIC_BASE_URL=https://theporndb.net`
- `NAKO_METADATA_SCRAPER_THEPORNDB_TIMEOUT_MS=10000`
- `NAKO_METADATA_SCRAPER_THEPORNDB_PROXY_URL=http://127.0.0.1:10809`

AniList environment knobs:

- `NAKO_METADATA_SCRAPER_PROVIDER_ANILIST_ENABLED=true`
- `NAKO_METADATA_SCRAPER_ANILIST_ACCESS_TOKEN=<optional AniList bearer token>`
- `NAKO_METADATA_SCRAPER_ANILIST_GRAPHQL_URL=https://graphql.anilist.co`
- `NAKO_METADATA_SCRAPER_ANILIST_INCLUDE_ADULT=false`
- `NAKO_METADATA_SCRAPER_ANILIST_TIMEOUT_MS=10000`
- `NAKO_METADATA_SCRAPER_ANILIST_PROXY_URL=http://127.0.0.1:10809`

JavBus environment knobs:

- `NAKO_METADATA_SCRAPER_PROVIDER_JAVBUS_ENABLED=true`
- `NAKO_METADATA_SCRAPER_JAVBUS_BASE_URL=https://www.javbus.com`
- `NAKO_METADATA_SCRAPER_JAVBUS_TIMEOUT_MS=10000`
- `NAKO_METADATA_SCRAPER_JAVBUS_COOKIE=<cookie header value>`

Jav321 environment knobs:

- `NAKO_METADATA_SCRAPER_PROVIDER_JAV321_ENABLED=true`
- `NAKO_METADATA_SCRAPER_JAV321_BASE_URL=https://www.jav321.com`
- `NAKO_METADATA_SCRAPER_JAV321_TIMEOUT_MS=10000`
- `NAKO_METADATA_SCRAPER_JAV321_PROXY_URL=http://127.0.0.1:10809`

AV candidates also expose a response-side `av` object for AV-specific evidence:
actors, all actors, directors, series, studio, publisher, maker, label, wanted
count, thumbnail URL, trailer URL, and extra fanart URLs. For writeback, the
selected AV facts are materialized into the native metadata patch: actors and
directors become credits, series becomes a collection, studio/maker/publisher
and label become studios, provider IDs become external IDs, and artwork/AV image
URLs become image references. The `av` object remains useful for diagnostics and
provider field-source evidence.

The runtime then resolves exact duplicate candidates and candidates that share
declared provider-emitted external IDs, caps the final list, and applies a
small generic bonus from the shared community score/vote-count facts exposed by
TMDB, Bangumi, Douban, and AniList. The resource response envelope does not
change.

Each metadata response includes `provider_execution`, which records the
provider IDs selected, skipped by AV route, suppressed by request policy,
skipped by provider budget, returned, empty, or failed with a safe failure
category. This is the single-scrape counterpart to the bulk provider summary.
A direct metadata request may include `provider_execution_policy` to suppress
providers or cap selected providers for that one request:

```json
{
  "provider_execution_policy": {
    "disabled_provider_ids": ["javlibrary"],
    "max_selected_providers": 3
  }
}
```

The response echoes this as `provider_execution.applied_policy` and reports
budget-skipped providers in `provider_execution.budget_exhausted_provider_ids`.
Operators may set `NAKO_METADATA_SCRAPER_PROVIDER_MAX_SELECTED_PER_REQUEST` as
the default sidecar-wide provider budget. Bulk scrape injects the same policy
shape into each item when applying its explicit batch provider policy and
resume state.

Requests may optionally include `provider_field_policy` for field-level source
priority inside an already-merged candidate cluster:

```json
{
  "av_number": "SSNI-644",
  "provider_field_policy": {
    "title": ["javdb"],
    "overview": ["dmm"],
    "tags": ["dmm"],
    "actors": ["javdb"],
    "studio": ["dmm"],
    "score": ["jav321", "javlibrary", "javdb"]
  }
}
```

Supported AV alias fields include `outline`, `actor`, `thumb`, `extrafanart`,
`trailer`, `tag`, `release`, `runtime`, `director`, `wanted`, and `score`.
Canonical fields such as `community_score_milli` and `community_vote_count`
also work for callers that want the exact internal fact names.

The policy does not merge unrelated candidates by itself; it only chooses fields
after providers have emitted compatible external IDs such as the same
`av_number`. When no request policy is supplied, AV clusters use
`NAKO_METADATA_SCRAPER_AV_FIELD_POLICY_PRESET` and default to `default`. Passing
an explicit `provider_field_policy` object replaces that configured default for the
request.

Future provider breadth will come through the runtime seam, not by turning each
provider into its own addon. The browser-worker companion service now owns the
Playwright/Crawlee path for rendered-page extraction, while provider-specific
metadata interpretation and render intent declaration remain in
`nako-metadata-scraper`. AV providers share parser primitives for row-level
metadata labels, but each provider keeps its own row selectors and media/link
rules so parser quality improves without flattening site-specific page models.
