# Official Metadata Addon Non-AV Provider Expansion - TODO

Status: Active
Last updated: 2026-05-27

## M0 - Scope

- [x] OMANV-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-metadata-addon-non-av-provider-expansion]
  Goal: Freeze non-AV provider expansion scope, priorities, and validation strategy.
  Validation: `python -m json.tool docs/workstreams/official-metadata-addon-non-av-provider-expansion/WORKSTREAM.json`
  Evidence: `DESIGN.md`
  Handoff: Opened from the user-approved 1/2/4/5 provider plan.

## M1 - TMDB TV Foundation

- [x] OMANV-020 [owner=codex] [deps=OMANV-010] [scope=crates/nako-metadata-scraper/src/providers/tmdb.rs,crates/nako-metadata-scraper/src/providers/tmdb]
  Goal: Add TMDB TV search/direct lookup and map TV series candidates with external IDs, artwork, dates, runtime, genres, titles, overview, and score facts.
  Validation: `cargo nextest run -p nako-metadata-scraper tmdb --no-fail-fast`
  Review: Existing movie behavior and `tmdb_id` direct lookup must remain stable; TV gets explicit `tmdb_tv_id`.
  Evidence: TMDB provider tests; `cargo nextest run -p nako-metadata-scraper tmdb --no-fail-fast`; `cargo nextest run -p nako-metadata-scraper config registry manifest routes tmdb --no-fail-fast`.

- [ ] OMANV-030 [owner=codex] [deps=OMANV-020] [scope=crates/nako-metadata-scraper/src/providers/tmdb.rs,crates/nako-metadata-scraper/src/providers/tmdb]
  Goal: Add TMDB season/episode direct lookup seeds and candidate mapping where the current metadata patch contract can represent them cleanly.
  Validation: `cargo nextest run -p nako-metadata-scraper tmdb season episode --no-fail-fast`
  Review: If the Addon Protocol cannot represent an episode field cleanly, split an ADR/follow-up instead of forcing lossy fields.
  Evidence: TMDB provider tests and any protocol-gap note.

## M2 - Bangumi Anime Enrichment

- [ ] OMANV-040 [owner=codex] [deps=OMANV-020] [scope=crates/nako-metadata-scraper/src/providers/bangumi.rs,crates/nako-metadata-scraper/src/providers/bangumi]
  Goal: Enrich Bangumi anime mapping with stronger aliases, relations/tags, production facts, image variants, score/vote evidence, and subject-type-aware titles.
  Validation: `cargo nextest run -p nako-metadata-scraper bangumi --no-fail-fast`
  Review: Avoid leaking raw user/authenticated-sensitive data into evidence or docs.
  Evidence: Bangumi mapper/parser tests.

## M3 - BrowserWorker Recipe Layer

- [ ] OMANV-050 [owner=codex] [deps=OMANV-010] [scope=crates/nako-metadata-scraper/src/providers/browser_worker.rs,crates/nako-metadata-scraper/src/providers/rendered_page.rs]
  Goal: Add a typed rendered-page extraction recipe path that can map title, overview, dates, tags, artwork, score, and external IDs from rendered HTML.
  Validation: `cargo nextest run -p nako-metadata-scraper browser_worker rendered --no-fail-fast`
  Review: Recipe execution must remain redaction-safe and bounded; selectors/config do not belong in logs.
  Evidence: BrowserWorker fake-runtime tests.

## M4 - New Provider First Wave

- [ ] OMANV-060 [owner=codex] [deps=OMANV-040] [scope=crates/nako-metadata-scraper/src/providers/anilist.rs,crates/nako-metadata-scraper/src/config.rs,crates/nako-metadata-scraper/src/providers/registry.rs]
  Goal: Add AniList as the first new non-AV provider with GraphQL search/direct lookup, anime fields, artwork, score, external IDs, config, manifest, and docs.
  Validation: `cargo nextest run -p nako-metadata-scraper anilist config registry manifest --no-fail-fast`
  Review: Keep GraphQL queries provider-local and avoid coupling query parsing to provider internals.
  Evidence: AniList provider tests.

- [ ] OMANV-070 [owner=planner] [deps=OMANV-050,OMANV-060] [scope=docs/workstreams/official-metadata-addon-non-av-provider-expansion]
  Goal: Decide whether TVDB, MAL, and IMDb should be implemented in this lane or split based on API/reliability evidence after TMDB TV, Bangumi, recipes, and AniList land.
  Validation: `python -m json.tool docs/workstreams/official-metadata-addon-non-av-provider-expansion/WORKSTREAM.json`
  Review: Do not add brittle providers just to increase count.
  Evidence: `HANDOFF.md`

## M5 - Verification And Closeout

- [ ] OMANV-080 [owner=codex] [deps=OMANV-020,OMANV-040,OMANV-050,OMANV-060] [scope=docs/workstreams/official-metadata-addon-non-av-provider-expansion,addons/metadata-scraper/README.md,crates/nako-metadata-scraper/README.md]
  Goal: Run full gates, update docs, record evidence, and close or split follow-ups.
  Validation: `cargo nextest run -p nako-metadata-scraper --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`; `python -m json.tool docs/workstreams/official-metadata-addon-non-av-provider-expansion/WORKSTREAM.json`; `git diff --check`
  Review: Worktree must contain only intended changes before commit.
  Evidence: `EVIDENCE_AND_GATES.md`
