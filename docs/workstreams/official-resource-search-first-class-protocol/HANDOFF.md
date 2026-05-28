# Handoff

Status: closed.

Completed:

- Migrate the official resource-search sidecar from temporary `automation`
  alpha semantics to the first-class Nako `resource_search` contract.
- Keep Admin UI out of scope.
- Keep search read-only; downstream link checking, downloader execution,
  cloud-drive transfer, and password/code reference handling remain separate
  contracts.

Follow-ons:

- Admin UI remains intentionally untouched.
- Link-check, downloader/external runner, cloud-drive transfer, and
  password/code references require separate workstreams or issues.
- Nako ADR 0050 records the host-side authority split.

Watch points:

- Do not reintroduce `automation_run` for search.
- Do not leak raw provider errors, URLs, tokens, passwords, or opaque context in
  diagnostics.
- Keep PanSou-compatible provider disabled by default.
- Keep runtime manifest equal to checked-in `manifest.example.json`.
