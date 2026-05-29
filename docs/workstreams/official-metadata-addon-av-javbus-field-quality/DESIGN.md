# Design

## Problem

JavBus is reachable through the browser-worker proxy path, but the first live crawl showed weak field coverage: a candidate can be found, while release date, runtime, actors, artwork, and other AV facts may be absent. The user wants MDCx-level maturity, especially for provider strategy and batch scraping quality.

## Target State

- JavBus detail parsing should extract the fields that JavBus is expected to own well: title, AV number, release date, runtime, actors, genres, studio, publisher, director, series, primary image, and sample images.
- Search behavior should tolerate JavBus returning a detail page directly from a search URL.
- Live drift evidence should stay redaction-safe while proving field coverage against browser-rendered pages.
- MDCx remains a behavioral reference only. This workstream must not copy GPL source, comments, selectors, fixtures, or module structure.

## MDCx Reference Findings

- JavBus is treated as a strong source for actors, thumbnails/posters, extra fanart, tags, release dates, runtime, director, series, studio, and publisher.
- The mature route strategy tries direct detail URLs for ordinary censored IDs and falls back to typed searches for censored, uncensored, or western-style identifiers.
- Batch maturity is mostly orchestration: bounded concurrency, delays, resumable failed items, provider field preferences, and side-effect handling. Nako already owns these concepts in `bulk-metadata-scrape`, so this lane should improve provider facts rather than add a parallel batch executor.
- Proxy and cookie support are operational requirements. Proxy is handled by browser-worker; JavBus can pass an optional operator cookie to browser-worker as a page request header when live pages demand age or region access.

## Scope

- Harden `crates/nako-metadata-scraper/src/providers/javbus.rs`.
- Update redaction-safe tests and workstream evidence.
- Run targeted and package validation before closeout.

## Non-Goals

- No direct MDCx code reuse.
- No new batch scheduler.
- No raw local file names, raw AV numbers from the user's disk, cookies, or proxy URLs in committed fixtures.

## Architecture Direction

Keep JavBus as a browser-worker-backed `RenderedAvFlow` provider. Put page-shape tolerance, access-gate rejection, and field extraction in the provider parser, while shared retry, route filtering, merge policy, and bulk accounting stay in the existing engine. Browser-worker owns only page mechanics: proxy, waits, headers, optional page actions, and rendered HTML capture.
