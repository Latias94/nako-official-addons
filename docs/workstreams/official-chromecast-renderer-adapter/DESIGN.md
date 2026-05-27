# Official Chromecast Renderer Adapter

## Problem

Nako now has a host-side renderer adapter boundary for external playback
targets. The first real protocol adapter should prove that the official addon
repository can host protocol-specific code without pulling Chromecast
discovery, command mapping, LAN permissions, or hardware smoke requirements
into Nako core.

The sidecar must stay a protocol adapter. Nako remains responsible for playback
policy, renderer target selection, authorization, cast-safe transport ticket
generation, and the command envelope contract.

## Target State

- `nako-chromecast-renderer` is an official HTTP Addon Sidecar.
- The sidecar declares one `renderer_adapter` resource at `/renderer-adapter`.
- The sidecar exposes manifest, health, renderer adapter, and diagnostics
  endpoints.
- Readiness is redaction-safe and does not expose LAN device addresses.
- Discovery supports deterministic manually configured Chromecast targets by
  default, and optional live LAN discovery behind an explicit environment flag.
- Dispatch validates Nako-owned command envelopes and maps them to a
  Chromecast command plan.
- Live Chromecast control is optional and must remain a manual/live smoke gate
  until test fixtures or stable virtual devices exist.

## Scope

- Add the new Rust sidecar crate.
- Add package manifest, Dockerfile, compose example, local smoke script, and
  README for the Chromecast renderer sidecar.
- Use `nako-addon-protocol` renderer adapter DTOs and
  `nako-official-addon-catalog::chromecast_renderer` constants.
- Link the `oxicast` crate as the selected Cast protocol dependency and keep
  hardware execution behind explicit configuration.

## Non-goals

- DLNA, AirPlay, Miracast, or DIAL support.
- Host-side renderer scheduling or media policy changes.
- Admin Web UX changes.
- Persisted device inventory.
- Chromecast hardware CI.
- Copying implementation details from Jellyfin, Plex, or other reference
  projects.

## Architecture Direction

The adapter boundary follows the same split as the host ECAB workstream:

- Nako host owns the renderer command envelope.
- The official sidecar owns protocol-specific readiness, target discovery, and
  command translation.
- Protocol dependencies live in `nako-official-addons`, not in Nako core.
- All diagnostics and errors are safe by construction: device IDs and counts may
  be reported, but raw cast URLs, bearer tokens, ticket data, and LAN addresses
  must not be echoed.

The first implementation is intentionally a small deep module:

- `config`: redaction-safe environment parsing and manual device facts.
- `manifest`: manifest generation from the Nako official catalog constants.
- `chromecast`: readiness, target mapping, transport validation, and
  command-plan construction.
- `routes`: HTTP Addon Protocol boundary.

## Decisions

- Use Chromecast as the first protocol because it exercises local network
  discovery and external renderer control without changing host media policy.
- Use `oxicast` as the selected Rust Cast dependency for this lane. The first
  slice links it for live discovery/control boundaries but keeps hardware
  execution out of default tests.
- Keep default runtime in a safe plan-only mode. Operators may opt into live LAN
  discovery/control with explicit environment variables and local smoke scripts.
