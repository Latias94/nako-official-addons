# Official Resource Link Check Provider

Status: Closed
Last updated: 2026-05-28

## Problem

Nako host now has a first-class `resource_link_check` product route, but the
official resource-search sidecar still declares only `resource_search`. Operators
can search and select links, but there is no official addon-side implementation
for safe link classification and lightweight reachability checks.

## Target State

- `nako-resource-search` declares a first-class `resource_link_check` resource.
- The route accepts Nako protocol envelopes and returns
  `nako.addon.resource_link_check.response.v1`.
- Internal checker provider boundaries are separate from search providers.
- The first slice supports deterministic fixture checks plus generic safe link
  classification for unsupported/live-risky link types.
- The response is safe: no raw URL, password/code, note, token, or provider raw
  body is returned.

## Delivered Shape

- Manifest declares both `resource_search` and `resource_link_check`.
- Router exposes `/resource-link-check`.
- Runtime calls a separate `ResourceLinkCheckProvider`.
- The initial provider is conservative: fixture cloud links are reachable,
  ordinary cloud/web links are unknown without live network, and peer-to-peer
  links are unsupported.
- Health/diagnostics expose only safe checker provider facts.

## In Scope

- `crates/nako-resource-search` manifest, routes, domain, runtime, and tests.
- `addons/resource-search/manifest.example.json` and local smoke coverage.
- Documentation updates for the official resource-search addon.

## Out Of Scope

- Nako Admin UI.
- Downloader execution.
- Cloud-drive transfer or save-to-drive automation.
- Password/code persistence.
- Site-specific authenticated cloud APIs.
- Real BitTorrent/DHT probing.

## Architecture Direction

Add a checker module parallel to search:

```text
routes
  -> resource_protocol decode/encode
  -> ResourceSearchRuntime::check_link
  -> link_check provider boundary
```

The first provider should be deterministic and conservative:

- invalid or empty links: `unsupported`;
- magnet/ed2k links: `unsupported` with safe facts only;
- known cloud-drive/web links: `unknown` or `password_needed` when password is
  required by the link metadata, without performing transfer or login;
- fixture links can return `reachable` so host/product tests and local smoke
  can prove the contract end to end.

Later site-specific providers can add live checks behind explicit config.

## Validation

```bash
cargo nextest run -p nako-resource-search resource_link_check --no-fail-fast
cargo nextest run -p nako-resource-search manifest --no-fail-fast
cargo fmt --all -- --check
cargo check -p nako-resource-search --tests
git diff --check
```
