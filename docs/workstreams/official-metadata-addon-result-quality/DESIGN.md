# Official Metadata Addon Result Quality

Status: Complete
Last updated: 2026-05-23

## Why This Lane Exists

The addon is published and the protocol boundary is stable for alpha.1, but
the current result set still leaves room for improvement:

- provider candidates are sorted, but noisy or duplicate candidates can still
  bubble into the top list;
- provider-specific facts are good enough for baseline support, but some
  signals can be made more consistent and more useful for ranking;
- future providers should inherit a stronger quality bar than the current
  TMDB/Bangumi baselines.

This lane focuses on result quality, not Admin Web. Admin Web is out of scope
and will be redesigned separately.

## Problem

1. Candidate lists are only sorted today. The runtime does not yet dedupe exact
   provider duplicates or cap the final result set.
2. Ranking works, but its output can still benefit from a clearer candidate
   quality policy and stronger deterministic ordering.
3. TMDB and Bangumi baselines should continue to improve their fact quality
   without changing the stable protocol contract.
4. We should avoid adding provider-specific ranking hacks in the providers
   themselves.

## Target State

When this lane closes:

- `MetadataScrapeRuntime` owns candidate shaping, de-duplication, and any final
  result capping policy.
- Provider output remains normalized and provider-neutral.
- Ranking stays deterministic and redaction-safe.
- TMDB and Bangumi expose the strongest cheap match signals they already know
  without changing the Addon Protocol envelope.
- Default tests keep using synthetic fake HTTP responses.

## In Scope

- Runtime candidate quality policy.
- Deterministic result capping and duplicate suppression.
- Ranking and evidence refinements.
- TMDB/Bangumi provider signal quality where it can be done cheaply and safely.
- README/workstream evidence updates.

## Out Of Scope

- Admin Web changes.
- Nako core server changes.
- Addon Protocol shape changes unless a real contract gap is found.
- New crawler framework design for Douban or similar lanes.

## Architecture Direction

Preferred module direction:

1. `engine::mod`
   - Own final candidate shaping.
   - Deduplicate exact repeats before returning sorted results.
   - Keep a deterministic cap on the final list size.
2. `engine::ranking`
   - Own final confidence and evidence details.
   - Keep score reasons redaction-safe.
3. `providers::tmdb` and `providers::bangumi`
   - Keep exposing normalized facts that are cheap to gather.
   - Do not hardcode final ordering policy.

## Follow-On Split Rules

Split rather than expand this lane if:

- a provider needs a new transport or crawler runtime;
- a provider wants to change the Addon Protocol manifest or resource schema;
- a new provider class requires browser automation or Playwright;
- candidate quality work starts to need persisted user feedback or Nako-side
  curation state.

## Closeout Condition

This lane can close when:

- runtime candidate shaping is deterministic and tested;
- provider result quality is improved in at least one concrete area;
- fresh validation passes with `cargo fmt`, targeted nextest, workspace
  nextest, and `git diff --check`;
- docs explain the current result quality behavior truthfully.
