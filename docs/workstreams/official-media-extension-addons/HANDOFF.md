# Official Media Extension Addons - Handoff

Status: complete.

Current task: none.

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
- OMEA-050 synced Nako core official catalog in `../nako` commit `52da469d`:
  `nako-official-addon-catalog` and server catalog resolution now include
  `nako.official.subtitle-provider` and `nako.official.dlna-renderer`; no
  `../nako/web` changes were made.
- OMEA-060 closed the lane with fresh plugin gates: subtitle + DLNA nextest,
  double-package check, fmt check, and diff hygiene all passed.

Next:

- Future work should split DLNA live SSDP/UPnP, subtitle host import/write
  product flow, and External Acquisition Runner into separate lanes.

Watch points:

- Do not add downloader execution or cloud-drive transfer behavior.
- Do not modify `../nako/web`.
- Keep DLNA live SSDP/UPnP and External Acquisition Runner implementation as
  separate future lanes.
