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

Runtime candidate shaping deduplicates exact duplicate provider candidates,
caps the final result set, and uses shared community score/vote-count facts
from TMDB and Bangumi as a small generic ranking bonus.

Version `0.1.0-alpha.1` targets Nako Addon Protocol `0.1.0-alpha.1` and
`nako-addon-protocol` Rust crate `0.1.0-alpha.1`.

Run locally:

```bash
cargo run -p nako-metadata-scraper
```

Default endpoint: `http://127.0.0.1:9100/manifest.json`.
