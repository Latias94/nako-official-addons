# Handoff

Status: active. Workstream opened for the official resource search addon
foundation.

Current state:

- The resource search boundary is documented as a dedicated sidecar.
- PanSou reference lessons are captured without adopting its implementation.
- Nako core protocol changes are deferred and must be handled in a separate
  host-side lane.

Next steps:

- Add `nako-resource-search` crate with the alpha-local typed search contract.
- Add deterministic fixture provider, link taxonomy, and result fusion tests.
- Write the deferred Nako protocol proposal before touching `../nako`.

Watch points:

- Do not add resource search providers to `nako-metadata-scraper`.
- Do not claim catalog or metadata resources are the final protocol shape for
  resource search.
- Do not make live provider scraping or downloader invocation a default CI gate.
