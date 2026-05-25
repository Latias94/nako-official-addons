# Official Metadata Addon Provider Fact Resolver - Milestones

Status: Complete
Last updated: 2026-05-25

## M0 - Scope And License Guardrails

- Workstream docs define target state, task order, gates, and license guardrails.
- Reference repositories remain ignored and are not part of the commit surface.

## M1 - Resolver Model

- Internal resolver model exists.
- Existing `ProviderMetadataCandidate` can be adapted into resolver facts.
- Tests prove cluster identity and provenance basics without changing provider
  mapper code.

## M2 - Resolver-Backed Orchestration

- `suggest_candidates` runs through resolver clustering before final ranking.
- Existing response shape stays compatible.
- Tests prove exact provider-ID dedupe and shared external-ID clustering.

## M3 - External ID Capability Catalog

- Provider catalog entries describe external ID capabilities.
- Existing top-level aliases still parse.
- Positive numeric validation remains covered.
- Resolver can rely on provider-emitted external ID descriptors.

## M4 - Integration And Documentation

- Full package gate passes.
- README or workstream docs explain resolver behaviour and provider capability
  boundaries where useful.
- No reference-source copy or license violation risk is introduced.

## M5 - Closeout

- Review and verification are recorded.
- Workstream is closed or follow-ons are split explicitly.

## Closeout

Closed on 2026-05-25 with all milestones complete. No immediate follow-on is
split from this lane.
