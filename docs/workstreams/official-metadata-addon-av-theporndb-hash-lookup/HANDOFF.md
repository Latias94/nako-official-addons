# Handoff

Status: Complete
Last updated: 2026-05-26

Current task: None.

Completed:

1. Added `file_oshash` and `file_phash` provider external-id capabilities.
2. Added ThePornDB `/scenes/hash/{hash}` direct lookup before ID/AV/title search.
3. Mapped returned hash rows into candidate external IDs.
4. Updated README docs and verification evidence.

Guardrails:

- Do not print raw hash values in diagnostics or workstream evidence.
- Do not implement movie hash lookup until scene/movie intent is explicit.
- Do not add local file hashing to this provider slice.

Remaining follow-ups:

- Add a provider-neutral `MetadataQuery.file_hashes` value object for multi-hash arrays.
- Add ThePornDB movie hash lookup once scene/movie intent is explicit.
