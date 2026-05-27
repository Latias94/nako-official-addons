# Official Browser Worker Live AV Provider Tuning - TODO

Status: Completed
Last updated: 2026-05-27

## Task Ledger

| ID | Status | Owner | Scope | Validation |
| --- | --- | --- | --- | --- |
| OBWLAPT-010 | Completed | Codex | Open workstream and connect it to the live sampling evidence. | Workstream docs exist and JSON validates. |
| OBWLAPT-020 | Completed | Codex | Add selector wait budgets, safe header env refs, production render budgets, and realistic DMM/official uncensored defaults. | `cargo nextest run -p nako-metadata-scraper rendered_page render_drift dmm_provider official_1pondo fc2_provider javbus_provider addon_manifest_configuration_schema_reflects_configured_provider_defaults config --no-fail-fast` |
| OBWLAPT-030 | Completed | Codex | Re-run live AV drift with local proxy and classify improvements vs operator-bound failures. | `npm --prefix addons/browser-worker run live:render-drift` with generated cases. |
| OBWLAPT-040 | Completed | Codex | Document access/proxy/cookie guidance for failures that code should not bypass. | Evidence, README, and handoff updated. |
| OBWLAPT-050 | Completed | Codex | Final hygiene and commit. | `cargo fmt`, targeted tests, `git diff --check`. |

## Notes

- Live output must stay redaction-safe.
- Provider cookies may be configured by operators, but drift cases must not
  emit cookie values; use `headers_from_env`.
- Selector/time budgets should be provider-owned so future parser changes do
  not require Browser Worker special cases.
- Selector waits should use DOM attachment semantics for scraper health. UI
  visibility is too strict for pages whose parseable links are hidden by layout.
