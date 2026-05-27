# Official Browser Worker Live AV Provider Tuning - Handoff

Status: Completed
Last updated: 2026-05-27

## Current State

The lane implemented the provider-owned tuning and recorded a second live run.
Generated live drift cases now support safe `headers_from_env` references,
selector waits use DOM attachment semantics, DMM sends an age-confirmation
cookie by default, and slow rendered AV providers carry explicit render budgets
through to Browser Worker.

## Final State

The lane is complete. Keep unrelated `Cargo.toml`, `Cargo.lock`, and
Chromecast renderer changes out of this commit. They are separate work.

## Evidence Summary

- Focused Rust test gate passed: 45 passed, 230 skipped.
- Browser Worker test gate passed: 13 passed.
- Live AV drift with local proxy improved from the previous 5/14 passing
  baseline to 9/14 passing. Remaining failures are access/network classes:
  JavBus gated access without a real operator cookie, JavLibrary/MGStage
  blocking, and AVSox/FC2PPVDB network failures.

## Guardrails

- Do not commit raw live HTML, URLs, sample numbers, cookies, or proxy values.
- Do not hardcode bypasses for forbidden or account-gated pages.
- Keep provider-specific knowledge in provider modules, not Browser Worker.
