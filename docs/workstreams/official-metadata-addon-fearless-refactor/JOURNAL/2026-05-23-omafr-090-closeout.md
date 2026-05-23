# OMAFR-090 — Closeout

Date: 2026-05-23
Status: DONE

## Closeout Decision

Close this architecture lane. The core target state is satisfied:

- configuration and manifest truth are unified;
- provider construction and diagnostics live behind `ProviderRegistry`;
- metadata orchestration lives behind `MetadataScrapeRuntime`;
- outbound provider HTTP behavior lives behind `ProviderHttpRuntime`;
- TMDB movie search proves the first real provider path without becoming a
  broad provider matrix;
- local direct sidecar smoke is scripted and verified;
- docs/examples match runtime truth.

The full Nako Admin-mediated smoke is a follow-on evidence item because it
requires an external running Nako server plus administrator token.

## Final Gates

```text
cargo fmt --all -- --check
```

Result: pass.

```text
pwsh -NoProfile -Command '$null = [scriptblock]::Create((Get-Content -Raw -Path "addons\metadata-scraper\smoke.local.ps1")); "syntax-ok"'
```

Result: pass.

```text
python -m json.tool docs\workstreams\official-metadata-addon-fearless-refactor\WORKSTREAM.json | Out-Null
```

Result: pass.

```text
cargo nextest run -p nako-metadata-scraper --no-fail-fast
```

Result: pass. 27 tests run, 27 passed.

```text
cargo nextest run --workspace --no-fail-fast
```

Result: pass. 27 tests run, 27 passed.

```text
git diff --check
```

Result: pass with a Cargo.lock line-ending warning.

## Follow-Ons

- Run full Nako Admin-mediated smoke with a live local Nako server and admin
  token.
- Expand TMDB beyond bounded movie search.
- Add Bangumi/Douban provider adapters.
- Add artwork and subtitle provider lanes.
- Add rename planning and NFO-compatible sidecar workflows.
- Add bulk scrape, provider scoring, and ranking hardening.
