# Task Ledger

Prefix: APQ

## Active

- None.

## Pending

- None.

## Completed

- [x] APQ-010 [owner=codex] [deps=-] [scope=crates/nako-metadata-scraper/src/providers/rendered_av.rs,crates/nako-metadata-scraper/src/providers/official_uncensored.rs,crates/nako-metadata-scraper/src/providers/fc2ppvdb.rs]
  Goal: Promote row-level structured label parsing into `rendered_av` and migrate official uncensored plus FC2PPVDB to it.
  Validation: `cargo nextest run -p nako-metadata-scraper rendered_av official_uncensored fc2ppvdb --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`
  Review: Shared helper owns row-boundary behavior while provider-specific row selectors remain local. Tests prove row-level values do not swallow following description/trailer text and that FC2PPVDB avoids selecting the outer details block.
  Evidence: PASS on 2026-05-26: 13 rendered_av/official_uncensored/fc2ppvdb tests; fmt, workstream JSON, and diff hygiene passed.
  Handoff: DONE. APQ-020 is active.

- [x] APQ-020 [owner=codex] [deps=APQ-010] [scope=crates/nako-metadata-scraper/src/providers/dmm,crates/nako-metadata-scraper/src/providers/mgstage.rs,crates/nako-metadata-scraper/src/providers/javbus.rs]
  Goal: Migrate official/rendered providers with row-like detail pages to shared structured labels where tests prove a quality gain.
  Validation: `cargo nextest run -p nako-metadata-scraper dmm mgstage javbus rendered_av --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`
  Review: DMM, MGStage, and JavBus now use provider-local row selectors with shared structured label fallback. Tests assert release/runtime plus studio/publisher/series/director facts across row-like fixtures.
  Evidence: PASS on 2026-05-26: 14 dmm/mgstage/javbus/rendered_av tests; fmt check passed.
  Handoff: DONE. APQ-030 is active.

- [x] APQ-030 [owner=codex] [deps=APQ-010] [scope=crates/nako-metadata-scraper/src/providers/javlibrary.rs,crates/nako-metadata-scraper/src/providers/javdb,crates/nako-metadata-scraper/src/providers/fc2]
  Goal: Audit community and FC2 parsers for reusable list/media helpers and migrate only where the abstraction is cleaner.
  Validation: `cargo nextest run -p nako-metadata-scraper javlibrary javdb fc2 rendered_av --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`
  Review: JavLibrary, JavDB, and FC2 now reuse structured label fallback for metadata rows while keeping provider-specific text, link, image, and URL helpers local.
  Evidence: PASS on 2026-05-26: 26 javlibrary/javdb/fc2/rendered_av tests; fmt check passed.
  Handoff: DONE. APQ-040 is active.

- [x] APQ-040 [owner=codex] [deps=APQ-020,APQ-030] [scope=crates/nako-metadata-scraper/README.md,addons/metadata-scraper/README.md,docs/workstreams/official-metadata-addon-av-parser-quality]
  Goal: Verify parser-quality lane, document drift strategy, and close or split remaining provider parser work.
  Validation: `cargo nextest run -p nako-metadata-scraper --no-fail-fast`; `python -m json.tool docs/workstreams/official-metadata-addon-av-parser-quality/WORKSTREAM.json`; `git diff --check`
  Review: No parser-quality decision exists only in journal notes. All current AV providers use shared row-level label parsing where relevant; provider-specific selectors and media/link parsing remain local.
  Evidence: PASS on 2026-05-26: full metadata-scraper package gate passed with 222 tests; README/docs, workstream JSON, fmt, and diff hygiene passed.
  Handoff: DONE. Workstream complete.

## Follow-Up Candidates

- Wave 4 providers such as ThePornDB, Jav321, or additional FC2 sources.
- Manual live drift tooling that stores no adult payloads in the repository.
- Nako core refresh/locked-field/local metadata/local artwork priority.
