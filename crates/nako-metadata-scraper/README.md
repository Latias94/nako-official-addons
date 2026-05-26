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
  It also accepts `NAKO_METADATA_SCRAPER_TMDB_PROXY_URL` for proxied access.
- `bangumi`: disabled by default; public subject search works without a token
  and requires a compliant User-Agent. It also accepts
  `NAKO_METADATA_SCRAPER_BANGUMI_PROXY_URL` for proxied access. It maps
  official subject facts such as NSFW/locked/series flags, episode and
  collection counts, ratings, selected infobox facts, tags, and poster artwork.
- `browser_worker`: disabled by default; uses the companion browser worker for
  rendered-page extraction when an external browser-worker URL is supplied.
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
- `javbus`: disabled by default; calls the companion browser worker for
  rendered HTML and acts as a broad AV fallback for normalized censored and
  uncensored numbers. It emits `javbus`, `javbus_url`, and `av_number`
  external IDs.
- `javlibrary`: disabled by default; calls the companion browser worker for
  rendered HTML and contributes community AV facts such as actors, score, and
  wanted count. It emits `javlibrary`, `javlibrary_url`, and `av_number`
  external IDs.
- `mgstage`: disabled by default; calls the companion browser worker for
  rendered HTML and acts as a route-specific official source for amateur/MGS
  numbers such as `300MIUM-382`. It emits `mgstage`, `mgstage_url`, and
  `av_number` external IDs.

Metadata requests may provide explicit `external_ids` or top-level aliases:
`tmdb_id`, `imdb_id`, `bangumi_id`, `browser_worker_url`, `javdb_id`, `dmm_id`,
`dmm_url`, `fc2_id`, `javbus_id`, `javbus_url`, `javlibrary_id`,
`javlibrary_url`, `mgstage_id`, `mgstage_url`, and `av_number`. These aliases
are derived from provider-owned external ID capabilities.

AV-oriented requests may also provide `number`, `file_name`, `filename`, or
`path`. The scraper normalizes common AV number shapes such as `SSNI-00644` and
`FC2PPV-1723984` before provider search. Normal scrape responses include
redaction-safe `query.av` facts when a number is recognized; full local paths
are not echoed.

When `javdb_id`, `dmm_id`, `dmm_url`, `fc2_id`, `javbus_id`, `javbus_url`,
`javlibrary_id`, `javlibrary_url`, `mgstage_id`, or `mgstage_url` is supplied,
the matching provider performs direct detail lookup before falling back to
inferred AV-number search. This is useful for appointed-source corrections
where a user already knows the authoritative site record.

Every metadata response includes `provider_execution`, a redaction-safe summary
of the provider wave. It records provider IDs that were selected, skipped by AV
route, suppressed by request policy, returned candidates, returned no
candidates, or failed with a safe failure category. Provider errors are logged
with a safe category and are not echoed as raw error text in the response. A
request may include `disabled_provider_ids` to suppress providers for that one
scrape; the response then reports `provider_execution.suppressed_provider_ids`.

Requests may optionally include `provider_field_policy` to choose field-level
source priority within a merged candidate cluster. For example, a request can
prefer JavDB for `title` while using another provider for `overview` and
`tags`:

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

The policy only mixes fields inside candidates that already share an identity
such as `av_number`; unrelated candidates are not merged by policy alone.
When no request policy is supplied, AV clusters use a conservative built-in
policy inspired by MDCx's field-priority behavior: DMM is preferred before
MGStage, JavDB, FC2, JavBus, and JavLibrary for official title, overview,
release/runtime, and studio-like facts. Community actor and wanted-count fields
prefer JavLibrary/JavDB first. Trailer and image fields prefer providers that
usually carry media URLs, starting with MGStage/DMM/JavDB. Passing an explicit
`provider_field_policy` object replaces that default for the request.

Runtime candidate shaping resolves exact duplicate provider candidates and
candidates that share declared provider-emitted external IDs before ranking,
caps the final result set, and uses shared community score/vote-count facts
from TMDB, Bangumi, and Douban as a small generic ranking bonus.
AV provider routing now uses declared route support so FC2 numbers stay on the
FC2 path, while censored AV numbers can fan out to enabled JavDB/DMM/JavBus
providers.
Ranked candidate evidence also carries redaction-safe provider-source and
field-source metadata when shared external IDs merge multiple provider facts.

The `/health` diagnostics report whether TMDB and Bangumi proxy policy is
configured without exposing the proxy URL itself. Browser-rendered AV providers
use proxy configuration from the companion browser worker, for example
`NAKO_BROWSER_WORKER_PROXY_URL` or `NAKO_BROWSER_WORKER_PROXY_LIST`.

Explicit `metadata_write` submission is available only when the request payload
contains a `writeback` object and the disabled-by-default Nako runtime side
effect config is enabled. Ordinary metadata calls remain suggestion-only. When
writeback is requested, selected AV facts are materialized into the native
metadata patch as credits, studios, collections, external IDs, and image
references instead of staying response-only.

Typed artwork candidates are returned with ranked metadata candidates. Explicit
`artwork_write` submission is available only when the request payload contains
an `artwork_writeback` object and Nako grants `artwork_write` for the target
library.

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
    "cooldown_items": 3
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
counts, and per-item `suppressed_provider_ids`.

Rendered AV providers use the companion browser worker through `POST /render`.
The worker is a Crawlee/Playwright execution boundary: it loads pages and
returns rendered HTML/text/excerpts, while Rust providers own site-specific
search, detail parsing, mapping, and source policy.

Optional live drift smoke checks are available for manual use only:

```bash
NAKO_METADATA_SCRAPER_LIVE_PROVIDER_DRIFT=1 cargo test -p nako-metadata-scraper --test live_provider_drift -- --ignored
```

TMDB requires `NAKO_METADATA_SCRAPER_TMDB_READ_ACCESS_TOKEN` to be set in the
environment before that command can do anything useful.

Version `0.1.0-alpha.2` targets Nako Addon Protocol `0.1.0-alpha.1` and
`nako-addon-protocol` Rust crate `0.1.0-alpha.2`.

Run locally:

```bash
cargo run -p nako-metadata-scraper
```

Default endpoint: `http://127.0.0.1:9100/manifest.json`.
