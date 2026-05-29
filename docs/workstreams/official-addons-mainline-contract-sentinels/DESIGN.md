# Official Addons Mainline Contract Sentinels

Status: Complete
Last updated: 2026-05-27

## Why This Lane Exists

The main `nako` repository is still moving quickly around casting, playback
capability planning, and transcode readiness. The official addon repository
depends on local path SDK crates from `../nako`, so clean CI, Docker builds,
and publish dry-runs must exercise the same cross-repo shape that developers
use locally.

## Relevant Authority

- Main repository context: `../nako/CONTEXT.md`
- Main repository workstreams:
  - `../nako/docs/workstreams/external-casting-adapter-boundary/`
  - `../nako/docs/workstreams/casting-renderer-runtime/`
  - `../nako/docs/workstreams/playback-capability-profile-planner/`
  - `../nako/docs/workstreams/cpu-transcode-readiness/`
- Addon release readiness:
  - `docs/workstreams/official-addons-v0-1-0-alpha-2-release-readiness/`
- Related completed addon lanes:
  - `docs/workstreams/official-addons-cross-repo-fearless-refactor/`
  - `docs/workstreams/official-chromecast-renderer-adapter/`

## Problem

The repository has three official Rust sidecars, but some automation still
assumes a metadata-scraper-only world. The release gate also checks out only
`nako-official-addons` even though the workspace has `../nako` path
dependencies, and the container Dockerfiles copy only the addon repository into
the image build context. That shape can pass local developer workflows while
failing or silently drifting in clean CI.

There is also one remaining catalog ownership inconsistency: metadata scraper
and Chromecast renderer derive their manifest facts from
`nako-official-addon-catalog`, while notification bridge still duplicates most
official manifest facts locally.

## Target State

- CI checks out the main `nako` repo as a sibling before Cargo commands that
  require local SDK path dependencies.
- Release gates build and smoke all current official sidecars:
  `nako-metadata-scraper`, `nako-notification-bridge`, and
  `nako-chromecast-renderer`.
- Container Dockerfiles have an explicit cross-repo build-context contract
  instead of relying on an unavailable `../nako` path inside Docker.
- Publish dry-run automation tracks every publishable official addon crate.
- Addon runtime manifest facts are delegated to the shared official catalog
  whenever that catalog already owns the official facts.

## In Scope

- `.github/workflows/release-gate.yml`
- `.github/workflows/crates-publish.yml`
- `addons/*/Dockerfile`
- `crates/nako-notification-bridge/src/manifest.rs`
- Workstream evidence and handoff docs

## Out Of Scope

- Publishing crates or pushing images.
- Editing main `../nako` source files.
- Changing the Addon Protocol wire contract.
- Adding live hardware Chromecast CI.
- Reworking provider internals unrelated to cross-repo drift detection.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Clean CI must have a sibling `nako` checkout for path dependencies. | High | `Cargo.toml` depends on `../nako/crates/*`. | CI fails before tests, or developers rely on local-only state. |
| Docker image builds need explicit access to both repositories until SDK crates are fully published and consumed from crates.io. | High | Current Dockerfiles copy only `.` and Cargo resolves `../nako`. | Container release gates fail after checkout isolation. |
| Notification bridge can delegate manifest facts to `nako-official-addon-catalog`. | High | The catalog already exposes a matching `notification_bridge` module. | Local duplicate constants continue drifting from main catalog facts. |
| Main casting/transcode work does not require addon source changes yet. | Medium | Recent main commits expose renderer-adapter/capability planning contracts, while addon Chromecast sidecar already implements the adapter proof. | This lane may need a follow-on if main adds a new required addon resource or manifest field. |

## Architecture Direction

Treat `../nako` as the source of shared protocol and official catalog facts, and
treat this repository as the source of sidecar runtime behavior, packaging, and
operational smoke. The boundary is intentionally asymmetric:

- protocol types and official catalog facts live in the main repository;
- sidecar execution, Docker packaging, and local smoke live here;
- CI must make that split explicit by checking out both repositories and by
  building containers from this repository plus a BuildKit `nako=../nako` named
  context.

This keeps addon development fast while making drift visible at the cheapest
possible layer.

## Closeout Condition

This lane can close when:

- the first cross-repo CI and Docker gate hardening lands,
- notification bridge no longer duplicates shared official manifest facts,
- publish dry-runs list all current official addon crates,
- fresh local Cargo evidence passes,
- Docker evidence is either passing or blocked by a concrete environment issue,
- and remaining mainline drift risks are recorded as follow-ons.

Closed 2026-05-27 with Docker image build/smoke blocked by the local Docker
daemon being unavailable, not by a code failure.
