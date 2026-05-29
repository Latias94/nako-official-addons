# Official Addons Architecture Boundary Hardening - TODO

Status: Complete
Last updated: 2026-05-29

Task IDs use the `OAABH` prefix.

## M0 - Scope And Evidence Freeze

- [x] OAABH-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-addons-architecture-boundary-hardening]
  Goal: Record the architecture findings, selected order, non-goals, cross-repo
  constraints, and validation gates before code changes.
  Validation: DESIGN.md, TODO.md, MILESTONES.md, EVIDENCE_AND_GATES.md,
  WORKSTREAM.json, and HANDOFF.md agree.
  Review: Confirm this lane does not duplicate completed resolver, provider
  capability, field-policy, release-readiness, or resource-search protocol
  workstreams.
  Evidence: this workstream directory.
  Handoff: Continue with OAABH-020.

## M1 - Official Manifest Source Of Truth

- [x] OAABH-020 [owner=codex] [deps=OAABH-010] [scope=crates/nako-resource-search/src/manifest.rs,crates/nako-subtitle-provider/src/manifest.rs,crates/nako-dlna-renderer/src/manifest.rs,../nako/crates/nako-official-addon-catalog/src/lib.rs]
  Goal: Make resource-search, subtitle-provider, and DLNA renderer runtime
  manifests use `nako-official-addon-catalog` builders instead of duplicating
  official addon facts locally.
  Validation: focused manifest/catalog tests in both repositories, format
  checks for touched packages, and path-scoped `git diff --check`.
  Review: Sidecars may still own runtime-specific configuration fragments, but
  addon IDs, names, resource kinds, paths, scopes, hosted pages, and install
  descriptor facts should come from the catalog where possible.
  Evidence: command transcript in EVIDENCE_AND_GATES.md.
  Handoff: Continue with OAABH-030. Catalog/runtime/example manifest shapes
  match for the affected sidecars.

## M2 - Nako Addon App Service Boundary Deepening

- [x] OAABH-030 [owner=codex] [deps=OAABH-020] [scope=../nako/crates/nako-server/src/app/addons.rs,../nako/crates/nako-server/src/app/addons]
  Goal: Split user-facing addon workflows out of the large parent app service
  file into cohesive local modules without changing public Admin API behavior.
  Validation: focused `nako-server` addon tests plus compile checks for touched
  crates.
  Review: Preserve repository traits, DTOs, error behavior, redaction, and
  active workstream boundaries. Do not touch `../nako/web`.
  Evidence: module map and command transcript in EVIDENCE_AND_GATES.md.
  Handoff: Continue with OAABH-040. Parent app service now keeps the service
  skeleton, shared registration/token/grant helpers, and cross-cutting helpers;
  user-facing workflows live in local modules.

## M3 - Provider HTTP Operation Policy

- [x] OAABH-040 [owner=codex] [deps=OAABH-020] [scope=crates/nako-metadata-scraper/src/providers/http_runtime.rs,crates/nako-metadata-scraper/src/providers]
  Goal: Add explicit provider operation policy for retry-after handling and
  bounded provider-local safe caching/throttling inputs without adding hidden
  scheduler state.
  Validation: focused HTTP runtime tests, provider call-site tests where policy
  is wired, and `cargo nextest run -p nako-metadata-scraper http_runtime provider --no-fail-fast`.
  Review: Keep provider quirks provider-local. Do not cache sensitive or
  authenticated responses unless a provider explicitly marks the operation safe.
  Evidence: command transcript in EVIDENCE_AND_GATES.md.
  Handoff: Continue with OAABH-050. HTTP runtime now carries explicit retry,
  cache, and throttle operation facts to transports; TMDB detail enrichment
  declares authenticated safe-cache and provider-local throttle intent.

## M4 - Notification Bridge Route Locality

- [x] OAABH-050 [owner=codex] [deps=OAABH-020] [scope=crates/nako-notification-bridge/src/routes.rs,crates/nako-notification-bridge/src]
  Goal: Move provider send orchestration and diagnostics rendering out of the
  route module while preserving public routes and response payloads.
  Validation: focused notification bridge route/provider/diagnostics tests plus
  package format check.
  Review: Keep redaction guarantees and provider attempt history semantics
  unchanged.
  Evidence: command transcript in EVIDENCE_AND_GATES.md.
  Handoff: Continue with OAABH-060. Provider fan-out now lives in
  `provider_send`; diagnostics HTML rendering lives in `diagnostics`; route
  handlers preserve public response payloads.

## M5 - Stale Architecture Docs Cleanup

- [x] OAABH-060 [owner=codex] [deps=OAABH-040] [scope=docs/workstreams/official-metadata-addon-mature-provider-model-research]
  Goal: Update mature provider model research docs so completed resolver,
  external ID capability, and field-policy work is no longer presented as
  future P0.
  Validation: docs agree with current code and this workstream's follow-on
  list.
  Review: Do not delete useful reference findings; mark completed items and
  keep remaining follow-ons accurate.
  Evidence: docs diff and closeout notes.
  Handoff: Continue with OAABH-070. Mature-provider research now records
  resolver, external ID capabilities, field-policy fusion, and HTTP operation
  policy as completed baseline architecture. Remaining follow-ons are host
  policy context, artwork source separation, matching strategy refinement, and
  only-if-needed cache/throttle execution state.

## M6 - Closeout

- [x] OAABH-070 [owner=planner] [deps=OAABH-020,OAABH-030,OAABH-040,OAABH-050,OAABH-060] [scope=docs/workstreams/official-addons-architecture-boundary-hardening]
  Goal: Review and verify the whole lane, record residual risks, and split
  External Acquisition Runner or Addon Manager work into separate lanes if it
  has become actionable.
  Validation: review-workstream and verify-rust-workstream evidence; final
  focused gates pass or blockers are concrete external constraints.
  Review: Close only when code, docs, and evidence agree.
  Evidence: EVIDENCE_AND_GATES.md, WORKSTREAM.json, HANDOFF.md, and closeout
  journal.
  Handoff: DONE. Lane closed with focused gates passing in both repositories.
