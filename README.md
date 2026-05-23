# Nako Official Addons

Official Nako-maintained Addon Sidecars.

Current release target: `v0.1.0-alpha.1`.

Main Nako repository: <https://github.com/Latias94/nako>.
Official addons repository: <https://github.com/Latias94/nako-official-addons>.

This repository intentionally exposes one user-installable metadata addon while
keeping provider implementations modular inside the codebase:

- users install `nako-metadata-scraper` once;
- the current runtime supports the fixture provider by default and includes
  default-disabled TMDB and Bangumi baselines behind the same provider seam;
- runtime candidate shaping deduplicates exact duplicate provider candidates,
  caps the final result set, and uses shared TMDB/Bangumi community score and
  vote-count signals as a small generic ranking bonus without changing the
  protocol contract;
- future Douban, artwork, subtitle, or local rule providers should be added as
  internal adapters behind the shared configuration, registry, and runtime
  seams;
- provider modules may later become internal crates, but the install artifact
  should remain one addon unless a provider has a different trust, license, or
  deployment boundary.

## Current Addon

- `crates/nako-metadata-scraper`: Rust HTTP sidecar that implements the Nako
  Addon Protocol metadata resource.

## Development

```bash
cargo fmt --all
cargo nextest run --workspace --no-fail-fast
cargo run -p nako-metadata-scraper
```

Default listen address: `127.0.0.1:9100`.

Provider defaults:

- `fixture`: enabled by default for local smoke.
- `tmdb`: disabled by default; requires
  `NAKO_METADATA_SCRAPER_TMDB_READ_ACCESS_TOKEN` when enabled.
- `bangumi`: disabled by default; public subject search works without a token,
  but `NAKO_METADATA_SCRAPER_BANGUMI_USER_AGENT` should identify the
  developer/app/version and an optional
  `NAKO_METADATA_SCRAPER_BANGUMI_ACCESS_TOKEN` can be supplied for
  authenticated visibility.

Local sidecar smoke:

```powershell
pwsh -File addons/metadata-scraper/smoke.local.ps1 `
  -SidecarBaseUrl http://127.0.0.1:9100
```

Optional Nako Admin-mediated smoke:

```powershell
$env:NAKO_ADMIN_TOKEN = '<admin bearer token>'
pwsh -File addons/metadata-scraper/smoke.local.ps1 `
  -SidecarBaseUrl http://127.0.0.1:9100 `
  -NakoBaseUrl http://127.0.0.1:3000 `
  -RegisterInNako `
  -Enable `
  -RunResourceCall
```

## Relationship to Nako Core

This alpha targets Nako Addon Protocol `0.1.0-alpha.1` and the matching
`nako-addon-protocol` Rust crate version `0.1.0-alpha.1`.

The main Nako repository is <https://github.com/Latias94/nako>. This repository
contains the official addon sidecars that integrate with that core project.

The Rust implementation depends on the published `nako-addon-protocol`
`0.1.0-alpha.1` crate and imports it in code as `nako_addon_protocol`.

Versioning has three separate layers:

- Addon `version`: this sidecar's own release version.
- Addon `protocol_version`: the runtime HTTP wire compatibility version that
  Nako uses for registration, health checks, and resource calls.
- Rust crate version: the Cargo package version for Rust SDK/dependency users.

## Licensing

This addon workspace is licensed as `AGPL-3.0-or-later`.

The Nako Addon Protocol crate is licensed separately as `Apache-2.0 OR MIT`;
this repository consumes that crate as a dependency and does not relicense it.

## Reference Code Policy

Reference repositories may be checked out under `../repo-ref/nako-scraper/` for
behavior and architecture research only. Do not copy, translate line by line,
port, import, or derive implementation code, schemas, tests, fixtures, artwork,
or generated files from MediaElch, Kodi scrapers, Jellyfin plugins, or similar
projects.

Allowed use:

- compare product capabilities and user workflows;
- study high-level provider responsibilities;
- record original Nako design notes in this repository;
- write fresh Rust implementations against Nako's own Addon Protocol and tests.

If a reference project's license or terms are unclear, treat it as inspiration
only and do not use its source text as implementation material.
