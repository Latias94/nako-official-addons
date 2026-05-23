# Official Metadata Browser Worker

Status: Active
Last updated: 2026-05-23

## Why This Lane Exists

Douban and similar anti-bot metadata sources require browser automation that
is heavier than the current Rust HTTP provider model. The official metadata
addon should stay thin and delegate rendered-page acquisition to a dedicated
browser worker.

## Relevant Authority

- Existing addon docs:
  - `README.md`
  - `addons/metadata-scraper/README.md`
  - `addons/metadata-scraper/compose.example.yml`
- Existing addon architecture:
  - `docs/workstreams/official-metadata-addon-fearless-refactor/DESIGN.md`
  - `docs/workstreams/official-metadata-addon-result-quality/DESIGN.md`
- Deferred crawler scope:
  - `docs/workstreams/official-metadata-bangumi-provider-baseline/DESIGN.md`
  - `docs/workstreams/official-metadata-addon-side-effect-writer/DESIGN.md`
- Related workstreams:
  - `docs/workstreams/official-metadata-addon-side-effect-writer`
  - `docs/workstreams/official-metadata-addon-result-quality`

## Problem

The metadata scraper can already fan out over HTTP providers, but that model is
not enough for sites that actively resist direct scraping. There is no worker
service contract for browser automation, no compose topology for the worker,
and no metadata-provider adapter for sites that need rendered-page extraction.

## Target State

When this lane closes:

- a dedicated browser worker service exists;
- the worker uses Playwright/Crawlee-style browser automation;
- the worker exposes a small internal HTTP API for health and extraction;
- the metadata scraper can call that worker through configuration;
- Docker Compose can deploy the metadata scraper and the worker together;
- the first proof covers a deterministic rendered-page extraction path;
- Douban becomes the first real consumer, without making the worker Douban-only.

## In Scope

- Browser worker service design and containerization.
- Internal HTTP contract for browser-backed extraction.
- Docker Compose integration for local and self-hosted deployment.
- Metadata scraper client adapter for the worker.
- Local fixture-based browser automation tests.
- First Douban-backed provider baseline once the worker exists.

## Out Of Scope

- Nako Addon Protocol changes.
- Nako core server changes.
- Requiring the browser worker to be a public user-facing addon.
- Embedding Playwright directly into the Rust addon binary.
- Broad provider breadth beyond the first browser-backed proof.
- CAPTCHA solving products or login automation beyond what the proof needs.
- Copying code from third-party scrapers or reference repositories.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Docker Compose is the right local/self-hosted orchestration layer for this lane. | High | Existing addon examples already use Compose and systemd patterns. | Split into separate deployment docs later. |
| Browser automation belongs in a dedicated service, not inside the Rust sidecar. | High | Current docs already defer Douban/browser automation to a separate lane. | A monolithic image may still work, but the lane would need to be reshaped. |
| Playwright/Crawlee is the best initial browser stack. | Medium | Current research points to the strongest scraping-oriented browser automation ecosystem. | The worker contract should still survive an implementation swap. |

## Architecture Direction

Keep `nako-metadata-scraper` as the only public Nako-facing addon. Introduce a
separate browser worker service that owns browser sessions, proxies, cookies,
timeouts, and rendered DOM extraction.

The metadata scraper should treat the worker like an internal upstream service:
it calls a small HTTP API, receives normalized fields or candidate facts, and
then continues with the existing provider/ranking flow. The worker should not
know about Nako manifests or addon registration.

This keeps the anti-bot complexity out of the official addon surface while
still letting the metadata scraper rely on browser rendering when HTTP alone is
not enough.

## Closeout Condition

This lane can close when:

- the worker service exists and is documented;
- the metadata scraper can call it through configuration;
- the compose example shows the deployment topology;
- one rendered-page extraction proof passes;
- the Douban-backed baseline is implemented or explicitly split out;
- docs reflect the shipped behavior truthfully.
