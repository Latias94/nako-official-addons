# Official Resource Search First-Class Protocol - Closeout

Status: complete
Closed: 2026-05-28

## Outcome

The official resource-search addon now declares and serves the first-class Nako
`resource_search` contract instead of the temporary `automation` alpha contract.

Delivered:

- Manifest resource kind is `resource_search`.
- Required scope is `acquisition_search_read`.
- Request payload schema is `nako.addon.resource_search.request.v1`.
- Response payload schema is `nako.addon.resource_search.response.v1`.
- Route validation rejects non-`resource_search` envelopes.
- Route adapter maps first-class search intents into addon-owned domain
  requests and maps domain results back to protocol DTOs.
- Internal domain responses no longer carry obsolete alpha schema fields.
- Provider finality is preserved into first-class provider execution DTOs.
- Smoke manifest and docs reflect the shipped protocol.
- Follow-on contracts are documented separately from search.

## Review

Workstream compliance: no blocking findings.

Code quality: no blocking findings.

Fixed during review:

- Exact-link intent now uses the intent URL as the internal query even if the
  host `query` field contains non-URL display text. Covered by
  `decode_search_request_uses_exact_link_intent_url_as_query`.

## Gates

Passed on 2026-05-28:

```bash
cargo nextest run -p nako-resource-search resource_search --no-fail-fast
cargo nextest run -p nako-resource-search --no-fail-fast
cargo fmt --all -- --check
cargo check -p nako-resource-search --tests
git diff --check
```

`git diff --check` reported Windows line-ending warnings only.

`../nako` ADR changes also passed:

```bash
git diff --check
```

## Follow-Ons

- Admin UI remains intentionally out of scope.
- Link checking needs its own read-only probe contract.
- Downloader/external runner actions need a separate audited action contract.
- Cloud-drive save/transfer needs a separate provider-account write contract.
- Password/code references need host-owned selected-link metadata or secret
  reference handling.

Nako-side authority is recorded in
`../nako/docs/adr/0050-acquisition-resource-action-boundaries.md`.

## Residual Risk

Hosts pinned to the previous temporary automation alpha declaration need a
compatibility branch or release pin. Mainline no longer preserves that contract.
