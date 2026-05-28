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

Expected future host-to-runner envelope:

- `selected_link_ref`: opaque Nako-owned reference minted after search/link
  check, not a raw provider URL with credentials;
- `runner_profile_id`: user-selected external runner profile such as
  qBittorrent, Transmission, aria2, or an HTTP downloader;
- `idempotency_key`: host-owned key for retry-safe action submission;
- `requested_operation`: explicit action such as enqueue, cancel, pause,
  resume, or query status;
- `callback_ref` or event stream binding for progress and terminal states;
- safe metadata facts only, with all provider tokens, passwords, and local file
  locations redacted or omitted.

Relationship to Resource Search:

- `resource_search` may discover candidates, classify links, and return
  conservative link-check facts.
- `resource_search` must not enqueue downloads, call downloader APIs, transfer
  cloud-drive files, or persist provider access codes.
- Any UI flow that turns a candidate into an action must route through Nako host
  policy first, then call a dedicated action addon with explicit user consent.

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

Completed in `../nako` commit `52da469d`: core official addon catalog facts and
server catalog resolve coverage now include `nako.official.subtitle-provider`
and `nako.official.dlna-renderer`. The sync did not touch `../nako/web`.
