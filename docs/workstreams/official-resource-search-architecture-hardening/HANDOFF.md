# Handoff

Status: active. Workstream opened for resource search architecture hardening.

Current state:

- Current `nako-resource-search` implementation is verified and committed.
- Two provider adapters exist: fixture and PanSou-compatible.
- The next refactor should deepen the search domain before adding more
  providers.

Next steps:

- Start ORSAH-020 by splitting query/result/link ownership and adding
  `ResourceSearchIntent`.
- Keep the alpha wire shape stable.
- Do not add live provider scraping, link checking, or downloader hooks during
  the domain split.

Watch points:

- Default local smoke must remain no-network.
- PanSou-compatible provider must remain disabled by default.
- Nako core protocol changes are still deferred.
