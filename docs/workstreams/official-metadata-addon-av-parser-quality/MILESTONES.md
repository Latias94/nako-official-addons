# Milestones

Status: Complete
Last updated: 2026-05-26

## M1 - Workstream Opened

Exit criteria:

- Workstream docs exist and are JSON-valid.
- Task ledger separates parser-quality work from provider breadth work.
- Reference-only guardrails are explicit.

## M2 - Shared Structured Labels

Exit criteria:

- `rendered_av` owns a small row-level label parser.
- Official uncensored and FC2PPVDB use the shared parser.
- Tests prove adjacent field boundaries are respected.

## M3 - Provider Parser Migration

Exit criteria:

- DMM/MGStage/JavBus/JavLibrary/JavDB/FC2 are either migrated with focused tests
  or explicitly left provider-local with a reason.

## M4 - Verification And Closeout

Status: Complete on 2026-05-26.

Exit criteria:

- Relevant targeted and package gates pass.
- Workstream JSON and diff hygiene pass.
- Remaining parser or provider breadth work is explicit follow-up scope.

Closeout result:

- `cargo nextest run -p nako-metadata-scraper --no-fail-fast` passed with 222
  tests.
- `cargo fmt -p nako-metadata-scraper -- --check`, workstream JSON validation,
  and `git diff --check` passed.
- No hidden parser migrations remain for current AV providers.
