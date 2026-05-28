# Official Media Extension Addons - Handoff

Status: active.

Current task: OMEA-020.

Completed:

- OMEA-010 opened the durable workstream.
- Subtitle Provider is scoped as read-only.
- DLNA Renderer is scoped as plan-only.
- External Acquisition Runner is recorded as a follow-on contract only.

Next:

- Implement `nako-subtitle-provider` foundation.
- Keep fixture provider deterministic.
- Do not write subtitle files or call live subtitle services.

Watch points:

- Do not add downloader execution or cloud-drive transfer behavior.
- Do not modify `../nako/web`.
- Split Nako official catalog sync after manifests stabilize if this lane gets
  too large.
