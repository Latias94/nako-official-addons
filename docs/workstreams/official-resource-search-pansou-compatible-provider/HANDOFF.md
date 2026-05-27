# Handoff

Status: active. Workstream opened for a disabled-by-default
PanSou-compatible HTTP provider.

Current state:

- Foundation sidecar exists and is committed.
- This lane should only add an optional provider adapter.

Next steps:

- Add config and manifest provider toggle.
- Add request/response mapping tests.
- Keep live PanSou service checks optional.

Watch points:

- Do not copy PanSou source code.
- Do not enable network providers by default.
- Do not add downloader execution.
