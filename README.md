# Nako Official Addons

Official Nako-maintained Addon Sidecars.

Current release target: `v0.1.0-alpha.2`.

Main Nako repository: <https://github.com/Latias94/nako>.
Official addons repository: <https://github.com/Latias94/nako-official-addons>.

This repository intentionally keeps official Addon Sidecars small and
capability-focused while preserving the option to package related capabilities
as one suite later:

- users can install `nako-metadata-scraper` for metadata suggestions,
  protected writeback, bulk metadata scrape, and the existing event proof;
- users can install `nako-notification-bridge` for the first notification
  bridge ACK proof plus sidecar-owned, fixture-tested `http_webhook` provider
  and `discord_webhook` provider sends when exactly one provider is explicitly
  configured;
- the current runtime supports the fixture provider by default and includes
  default-disabled TMDB, Bangumi, and Douban baselines behind the same provider
  seam;
- the same sidecar also declares a small `library.scanned` event subscription
  proof path, so resource, task, and event capabilities can share one official
  deployment unit when they share trust and lifecycle;
- runtime candidate shaping deduplicates exact duplicate provider candidates,
  caps the final result set, and uses shared TMDB/Bangumi/Douban community
  score and vote-count signals as a small generic ranking bonus without
  changing the protocol contract;
- provider image facts are surfaced as typed artwork candidates, and explicit
  metadata/artwork writeback goes through Nako-owned Addon Side Effects;
- future subtitle or local rule providers should be added as internal adapters
  behind the shared configuration, registry, and runtime seams;
- provider modules may later become internal crates, but the install artifact
  should remain one addon unless a provider has a different trust, license, or
  deployment boundary.

## Current Addon

- `crates/nako-metadata-scraper`: Rust HTTP sidecar that implements the Nako
  Addon Protocol metadata resource, bulk task, and library-scanned event proof.
- `crates/nako-notification-bridge`: Rust HTTP sidecar that implements the
  first ACK-only notification bridge proof for scheduled `library.scanned`
  Addon Events and redaction-safe `http_webhook` / `discord_webhook` provider
  sends.

## Development

```bash
cargo fmt --all
cargo nextest run --workspace --no-fail-fast
cargo run -p nako-metadata-scraper
cargo run -p nako-notification-bridge
```

Default listen addresses:

- metadata scraper: `127.0.0.1:9100`
- notification bridge: `127.0.0.1:9110`

Provider defaults:

- `fixture`: enabled by default for local smoke.
- `tmdb`: disabled by default; requires
  `NAKO_METADATA_SCRAPER_TMDB_READ_ACCESS_TOKEN` when enabled.
- `bangumi`: disabled by default; public subject search works without a token,
  but `NAKO_METADATA_SCRAPER_BANGUMI_USER_AGENT` should identify the
  developer/app/version and an optional
  `NAKO_METADATA_SCRAPER_BANGUMI_ACCESS_TOKEN` can be supplied for
  authenticated visibility.
- `browser_worker`: disabled by default; used for rendered-page extraction
  through the companion browser worker service.
- `douban`: disabled by default; uses the companion browser worker `POST /render`
  contract through `NAKO_METADATA_SCRAPER_BROWSER_WORKER_BASE_URL`, with
  provider-specific parsing kept in the Rust sidecar.
- `notification_bridge.http_webhook`: disabled by default; configured through
  `NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_*` sidecar environment variables and
  reported only through redaction-safe diagnostics. When enabled with a valid
  URL, it sends a fixed JSON summary containing event facts and payload keys
  only.
- `notification_bridge.discord_webhook`: disabled by default; configured
  through `NAKO_NOTIFICATION_BRIDGE_DISCORD_WEBHOOK_*` sidecar environment
  variables and reported only through redaction-safe diagnostics. When enabled
  with a valid URL, it sends a fixed Discord-compatible payload containing event
  facts and payload keys only. The sidecar rejects multiple simultaneously
  configured provider send paths.
- `notification_bridge.template`: default-safe summary rendering through
  `NAKO_NOTIFICATION_BRIDGE_TEMPLATE_SUMMARY`, with whitelisted event fact
  tokens only and no raw event payload value access.
- `notification_bridge.configuration_status`: redaction-safe health and
  diagnostics status for ACK-only mode, ready provider sends, invalid provider
  configuration, multiple configured send paths, and invalid enabled-provider
  templates.
- `notification_bridge.provider_attempt_history`: bounded in-memory recent
  provider send outcome and failure history for redaction-safe sidecar
  diagnostics, controlled by
  `NAKO_NOTIFICATION_BRIDGE_PROVIDER_ATTEMPT_HISTORY_CAPACITY`. ACK-only events
  and disabled providers do not create history records.

Bulk Metadata Scrape is tracked in
`docs/workstreams/official-metadata-addon-bulk-task-design/` and is now
declared as the `bulk-metadata-scrape` Addon Task at
`/tasks/bulk-metadata-scrape`. The task processes bounded batches of the same
metadata payload shape used by `POST /metadata`.

Local sidecar smoke:

```powershell
pwsh -File addons/metadata-scraper/smoke.local.ps1 `
  -SidecarBaseUrl http://127.0.0.1:9100

pwsh -File addons/notification-bridge/smoke.local.ps1 `
  -SidecarBaseUrl http://127.0.0.1:9110
```

Optional notification provider live smoke is skipped by default and must be
enabled explicitly with `NAKO_NOTIFICATION_BRIDGE_LIVE_SMOKE=1` before running
`addons/notification-bridge/smoke.live.ps1` against a locally configured
sidecar. A configured notification bridge can also be checked locally with
`POST /providers/test-send`; the endpoint sends a synthetic safe provider
notification and returns only redaction-safe status.

Optional Nako Admin-mediated smoke:

```powershell
$env:NAKO_ADMIN_TOKEN = '<admin bearer token>'
pwsh -File addons/metadata-scraper/smoke.local.ps1 `
  -SidecarBaseUrl http://127.0.0.1:9100 `
  -NakoBaseUrl http://127.0.0.1:3000 `
  -RegisterInNako `
  -Enable `
  -RunResourceCall `
  -RunTaskPath
```

`-RunTaskPath` creates a Nako-owned direct Addon Task run for
`bulk-metadata-scrape`, waits for it to succeed, and verifies the result came
from the sidecar's `/tasks/bulk-metadata-scrape` path.

When a smoke option asks for a Nako-owned path, such as `-RunTaskPath`,
`-RunResourceCall`, `-Enable`, or `-IssueAddonToken`, the script requires
`-RegisterInNako` so release gates cannot silently fall back to sidecar-only
checks. For writeback smoke, pass `-ExpectedWritebackStatus` and optionally
`-ExpectedWritebackSafeErrorCode` to assert the exact side-effect gate result.

## Relationship to Nako Core

This alpha targets Nako Addon Protocol `0.1.0-alpha.1` and the matching
`nako-addon-protocol` Rust crate version `0.1.0-alpha.2`.

The main Nako repository is <https://github.com/Latias94/nako>. This repository
contains the official addon sidecars that integrate with that core project.

The Rust implementation depends on the published `nako-addon-protocol`
`0.1.0-alpha.2` crate and imports it in code as `nako_addon_protocol`.

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
