# Official Addons Cross-Repo Fearless Refactor - Design

Status: Complete
Last updated: 2026-05-24

## Problem

The official metadata Addon Sidecar is now useful enough that its next risks are
cross-repo contract drift and provider-adapter scale, not missing first
features.

Recent review found three concrete problems:

1. Protected-write client logic is duplicated in the official addon. The
   sidecar owns a private Nako runtime client, permissions, side-effect request
   DTOs, token-placement guard, and safe-error mapping in
   `crates/nako-metadata-scraper/src/nako_runtime.rs`. That was a good first
   slice, but it now overlaps with `../nako` public Addon Protocol/client
   responsibilities and will drift as `metadata_write`, `artwork_write`,
   `library_file_write`, and future official addons grow.
2. Provider adapters have become too broad. TMDB has already been split into
   provider-local client/search/parser/mapper/enrichment/test-support modules,
   but Bangumi remains a large mixed adapter and Douban still mixes rendered
   page dispatch, HTML parsing, search result extraction, detail mapping, and
   candidate shaping.
3. The official release smoke proves registration, hosted health, and metadata
   resource calls, but does not prove host-dispatched Addon Task path execution.
   Nako now has a host-owned Addon Task runtime, and the metadata sidecar
   declares `bulk-metadata-scrape`; that contract needs a published smoke gate.

## Target State

- Official addon protected-write calls use a public Nako addon client/protocol
  interface where appropriate, while preserving sidecar-owned transport,
  timeout, redaction, and fake-transport testability.
- Bangumi and Douban provider adapters are split into deep provider-local
  modules with unchanged public payloads and focused provider tests.
- The official Nako + official metadata addon smoke covers Addon Task path
  dispatch for `bulk-metadata-scrape`, including safe result/diagnostic
  assertions and no sidecar process-supervision assumption.

## Scope

- `crates/nako-metadata-scraper/src/nako_runtime.rs`
- `crates/nako-metadata-scraper/src/providers/bangumi.rs`
- `crates/nako-metadata-scraper/src/providers/douban.rs`
- provider-local module directories under `crates/nako-metadata-scraper/src/providers/`
- metadata scraper smoke/docs under `addons/metadata-scraper/` and root docs
- cross-repo alignment with `../nako` public addon client/protocol and
  official smoke scripts, coordinated carefully because `../nako` may have an
  active dirty worktree and active addon credential workstreams

## Non-Goals

- Copying, porting, translating, or deriving implementation code from
  `repo-ref/nako-scraper`.
- Adding a new provider solely to increase provider count.
- Turning `nako-metadata-scraper` into a general media organizer.
- Moving reqwest-heavy runtime behavior into the permissive protocol crate
  without an explicit ADR.
- Changing public Addon Protocol compatibility unless a task proves it is
  necessary and records the versioning impact.
- Making Nako install, start, stop, or supervise the addon sidecar process.
- Implementing new subtitle, NFO, artwork, AI, package signing, marketplace, or
  Addon Manager lifecycle features in this lane.

## Architecture Direction

The protected-write work should deepen the host/client seam instead of
duplicating private DTOs in every official addon. The desired direction is a
small public client module that owns authorization header placement,
side-effect request/response typing, safe error mapping, and version-tolerant
parsing, while each sidecar keeps its own runtime policy and transport adapter.

The provider work should follow the existing TMDB split. A provider module
should expose one small adapter type satisfying `MetadataProvider`; provider
HTTP, search planning, parser structs, mapping, enrichment, and test support
should be local modules behind that adapter.

The task smoke work should prove the Nako-owned Addon Task runtime calls the
declared sidecar task path and receives a bounded result. The addon remains an
external process; smoke orchestration may start local processes for evidence,
but product semantics stay outside process supervision.

## Reference Notes

`F:/SourceCodes/Rust/repo-ref/nako-scraper` is useful only as product and
responsibility inspiration. It shows mature scraper products need batch work,
review flows, NFO handling, subtitles, artwork, actor/tag tooling, and network
diagnostics. In Nako those concerns must be split across Addon Task, Addon Side
Effect, Candidate/Acceptance, Library File Write, and Managed Artwork seams.

## Closeout Result

The three selected refactors are complete:

- protected-write request/response DTOs and runtime submission helpers now live
  in public Nako addon protocol/client crates, and the official metadata addon
  uses a thin local facade instead of a private duplicate client;
- Bangumi and Douban now follow provider-local facade plus
  client/enrichment/parser/mapper/test-support module boundaries;
- official smoke scripts now expose a `RunTaskPath` path that creates and waits
  for a host-dispatched `bulk-metadata-scrape` task run.

The live Docker/server smoke was not executed in this closeout because this
session did not own a running Nako server/admin token. The task path behavior is
covered by script parser checks, addon route/task tests, and Nako server direct
dispatch integration tests.
