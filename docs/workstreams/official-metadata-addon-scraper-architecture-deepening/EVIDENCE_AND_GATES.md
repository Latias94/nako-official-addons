# Evidence And Gates

Status: Active
Last updated: 2026-05-26

## Required Gates

| Gate | Command | Required before |
| --- | --- | --- |
| Workstream JSON | `python -m json.tool docs/workstreams/official-metadata-addon-scraper-architecture-deepening/WORKSTREAM.json` | OMSAD-010 completion and closeout |
| Workstream hygiene | `git diff --check` | every task completion |
| Typed scrape outcome | `cargo nextest run -p nako-metadata-scraper bulk runtime metadata_endpoint --no-fail-fast` | OMSAD-020 completion |
| Render intent | `cargo nextest run -p nako-metadata-scraper rendered_page browser_worker douban javbus javlibrary mgstage --no-fail-fast`; `npm --prefix addons/browser-worker test` | OMSAD-030 completion |
| Rendered AV flow | `cargo nextest run -p nako-metadata-scraper javbus javlibrary mgstage av --no-fail-fast` | OMSAD-040 completion |
| Provider quality descriptors | `cargo nextest run -p nako-metadata-scraper config registry manifest field_policy resolver av --no-fail-fast` | OMSAD-050 completion |
| Resolver/fusion/ranking | `cargo nextest run -p nako-metadata-scraper resolver ranking artwork writeback av --no-fail-fast` | OMSAD-060 completion |
| Side-effect writeback | `cargo nextest run -p nako-metadata-scraper writeback artwork runtime --no-fail-fast` | OMSAD-070 completion |
| Package validation | `cargo nextest run -p nako-metadata-scraper --no-fail-fast` | closeout |
| Browser-worker validation | `npm --prefix addons/browser-worker test` | closeout when browser-worker changed |
| Formatting | `cargo fmt -p nako-metadata-scraper -- --check` | every Rust task completion |

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-26 | OMSAD-010 | `python -m json.tool docs/workstreams/official-metadata-addon-scraper-architecture-deepening/WORKSTREAM.json`; `git diff --check` | Pass |
| 2026-05-26 | OMSAD-020 | `cargo nextest run -p nako-metadata-scraper runtime_builds_typed_scrape_outcome_before_response_rendering --no-fail-fast` red/green tracer; `cargo nextest run -p nako-metadata-scraper bulk runtime metadata_endpoint --no-fail-fast`; `cargo nextest run -p nako-metadata-scraper --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`; `python -m json.tool docs/workstreams/official-metadata-addon-scraper-architecture-deepening/WORKSTREAM.json`; `git diff --check` | Pass: typed scrape outcome implemented; focused gate 41 passed; full package gate 193 passed |
| 2026-05-26 | OMSAD-030 | `cargo nextest run -p nako-metadata-scraper rendered_page_runtime_serializes_render_intent_options --no-fail-fast` red/green tracer; `cargo nextest run -p nako-metadata-scraper rendered_page --no-fail-fast`; `cargo nextest run -p nako-metadata-scraper rendered_page browser_worker douban javbus javlibrary mgstage --no-fail-fast`; `npm --prefix addons/browser-worker test`; `cargo fmt -p nako-metadata-scraper -- --check`; `python -m json.tool docs/workstreams/official-metadata-addon-scraper-architecture-deepening/WORKSTREAM.json`; `git diff --check` | Pass: render intent implemented; focused gate 19 passed; browser-worker gate 4 passed |
| 2026-05-26 | OMSAD-040 | `cargo nextest run -p nako-metadata-scraper rendered_av_flow_searches_then_renders_first_detail_result javbus javlibrary mgstage --no-fail-fast` red/green tracer; `cargo nextest run -p nako-metadata-scraper javbus javlibrary mgstage av --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`; `python -m json.tool docs/workstreams/official-metadata-addon-scraper-architecture-deepening/WORKSTREAM.json`; `git diff --check` | Pass: rendered AV flow implemented; focused gate 33 passed |
| 2026-05-26 | OMSAD-050 | `cargo nextest run -p nako-metadata-scraper registry_builds_default_av_field_policy_from_provider_quality_descriptors --no-fail-fast` red/green tracer; `cargo nextest run -p nako-metadata-scraper config registry manifest field_policy resolver av --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`; `python -m json.tool docs/workstreams/official-metadata-addon-scraper-architecture-deepening/WORKSTREAM.json`; `git diff --check` | Pass: provider quality descriptors implemented; focused gate 62 passed |
