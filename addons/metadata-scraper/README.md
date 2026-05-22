# Nako Metadata Scraper Addon

Official metadata scraper Addon Sidecar for Nako.

The first version is intentionally small: it validates the Addon Protocol shape,
returns fixture metadata suggestions, and establishes provider-module seams for
future TMDB, Bangumi, Douban, NFO, artwork, scoring, and rename-planning work.

## Run locally

```bash
cargo run -p nako-metadata-scraper
```

Endpoints:

- `GET /manifest.json`
- `POST /health`
- `POST /metadata`
- `GET /ui/diagnostics`

## Register in Nako Admin Web

1. Start this sidecar or generate the manifest from `/manifest.json`.
2. Paste the manifest JSON into Nako Admin Web Addon Onboarding.
3. Register as disabled.
4. Follow the generated Install Guide.
5. Run Addon Health Check.
6. Configure future token/grant flow and enable the Addon.

## Docker example while the protocol crate is local

The current workspace depends on the local core checkout:

```text
../taru/crates/taru-addon-protocol
```

Therefore `compose.example.yml` expects this directory layout:

```text
<parent>/
  taru/
  nako-official-addons/
```

After the protocol crate is renamed/published, the Docker context can shrink
back to this repository only.

## Current provider strategy

Users install one Addon: `nako-metadata-scraper`.

Providers are code modules inside the Addon, not separate user-visible Addons.
This keeps installation simple while preserving internal modularity.
