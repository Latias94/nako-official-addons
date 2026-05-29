# Official Addons Cross-Repo Fearless Refactor - TODO

Status: Complete
Last updated: 2026-05-24

Task IDs use the `OACR` prefix.

## M0 - Scope And Evidence Freeze

- [x] OACR-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-addons-cross-repo-fearless-refactor]
  Goal: Record the three discovered architecture problems, selected task
  order, non-goals, cross-repo constraints, and validation gates before code
  changes.
  Validation: DESIGN.md, TODO.md, MILESTONES.md, EVIDENCE_AND_GATES.md,
  WORKSTREAM.json, and HANDOFF.md agree.
  Review: Confirm this lane does not duplicate completed provider-hardening,
  browser-worker, bulk-task, or release-prep workstreams.
  Evidence: this workstream directory.
  Handoff: Continue with OACR-020 unless `../nako` active work blocks the
  protected-write public client alignment.

## M1 - Protected-Write Client Alignment

- [x] OACR-020 [owner=codex] [deps=OACR-010] [scope=../nako/crates/nako-addon-client,../nako/crates/nako-addon-protocol,crates/nako-metadata-scraper/src/nako_runtime.rs]
  Goal: Align official addon protected-write host-client responsibilities with
  public Nako addon client/protocol crates without depending on private server
  crates.
  Validation: focused protocol/client tests plus addon fake-transport tests
  proving bearer token placement, request-body token rejection, redaction-safe
  error mapping, and version-tolerant response parsing.
  Review: Do not move reqwest-heavy sidecar runtime policy into the protocol
  crate unless an ADR explicitly requires it. Preserve sidecar-owned transport
  and timeout behavior.
  Evidence: public client/protocol runtime tests, metadata scraper runtime
  facade tests, Nako server direct-dispatch compile integration, and
  EVIDENCE_AND_GATES.md command transcript.
  Handoff: Public crate release/versioning remains a follow-on release concern;
  compatibility claims were kept within current local path workspace evidence.

## M2 - Provider Adapter Deepening

- [x] OACR-030 [owner=codex] [deps=OACR-010] [scope=crates/nako-metadata-scraper/src/providers/bangumi.rs,crates/nako-metadata-scraper/src/providers/bangumi]
  Goal: Split Bangumi into provider-local client/search/parser/mapper/
  enrichment/test-support modules while preserving runtime behavior and public
  response payloads.
  Validation: `cargo nextest run -p nako-metadata-scraper bangumi ranking title --no-fail-fast`; `cargo fmt --all -- --check`; path-scoped `git diff --check`.
  Review: The split must improve locality. Do not change relevance policy,
  search-variant resilience, degraded candidates, or partial-search diagnostics
  except where tests prove behavior is unchanged.
  Evidence: 2026-05-24 focused nextest passed 49/49; fmt and path-scoped
  diff whitespace gates passed; module map recorded in HANDOFF.md.
  Handoff: Continue with OACR-040 after Bangumi tests pass.

- [x] OACR-040 [owner=codex] [deps=OACR-030] [scope=crates/nako-metadata-scraper/src/providers/douban.rs,crates/nako-metadata-scraper/src/providers/douban]
  Goal: Split Douban into rendered-page client, search parser, detail parser,
  mapper, and test-support modules while preserving behavior.
  Validation: `cargo nextest run -p nako-metadata-scraper douban browser_worker ranking title --no-fail-fast`; `cargo fmt --all -- --check`; path-scoped `git diff --check`.
  Review: Keep browser-worker integration explicit; parsing failures must stay
  redaction-safe and must not introduce live-network default tests.
  Evidence: 2026-05-24 focused nextest passed 30/30; fmt and path-scoped
  diff whitespace gates passed; module map recorded in HANDOFF.md.
  Handoff: Continue with OACR-050 after Douban tests pass.

## M3 - Official Addon Task Path Smoke

- [x] OACR-050 [owner=codex] [deps=OACR-020] [scope=addons/metadata-scraper/smoke.local.ps1,README.md,addons/metadata-scraper/README.md,../nako/scripts/official-addon-e2e-smoke.ps1]
  Goal: Extend the official Nako + metadata addon smoke to prove
  host-dispatched Addon Task path execution for `bulk-metadata-scrape`.
  Validation: local smoke evidence when Nako server/admin token is available,
  plus script parse/static checks and focused Rust tests for task envelope
  compatibility.
  Review: Smoke must prove Nako owns task run creation, retry/result semantics,
  and sidecar task-path dispatch. Do not add Addon process supervision.
  Evidence: smoke script parser checks, addon task/route/manifest tests, Nako
  server direct-dispatch tests, and docs updates pointing at the task smoke
  gate.
  Handoff: Live Docker/server smoke was not run in this session; run it when a
  local Nako server/admin token is available.

## M4 - Closeout

- [x] OACR-060 [owner=planner] [deps=OACR-020,OACR-030,OACR-040,OACR-050] [scope=docs/workstreams/official-addons-cross-repo-fearless-refactor]
  Goal: Verify final evidence, record residual risks, update status, and split
  any remaining new official plugin work into separate lanes.
  Validation: review-workstream and verify-rust-workstream evidence; final
  focused gates pass or documented blockers are external and concrete.
  Review: Close only when code, docs, smoke behavior, and workstream records
  agree.
  Evidence: EVIDENCE_AND_GATES.md, WORKSTREAM.json, HANDOFF.md, and closeout
  journals.
  Handoff: Remaining subtitle/NFO/artwork/new-provider work is explicitly
  deferred to separate future lanes.
