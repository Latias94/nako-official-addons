# Official Metadata Addon AV Provider Wave 3

Status: Complete
Last updated: 2026-05-26

## Why This Lane Exists

The metadata scraper now has the architecture needed for broader AV provider
coverage: rendered AV flow, typed render intent, descriptor-derived field
quality, resolver/fusion boundaries, native writeback projection, and a shared
side-effect writeback state machine. The next bottleneck is no longer the
engine shape. It is provider breadth, repeatable provider tests, and protection
against real-site operational pain.

This lane covers three connected deliverables:

- add a reusable rendered AV provider fixture/drift harness;
- add AV provider wave 3 behind disabled-by-default config;
- add explicit real-use protection such as provider budgets, cache, cooldown,
  and render-session policy where it can be tested safely.

## Relevant Authority

- Closed scraper architecture lane:
  - `docs/workstreams/official-metadata-addon-scraper-architecture-deepening/`
- Closed AV native writeback/provider wave 2 lane:
  - `docs/workstreams/official-metadata-addon-av-native-writeback-and-provider-wave2/`
- Closed AV MDCx parity lane:
  - `docs/workstreams/official-metadata-addon-av-mdcx-parity/`
- Reference-only MDCx checkout:
  - `repo-ref/mdcx/`

## Domain Vocabulary

- Provider Wave 3: the next batch of disabled-by-default AV providers after
  JavBus, JavLibrary, and MGStage.
- Fixture Harness: shared test helpers and fixture contracts for rendered AV
  provider parser/mapper behavior.
- Drift Harness: optional manual tooling that detects selector or layout drift
  without storing adult-site payloads in CI.
- Real-Use Protection: explicit limits, cache, cooldown, and render session
  policy that prevent provider fan-out from becoming noisy or fragile.

## Problem

Adding more AV providers directly to the current modules would work, but it
would recreate copy-heavy tests and repeat operational decisions. The prior
architecture work made providers easier to add; this lane should prove that by
adding providers while also making future providers cheaper and safer.

The system also needs a clearer policy for real usage. Bulk has explicit
provider suppression state, but single-scrape provider execution still has no
shared provider budget/cache boundary. Browser-worker proxy and session
support exists, but providers do not yet have a provider-level policy story for
how aggressive rendered scraping should be.

## Target State

When this lane closes:

- rendered AV providers share a fixture harness that verifies search/direct
  lookup, parser mapping, external IDs, AV facts, artwork facts, and field
  quality behavior;
- wave 3 providers are registered, disabled by default, documented, covered by
  synthetic rendered-HTML tests, and represented in default field-quality
  descriptors;
- provider execution can apply explicit real-use protections without hidden
  scheduler state;
- browser-worker proxy/session/wait policy remains the browser boundary, but
  provider-facing config can express conservative defaults where needed;
- no source or fixture material is copied from GPL reference projects.

## Provider Candidates

Priority order:

1. Prestige as an official censored-release provider.
2. FC2 long-tail sources such as FC2PPVDB, FC2Hub, or FC2Club.
3. Caribbeancom, 1Pondo, and 10Musume as official uncensored sources.
4. ThePornDB or Jav321 only after provider harness and policy are stable.

The exact provider sequence can change if a target site requires credentials,
is unstable, or cannot be tested with independently authored synthetic HTML.

## In Scope

- Rust provider modules, config, registry, manifest, README, and tests.
- Shared test fixture helpers for rendered AV providers.
- Provider execution budget/cache/cooldown state that is explicit in request,
  config, or task output.
- Browser-worker contract updates only when required by provider policy.
- Documentation of provider IDs, env vars, aliases, and operational behavior.

## Out Of Scope

- Copying MDCx code, selectors, regex tables, comments, or fixtures.
- Live scraping against adult websites in CI.
- Nako core refresh mode, locked fields, local metadata priority, local artwork
  priority, NFO/rename workflows, or actor image workflows.
- User-facing review UI.
- Release packaging.

## Architecture Direction

Do not add provider wave 3 by copy-pasting the existing providers. Add a small
test harness first, then require each provider to pass the same fixture contract
before registry/manifest wiring. Provider-local modules should still own URL
construction, parser details, and mapping quirks. Shared modules should own
rendered AV flow, redaction-safe execution policy, and reusable test mechanics.

Real-use protection should remain explicit. A hidden global scheduler would be
hard to reason about in an addon sidecar. Prefer request/config/task-visible
budgets, bounded cache entries, and cooldown facts that can be tested and
reported.

## Closeout Condition

This lane can close when:

- fixture harness and drift plan are documented and tested;
- selected wave 3 providers are disabled by default and covered by synthetic
  tests;
- provider protection behavior is explicit and tested;
- full package, browser-worker, format, JSON, and diff hygiene gates pass;
- remaining provider breadth is split into follow-up candidates.

## Closeout Result

Complete on 2026-05-26. The lane shipped the shared rendered AV fixture harness,
explicit provider execution protection, Prestige, FC2PPVDB, Caribbean, 1Pondo,
and 10Musume. All selected providers are disabled by default, documented in the
manifest/README surface, covered by synthetic tests, and represented in route
support plus field-quality descriptors. Remaining provider breadth is deferred
to follow-up candidates rather than blocking this lane.
