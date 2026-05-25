# Milestones

## M0 - Workstream Opened

Status: Complete

- MDCx strategy summarized as reference-only input.
- Scope and GPL guardrails recorded.
- Task ledger and validation gates created.

## M1 - AV Query Facts

Status: Complete

- AV number extraction and route classification are available to all providers.
- Metadata scrape responses include optional redaction-safe `query.av`.

## M2 - JavDB Baseline

Status: Complete

- JavDB provider can be enabled through config.
- Synthetic rendered HTML tests cover search, detail parsing, mapping, and artwork.
- FC2 provider can be enabled through config and handles route-specific direct article lookup for FC2 numbers.

## M3 - Bulk Diagnostics

Status: Complete

- Bulk task item output includes optional AV planning summary.
- README documents AV fields and batch behavior.
- Duplicate AV numbers within one bounded batch reuse the first scrape result when no side-effect request is present.
- Empty candidate items report a redaction-safe failure reason.

## M4 - Verified Closeout

Status: In progress

- Package tests, format, JSON validation, and diff checks pass.
- Implementation is ready to commit with conventional commit message.

## Follow-Up - Multi-Provider Maturity

Status: Candidate

- Add additional AV provider groups using the shared AV route facts.
- Add field-level provenance and cross-batch failure accounting beyond this first vertical slice.
