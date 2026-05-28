# Handoff

Status: active. ORSAH-030 is implemented and the lane is ready for fusion
extraction.

Current state:

- Resource-search foundation and PanSou-compatible lanes are verified and
  committed.
- Two provider adapters exist: fixture and PanSou-compatible.
- Search domain ownership is split into query, link, and result modules.
- `ResourceSearchIntent` is internal and inferred from request query/ext
  context while preserving the alpha wire shape.
- `links` owns URL classification, normalization, and `ResourceLink`
  construction.
- Provider descriptors and source policy are in place.
- Provider registry assembly owns fixture/PanSou activation and exposes
  diagnostics.

Next steps:

- Start ORSAH-040 by moving result fusion out of `engine.rs`.
- Keep PanSou-compatible disabled by default.
- Do not add live provider scraping, link checking, or downloader hooks during
  fusion work.

Watch points:

- Default local smoke must remain no-network.
- PanSou-compatible provider must remain disabled by default.
- Registry diagnostics must stay redaction-safe.
- Nako core protocol changes are still deferred.
