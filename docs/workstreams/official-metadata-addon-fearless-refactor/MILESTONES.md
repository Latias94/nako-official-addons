# Official Metadata Addon Fearless Refactor — Milestones

Status: Complete
Last updated: 2026-05-23

## M0 — Scope And Evidence Freeze

Exit criteria:

- Workstream docs exist and agree.
- Reference repository policy is explicit.
- First executable task is selected.

Primary evidence:

- `docs/workstreams/official-metadata-addon-fearless-refactor/DESIGN.md`
- `docs/workstreams/official-metadata-addon-fearless-refactor/TODO.md`

## M1 — Configuration And Manifest Truth

Exit criteria:

- Runtime configuration and Addon Manifest declarations share one provider
  model.
- Secret Reference field declarations match actual provider needs.
- Example manifest, Docker Compose, and README do not claim unsupported
  providers.

Primary gates:

- `cargo nextest run -p nako-metadata-scraper config manifest --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`

## M2 — Provider Registry And Diagnostics

Exit criteria:

- Provider construction moves out of `default_providers()`.
- Registry reports capabilities and availability without secrets.
- Fixture provider is an adapter behind the registry.

Primary gates:

- `cargo nextest run -p nako-metadata-scraper provider --no-fail-fast`

## M3 — Metadata Scrape Runtime

Exit criteria:

- HTTP routes are thin adapters.
- Runtime owns request normalization, provider fan-out, ranking, failure
  classification, artifacts, and response shaping.
- Existing fixture metadata behavior remains covered through the new seam.

Primary gates:

- `cargo nextest run -p nako-metadata-scraper metadata --no-fail-fast`

## M4 — Provider HTTP Runtime

Exit criteria:

- Provider adapters share one outbound HTTP runtime.
- Timeout, retry, proxy, user-agent, response-size, and redaction behavior are
  testable without live network calls.

Primary gates:

- `cargo nextest run -p nako-metadata-scraper http_runtime --no-fail-fast`

## M5 — First Real Provider Proof

Exit criteria:

- One provider adapter proves the runtime shape, preferably TMDB.
- Missing credentials and disabled provider states are diagnosable.
- Tests do not require live network by default.

Primary gates:

- `cargo nextest run -p nako-metadata-scraper tmdb --no-fail-fast`

## M6 — Nako End-To-End Smoke

Exit criteria:

- A local operator can start the sidecar and exercise it through Nako Admin
  Addon onboarding/health/resource-call flow.
- Smoke docs keep Nako process lifecycle responsibilities separate from Addon
  Sidecar operation.

Primary evidence:

- `addons/metadata-scraper/README.md`
- Optional smoke script or recorded manual smoke commands.

## M7 — Docs, Examples, And Deletion Sweep

Exit criteria:

- README, addon README, example manifest, Dockerfile, compose, and systemd
  examples match runtime truth.
- Obsolete shallow helpers and stale docs are deleted.
- No copied reference repository code, schemas, fixtures, generated files, or
  comments are introduced.

Primary gates:

- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`

## M8 — Closeout Or Follow-On Split

Exit criteria:

- Final gate evidence is fresh.
- Remaining provider/product breadth is complete or split into named
  follow-ons.
- `WORKSTREAM.json` and `HANDOFF.md` reflect the final state.

Primary gates:

- `cargo nextest run --workspace --no-fail-fast`
- `git diff --check`

Closeout result:

- Final gates passed on 2026-05-23.
- Remaining provider/product breadth is split into follow-ons:
  - Nako Admin-mediated local smoke with a live Nako server and admin token;
  - TMDB provider expansion beyond movie search;
  - Bangumi/Douban provider adapters;
  - artwork/subtitle provider lanes;
  - rename planning and NFO-compatible sidecar workflows;
  - bulk scrape and provider scoring/ranking hardening.
