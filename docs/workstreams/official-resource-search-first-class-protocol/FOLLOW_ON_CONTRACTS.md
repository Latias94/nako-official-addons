# Follow-On Contracts

Status: Draft
Last updated: 2026-05-28

This lane migrates the official resource-search addon to the first-class
read-only `resource_search` contract. It does not implement the adjacent
external-action capabilities.

Host authority is tracked in
`../nako/docs/adr/0050-acquisition-resource-action-boundaries.md`.

## Resource Link Check

Purpose: determine whether a selected host-owned resource reference is usable.

Boundary:

- consumes a host-owned selected-link reference, not a browser-submitted raw URL;
- returns safe facts such as reachable, unavailable, password_needed,
  unsupported, rate_limited, checked_at, and safe_message;
- has its own read-only scope, timeout, retry, cache, and diagnostics policy;
- does not enqueue a downloader or cloud-drive action.

## Downloader Or External Acquisition Runner

Purpose: execute an explicit acquisition action through a configured external
runner such as qBittorrent, Transmission, aria2, ed2k, or HTTP downloaders.

Boundary:

- consumes an intake candidate or selected-link reference after host policy has
  approved it;
- owns idempotency keys, cancellation, audit events, progress, and failure
  states;
- uses separate addon scopes from `acquisition_search_read`;
- is not callable from search responses.

## Cloud-Drive Save Or Transfer

Purpose: perform provider-account operations such as save, transfer, or copy.

Boundary:

- is a write/action contract with provider account secret references;
- must be governed by host acquisition policy;
- must not be implied by link discovery or link checking;
- likely belongs in separate official or third-party addons because trust and
  credentials differ from read-only search.

## Password Or Code References

Purpose: carry selected resource access metadata safely through acquisition.

Boundary:

- search responses may contain raw provider-supplied codes only inside the
  host-owned transient session;
- product API responses expose display-safe facts such as `has_password`;
- durable storage should use host-owned selected-link metadata or secret
  references;
- provider authentication secrets and resource extraction/access codes remain
  different secret classes.

## Official Versus Third-Party Support

The official addon should keep fixture search and generic disabled-by-default
external search adapters. Site-specific search providers, downloader clients,
and cloud-drive write integrations should be third-party or separately packaged
official capabilities unless they share the same trust, license, deployment,
and audit boundary.
