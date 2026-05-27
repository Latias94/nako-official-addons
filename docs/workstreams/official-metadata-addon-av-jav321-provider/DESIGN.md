# Official Metadata Addon AV Jav321 Provider

Status: Active
Last updated: 2026-05-27

## Why This Lane Exists

The AV scraper has broad coverage, field fusion presets, and browser-rendered
providers, but it still lacks Jav321. Jav321 is a useful fallback for title,
outline, score, actors, release date, runtime, studio/publisher, series, tags,
cover/poster, and extra fanart. Its search flow is form POST returning an HTML
detail page, so field parity needs a tested parser contract instead of a generic
best-effort rendered-page lookup.

## Relevant Authority

- Reference-only upstream:
  - `repo-ref/mdcx/mdcx/crawlers/jav321.py`
  - `repo-ref/mdcx/mdcx/config/v1.py`
- Related workstreams:
  - `docs/workstreams/official-metadata-addon-av-field-policy-presets/`
  - `docs/workstreams/official-metadata-addon-av-javbus-field-quality/`
- License guardrail:
  - `repo-ref/mdcx` is GPLv3/reference-only; do not copy source, comments,
    fixtures, selector implementations, regex tables, or structure.

## Problem

Adding Jav321 without pinning its field contract would likely produce a provider
that returns candidates but misses the exact AV facts operators expect.

## Target State

- Jav321 is a disabled-by-default AV provider.
- It supports `jav321_id`, `jav321_url`, and `av_number` lookup.
- It posts `sn=<av-number>` to `/search` and parses the returned detail HTML.
- Parser tests cover title, outline, actors, release date, runtime, score,
  studio/publisher, series, tags, cover/poster, extra fanart, source URL, and
  external IDs.
- HTTP runtime has bounded form/text support reused by Jav321.
- Config, manifest, diagnostics, README, and default field policy know Jav321.

## In Scope

- Add raw text/form support to `ProviderHttpRuntime`.
- Add a Jav321 provider module and catalog/config wiring.
- Add synthetic HTML fixture tests based on observed field shapes, not copied
  upstream source.
- Update docs and workstream evidence.

## Out Of Scope

- Live Jav321 proof if the site blocks the current environment.
- Adding new batch scheduler behavior.
- Copying upstream implementation text or fixtures.

## Architecture Direction

Use HTTP runtime for Jav321 because the search flow is a form POST that returns
HTML. Browser-worker remains for JavaScript-rendered GET/detail flows. The parser
should be provider-owned but reuse generic AV helpers where they fit.

## Closeout Condition

This lane can close when Jav321 is wired, parser coverage proves the field
contract, package tests pass, docs are updated, and any live-network limitation
is recorded explicitly.
