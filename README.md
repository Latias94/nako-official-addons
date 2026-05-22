# Nako Official Addons

Official Nako-maintained Addon Sidecars.

This repository intentionally exposes one user-installable metadata addon while
keeping provider implementations modular inside the codebase:

- users install `nako-metadata-scraper` once;
- the addon can enable multiple providers such as TMDB, Bangumi, Douban, or
  local rule providers through configuration;
- provider modules may later become internal crates, but the install artifact
  should remain one addon unless a provider has a different trust, license, or
  deployment boundary.

## Current Addon

- `crates/nako-metadata-scraper`: Rust HTTP sidecar that implements the Nako
  Addon Protocol metadata resource.

## Development

```bash
cargo fmt --all
cargo test --workspace
cargo run -p nako-metadata-scraper
```

Default listen address: `127.0.0.1:9100`.

## Relationship to Taru/Nako Core

The core project is being rebranded from Taru to Nako. Until the protocol crate
is renamed and published, this repository depends on the local core checkout at
`../taru/crates/taru-addon-protocol` and aliases it in code as
`nako_addon_protocol`.
