# Milestones

## M1 - Boundary Is Written Down

Exit criteria:

- Workstream docs exist.
- Resource search is explicitly separate from metadata scraping.
- PanSou reference lessons are captured at the architectural level.
- Nako core protocol changes are marked deferred.

## M2 - Alpha Sidecar Runs Locally

Exit criteria:

- Workspace contains `nako-resource-search`.
- The sidecar exposes manifest, health, search, and diagnostics routes.
- The manifest validates under the current addon protocol.
- The alpha search response is typed and covered by route tests.

## M3 - Search Domain Is Testable

Exit criteria:

- Provider trait and deterministic fixture provider exist.
- Link classification covers common cloud-drive, magnet, ed2k, and fallback
  links.
- Result fusion deduplicates normalized URLs and preserves source provenance.
- Search output groups links by classified type.

## M4 - Ready For Host Protocol Lane

Exit criteria:

- Focused tests pass.
- Formatting and whitespace gates pass for touched paths.
- Deferred Nako protocol proposal is current.
- Workstream evidence and handoff are current.
