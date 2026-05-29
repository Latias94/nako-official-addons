# Milestones

Status: Closed
Last updated: 2026-05-26

## M1 - Lane Opened

Exit criteria:

- Workstream docs exist and are JSON-valid.
- The broken-protocol decision is explicit.
- Cross-repo ownership and dirty-worktree guardrails are documented.

## M2 - Native Nako Metadata Writeback

Exit criteria:

- Addon metadata write payload mirrors canonical graph fields.
- Server metadata write adapter validates and maps graph fields.
- Metadata write apply runs full catalog projection.
- Focused protocol/client/server tests pass.

## M3 - Addon AV Materialization

Exit criteria:

- Selected AV facts populate canonical writeback fields.
- Response evidence still exposes provider fact sources.
- Existing AV providers remain fixture-tested after the protocol break.

## M4 - Bulk Mature Scrape Accounting (Complete)

Exit criteria:

- Bulk output classifies retryable, permanent, empty, and suppressed outcomes.
- Provider suppression/cooldown state is resume-safe.
- Tests prove duplicate reuse and provider summaries still work.

## M5 - Provider Wave 2 (Complete)

Exit criteria:

- JavLibrary plus one route-specific provider are disabled by default.
- Config, registry, manifest, docs, and synthetic fixtures cover the providers.
- Provider parsing remains independently implemented.

## M6 - Verification And Closeout (Complete)

Exit criteria:

- Nako and official-addon focused gates pass.
- Package-level official-addon gates pass or failures are documented.
- Commits are split cleanly by repo when necessary.
- Remaining AV parity work is explicit follow-up scope.
