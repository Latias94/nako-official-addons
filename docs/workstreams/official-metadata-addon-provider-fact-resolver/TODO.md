# Official Metadata Addon Provider Fact Resolver - TODO

Status: Active
Last updated: 2026-05-25

## M0 - Scope And License Guardrails

- [x] OMAPFR-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-metadata-addon-provider-fact-resolver]
  Goal: Open the resolver implementation lane, define task order, and record license guardrails against copying reference repository source.
  Validation: `python -m json.tool docs/workstreams/official-metadata-addon-provider-fact-resolver/WORKSTREAM.json`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: Confirm this lane implements local Nako architecture and does not vendor or copy reference code.
  Evidence: `python -m json.tool docs/workstreams/official-metadata-addon-provider-fact-resolver/WORKSTREAM.json`; `cargo fmt --all -- --check`; `git diff --check`; `git status --short --ignored repo-ref docs/workstreams/official-metadata-addon-provider-fact-resolver`.
  Handoff: DONE. Workstream opened with license guardrails; start OMAPFR-020 resolver model.

## M1 - Resolver Model

- [x] OMAPFR-020 [owner=codex] [deps=OMAPFR-010] [scope=crates/nako-metadata-scraper/src/engine]
  Goal: Add an internal resolver model that adapts current `ProviderMetadataCandidate` values into provenance-preserving facts and candidate clusters without changing provider outputs.
  Validation: `cargo nextest run -p nako-metadata-scraper resolver --no-fail-fast`.
  Review: Confirm the model is local, minimal, redaction-safe, and does not copy mature-project implementation structure.
  Evidence: `cargo nextest run -p nako-metadata-scraper resolver --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`.
  Handoff: DONE. Resolver model exists and is intentionally not yet wired into orchestration; start OMAPFR-030.

## M2 - Resolver-Backed Orchestration

- [x] OMAPFR-030 [owner=codex] [deps=OMAPFR-020] [scope=crates/nako-metadata-scraper/src/engine/orchestration.rs,crates/nako-metadata-scraper/src/engine/ranking.rs,crates/nako-metadata-scraper/src/engine]
  Goal: Route `suggest_candidates` through resolver clustering so exact provider IDs and shared external IDs are resolved before final ranking while preserving the `/metadata` response shape.
  Validation: `cargo nextest run -p nako-metadata-scraper resolver orchestration ranking --no-fail-fast`.
  Review: Confirm output compatibility, deterministic ordering, and raw provider provenance are preserved.
  Evidence: `cargo nextest run -p nako-metadata-scraper resolver orchestration ranking --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`.
  Handoff: DONE. Orchestration now resolves provider facts before ranking; start OMAPFR-040 external ID capability catalog.

## M3 - External ID Capability Catalog

- [ ] OMAPFR-040 [owner=codex] [deps=OMAPFR-030] [scope=crates/nako-metadata-scraper/src/providers/registry.rs,crates/nako-metadata-scraper/src/providers,crates/nako-metadata-scraper/src/engine/query.rs]
  Goal: Replace alias-only external ID descriptors with executable provider external ID capabilities while preserving current top-level alias parsing and numeric validation behaviour.
  Validation: `cargo nextest run -p nako-metadata-scraper external_id tmdb bangumi browser_worker resolver --no-fail-fast`.
  Review: Confirm descriptors are used by behaviour, not just documentation, and provider quirks stay provider-local.
  Evidence: pending.
  Handoff: TODO.

## M4 - Integration And Docs

- [ ] OMAPFR-050 [owner=codex] [deps=OMAPFR-040] [scope=crates/nako-metadata-scraper,addons/metadata-scraper,docs/workstreams/official-metadata-addon-provider-fact-resolver]
  Goal: Run full package validation, update user-facing docs if resolver or external ID capability behaviour needs explanation, and record integration evidence.
  Validation: `cargo nextest run -p nako-metadata-scraper --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: Confirm no public protocol break, no reference-source copy, and no host policy was moved into the sidecar.
  Evidence: pending.
  Handoff: TODO.

## M5 - Closeout

- [ ] OMAPFR-060 [owner=planner] [deps=OMAPFR-050] [scope=docs/workstreams/official-metadata-addon-provider-fact-resolver]
  Goal: Review, verify, and close the resolver workstream or split explicit follow-ons.
  Validation: `cargo nextest run -p nako-metadata-scraper --no-fail-fast`; `cargo fmt --all -- --check`; `python -m json.tool docs/workstreams/official-metadata-addon-provider-fact-resolver/WORKSTREAM.json`; `git diff --check`.
  Review: review-workstream has no blocking findings.
  Evidence: pending.
  Handoff: TODO.
