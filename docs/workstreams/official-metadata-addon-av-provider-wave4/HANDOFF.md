# Handoff

Status: Complete
Last updated: 2026-05-26

Current task: None

## Context

The user wants MDCx-style mature AV scraping breadth and accepts breaking
internal refactors for clean architecture. Previous lanes already added parser
hardening, official uncensored providers, provider presets, proxy/browser-worker
controls, and redaction-safe manual drift checks.

## Reference Rules

`repo-ref/mdcx` is GPLv3/reference-only. Use it only for high-level provider
coverage and strategy comparison. Do not copy source, selectors, regex tables,
comments, fixtures, or structure.

## Closeout

Wave 4 is complete. AirAV, AVSox, and XCity are implemented through a shared
rendered-search provider base; config, manifest, presets, field policy,
diagnostics, external-id aliases, drift support, docs, and evidence are updated.
Full package, fmt, JSON, and diff hygiene gates passed on 2026-05-26.
