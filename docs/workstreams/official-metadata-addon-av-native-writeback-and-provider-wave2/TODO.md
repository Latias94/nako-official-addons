# Task Ledger

Prefix: OMAV2

## Active

- [ ] OMAV2-040 [owner=codex] [deps=OMAV2-030] [scope=crates/nako-metadata-scraper/src/engine/bulk.rs,addons/metadata-scraper/README.md,crates/nako-metadata-scraper/README.md]
  Goal: Add bulk scrape mature accounting: retry classes, provider temporary suppression, cooldown hints, and resume-safe provider state.
  Validation: `cargo nextest run -p nako-metadata-scraper bulk --no-fail-fast`
  Review: Confirm bulk remains stateless from Nako's perspective and does not add a hidden scheduler.
  Evidence:
  Handoff:

## Pending

- [ ] OMAV2-050 [owner=codex] [deps=OMAV2-030,OMAV2-040] [scope=crates/nako-metadata-scraper/src/providers,crates/nako-metadata-scraper/src/config.rs,crates/nako-metadata-scraper/src/manifest.rs,addons/metadata-scraper]
  Goal: Add provider wave 2 behind disabled-by-default config, starting with JavLibrary plus one high-value route-specific provider.
  Validation: `cargo nextest run -p nako-metadata-scraper config registry manifest av javlibrary --no-fail-fast`
  Review: Confirm selectors/parsers are independently implemented from reference-only MDCx code.
  Evidence:
  Handoff:

- [ ] OMAV2-060 [owner=codex] [deps=OMAV2-050] [scope=docs/workstreams/official-metadata-addon-av-native-writeback-and-provider-wave2,addons/metadata-scraper/README.md,crates/nako-metadata-scraper/README.md]
  Goal: Run closeout gates, document shipped behavior and remaining provider parity, then close or split follow-ups.
  Validation: `cargo nextest run -p nako-metadata-scraper --no-fail-fast`; `npm --prefix addons/browser-worker test`; Nako focused gates from OMAV2-020; JSON validation; `git diff --check`
  Review: Confirm both repos contain only intended changes and follow-ups are explicit.
  Evidence:
  Handoff:

## Completed

- [x] OMAV2-010 [owner=codex] [deps=-] [scope=docs/workstreams/official-metadata-addon-av-native-writeback-and-provider-wave2]
  Goal: Open the durable second-stage AV lane with broken-protocol scope, cross-repo guardrails, task ledger, and gates.
  Validation: `python -m json.tool docs/workstreams/official-metadata-addon-av-native-writeback-and-provider-wave2/WORKSTREAM.json`; `git diff --check`
  Review: Confirmed the lane explicitly allows breaking the old minimal metadata patch contract.
  Evidence: PASS on 2026-05-26.
  Handoff: DONE. OMAV2-020 is active in `../nako`.

- [x] OMAV2-020 [owner=codex] [deps=OMAV2-010] [scope=../nako/crates/nako-addon-protocol,../nako/crates/nako-addon-client,../nako/crates/nako-reference-addon,../nako/crates/nako-server/src/app/addons/metadata_write.rs,../nako/docs/adr]
  Goal: Break and replace the narrow addon metadata writeback payload with a canonical graph-shaped payload and full catalog projection on apply.
  Validation: `cargo nextest run -p nako-addon-protocol -p nako-addon-client -p nako-reference-addon --no-fail-fast`; `cargo nextest run -p nako-server addon_side_effect_metadata_write --no-fail-fast`; `cargo fmt -p nako-addon-protocol -p nako-addon-client -p nako-reference-addon -p nako-server -- --check`
  Review: Confirmed no old partial graph writeback path remains; `metadata_write` now maps graph payloads and applies full catalog projection.
  Evidence: PASS on 2026-05-26; Nako commit `a0ad9a8`.
  Handoff: DONE. OMAV2-030 is active in official addons.

- [x] OMAV2-030 [owner=codex] [deps=OMAV2-020] [scope=crates/nako-metadata-scraper/src/engine,crates/nako-metadata-scraper/src/providers,addons/metadata-scraper/README.md,crates/nako-metadata-scraper/README.md]
  Goal: Materialize selected AV facts into native writeback fields and update response/writeback docs.
  Validation: `cargo nextest run -p nako-metadata-scraper av field_policy resolver writeback javdb dmm fc2 javbus --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`
  Review: Confirmed response `av` facts remain evidence, while writeback patch carries credits, studios, collections, external IDs, and image references.
  Evidence: PASS on 2026-05-26; 36 related tests passed.
  Handoff: DONE. OMAV2-040 is active.

## Follow-Up Candidates

- Additional provider wave: FC2PPVDB/FC2Hub/FC2Club, MGStage, Prestige,
  ThePornDB, Jav321, Caribbeancom, 1Pondo, 10Musume, and region-specific
  fallbacks.
- UI review tools for graph AV writeback review before apply.
- NFO/rename/actor-image lanes after native writeback is stable.
