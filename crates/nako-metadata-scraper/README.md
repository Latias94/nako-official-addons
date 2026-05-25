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
- `fc2`: disabled by default; calls the companion browser worker for rendered
  HTML and uses FC2 AV numbers for direct article lookup. It emits `fc2`,
  `fc2_url`, and `av_number` external IDs.

Metadata requests may provide explicit `external_ids` or top-level aliases:
`tmdb_id`, `imdb_id`, `bangumi_id`, `browser_worker_url`, `javdb_id`, `fc2_id`,
and `av_number`. These aliases are derived from provider-owned external ID
capabilities.

AV-oriented requests may also provide `number`, `file_name`, `filename`, or
`path`. The scraper normalizes common AV number shapes such as `SSNI-00644` and
`FC2PPV-1723984` before provider search. Normal scrape responses include
redaction-safe `query.av` facts when a number is recognized; full local paths
are not echoed.

Runtime candidate shaping resolves exact duplicate provider candidates and
candidates that share declared provider-emitted external IDs before ranking,
caps the final result set, and uses shared community score/vote-count facts
from TMDB, Bangumi, and Douban as a small generic ranking bonus.
AV provider routing now uses declared route support so FC2 numbers stay on the
FC2 path and non-FC2 AV numbers stay on the JavDB path. Ranked candidate
evidence also carries redaction-safe provider-source and field-source metadata
when shared external IDs merge multiple provider facts.

The `/health` diagnostics report whether TMDB and Bangumi proxy policy is
configured without exposing the proxy URL itself.

Explicit `metadata_write` submission is available only when the request payload
contains a `writeback` object and the disabled-by-default Nako runtime side
effect config is enabled. Ordinary metadata calls remain suggestion-only.

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
candidate lists report `safe_failure_reason: "no_candidates"`.

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
