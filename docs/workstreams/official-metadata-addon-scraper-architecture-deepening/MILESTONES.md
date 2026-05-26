# Milestones

Status: Active
Last updated: 2026-05-26

## M1 - Lane Opened

Exit criteria:

- Workstream docs exist and are JSON-valid.
- Task ledger covers all six architecture review candidates.
- Breaking-change policy is explicit.

## M2 - Typed Scrape Outcome (Complete)

Exit criteria:

- Runtime produces an internal typed outcome for one scrape.
- Response rendering is a projection from typed outcome.
- Bulk no longer reads provider execution, AV facts, or failure classes from
  public response JSON.

## M3 - Render Intent And Rendered AV Flow

Exit criteria:

- RenderedPageRuntime accepts provider-owned render intent.
- Browser-worker contract tests cover wait/proxy/session payloads.
- JavBus/JavLibrary/MGStage share rendered AV flow for direct lookup, route
  gating, search-to-detail, and empty/failure behavior.

## M4 - Provider Quality And Candidate Fusion

Exit criteria:

- Provider descriptors contribute default AV field quality/profile facts.
- Engine policy executes descriptor-derived defaults without provider identity
  lists in query parsing.
- Resolver, fusion, ranking, artwork, and native writeback projection have
  narrower Interfaces with direct tests.

## M5 - Side Effect Writeback Consolidated

Exit criteria:

- Metadata and artwork writeback use one shared state machine.
- Type-specific adapters own selection, payload, provenance, and summary shape.
- Runtime writeback tests prove old state paths through the shared Module.

## M6 - Verification And Closeout

Exit criteria:

- `cargo nextest run -p nako-metadata-scraper --no-fail-fast` passes.
- `npm --prefix addons/browser-worker test` passes if browser-worker changed.
- Workstream docs describe shipped behavior and split follow-ups.
- Worktree has only intended changes staged/committed.
