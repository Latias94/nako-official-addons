# Official Browser Worker Render Runtime Hardening - TODO

Status: Complete
Last updated: 2026-05-27

## M0 - Planning

- [x] OBWR-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-browser-worker-render-runtime-hardening]
  Goal: Open the render runtime hardening lane and freeze the target contract.
  Validation: `python -m json.tool docs/workstreams/official-browser-worker-render-runtime-hardening/WORKSTREAM.json`
  Review: Keep the worker as an internal execution boundary, not a metadata parser.
  Evidence: `DESIGN.md`

## M1 - Render Intent Contract

- [x] OBWR-020 [owner=codex] [deps=OBWR-010] [scope=addons/browser-worker/src/render-contract.mjs,addons/browser-worker/src/extract.mjs,addons/browser-worker/src/app.mjs,addons/browser-worker/test]
  Goal: Move render request parsing and option normalization into a dedicated contract module while preserving `/render` and `/extract` request compatibility.
  Validation: `npm test`
  Review: Invalid inputs must return redaction-safe codes without starting browser work.
  Evidence: `render-contract.mjs`; worker contract tests in `app.test.mjs`; `npm test`.

## M2 - Safety Policy

- [x] OBWR-030 [owner=codex] [deps=OBWR-020] [scope=addons/browser-worker/src/render-safety.mjs,addons/browser-worker/src/render-contract.mjs,addons/browser-worker/test]
  Goal: Add explicit URL, timeout, header/action budget, and response-size policy with safe invalid-request errors.
  Validation: `npm test`
  Review: Preserve local fixture tests while allowing production operators to configure stricter policy later.
  Evidence: `render-safety.mjs`; invalid URL/action/size tests; `npm test`.

## M3 - Runtime Seam

- [x] OBWR-040 [owner=codex] [deps=OBWR-030] [scope=addons/browser-worker/src/render-runtime.mjs,addons/browser-worker/src/crawlee-render-adapter.mjs,addons/browser-worker/src/extract.mjs,addons/browser-worker/test]
  Goal: Introduce a deep Render Runtime interface and move Crawlee/Playwright execution behind an adapter without changing response shape.
  Validation: `npm test`; `npm run smoke`
  Review: Runtime seam should make browser lifecycle and page execution local to one module.
  Evidence: `render-runtime.mjs`; `crawlee-render-adapter.mjs`; `npm test`; `npm run smoke`.

## M4 - Failure Taxonomy

- [x] OBWR-050 [owner=codex] [deps=OBWR-040] [scope=addons/browser-worker/src,addons/browser-worker/test,crates/nako-metadata-scraper/src/providers/rendered_page.rs]
  Goal: Return typed redaction-safe render failure kinds and map them through the Rust rendered-page runtime without leaking URLs, selectors, cookies, or proxy values.
  Validation: `npm test`; `cargo nextest run -p nako-metadata-scraper rendered browser_worker --no-fail-fast`
  Review: Do not expose raw provider page details in HTTP responses or Rust diagnostics.
  Evidence: Worker `failure_kind` responses; Rust rendered-page and provider execution tests.

## M5 - Closeout

- [x] OBWR-060 [owner=codex] [deps=OBWR-020,OBWR-030,OBWR-040,OBWR-050] [scope=addons/browser-worker/README.md,docs/workstreams/official-browser-worker-render-runtime-hardening]
  Goal: Update docs, run gates, record evidence, and commit.
  Validation: `npm test`; `npm run smoke`; `cargo nextest run -p nako-metadata-scraper rendered browser_worker --no-fail-fast`; `python -m json.tool docs/workstreams/official-browser-worker-render-runtime-hardening/WORKSTREAM.json`; `git diff --check`
  Review: Worktree must contain only intended browser-worker hardening changes.
  Evidence: `EVIDENCE_AND_GATES.md`
