# Handoff

Status: Complete
Last updated: 2026-05-26

Current task: None.

Completed:

1. Added disabled-by-default `theporndb` provider using `ProviderHttpRuntime`.
2. Added bearer token config, direct Rust proxy config, scene search, scene
   detail lookup, metadata/AV/artwork/external-id mapping, and tests.
3. Wired provider catalog/config/presets/manifest/health diagnostics/live drift
   and README documentation.

Guardrails:

- Do not copy MDCx code, comments, selectors, fixtures, regex tables, or structure.
- Do not commit live adult payloads or secrets.
- Keep token and proxy diagnostics boolean-only.

Remaining follow-ups:

- Add explicit hash input facts before using ThePornDB `/scenes/hash/{hash}` or
  `/movies/hash/{hash}` routes.
- Add movie-route provider support once the scrape contract can distinguish
  scene and movie intent.
