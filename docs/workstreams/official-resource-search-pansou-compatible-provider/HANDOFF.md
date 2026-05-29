# Handoff

Status: complete. Disabled-by-default PanSou-compatible HTTP provider adapter
landed for `nako-resource-search`.

Current state:

- Foundation sidecar exists and is committed.
- Config and manifest expose a disabled-by-default `pansou_compatible`
  provider.
- Runtime registers it only when explicitly enabled and a valid HTTP(S) base URL
  is configured.
- Adapter maps PanSou `results` and `merged_by_type` response shapes into
  internal resource search results.
- Tests cover request shaping, response mapping, provider registration, and
  token redaction.

Next steps:

- Keep live PanSou service checks optional.
- Consider adding a small mock HTTP server test later if the workspace adds a
  shared HTTP test dependency.
- Do not start link checking or downloader hooks until host/operator policy is
  explicit.

Watch points:

- Do not copy PanSou source code.
- Do not enable network providers by default.
- Do not add downloader execution.
