# Official Metadata Bangumi Provider Baseline - Milestones

Status: Active
Last updated: 2026-05-23

## M0 - Workstream Opened

Exit criteria:

- Official Bangumi API facts and User-Agent constraint are recorded.
- Task ledger is split into provider surface, adapter, docs, and closeout.

## M1 - Provider Surface Ready

Exit criteria:

- `ProviderId::Bangumi` exists.
- Bangumi config is read from environment and defaults to disabled.
- Manifest configuration schema lists Bangumi only because runtime supports it.
- Diagnostics report Bangumi disabled/ready/unavailable safely.

## M2 - Adapter Ready

Exit criteria:

- Bangumi provider uses shared HTTP runtime.
- Search and detail request shapes are tested with fake transport.
- Subject mapping produces provider-neutral facts and patch fields.
- No default test performs live network I/O.

## M3 - Operator Truth Updated

Exit criteria:

- README/examples describe actual runtime behavior.
- User-Agent and optional token configuration are visible.
- Douban/crawler/Playwright are explicitly deferred.

## M4 - Closed

Exit criteria:

- Package and workspace tests pass.
- Formatting and diff checks pass.
- Workstream status is complete or split with named follow-ons.
