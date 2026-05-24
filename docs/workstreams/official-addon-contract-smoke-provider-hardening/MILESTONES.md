# Official Addon Contract Smoke Provider Hardening - Milestones

Status: Complete
Last updated: 2026-05-24

## M0 - Scope Freeze

Exit criteria:

- ADR 0033 is referenced as the existing decision for protocol/crate/addon
  version separation.
- Tasks cover only task envelope unification, live smoke hardening, and provider
  descriptor boundary.
- Sidecar installation experience is a written constraint.

## M1 - Task Envelope Contract Unification

Exit criteria:

- Metadata scraper task endpoint and bulk planner use public
  `nako-addon-protocol` task envelope types.
- Sidecar-specific result payloads remain sidecar-owned.
- Focused bulk/task/route tests pass.

Evidence:

- OACSH-020 replaced local task envelope mirrors with public protocol types.

## M2 - Live Smoke Harness

Exit criteria:

- Smoke commands clearly support direct sidecar checks and Nako-mediated live
  checks.
- Task-path smoke proves host-created direct task run reaches the sidecar task
  endpoint and records a bounded successful result.
- If live smoke cannot run, the blocker is exact: missing server, missing admin
  token, missing sidecar, or port/process conflict.

Evidence:

- OACSH-030 added no-silent-skip guards, writeback assertions, manifest task
  checks, and E2E preflight mode.

## M3 - Provider Descriptor Boundary

Exit criteria:

- Provider metadata needed by registry, diagnostics, and manifest schema is
  declared close to each provider.
- Adding a provider no longer requires central modules to know unrelated
  provider-specific details.
- Registry/manifest/config tests pass.

Evidence:

- OACSH-040 moved catalog entries to provider modules and manifest provider
  schema/secret references to registry-derived descriptors.

## M4 - Closeout

Exit criteria:

- Workstream evidence is fresh and reproducible.
- Remaining sidecar packaging/plugin-family questions are recorded as a
  separate design topic, not implemented accidentally.

Evidence:

- OACSH-050 closeout records live smoke and addon-family splitting as
  follow-ons.
