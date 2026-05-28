# Handoff

Status: active. ORSAH-060 is implemented and the lane is ready for verification
and closeout.

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
- Fusion/ranking/deduplication lives in `engine::fusion`.
- Providers return `ProviderSearchBatch` so warnings and partial finality can
  be represented without becoming runtime errors.
- Provider descriptors now own provider-specific manifest configuration schema
  fragments.
- Hardened protocol proposal records the deferred Nako host work.

Next steps:

- Start ORSAH-070 by running focused and workspace validation.
- Keep PanSou-compatible disabled by default.
- Do not add live provider scraping, link checking, or downloader hooks during
  closeout.

Watch points:

- Default local smoke must remain no-network.
- PanSou-compatible provider must remain disabled by default.
- Registry diagnostics must stay redaction-safe.
- Provider execution errors must not expose raw provider exception text.
- Runtime and checked-in container manifests must remain equal.
- Nako core protocol changes are still deferred.
