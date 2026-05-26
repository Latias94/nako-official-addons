# Milestones

Status: Complete
Last updated: 2026-05-26

## M1 - Workstream Open

Exit criteria:

- Workstream docs exist.
- GPL/reference boundary is explicit.
- `WORKSTREAM.json` is valid JSON.

Result: Done.

## M2 - Provider Slice

Exit criteria:

- `theporndb` provider builds only with token configuration.
- Search and direct slug flows are covered by synthetic JSON tests.
- Candidate mapping includes core metadata, AV facts, artwork, and external IDs.

Result: Done. The provider uses the official JSON API with bearer-token auth,
redaction-safe proxy config, AV-number/title scene search, and explicit scene
detail lookup through `theporndb_id` or `theporndb_url`.

## M3 - Integration Slice

Exit criteria:

- Provider catalog/config/manifest/diagnostics/presets know about `theporndb`.
- Secret reference field is exposed only when provider is enabled.
- Proxy/token configured checks are redaction-safe.

Result: Done. The provider is disabled by default, participates in `fast_safe`
and `community_first`, appears in diagnostics/manifest schema/live drift lists,
and is unavailable instead of built when enabled without a token.

## M4 - Closeout

Exit criteria:

- Targeted and package gates pass.
- README and addon docs mention token, proxy, and hash follow-up.
- Workstream evidence and handoff are current.

Result: Done. Hash and movie route support remain explicit follow-ups.
