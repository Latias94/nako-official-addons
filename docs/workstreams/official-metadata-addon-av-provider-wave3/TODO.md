# Task Ledger

Prefix: OMAV3

## Active

- [ ] OMAV3-020 [owner=codex] [deps=OMAV3-010] [scope=crates/nako-metadata-scraper/src/providers/rendered_av.rs,crates/nako-metadata-scraper/src/providers/*,crates/nako-metadata-scraper/src/engine]
  Goal: Build a reusable rendered AV provider fixture harness for parser/mapper/search/direct lookup contracts.
  Validation: `cargo nextest run -p nako-metadata-scraper rendered_av provider_fixture av --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`
  Review: Confirm existing JavBus/JavLibrary/MGStage tests use the shared harness or prove why a provider-local test is still needed.
  Evidence:
  Handoff:

## Pending

- [ ] OMAV3-030 [owner=codex] [deps=OMAV3-020] [scope=crates/nako-metadata-scraper/src/engine,crates/nako-metadata-scraper/src/providers,addons/metadata-scraper/README.md,crates/nako-metadata-scraper/README.md]
  Goal: Add explicit provider execution protection: request/config-visible budgets, bounded cache/cooldown policy, and redaction-safe reporting.
  Validation: `cargo nextest run -p nako-metadata-scraper provider_guard bulk runtime provider_execution --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`
  Review: Confirm protection state is explicit and does not introduce hidden scheduler memory that Nako cannot reason about.
  Evidence:
  Handoff:

- [ ] OMAV3-040 [owner=codex] [deps=OMAV3-020] [scope=crates/nako-metadata-scraper/src/providers,crates/nako-metadata-scraper/src/config.rs,crates/nako-metadata-scraper/src/manifest.rs,addons/metadata-scraper/README.md,crates/nako-metadata-scraper/README.md]
  Goal: Add the first wave 3 provider, preferring Prestige if synthetic fixture mapping proves stable.
  Validation: `cargo nextest run -p nako-metadata-scraper prestige config registry manifest av --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`
  Review: Confirm the provider is disabled by default, emits declared external IDs, supports only correct AV routes, and uses independent parser fixtures.
  Evidence:
  Handoff:

- [ ] OMAV3-050 [owner=codex] [deps=OMAV3-020,OMAV3-030] [scope=crates/nako-metadata-scraper/src/providers,crates/nako-metadata-scraper/src/config.rs,crates/nako-metadata-scraper/src/manifest.rs,addons/metadata-scraper/README.md,crates/nako-metadata-scraper/README.md]
  Goal: Add one FC2 long-tail provider after evaluating FC2PPVDB, FC2Hub, and FC2Club for testable value.
  Validation: `cargo nextest run -p nako-metadata-scraper fc2 av config registry manifest --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`
  Review: Confirm it does not duplicate the existing FC2 official source and improves fallback coverage.
  Evidence:
  Handoff:

- [ ] OMAV3-060 [owner=codex] [deps=OMAV3-020,OMAV3-030] [scope=crates/nako-metadata-scraper/src/providers,crates/nako-metadata-scraper/src/config.rs,crates/nako-metadata-scraper/src/manifest.rs,addons/metadata-scraper/README.md,crates/nako-metadata-scraper/README.md]
  Goal: Add the official uncensored provider trio path for Caribbeancom, 1Pondo, and 10Musume, or split if one site needs a separate lane.
  Validation: `cargo nextest run -p nako-metadata-scraper caribbean 1pondo 10musume av config registry manifest --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`
  Review: Confirm each provider has independent fixtures, route gates, external IDs, field quality descriptors, and docs.
  Evidence:
  Handoff:

- [ ] OMAV3-070 [owner=codex] [deps=OMAV3-040,OMAV3-050,OMAV3-060] [scope=crates/nako-metadata-scraper/src/providers,addons/metadata-scraper/README.md,crates/nako-metadata-scraper/README.md,docs/workstreams/official-metadata-addon-av-provider-wave3]
  Goal: Run full gates, document provider wave 3 behavior, and close or split remaining provider candidates.
  Validation: `cargo nextest run -p nako-metadata-scraper --no-fail-fast`; `npm --prefix addons/browser-worker test`; `python -m json.tool docs/workstreams/official-metadata-addon-av-provider-wave3/WORKSTREAM.json`; `git diff --check`
  Review: Confirm no provider or protection work remains hidden in handoff notes.
  Evidence:
  Handoff:

## Completed

- [x] OMAV3-010 [owner=codex] [deps=-] [scope=docs/workstreams/official-metadata-addon-av-provider-wave3]
  Goal: Open the durable AV Provider Wave 3 lane with harness, provider breadth, and real-use protection tasks.
  Validation: `python -m json.tool docs/workstreams/official-metadata-addon-av-provider-wave3/WORKSTREAM.json`; `git diff --check`
  Review: Confirmed the lane covers all three user-approved streams and keeps MDCx as reference-only.
  Evidence: PASS on 2026-05-26.
  Handoff: DONE. OMAV3-020 is active.

## Follow-Up Candidates

- ThePornDB, Jav321, region-specific fallbacks, or additional FC2 sources not
  selected in this lane.
- Nako core refresh/locked-field/local metadata/local artwork priority.
- User-facing review UI, NFO/rename, and actor-image workflows.
