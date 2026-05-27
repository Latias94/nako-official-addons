# Handoff

Status: complete for the first resource search addon foundation slice.

Current state:

- The resource search boundary is documented as a dedicated sidecar.
- PanSou reference lessons are captured without adopting its implementation.
- `nako-resource-search` is a workspace crate with manifest, health, search,
  diagnostics, fixture provider, link taxonomy, and fusion tests.
- Packaging files, checked-in example manifest, and local smoke script exist
  under `addons/resource-search`.
- Nako core protocol changes are deferred and captured in
  `PROTOCOL_PROPOSAL.md`.

Next steps:

- Open a separate `../nako` host-side lane for the `resource_search` resource
  only after this plugin-side contract is accepted.
- Add optional live/search provider adapters behind disabled-by-default
  configuration.
- Add link checking and downloader hooks only after host/operator policy exists.

Watch points:

- Do not add resource search providers to `nako-metadata-scraper`.
- Do not claim catalog or metadata resources are the final protocol shape for
  resource search.
- Do not make live provider scraping or downloader invocation a default CI gate.
