# Official Metadata Addon Bangumi Metadata Enrichment

Status: Complete
Last updated: 2026-05-26

## Why This Lane Exists

Bangumi is now a first-class provider, but its candidate model still uses a
small subset of the official subject payload. This leaves useful animation
metadata invisible to Nako users and weakens ranking/provenance for Bangumi
results.

## Relevant Authority

- ADRs: none.
- Existing docs:
  - `docs/workstreams/official-metadata-bangumi-provider-baseline/DESIGN.md`
  - `docs/workstreams/official-metadata-addon-provider-fact-resolver/DESIGN.md`
- Related workstreams:
  - `docs/workstreams/official-metadata-bangumi-provider-baseline/`
  - `docs/workstreams/official-metadata-addon-provider-fact-resolver/`
  - `docs/workstreams/official-metadata-addon-bangumi-year-air-date-filter/`

## Reference Inputs

- `repo-ref/bangumi-api`: official OpenAPI schema for subject fields and
  endpoint behavior.
- `repo-ref/bangumi-server`: official server implementation used only to
  confirm response field semantics such as `nsfw`, `eps`,
  `total_episodes`, `air_weekday`, and parsed infobox shape.
- `repo-ref/jellyfin-plugin-bangumi`: GPL-2.0 media-library provider used only
  for high-level behavior comparison. Do not copy implementation, structure, or
  code expressions from this repository.

## Problem

Current Bangumi mapping already reads title, summary, date, platform, images,
rating, aliases, meta tags, tags, and episode counters. It does not read or
normalize several official subject facts that are valuable in Nako:

- `nsfw`, `locked`, `volumes`, and `air_weekday` are ignored.
- Infobox values are only used for alternate titles; official website,
  broadcast end date, and production/studio/staff-like values are dropped.
- Bangumi tag handling mixes meta tags and arbitrary popular tags without a
  provider-owned taxonomy boundary.
- The code has no durable evidence comparing our choices against official
  Bangumi schema and a mature media-library provider.

## Target State

- Bangumi subject parsing accepts the official enrichment fields without
  making them required.
- Candidate mapping emits richer, deterministic Bangumi tags for official
  subject facts and safe infobox-derived facts.
- User-facing patch fields remain protocol-compatible and only use existing
  `AddonMetadataPatch` fields.
- Provenance remains visible through provider tags/outcomes without leaking
  secrets or raw request details.
- Tests cover malformed/blank infobox values, subject fact tags, degraded
  fallback, and existing direct lookup/search behavior.
- Reference findings are documented with clear license boundaries.

## In Scope

- Rust code under `crates/nako-metadata-scraper/src/providers/bangumi*`.
- Synthetic fake-transport tests for Bangumi provider behavior.
- Workstream docs and README notes if behavior visible to users changes.

## Out Of Scope

- Copying GPL-2.0 reference code.
- Adding new `AddonMetadataPatch` fields or changing Nako protocol.
- Episode, cast/person, relation, or collection synchronization.
- Live Bangumi network tests in default gates.
- Cache/archive systems.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Official v0 subject fields are the correct source of truth. | High | `repo-ref/bangumi-api/open-api/v0.yaml`, `repo-ref/bangumi-server/web/res/subject.go` | Mapping may need to change if Bangumi API schema changes. |
| Existing protocol should not be expanded for this lane. | High | Current `nako-addon-protocol::AddonMetadataPatch` only supports generic metadata patch fields. | Rich facts must stay in tags/facts until a protocol ADR exists. |
| GPL reference can be used for behavior comparison only. | High | `repo-ref/jellyfin-plugin-bangumi/LICENSE` is GPL-2.0. | Any copied implementation would contaminate this AGPL project and must be avoided. |
| Tags are the least disruptive place for extra provider facts. | Medium | Existing TMDB/Douban/Bangumi mappers already emit provider-prefixed tags. | If tags become too noisy, a future protocol/facts field may be needed. |

## Architecture Direction

Provider-specific parsing and normalization stays inside `providers::bangumi`.
The shared engine continues to own ranking, resolver clustering, response
shape, and writeback. Bangumi should emit more complete facts, but it should
not bypass provider-neutral scoring or introduce provider-specific response
schema.

The mapper will use small local helpers for infobox extraction and
provider-prefixed tags. Tags must be deterministic, trimmed, deduplicated, and
bounded. Infobox keys are treated as optional hints because Bangumi data is
community-authored and structurally flexible.

## Closeout Condition

This lane can close when:

- the target state is implemented,
- evidence gates pass,
- docs reflect the shipped behavior,
- and follow-on work is either split or explicitly deferred.

Closeout result: complete on 2026-05-26. Cast/person, episode, relation, and
protocol expansion work remain explicitly out of scope and should be split into
separate lanes if needed.
