# Official Metadata Browser Worker - Evidence And Gates

Status: Closed
Last updated: 2026-05-23

## Smallest Current Repro

```bash
docker compose -f addons/metadata-scraper/compose.example.yml config
```

## Gate Set

### Targeted Iteration Gate

```bash
cargo nextest run -p nako-metadata-scraper --no-fail-fast
```

### Compose Gate

```bash
docker compose -f addons/metadata-scraper/compose.example.yml config
```

### Broader Closeout Gate

```bash
cargo nextest run --workspace --no-fail-fast
```

Use a narrower closeout gate when the workspace is too large, and explain why.

### Review Gate

Run `review-workstream` before accepting task or lane completion. Record blocking findings,
missing gates, and residual risks here or link to the review note.

## Evidence Anchors

- `docs/workstreams/official-metadata-browser-worker/DESIGN.md`
- `docs/workstreams/official-metadata-browser-worker/TODO.md`
- `docs/workstreams/official-metadata-browser-worker/MILESTONES.md`
- `addons/metadata-scraper/compose.example.yml`
- `addons/metadata-scraper/README.md`
- `crates/nako-metadata-scraper/src/providers`

## Notes

Record what each gate proves. Do not list commands without explaining the behavior they cover.

Fresh verification is required before marking a task, Codex goal, or lane complete.

## 2026-05-23 Verification

### OMBW-020 Worker Proof

- `npm test` in `addons/browser-worker`: PASS. Proves the worker extraction logic can load a local HTML page, wait for browser rendering, and capture DOM text changed by JavaScript.
- `npm run smoke` in `addons/browser-worker`: PASS. Proves the HTTP app exposes `/health`, serves the deterministic fixture, and returns rendered text from `/extract`.

### OMBW-030 Sidecar Integration

- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`: PASS, 52 tests passed. Proves config, manifest defaults, provider registry diagnostics, HTTP provider plumbing, and the `browser_worker` provider mapping compile and behave through Rust test seams.
- `docker compose -f addons/metadata-scraper/compose.example.yml config`: PASS. Proves Compose can resolve the two-service topology and that the sidecar receives `NAKO_METADATA_SCRAPER_BROWSER_WORKER_BASE_URL=http://nako-browser-worker:3000`.
- Direct sidecar smoke with `pwsh -File addons/metadata-scraper/smoke.local.ps1 -SidecarBaseUrl http://127.0.0.1:9100`: PASS. Proves the default public addon path still serves manifest, health, and metadata suggestions.

### Boundary And Closeout Gates

- `cargo fmt --all -- --check`: PASS. Proves Rust formatting is stable.
- `cargo nextest run --workspace --no-fail-fast`: PASS, 52 tests passed. Proves the current Rust workspace remains green with the browser-worker adapter present.
- `git diff --check`: PASS. Proves the current diff has no whitespace errors.
- `rg -n "playwright|crawlee" Cargo.toml Cargo.lock crates addons/browser-worker addons/metadata-scraper -S`: PASS for boundary review. Matches are limited to `addons/browser-worker` implementation and README wording; no Playwright/Crawlee dependency is embedded in the Rust sidecar.

### Not Yet Proven

- OMBW-040 live Douban extraction is not proven. The implemented baseline is fixture-backed and verifies the browser-worker `/render` contract plus Rust-side Douban search/detail parsing without depending on current Douban availability.

## 2026-05-23 OMBW-040 Verification

- `npm test` in `addons/browser-worker`: PASS. Proves the render path still captures rendered DOM text and now returns both `html` and `text`.
- `npm run smoke` in `addons/browser-worker`: PASS. Proves the HTTP app exposes the stable `POST /render` contract in addition to the legacy `/extract` path.
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`: PASS, 53 tests passed. Proves the `douban` provider is configured, appears in diagnostics/manifest schema, calls the browser worker `/render` endpoint, and maps recorded search/detail HTML into provider-neutral metadata/artwork facts.
- `cargo nextest run --workspace --no-fail-fast`: PASS, 53 tests passed. Proves the current Rust workspace remains green with the Douban provider and `scraper` dependency.
- `docker compose -f addons/metadata-scraper/compose.example.yml config`: PASS. Proves Compose still resolves the sidecar + worker topology and keeps Douban disabled by default.
- Direct sidecar smoke with `pwsh -File addons/metadata-scraper/smoke.local.ps1 -SidecarBaseUrl http://127.0.0.1:9100`: PASS. Proves the default fixture-enabled public addon path still serves manifest, health, metadata candidates, and artifacts after adding Douban to the runtime schema.
- `cargo fmt --all -- --check`: PASS. Proves Rust formatting is stable.
- `git diff --check`: PASS. Proves the current diff has no whitespace errors.

## 2026-05-23 OMBW-050 Closeout Verification

Review result:

- Workstream compliance: PASS with no blocking findings after root README was corrected to describe Douban as a current default-disabled baseline rather than future scope.
- Code quality: PASS with no blocking findings. The worker owns Playwright/Crawlee, the Rust sidecar calls the worker over HTTP, and provider-specific Douban parsing stays in `nako-metadata-scraper`.
- Residual risk: live Douban smoke remains unproven and is split to follow-on hardening. This closeout claims deterministic rendering and fixture-backed Douban parsing, not production live-network reliability.

Fresh gates:

- `cargo fmt --all -- --check`: PASS. Proves Rust formatting remains stable after closeout docs.
- `npm test` in `addons/browser-worker`: PASS, 1 test passed. Proves the worker still captures JavaScript-rendered DOM text and returns rendered content.
- `npm run smoke` in `addons/browser-worker`: PASS. Proves `/health`, fixture serving, `/extract`, and stable `POST /render` still work through the HTTP app.
- `docker compose -f addons/metadata-scraper/compose.example.yml config`: PASS. Proves the sidecar + browser-worker topology still resolves and keeps Douban disabled by default.
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`: PASS, 53 tests passed. Proves the package-level runtime, provider registry, manifest schema, and Douban render-contract seam remain green.
- `cargo nextest run --workspace --no-fail-fast`: PASS, 53 tests passed. Proves the Rust workspace remains green at closeout.
- `rg -n "playwright|crawlee" Cargo.toml Cargo.lock crates addons/browser-worker addons/metadata-scraper -S`: PASS for boundary review. Matches are in `addons/browser-worker` implementation/lockfiles and documentation wording; no Rust sidecar dependency embeds Playwright or Crawlee.
- `git diff --check`: PASS. Proves the closeout diff has no whitespace errors.
