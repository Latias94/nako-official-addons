# Handoff

Status: active. ORSAH-020 is implemented and the lane is ready for provider
registry work.

Current state:

- Resource-search foundation and PanSou-compatible lanes are verified and
  committed.
- Two provider adapters exist: fixture and PanSou-compatible.
- Search domain ownership is split into query, link, and result modules.
- `ResourceSearchIntent` is internal and inferred from request query/ext
  context while preserving the alpha wire shape.
- `links` owns URL classification, normalization, and `ResourceLink`
  construction.

Next steps:

- Start ORSAH-030 by adding provider descriptors, source policy, and registry
  assembly.
- Keep PanSou-compatible disabled by default.
- Do not add live provider scraping, link checking, or downloader hooks during
  registry work.

Watch points:

- Default local smoke must remain no-network.
- PanSou-compatible provider must remain disabled by default.
- Nako core protocol changes are still deferred.
