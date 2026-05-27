# Official Resource Search Addon Foundation

## Problem

Nako has addon support for metadata scraping, renderer adapters, notifications,
and host-side acquisition intake. It does not yet have an official addon-side
resource discovery capability for searching external share links, magnet links,
or downloadable sources and handing useful candidates back to Nako.

The local `repo-ref/pansou` project is a good reference for the problem shape:
it proves that resource search is mostly a source aggregation, link
classification, result fusion, and availability-checking problem. That model is
different from metadata scraping. A metadata provider answers "what is this
media item?" Resource search answers "where can an operator find candidate
resources for this query?"

## Target State

- Add a dedicated `nako-resource-search` official addon sidecar.
- Keep resource search out of `nako-metadata-scraper`.
- Model resource search as a first-class addon domain with typed requests,
  typed results, provider execution diagnostics, link classification, result
  fusion, and later acquisition handoff.
- Make the first slice fully testable without live third-party sites.
- Record the missing Nako core protocol contract for tomorrow instead of
  forcing a premature core change in this lane.

## PanSou Reference Takeaways

Use these ideas, not the implementation:

- Source adapters should be independent and ranked.
- Fast partial results and slower background completion are different runtime
  paths.
- Links need a normalized type taxonomy before they can be grouped or checked.
- Result fusion should deduplicate by normalized URL while retaining source
  provenance.
- Link health checking is a separate concern from searching.
- External downloader integration should be a later hook, not part of the first
  search contract.

## Scope

- Open the durable workstream for resource search.
- Add a new official addon crate with manifest, health, search, diagnostics,
  and deterministic fixture provider paths.
- Define internal domain types for search request, result, link, provider
  execution, and grouped links.
- Add link classification for common cloud drive and peer-to-peer schemes.
- Add result fusion and package-focused tests.
- Document the deferred Nako protocol surface needed for a proper
  `resource_search` addon resource and acquisition-candidate handoff.

## Non-goals

- Changing `../nako` protocol code in this lane.
- Copying PanSou code or its provider implementations.
- Building live scraping providers before the local contract is proven.
- Starting downloads from search results.
- Making Nako trust arbitrary external URLs without explicit host policy.
- Replacing metadata scraping, catalog browsing, or acquisition intake.

## Architecture Direction

The sidecar should use the same outer shape as existing official addons:

- `config`: environment parsing and provider enablement.
- `manifest`: HTTP Addon manifest generation.
- `domain`: request, result, link, and provider execution DTOs.
- `engine`: search orchestration and result fusion.
- `links`: URL normalization and link type classification.
- `providers`: source adapters, starting with deterministic fixtures.
- `routes`: HTTP addon protocol boundary and diagnostics.

The long-term Nako-facing boundary should be:

- A new addon resource kind such as `resource_search`.
- A read/discovery scope such as `acquisition_search_read`.
- Optional task support for background refresh and link checking.
- Runtime handoff into Nako acquisition intake using the existing candidate
  model once the host grants that path explicitly.

Until the Nako protocol has that surface, the official addon implementation
must keep the search contract alpha-local and avoid pretending that catalog or
metadata resources are the correct semantic fit.

## Decisions

- Create a dedicated sidecar instead of adding resource search to
  `nako-metadata-scraper`; this preserves provider boundaries and avoids
  mixing metadata quality policy with acquisition discovery.
- Use deterministic fixture search first; live providers can be added only
  after the typed search and fusion contracts are stable.
- Treat link checking and downloader hooks as follow-on modules. They need
  different timeout, credential, proxy, and safety policies from search.
- Defer `../nako` protocol edits. This workstream records the proposed contract
  and leaves the host change for a separate Nako-side lane.
