# Handoff

Status: Complete
Current task: None
Last updated: 2026-05-27

## Context

The user approved continuing AV scraping work and explicitly allowed referencing `repo-ref/mdcx`. The prior MDCx-style AV scraping workstream is complete, so this lane focuses on JavBus field quality and redaction-safe live evidence.

## Current Findings

- Live JavBus access needs the browser-worker proxy path in this environment and currently lands on an age-verification flow without an operator cookie.
- Search URLs can render as detail pages directly, which was fixed in commit `56d45e1`.
- JavBus now prefers direct detail lookup before search, rejects age-verification pages as non-candidates, and can pass `NAKO_METADATA_SCRAPER_JAVBUS_COOKIE` as a browser-worker page Cookie header.

## Next Step

Follow-up work should be a new lane: cookie-backed JavBus live drift cases, uncensored/western route live coverage, or configurable MDCx-style per-field provider preferences.
