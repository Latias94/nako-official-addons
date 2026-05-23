# Nako Official Addons

Official Nako-maintained Addon Sidecars.

This repository intentionally exposes one user-installable metadata addon while
keeping provider implementations modular inside the codebase:

- users install `nako-metadata-scraper` once;
- the current runtime supports the fixture provider by default and includes a
  default-disabled TMDB movie baseline behind the same provider seam;
- future Bangumi, Douban, artwork, subtitle, or local rule providers should be
  added as internal adapters behind the shared configuration, registry, and
  runtime seams;
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

Until the protocol crate is published, this repository depends on the local core
checkout at `../nako/crates/nako-addon-protocol` and imports it in code as
`nako_addon_protocol`.

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
