# TODO

## ORSAH-010 - Open workstream

Status: complete
Owner: Codex
Dependencies: resource-search foundation and PanSou-compatible provider lanes

Scope:

- Record the resource search architecture review.
- Define official vs third-party ownership rules.
- Define the refactor brief and target module shape.

Validation:

- Workstream docs exist.

## ORSAH-020 - Deepen search domain and link taxonomy

Status: complete
Owner: Codex
Dependencies: ORSAH-010

Scope:

- Split query/result/link DTOs into deeper modules.
- Introduce internal `ResourceSearchIntent`.
- Move link construction out of `domain` and into `links`.
- Preserve alpha request/response wire shape.

Validation:

- Existing resource-search route tests pass unchanged.
- New tests cover query intent normalization and link construction.

## ORSAH-030 - Provider registry and source policy

Status: complete
Owner: Codex
Dependencies: ORSAH-020

Scope:

- Add provider descriptors and capability declarations.
- Add source policy classification: official, external-service, third-party.
- Move provider assembly out of `ResourceSearchRuntime::new`.
- Keep PanSou-compatible disabled by default.

Validation:

- Tests cover provider enablement, registry diagnostics, and no-network default.

## ORSAH-040 - Split orchestration from fusion

Status: pending
Owner: Codex
Dependencies: ORSAH-030

Scope:

- Move result fusion into a dedicated module.
- Keep provider execution accounting in orchestration.
- Add `ProviderSearchBatch` so providers can return warnings/finality hints
  without encoding them as global runtime errors.

Validation:

- Fusion tests move to the fusion interface.
- Provider error accounting remains redaction-safe.

## ORSAH-050 - Manifest schema composition

Status: pending
Owner: Codex
Dependencies: ORSAH-030

Scope:

- Move provider-specific manifest configuration schema fragments behind
  provider descriptors.
- Keep checked-in example manifest equal to runtime container manifest.

Validation:

- Manifest parity test passes.

## ORSAH-060 - Protocol handoff refresh

Status: pending
Owner: Codex
Dependencies: ORSAH-040

Scope:

- Update deferred Nako protocol proposal after addon-side architecture settles.
- Confirm DTO names, scopes, and acquisition handoff flow.

Validation:

- `PROTOCOL_PROPOSAL.md` reflects the final addon-side model.

## ORSAH-070 - Verify and close

Status: pending
Owner: Codex
Dependencies: ORSAH-020 ORSAH-030 ORSAH-040 ORSAH-050 ORSAH-060

Scope:

- Run focused and workspace validation.
- Update evidence and handoff.
- Commit only resource-search architecture-hardening files.

Validation:

- `cargo fmt --all -- --check`
- `cargo nextest run -p nako-resource-search --no-fail-fast`
- `cargo nextest run --workspace --no-fail-fast`
- `git diff --check -- Cargo.toml Cargo.lock README.md crates/nako-resource-search addons/resource-search docs/workstreams/official-resource-search-architecture-hardening`
