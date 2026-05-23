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
- `bangumi`: disabled by default; public subject search works without a token
  and requires a compliant User-Agent.
- `browser_worker`: disabled by default; uses the companion browser worker for
  rendered-page extraction when an external browser-worker URL is supplied.
- `douban`: disabled by default; calls the companion browser worker for rendered
  HTML and keeps Douban parsing/mapping inside the Rust provider.

Runtime candidate shaping deduplicates exact duplicate provider candidates,
caps the final result set, and uses shared community score/vote-count facts
from TMDB, Bangumi, and Douban as a small generic ranking bonus.

Explicit `metadata_write` submission is available only when the request payload
contains a `writeback` object and the disabled-by-default Nako runtime side
effect config is enabled. Ordinary metadata calls remain suggestion-only.

Typed artwork candidates are returned with ranked metadata candidates. Explicit
`artwork_write` submission is available only when the request payload contains
an `artwork_writeback` object and Nako grants `artwork_write` for the target
library.

Bulk Metadata Scrape is intentionally not declared as an Addon Task yet. Nako
can validate task declarations and routing plans today, but the generic Addon
Task scheduler/invoker must land before this crate exposes
`bulk-metadata-scrape`.

Version `0.1.0-alpha.1` targets Nako Addon Protocol `0.1.0-alpha.1` and
`nako-addon-protocol` Rust crate `0.1.0-alpha.1`.

Run locally:

```bash
cargo run -p nako-metadata-scraper
```

Default endpoint: `http://127.0.0.1:9100/manifest.json`.
