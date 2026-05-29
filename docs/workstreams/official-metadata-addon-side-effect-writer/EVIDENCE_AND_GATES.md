# Official Metadata Addon Side Effect Writer - Evidence And Gates

Status: Complete
Last updated: 2026-05-23

## Gates

- `cargo fmt --all -- --check`
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`
- `cargo nextest run --workspace --no-fail-fast`
- `git diff --check`

Focused task gates may be recorded before broader gates.

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-23 | OMASE-010 | Workstream docs created for side-effect writer, typed artwork candidate flow, and Bulk Metadata Scrape evaluation. | Pass. |
| 2026-05-23 | OMASE-020 | `cargo nextest run -p nako-metadata-scraper nako_runtime config --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check` | Pass. Nako runtime config and outbound client tests passed. |
| 2026-05-23 | OMASE-030 | `cargo nextest run -p nako-metadata-scraper side_effect metadata --no-fail-fast`; `cargo nextest run -p nako-metadata-scraper --no-fail-fast`; `cargo fmt --all -- --check`; `cargo build -p nako-metadata-scraper`; direct smoke on `127.0.0.1:19100`; direct writeback smoke on `127.0.0.1:19101`; `git diff --check` | Pass. Explicit `payload.writeback` submission, skipped fallback, smoke script support, and docs all verified. |
| 2026-05-23 | OMASE-040 | `cargo test -p nako-metadata-scraper --lib --no-run`; `cargo nextest run -p nako-metadata-scraper artwork side_effect --no-fail-fast`; `cargo fmt --all -- --check`; `cargo nextest run -p nako-metadata-scraper --no-fail-fast`; `cargo nextest run --workspace --no-fail-fast`; `git diff --check` | Pass. Typed artwork candidates now flow out of TMDB/Bangumi/browser worker providers, and explicit `artwork_write` submission is covered by runtime and engine tests. |
| 2026-05-23 | OMASE-050 | Host assessment against `../nako` Addon Task declarations, routing plans, author guide, and HTTP API docs; `cargo nextest run -p nako-metadata-scraper manifest bulk --no-fail-fast`; `python -m json.tool` on OMASE/OMAB workstream JSON; `cargo nextest run --workspace --no-fail-fast`; `git diff --check` | Pass. Bulk Metadata Scrape task declaration is deferred, manifest test proves `tasks` stays empty, and follow-on design lane `official-metadata-addon-bulk-task-design` is open. |
| 2026-05-23 | OMASE-060 | `cargo fmt --all -- --check`; `cargo nextest run --workspace --no-fail-fast`; `python -m json.tool` on OMASE/OMAB workstream JSON; `git diff --check` | Pass. Operator docs now describe explicit metadata/artwork writeback, Bulk Metadata Scrape is split into its own design lane, and the side-effect writer lane is closed. |

## Safety Requirements

- Addon Token values must not be serialized into request payloads,
  diagnostics, logs, docs examples, or response payloads.
- Ordinary `/metadata` calls must not write Canonical Metadata or artwork.
- Explicit side-effect submission must include library, target, permission,
  idempotency key, and redaction-safe provenance.
- Provider image URLs must stay as Artwork Candidate inputs; Nako owns fetch,
  validation, cache, selected artwork, and public image serving.
- Bulk Metadata Scrape must use a Nako-owned Addon Task seam when available.
