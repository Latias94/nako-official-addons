# Official Resource Search First-Class Protocol - Milestones

Status: Active
Last updated: 2026-05-28

## M0 - Scope And Evidence Freeze

Exit criteria:

- Workstream docs exist and agree on scope.
- Admin UI is explicitly out of scope.
- First task has a targeted validation command.

Status: done.

## M1 - First-Class Protocol Slice

Exit criteria:

- Manifest runtime and checked-in example both declare `resource_search`.
- Manifest scope is `acquisition_search_read`.
- Route rejects non-`resource_search` envelopes.
- Response payload uses `nako.addon.resource_search.response.v1`.
- Targeted tests pass.

Status: done.

## M2 - Docs And Follow-On Boundaries

Exit criteria:

- README and smoke docs no longer describe resource search as temporary
  automation.
- Link-check, downloader/external runner, cloud-drive transfer, and
  password/code reference boundaries are documented as separate contracts.
- No Admin UI work is included.

Status: done.

## M3 - Verification And Closeout

Exit criteria:

- Package tests pass.
- Formatting and check gates pass.
- Workstream evidence is fresh.
- Lane is committed with a Conventional Commit message.

Status: done pending commit.
