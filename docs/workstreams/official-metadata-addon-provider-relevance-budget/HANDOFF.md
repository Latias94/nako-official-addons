# Official Metadata Addon Provider Relevance Budget — Handoff

Status: Complete
Last updated: 2026-05-24

## Current State

The lane is complete. TMDB and Bangumi now rank deduped merged search results before spending their
three-candidate detail-enrichment budgets.

## Completed Tasks

- OMPRB-010: relevance-budget policy freeze.
- OMPRB-020: TMDB relevance-budget implementation and provider test.
- OMPRB-030: Bangumi relevance-budget implementation and provider test.
- OMPRB-040: closeout with fresh gate evidence.

## Decisions Since Last Update

- Keep raw search-result collection provider-local.
- Use provider-neutral facts and ranking behavior to select the enrichment budget.
- Do not change final runtime ranking or the Addon Protocol payload shape.
- Keep live provider gates out of default validation.

## Blockers

- None.

## Follow-Ons

- Live provider payload drift checks.
- Provider-specific transliteration or alias expansion if synthetic tests expose a concrete gap.
