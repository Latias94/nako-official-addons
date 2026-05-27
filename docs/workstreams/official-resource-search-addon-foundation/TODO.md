# TODO

## ORSAF-010 - Open workstream

Status: complete
Owner: Codex
Dependencies: none

Scope:

- Create durable workstream docs.
- Capture the PanSou reference lessons without copying implementation details.
- Fix the first boundary decision: resource search is a dedicated official
  addon sidecar, not a metadata provider extension.
- Record that Nako core protocol changes are deferred.

Validation:

- Docs exist under this workstream path.

## ORSAF-020 - Sidecar crate and alpha search contract

Status: complete
Owner: Codex
Dependencies: ORSAF-010

Scope:

- Add `nako-resource-search` as a workspace member.
- Add manifest, config, health, search, and diagnostics routes.
- Define typed request/response DTOs for the alpha-local search contract.
- Declare the temporary manifest boundary without misrepresenting resource
  search as metadata or catalog.

Validation:

- Manifest unit test validates the runtime manifest.
- Route tests cover health, search success, empty query rejection, and
  diagnostics.

## ORSAF-030 - Search domain, link taxonomy, and fusion

Status: complete
Owner: Codex
Dependencies: ORSAF-020

Scope:

- Add `ResourceSearchProvider` and deterministic fixture provider.
- Classify cloud-drive and peer-to-peer links into stable link types.
- Normalize URLs for deduplication.
- Merge results by normalized URL while preserving source provenance and link
  grouping by type.

Validation:

- Unit tests cover link classification, URL normalization, deduplication,
  source provenance, and grouped result output.

## ORSAF-040 - Deferred Nako protocol proposal

Status: complete
Owner: Codex
Dependencies: ORSAF-020

Scope:

- Write the proposed Nako core contract for a `resource_search` addon resource.
- Define the required scope, request envelope, response envelope, and
  acquisition intake handoff.
- Keep code changes to `../nako` out of this lane.

Validation:

- Workstream docs name the exact protocol additions needed in Nako.
- `PROTOCOL_PROPOSAL.md` exists.

## ORSAF-050 - Packaging, smoke, and handoff

Status: complete
Owner: Codex
Dependencies: ORSAF-030

Scope:

- Add package manifest, Dockerfile or compose entry, and local smoke script.
- Update repository README with the new addon.
- Run focused gates and update evidence.
- Commit only resource-search workstream and addon files.

Validation:

- `cargo nextest run -p nako-resource-search --no-fail-fast`
- `cargo fmt -p nako-resource-search -- --check`
- `git diff --check -- Cargo.toml Cargo.lock README.md crates/nako-resource-search addons/resource-search docs/workstreams/official-resource-search-addon-foundation`
- `pwsh -File addons/resource-search/smoke.local.ps1`
