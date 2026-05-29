# Official Metadata Bangumi Provider Baseline - Evidence And Gates

Status: Complete
Last updated: 2026-05-23

## Baseline

- Previous commit: `92e2542 feat(metadata-scraper): add provider runtime and tmdb baseline`.
- Current branch: `main`, ahead of origin by one commit before Bangumi work.
- Official Bangumi references:
  - https://github.com/bangumi/api
  - https://raw.githubusercontent.com/bangumi/api/master/open-api/v0.yaml
  - https://raw.githubusercontent.com/bangumi/api/master/docs-raw/user%20agent.md

## Gates

- `cargo fmt --all -- --check`
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`
- `cargo nextest run --workspace --no-fail-fast`
- `git diff --check`

## OMBGM-010

Status: DONE 2026-05-23.

Evidence:

- Workstream docs created with official API facts, scope, risks, task ledger,
  milestones, gates, and handoff.

## OMBGM-020

Status: DONE 2026-05-23.

Evidence:

- `cargo nextest run -p nako-metadata-scraper config manifest registry --no-fail-fast`
  passed with 18 tests and 15 skipped.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed with only the existing Cargo.lock line-ending
  warning.
- Config tests cover Bangumi defaults, environment overrides, invalid subject
  type fallback, and optional token/user-agent settings.
- Manifest tests cover Bangumi provider schema and optional secret field.
- Registry tests cover disabled Bangumi diagnostics and ready public Bangumi
  provider construction without a token.

Note:

- Running Cargo aligned `Cargo.lock`'s path dependency metadata for
  `nako-addon-protocol` to the currently resolved local crate version
  `0.1.0-alpha.1`.

## OMBGM-030

Status: DONE 2026-05-23.

Evidence:

- `cargo nextest run -p nako-metadata-scraper bangumi --no-fail-fast`
  passed with 3 tests and 31 skipped.
- `cargo fmt --all -- --check` passed after formatting.
- `git diff --check` passed with only the Cargo.lock line-ending warning.
- Fake transport proves `POST /v0/search/subjects` request shape with `limit`,
  `offset`, `keyword`, `sort`, `filter.type`, and `filter.nsfw`.
- Fake transport proves configured User-Agent reaches the HTTP runtime config.
- Fake transport proves optional bearer auth is sent when configured.
- Fake transport proves `GET /v0/subjects/{subject_id}` detail enrichment.
- Candidate mapping proves localized title/original title, summary, date/year,
  platform, subject type, eps/total episodes, ranking/score tags, genre tags,
  image metadata tags, and Bangumi external ID facts.

## OMBGM-040

Status: DONE 2026-05-23.

Evidence:

- Root README describes fixture/TMDB/Bangumi runtime defaults.
- Addon README describes Bangumi search/detail baseline, User-Agent, optional
  token, mapped fields, and Douban/Playwright deferral.
- Dockerfile, compose example, and systemd example include Bangumi disabled
  defaults and User-Agent configuration.
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast` passed with 34
  tests.
- `git diff --check` passed with only the Cargo.lock line-ending warning.

## OMBGM-050

Status: DONE 2026-05-23.

Evidence:

- `python -m json.tool docs/workstreams/official-metadata-bangumi-provider-baseline/WORKSTREAM.json`
  passed.
- `cargo fmt --all -- --check` passed.
- `cargo nextest run --workspace --no-fail-fast` passed with 34 tests.
- `git diff --check` passed with only the Cargo.lock line-ending warning.

Follow-ons:

- Live Bangumi provider QA with an operator-supplied compliant User-Agent and,
  optionally, access token.
- Douban provider and crawler/browser automation runtime design, potentially
  with Playwright, isolated from the shared API-provider HTTP runtime.
- Episode-level metadata mapping.
- Artwork/image materialization instead of image URL tags.
