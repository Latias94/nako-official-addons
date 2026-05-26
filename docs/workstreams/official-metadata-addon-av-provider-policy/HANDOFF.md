# Handoff

Status: Complete
Current task: None
Last updated: 2026-05-26

## Context

The previous MDCx-style AV scraping lane is complete. The user agreed to keep
the Crawlee browser-worker as the browser execution layer and continue with
documentation, more AV providers, and configurable multi-source field policy.

## Decisions

- Use MDCx only as high-level behavioral reference.
- Keep Rust providers responsible for site parsing and field facts.
- Keep `addons/browser-worker` responsible for Crawlee/Playwright rendering and
  future session/wait/proxy mechanics.
- Implement field policy first as request-level behavior before committing to
  manifest/global config shape.

## Completed In This Slice

- New workstream opened.
- OMAVP-010 JSON validation passed.
- OMAVP-020 README contract documentation passed `git diff --check`.
- OMAVP-030 request-level provider field policy passed targeted tests.
- OMAVP-030 also has a default AV policy inspired by MDCx field priority:
  DMM/JavDB/FC2 provider order for metadata and poster/backdrop artwork inside
  compatible merged clusters.
- OMAVP-040 added DMM as a disabled-by-default official censored-release AV
  tracer using browser-worker rendered HTML.
- OMAVP-050 closeout gates passed.

## Validation

- `python -m json.tool docs/workstreams/official-metadata-addon-av-provider-policy/WORKSTREAM.json`: passed.
- `git diff --check`: passed after README updates.
- `cargo nextest run -p nako-metadata-scraper field_policy resolver ranking --no-fail-fast`: 20 passed.
- `cargo nextest run -p nako-metadata-scraper provider_field_policy --no-fail-fast`: 2 passed.
- `cargo nextest run -p nako-metadata-scraper dmm --no-fail-fast`: 3 passed.
- `cargo nextest run -p nako-metadata-scraper config registry manifest --no-fail-fast`: 27 passed.
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`: 180 passed, 2 skipped.
- `cargo fmt -p nako-metadata-scraper -- --check`: passed.
- `python -m json.tool docs/workstreams/official-metadata-addon-av-provider-policy/WORKSTREAM.json`: passed.
- `python -m json.tool addons/metadata-scraper/manifest.example.json`: passed.
- `git diff --check`: passed.

## Follow-Ups

1. Add aggregator fallback providers such as JavBus/JavLibrary for missing
   overview, tags, actor, and artwork fields.
2. Add an FC2 fallback provider such as FC2PPVDB if FC2 coverage needs another
   route-specific source.
3. Promote `provider_field_policy` to manifest/global config once the request
   contract proves stable in real operator workflows.
4. Expand browser-worker session/wait/proxy controls only when a real provider
   needs more than the current `/render` contract.
