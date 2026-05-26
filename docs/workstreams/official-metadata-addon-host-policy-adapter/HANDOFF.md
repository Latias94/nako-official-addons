# Official Metadata Addon Host Policy Adapter - Handoff

Status: Complete
Last updated: 2026-05-26

## Current State

The official metadata scraper remains a facts-only adapter. It does not accept
or express host merge/application policy in writeback requests.

## Shipped Behavior

- `writeback` payloads with host-policy-looking fields are invalid.
- Metadata writeback targets remain `media_source` only.
- Host application policy remains in Nako.

## Blockers

None.

## Follow-Ons

- Keep future sidecar fields factual. If a field asks the host how to apply
  metadata, it belongs in Nako host policy, not in this repository.
