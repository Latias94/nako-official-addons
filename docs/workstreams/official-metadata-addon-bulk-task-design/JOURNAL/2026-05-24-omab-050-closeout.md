# 2026-05-24 - OMAB-050 Closeout

## Summary

The bulk task design lane is closed for the current official metadata addon release. Bulk Metadata
Scrape remains a valid future feature, but implementation is blocked on the Nako host owning a
generic Addon Task scheduler/invoker.

## Decisions

- Keep the official metadata addon manifest task-free for this release.
- Defer `bulk-metadata-scrape` manifest declaration and endpoint implementation until `../nako`
  owns task invocation, durable records, cancellation, retry, progress, and redaction-safe outcomes.
- Do not add a hidden addon-side scheduler.

## Follow-On

Open a host-side workstream in `../nako` for the generic Addon Task runtime. After that lands, open a
new addon implementation lane for the manifest declaration and task endpoint.
