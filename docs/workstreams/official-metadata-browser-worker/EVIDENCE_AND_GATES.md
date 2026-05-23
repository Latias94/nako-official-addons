# Official Metadata Browser Worker - Evidence And Gates

Status: Active
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
