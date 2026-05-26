# Milestones

Status: Active
Last updated: 2026-05-26

## M1 - Lane Opened

Exit criteria:

- Workstream docs exist and are JSON-valid.
- Task ledger covers provider harness, provider wave 3, and real-use
  protection.
- MDCx reference-only guardrail is explicit.

## M2 - Provider Harness

Exit criteria:

- Rendered AV provider tests can reuse one fixture contract.
- Existing rendered AV providers prove the harness can cover search, direct
  lookup, parser mapping, and AV/artwork facts.
- Drift tooling is documented as manual-only unless it can run without adult
  payloads.

## M3 - Real-Use Protection

Exit criteria:

- Provider execution can apply explicit budget/cache/cooldown policy.
- Protection facts are visible in response/task output where relevant.
- No hidden global scheduler state is introduced.

## M4 - Provider Wave 3

Exit criteria:

- Selected wave 3 providers are disabled by default.
- Registry, config, manifest, aliases, docs, and field-quality descriptors are
  updated.
- Providers have independent synthetic rendered-HTML tests.

## M5 - Verification And Closeout

Exit criteria:

- `cargo nextest run -p nako-metadata-scraper --no-fail-fast` passes.
- `npm --prefix addons/browser-worker test` passes if browser-worker changes.
- Workstream JSON and diff hygiene pass.
- Remaining provider candidates are explicit follow-ups.
