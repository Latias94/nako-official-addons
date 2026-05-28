# Official Media Extension Addons - Handoff

Status: active.

Current task: OMEA-050.

Completed:

- OMEA-010 opened the durable workstream.
- Subtitle Provider is scoped as read-only.
- DLNA Renderer is scoped as plan-only.
- External Acquisition Runner is recorded as a follow-on contract only.
- OMEA-020 added the read-only fixture-backed `nako-subtitle-provider`
  foundation, checked-in manifest, packaging docs, and local smoke script.
- OMEA-030 added the plan-only `nako-dlna-renderer` foundation, manual target
  discovery, command envelope validation, redaction-safe diagnostics,
  checked-in manifest, packaging docs, and local smoke script.
- OMEA-040 recorded External Acquisition Runner as a future action addon:
  search only passes candidate/link-check facts, runner execution must use
  host-owned selected-link references, and cloud-drive transfer/password/code
  persistence remain out of scope.

Next:

- Decide whether Nako official catalog sync should stay split after manifests
  stabilized.

Watch points:

- Do not add downloader execution or cloud-drive transfer behavior.
- Do not modify `../nako/web`.
- Split Nako official catalog sync after manifests stabilize if this lane gets
  too large.
