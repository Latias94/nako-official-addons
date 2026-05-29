# Official Media Extension Addons - Design

Status: Complete
Last updated: 2026-05-28

## Problem

The official addon set now covers metadata, notifications, Chromecast rendering,
and read-only acquisition resource search. The next useful media surfaces are
subtitles and additional renderer protocols, but they need to be added without
turning the official addon suite into a broad downloader or cloud-drive action
package.

## Target State

- Add a read-only official `nako-subtitle-provider` addon foundation.
- Add a plan-only official `nako-dlna-renderer` renderer adapter foundation.
- Keep both addons small, fixture-backed, and safe by default.
- Record External Acquisition Runner as a follow-on contract only.
- Preserve the existing boundary: resource search discovers and checks links;
  it does not download, transfer cloud resources, or persist access codes.

## Scope

- `crates/nako-subtitle-provider`
- `addons/subtitle-provider`
- `crates/nako-dlna-renderer`
- `addons/dlna-renderer`
- repository workspace metadata and operator docs
- this workstream's evidence and follow-on contract docs

## Non-Goals

- No real subtitle provider scraping or credentialed provider calls.
- No automatic subtitle file write/import into Nako media sources.
- No DLNA SSDP discovery, UPnP control, SOAP actions, or live device control.
- No External Acquisition Runner implementation.
- No qBittorrent, Transmission, aria2, ed2k, HTTP downloader, or cloud-drive
  transfer behavior.
- No durable password/code reference storage.

## Architecture Direction

Subtitle Provider should start as a read-only `AddonResource::Subtitle`
sidecar. Its first implementation may be a deterministic fixture provider with
typed request/response payloads local to the addon until Nako host grows a
first-class subtitle product flow.

DLNA Renderer should reuse the existing `renderer_adapter` protocol. The first
slice should expose readiness, manual target discovery, and safe command
planning only. Live discovery/control must remain opt-in follow-on work because
it touches the local network and protocol-specific failure modes.

External Acquisition Runner is deliberately separate. It requires explicit host
policy, idempotency, audit events, cancellation, progress, failure states, and
dedicated scopes before any official addon executes external downloads.

## Risks

| Risk | Impact | Mitigation |
| --- | --- | --- |
| Subtitle provider accidentally looks like import/write support. | Medium | Manifest uses read-only `subtitle_read`; docs state no writes. |
| DLNA plan-only behavior is mistaken for live control. | Medium | Diagnostics and safe reason codes must say plan-only. |
| Acquisition runner pressure leaks into resource search. | High | Keep runner in follow-on docs only; do not add scopes or routes. |
| Nako official catalog drifts after new addons land. | Medium | Split catalog sync into a bounded follow-on after addon manifests are stable. |

## Validation Strategy

- `cargo nextest run -p nako-subtitle-provider --no-fail-fast`
- `cargo nextest run -p nako-dlna-renderer --no-fail-fast`
- `cargo fmt --all -- --check`
- `cargo check -p nako-subtitle-provider -p nako-dlna-renderer --tests`
- `git diff --check`
