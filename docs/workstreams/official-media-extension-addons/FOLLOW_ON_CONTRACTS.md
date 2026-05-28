# Official Media Extension Addons - Follow-On Contracts

Status: Active
Last updated: 2026-05-28

## External Acquisition Runner

Purpose: execute explicit acquisition actions through a configured external
runner such as qBittorrent, Transmission, aria2, ed2k handlers, or HTTP
downloaders.

Boundary:

- consumes a host-owned selected-link or intake candidate reference;
- does not accept browser-submitted raw URLs or passwords;
- uses dedicated acquisition action scopes, not `acquisition_search_read`;
- owns idempotency keys, cancellation, progress, terminal states, and audit
  events;
- reports redaction-safe failure reasons;
- must not be callable from raw search results.

Non-goals for the current workstream:

- no downloader execution;
- no cloud-drive save, transfer, or copy;
- no durable password/code reference storage;
- no provider account credential handling.

## Cloud-Drive Transfer

Cloud-drive transfer is a separate write/action capability with provider
account secrets and platform-specific risk. It should be third-party or a
separately packaged official addon only after Nako host policy defines account
secrets, consent, idempotency, audit, and rollback behavior.

## Catalog Sync

Once Subtitle Provider and DLNA Renderer manifests stabilize, sync the official
addon catalog in `../nako` as a separate bounded task or follow-on. That task
must not touch `../nako/web`.
