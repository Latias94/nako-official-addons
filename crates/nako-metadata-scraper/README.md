# Nako Metadata Scraper

Official Nako metadata scraper Addon Sidecar.

This crate exposes one HTTP sidecar that implements the Nako Addon Protocol
metadata resource. Provider modules are internal implementation details behind
the shared provider registry, HTTP runtime, and ranking model.

Main Nako repository: <https://github.com/Latias94/nako>.
Official addons repository: <https://github.com/Latias94/nako-official-addons>.

Current alpha provider defaults:

- `fixture`: enabled by default for smoke tests.
- `tmdb`: disabled by default; requires a TMDB read access token when enabled.
  It maps movie and TV-series search/detail results, supports explicit
  `tmdb_id`, `tmdb_tv_id`, and `imdb_id` lookup, and accepts
  `NAKO_METADATA_SCRAPER_TMDB_PROXY_URL` for proxied access.
- `bangumi`: disabled by default; public subject search works without a token
  and requires a compliant User-Agent. It also accepts
  `NAKO_METADATA_SCRAPER_BANGUMI_PROXY_URL` for proxied access. It maps
  official subject facts such as NSFW/locked/series flags, episode and
  collection counts, ratings, selected infobox facts, tags, and poster artwork.
- `anilist`: disabled by default; calls the official AniList GraphQL API for
  anime search plus explicit `anilist_id` or `mal_id` lookup. Public metadata
  works without a token; `NAKO_METADATA_SCRAPER_ANILIST_ACCESS_TOKEN` is an
  optional secret for authenticated requests, and
  `NAKO_METADATA_SCRAPER_ANILIST_PROXY_URL` enables proxied API access.
- `browser_worker`: disabled by default; uses the companion browser worker for
  rendered-page extraction when an external browser-worker URL is supplied. It
  supports explicit `browser_worker_url` text extraction and
  `browser_worker_recipe_url` rendered metadata recipes.
- `douban`: disabled by default; calls the companion browser worker for rendered
  HTML and keeps Douban parsing/mapping inside the Rust provider.
- `javdb`: disabled by default; calls the companion browser worker for rendered
  HTML and searches by normalized AV number. It emits `javdb`, `javdb_url`, and
  `av_number` external IDs.
- `dmm`: disabled by default; calls the companion browser worker for rendered
  HTML and acts as an official censored-release AV tracer. It searches by
  normalized AV number, supports explicit `dmm_id` or `dmm_url` direct lookup,
  and emits `dmm`, `dmm_url`, and `av_number` external IDs.
- `fc2`: disabled by default; calls the companion browser worker for rendered
  HTML and uses FC2 AV numbers for direct article lookup. It emits `fc2`,
  `fc2_url`, and `av_number` external IDs.
- `fc2ppvdb`: disabled by default; calls the companion browser worker for
  rendered HTML and acts as an FC2 long-tail fallback. It searches deterministic
  FC2PPVDB article URLs by normalized FC2 number, supports `fc2ppvdb_id` or
  `fc2ppvdb_url` direct lookup, and emits `fc2ppvdb`, `fc2ppvdb_url`, and
  `av_number` external IDs.
- `caribbean`: disabled by default; calls the companion browser worker for
  rendered HTML and acts as an official uncensored source for date-style IDs
  such as `010116-001`. It supports `caribbean_id` or `caribbean_url` direct
  lookup and emits `caribbean`, `caribbean_url`, and `av_number` external IDs.
- `1pondo`: disabled by default; calls the companion browser worker for
  rendered HTML and acts as an official uncensored source for date-style IDs
  such as `010116_001`. It supports `1pondo_id` or `1pondo_url` direct lookup
  and emits `1pondo`, `1pondo_url`, and `av_number` external IDs.
- `10musume`: disabled by default; calls the companion browser worker for
  rendered HTML and acts as an official uncensored source for date-style IDs
  such as `010116_01`. It supports `10musume_id` or `10musume_url` direct
  lookup and emits `10musume`, `10musume_url`, and `av_number` external IDs.
- `jav321`: disabled by default; posts the normalized AV number to Jav321 search
  and parses the returned raw detail HTML without the browser worker. It
  contributes title, outline, score, actors, release date, runtime, studio,
  publisher, series, tags, thumbnail/poster art, and extra fanart. It supports
  `jav321_id` or `jav321_url` direct lookup, emits `jav321`, `jav321_url`, and
  `av_number` external IDs, and accepts
  `NAKO_METADATA_SCRAPER_JAV321_PROXY_URL` for proxied access.
- `javbus`: disabled by default; calls the companion browser worker for
  rendered HTML and acts as a broad AV fallback for normalized censored and
  uncensored numbers. It tries a direct detail URL before search, rejects
  age-verification pages as non-candidates, accepts
  `NAKO_METADATA_SCRAPER_JAVBUS_COOKIE` when operator cookie access is needed,
  and emits `javbus`, `javbus_url`, and `av_number` external IDs.
- `javlibrary`: disabled by default; calls the companion browser worker for
  rendered HTML and contributes community AV facts such as actors, score, and
  wanted count. It emits `javlibrary`, `javlibrary_url`, and `av_number`
  external IDs.
- `mgstage`: disabled by default; calls the companion browser worker for
  rendered HTML and acts as a route-specific official source for amateur/MGS
  numbers such as `300MIUM-382`. It emits `mgstage`, `mgstage_url`, and
  `av_number` external IDs.
- `prestige`: disabled by default; calls the official Prestige JSON API for
  censored AV search/detail lookup. It emits `prestige`, `prestige_url`, and
  `av_number` external IDs, and accepts `NAKO_METADATA_SCRAPER_PRESTIGE_PROXY_URL`
  for proxied API access.
- `theporndb`: disabled by default; calls the ThePornDB JSON API for AV scene
  search/detail lookup and requires `NAKO_METADATA_SCRAPER_THEPORNDB_API_TOKEN`
  when enabled. It emits `theporndb`, `theporndb_url`, and `av_number` external
  IDs, supports `file_oshash`/`file_phash` scene hash lookup, and accepts
  `NAKO_METADATA_SCRAPER_THEPORNDB_PROXY_URL` for proxied API access.

Metadata requests may provide explicit `external_ids` or top-level aliases:
`tmdb_id`, `tmdb_tv_id`, `imdb_id`, `bangumi_id`, `bgm_id`, `anilist_id`,
`mal_id`, `browser_worker_url`, `browser_worker_recipe_url`, `javdb_id`,
`dmm_id`, `dmm_url`, `fc2_id`, `fc2ppvdb_id`, `fc2ppvdb_url`, `caribbean_id`,
`caribbean_url`, `1pondo_id`, `1pondo_url`, `10musume_id`, `10musume_url`,
`jav321_id`, `jav321_url`, `javbus_id`, `javbus_url`, `javlibrary_id`,
`javlibrary_url`, `mgstage_id`, `mgstage_url`, `prestige_id`, `prestige_url`,
`theporndb_id`,
`theporndb_url`, `file_oshash`, `file_phash`, and `av_number`. These aliases
are derived from provider-owned external ID capabilities.

AV-oriented requests may also provide `number`, `file_name`, `filename`, or
`path`. The scraper normalizes common AV number shapes such as `SSNI-00644` and
`FC2PPV-1723984`, plus official uncensored date-style IDs such as
`010116-001`, before provider search. Normal scrape responses include
redaction-safe `query.av` facts when a number is recognized; full local paths
are not echoed.

When `javdb_id`, `dmm_id`, `dmm_url`, `fc2_id`, `fc2ppvdb_id`,
`fc2ppvdb_url`, `caribbean_id`, `caribbean_url`, `1pondo_id`, `1pondo_url`,
`10musume_id`, `10musume_url`, `jav321_id`, `jav321_url`, `javbus_id`,
`javbus_url`, `javlibrary_id`, `javlibrary_url`, `mgstage_id`, `mgstage_url`,
`prestige_id`, `prestige_url`, `theporndb_id`, or `theporndb_url` is supplied,
the matching provider performs direct detail lookup before falling back to
inferred AV-number search. This is useful for appointed-source corrections
where a user already knows the authoritative site record.

When `file_oshash` or `file_phash` is supplied, ThePornDB performs direct scene
hash lookup through `/scenes/hash/{hash}` before ID, AV-number, or title search.
The request can use top-level fields or `external_ids`, for example
`{"external_ids": {"file_oshash": "d7dae9cd888c5984"}}`. Movie hash lookup is
kept separate until the query contract can distinguish scene and movie intent.

Every metadata response includes `provider_execution`, a redaction-safe summary
of the provider wave. It records provider IDs that were selected, skipped by AV
route, suppressed by request policy, returned candidates, returned no
candidates, skipped by provider budget, or failed with a safe failure category.
Provider errors are logged with a safe category and are not echoed as raw error
text in the response. A request may include `provider_execution_policy` to
suppress providers for that scrape or cap the number of selected providers; the
applied policy is echoed in `provider_execution.applied_policy`, and suppressed
or budget-skipped providers are reported by provider ID only.

```json
{
  "provider_execution_policy": {
    "disabled_provider_ids": ["javlibrary"],
    "max_selected_providers": 3
  }
}
```

Operators may also set
`NAKO_METADATA_SCRAPER_PROVIDER_MAX_SELECTED_PER_REQUEST` as a default provider
budget for all metadata requests served by the sidecar.

AV field fusion has a sidecar-wide preset through
`NAKO_METADATA_SCRAPER_AV_FIELD_POLICY_PRESET`:

- `default`: uses the default field source order adapted to supported providers.
- `quality_scores`: descriptor-derived provider quality order.
- `none`: base candidate fields only unless a request override is supplied.

Requests may optionally include `provider_field_policy` to choose field-level
source priority within a merged candidate cluster. For example, a request can
prefer JavDB for `title` while using another provider for `overview` and
`tags`. AV-friendly aliases such as `outline`, `actor`, `thumb`, `trailer`,
`tag`, `release`, `runtime`, `director`, `wanted`, and `score` are accepted
alongside canonical fields such as `community_score_milli` and
`community_vote_count`:

```json
{
  "av_number": "SSNI-644",
  "provider_field_policy": {
    "title": ["javdb"],
    "overview": ["dmm"],
    "tags": ["dmm"],
    "score": ["jav321", "javlibrary", "javdb"]
  }
}
```

The policy only mixes fields inside candidates that already share an identity
such as `av_number`; unrelated candidates are not merged by policy alone.
When no request policy is supplied, AV clusters use the configured
`NAKO_METADATA_SCRAPER_AV_FIELD_POLICY_PRESET` and default to `default`. Passing
an explicit `provider_field_policy` object replaces that configured default for the
request.

Runtime candidate shaping resolves exact duplicate provider candidates and
candidates that share declared provider-emitted external IDs before ranking,
caps the final result set, and uses shared community score/vote-count facts as
a small generic ranking bonus.
AV provider routing now uses declared route support so FC2 numbers stay on the
FC2 path, while censored AV numbers can fan out to enabled
JavDB/DMM/Jav321/JavBus, Prestige, and ThePornDB providers. Official
uncensored date-style IDs fan out only to enabled Caribbean/1Pondo/10Musume and
uncensored-capable fallback providers. Western-style AV numbers can fan out to
ThePornDB when configured.
Ranked candidate evidence also carries redaction-safe provider-source and
field-source metadata when shared external IDs merge multiple provider facts.

The `/health` diagnostics report whether TMDB/Bangumi/AniList/Jav321/Prestige/
ThePornDB proxy policy and browser render proxy/session policy are configured
without exposing proxy URLs, credentials, or session key values. Browser-rendered AV
providers use proxy configuration from the companion browser worker, for example
`NAKO_BROWSER_WORKER_PROXY_URL` or `NAKO_BROWSER_WORKER_PROXY_LIST`. Rust
providers send a typed render intent to the worker; operators can set
`NAKO_METADATA_SCRAPER_BROWSER_WORKER_WAIT_STATE` (`load`, `domcontentloaded`,
or `networkidle`), `NAKO_METADATA_SCRAPER_BROWSER_WORKER_WAIT_SELECTOR`,
`NAKO_METADATA_SCRAPER_BROWSER_WORKER_WAIT_TIMEOUT_MS`,
`NAKO_METADATA_SCRAPER_BROWSER_WORKER_PROXY_POLICY` (`default`, `direct`, or
`required`), and `NAKO_METADATA_SCRAPER_BROWSER_WORKER_SESSION_KEY` to shape all
rendered-page requests without changing provider code. Browser-worker failures
can include redaction-safe `failure_kind` values such as `operator_action` or
`selector_timeout`; the sidecar maps these into provider execution failure
classes without exposing URLs, selectors, cookies, or proxy values.

JavBus may require an age or region cookie depending on network location. Set
`NAKO_METADATA_SCRAPER_JAVBUS_COOKIE` to the raw Cookie header value; it is sent
only to the browser worker as a page request header and is not emitted in
diagnostics. Without a valid cookie, age-verification pages are treated as
access gates and do not produce metadata candidates.

Jav321 raw HTML access is configured directly on the Rust sidecar:

- `NAKO_METADATA_SCRAPER_PROVIDER_JAV321_ENABLED=true`
- `NAKO_METADATA_SCRAPER_JAV321_BASE_URL=https://www.jav321.com`
- `NAKO_METADATA_SCRAPER_JAV321_TIMEOUT_MS=10000`
- `NAKO_METADATA_SCRAPER_JAV321_PROXY_URL=http://127.0.0.1:10809`

ThePornDB API access is configured directly on the Rust sidecar:

- `NAKO_METADATA_SCRAPER_PROVIDER_THEPORNDB_ENABLED=true`
- `NAKO_METADATA_SCRAPER_THEPORNDB_API_TOKEN=<ThePornDB bearer token>`
- `NAKO_METADATA_SCRAPER_THEPORNDB_API_BASE_URL=https://api.theporndb.net`
- `NAKO_METADATA_SCRAPER_THEPORNDB_PUBLIC_BASE_URL=https://theporndb.net`
- `NAKO_METADATA_SCRAPER_THEPORNDB_TIMEOUT_MS=10000`
- `NAKO_METADATA_SCRAPER_THEPORNDB_PROXY_URL=http://127.0.0.1:10809`

AniList API access is configured directly on the Rust sidecar:

- `NAKO_METADATA_SCRAPER_PROVIDER_ANILIST_ENABLED=true`
- `NAKO_METADATA_SCRAPER_ANILIST_ACCESS_TOKEN=<optional AniList bearer token>`
- `NAKO_METADATA_SCRAPER_ANILIST_GRAPHQL_URL=https://graphql.anilist.co`
- `NAKO_METADATA_SCRAPER_ANILIST_INCLUDE_ADULT=false`
- `NAKO_METADATA_SCRAPER_ANILIST_TIMEOUT_MS=10000`
- `NAKO_METADATA_SCRAPER_ANILIST_PROXY_URL=http://127.0.0.1:10809`

Explicit `metadata_write` submission is available only when the request payload
contains a `writeback` object and the disabled-by-default Nako runtime side
effect config is enabled. Ordinary metadata calls remain suggestion-only. When
writeback is requested, selected AV facts are materialized into the native
metadata patch as credits, studios, collections, external IDs, and image
references instead of staying response-only. Metadata writes require a
`media_source` target and return `invalid_metadata_target_kind` before access
checks when another target kind is requested.

Typed artwork candidates are returned with ranked metadata candidates. Explicit
`artwork_write` submission is available only when the request payload contains
an `artwork_writeback` object and Nako grants `artwork_write` for the target
library. Artwork writes require a `media_item` target and return
`invalid_artwork_target_kind` before access checks when another target kind is
requested.

Bulk Metadata Scrape is declared as the `bulk-metadata-scrape` Addon Task at
`/tasks/bulk-metadata-scrape`. Nako owns task execution, progress, retry, and
cancellation; this crate owns the bounded batch planner and metadata/item
scrape execution behind that task path. Each bulk item also includes an optional
`av` summary copied from `payload.query.av`, so batch runs can explain which AV
number and route were used without exposing raw file paths. Within one bounded
batch, duplicate AV numbers without metadata/artwork writeback requests reuse
the first scrape result and report `reused_from_index`; items with empty
candidate lists report `safe_failure_reason`.

Bulk requests may pass a previous output `resume_state` back into the next task
payload. The sidecar can then reuse safe duplicate AV-number results across
bounded batches while Nako still owns scheduling and retry. Bulk output also
includes `summary.failure_reasons`, `summary.failed_items`, and
`summary.provider_execution` so a batch runner can distinguish empty results,
provider failures, and route skips without parsing provider-specific payloads.
Reusable resume entries include typed `safe_failure_reason` and
`suppressed_provider_ids`, which keeps retry accounting separate from the
public item payload projection.

Bulk requests may also include a `provider_policy`:

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

The policy is explicit batch state, not a hidden scheduler. Retryable provider
failures (`timeout`, `rate_limited`, `provider_error`) increment a provider
failure streak and can add cooldown entries to
`resume_state.provider_states`; `auth_or_forbidden` is classified as
`operator_action`, while `not_found` and `parse_error` are permanent for
accounting. The next bulk request can pass the returned `resume_state` to keep
cooldown suppression across bounded batches. Output includes
`summary.suppressed_items`, `summary.retry_classes`, provider-level retry-class
counts, `summary.budget_exhausted_items`, provider-level budget counts, the
applied top-level `provider_policy`, and per-item `suppressed_provider_ids`.
`max_reusable_items` bounds the duplicate-AV resume cache and
`max_provider_states` bounds persisted cooldown state.

Rendered AV providers use the companion browser worker through `POST /render`.
The worker is a Crawlee/Playwright execution boundary: it loads pages and
returns rendered HTML/text/excerpts, while Rust providers own site-specific
search, detail parsing, mapping, source policy, and render intent declaration.
Provider parsers share a row-level structured label helper for AV detail pages:
each provider supplies its own metadata row selector, then falls back to
full-text label scanning when the page is not row-structured. This keeps
site-specific shape local while preventing one label value from swallowing
following description, trailer, or media text.

Optional live drift smoke checks are available for manual use only:

```bash
NAKO_METADATA_SCRAPER_LIVE_PROVIDER_DRIFT=1 cargo test -p nako-metadata-scraper --test live_provider_drift -- --ignored
```

TMDB requires `NAKO_METADATA_SCRAPER_TMDB_READ_ACCESS_TOKEN` to be set in the
environment before that command can do anything useful.

Rendered provider live render drift cases can be generated from provider-owned
URL, selector, and action presets:

```powershell
$env:NAKO_METADATA_SCRAPER_PROVIDER_DOUBAN_ENABLED = 'true'
$env:NAKO_METADATA_SCRAPER_PROVIDER_JAVBUS_ENABLED = 'true'
$env:NAKO_METADATA_SCRAPER_PROVIDER_JAVLIBRARY_ENABLED = 'true'
$env:NAKO_METADATA_SCRAPER_RENDER_DRIFT_SAMPLE_AV_NUMBER = 'SSNI-644'
cargo run -q -p nako-metadata-scraper -- render-drift-cases
```

The command prints the JSON array expected by Browser Worker
`NAKO_BROWSER_WORKER_LIVE_RENDER_DRIFT_CASES`. It currently emits cases for
enabled Douban, DMM, JavBus, JavLibrary, XCity, AirAV, AVSox, and MGStage
providers. Override samples with
`NAKO_METADATA_SCRAPER_RENDER_DRIFT_SAMPLE_DOUBAN_TITLE`,
`NAKO_METADATA_SCRAPER_RENDER_DRIFT_SAMPLE_AV_NUMBER`, or provider-specific AV
sample variables such as
`NAKO_METADATA_SCRAPER_RENDER_DRIFT_SAMPLE_MGSTAGE_AV_NUMBER`. Safe render
defaults such as `proxy_policy` are emitted; session keys, cookies, and header
values are not.

Version `0.1.0-alpha.2` targets Nako Addon Protocol `0.1.0-alpha.1` and
`nako-addon-protocol` Rust crate `0.1.0-alpha.2`.

Run locally:

```bash
cargo run -p nako-metadata-scraper
```

Default endpoint: `http://127.0.0.1:9100/manifest.json`.
