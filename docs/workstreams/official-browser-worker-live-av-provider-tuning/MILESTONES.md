# Official Browser Worker Live AV Provider Tuning - Milestones

Status: Completed
Last updated: 2026-05-27

## M1 - Provider-Owned Wait Budgets

Exit criteria:

- Done: Rust render drift cases can emit `selector_timeout_ms`.
- Done: Slow live AV provider presets use explicit selector wait budgets.
- Done: slow production render requests also pass their timeout budget to
  Browser Worker as `render_timeout_ms`.
- Done: focused Rust and Browser Worker tests pass.

## M2 - Live Evidence

Exit criteria:

- Done: the 14-case live AV suite was re-run through the local proxy.
- Done: results are summarized without raw URLs, sample numbers, secrets, or
  proxy values.
- Done: improved cases and remaining access/network failures are separated in
  `EVIDENCE_AND_GATES.md`.

## M3 - Closeout

Exit criteria:

- Done: final formatting, JSON validation, diff hygiene, and commit.
