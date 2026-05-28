# Milestones

## M1 - Search Language Is Stable

Status: complete via ORSAH-020.

Exit criteria:

- Search intent names are documented.
- Official vs third-party ownership rules are documented.
- Domain DTOs and internal query intent are separate.

## M2 - Providers Are Registry-Owned

Status: complete via ORSAH-030.

Exit criteria:

- Runtime no longer constructs concrete providers directly.
- Provider descriptors expose capabilities and source policy.
- Provider-specific config and schema fragments are local to each provider.

## M3 - Fusion Is A Deep Module

Status: complete via ORSAH-040.

Exit criteria:

- Orchestration does not own deduplication/ranking details.
- Fusion interface tests cover grouping, dedupe, filtering, and provenance.

## M4 - Ready For Nako Host Lane

Status: complete via ORSAH-060 and ORSAH-070.

Exit criteria:

- Addon-side model is reflected in the deferred Nako protocol proposal.
- Full validation passes.
- Workstream evidence and handoff are current.

Note:

- The workspace fmt gate required one formatting commit in `../nako`:
  `d9d74402 style: format admin contract`.
