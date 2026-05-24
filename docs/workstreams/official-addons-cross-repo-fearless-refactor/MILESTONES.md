# Official Addons Cross-Repo Fearless Refactor - Milestones

Status: Complete
Last updated: 2026-05-24

## M0 - Scope Freeze

Exit criteria:

- Three discovered problems are recorded with concrete file/workstream anchors.
- Non-goals explicitly exclude new provider count, all-in-one scraper behavior,
  and sidecar process supervision.
- Cross-repo dirty-worktree risk is recorded.

Evidence:

- `docs/workstreams/official-addons-cross-repo-fearless-refactor/DESIGN.md`
- `docs/workstreams/official-addons-cross-repo-fearless-refactor/TODO.md`
- `docs/workstreams/official-addons-cross-repo-fearless-refactor/WORKSTREAM.json`

## M1 - Protected-Write Client Alignment

Exit criteria:

- Official addon protected-write request/response typing and safe error mapping
  are aligned with public Nako addon client/protocol crates where appropriate.
- The sidecar keeps its own transport and runtime policy.
- Tests prove bearer/header placement, no token in request body, safe errors,
  and version tolerance.

Evidence:

- `../nako` public addon protocol/client crates now expose protected-write
  DTOs and runtime client helpers.
- `nako-official-addons` metadata scraper runtime facade delegates to the
  public client while preserving sidecar transport testability.
- Focused client/protocol/runtime tests passed.

## M2 - Provider Adapter Deepening

Exit criteria:

- Bangumi adapter is split into deep provider-local modules with unchanged
  behavior.
- Douban adapter is split into deep provider-local modules with unchanged
  behavior.
- Focused provider/ranking/title tests pass.

Evidence:

- Bangumi and Douban provider-local module splits are complete.
- Combined provider/ranking/title focused gate passed.

## M3 - Official Addon Task Path Smoke

Exit criteria:

- Official smoke can prove Nako host-dispatched task-path execution against
  `bulk-metadata-scrape`.
- The smoke documents task-result expectations and redaction boundaries.
- The smoke does not imply Nako process/container supervision.

Evidence:

- `smoke.local.ps1` and `../nako/scripts/official-addon-e2e-smoke.ps1` support
  the host-dispatched task path.
- PowerShell parser checks, addon task/route tests, and Nako server
  direct-dispatch integration tests passed.
- Live Docker/server smoke is deferred until a server/admin token is available.

## M4 - Closeout

Exit criteria:

- Workstream status is complete or blocked with concrete external blockers.
- Final evidence is recorded.
- Follow-on official plugin work is split into new lanes or explicitly
  deferred.

Evidence:

- Workstream docs and journals updated for closeout.
