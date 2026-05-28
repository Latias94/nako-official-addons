# Official Media Extension Addons - Handoff

Status: active.

Current task: OMEA-030.

Completed:

- OMEA-010 opened the durable workstream.
- Subtitle Provider is scoped as read-only.
- DLNA Renderer is scoped as plan-only.
- External Acquisition Runner is recorded as a follow-on contract only.
- OMEA-020 added the read-only fixture-backed `nako-subtitle-provider`
  foundation, checked-in manifest, packaging docs, and local smoke script.

Next:

- Implement `nako-dlna-renderer` foundation.
- Keep it plan-only with manual targets only.
- Do not add SSDP discovery or UPnP control.

Watch points:

- Do not add downloader execution or cloud-drive transfer behavior.
- Do not modify `../nako/web`.
- Split Nako official catalog sync after manifests stabilize if this lane gets
  too large.
