# Official Metadata Addon AV Parser Quality

Status: Active
Last updated: 2026-05-26

## Why This Lane Exists

AV provider breadth is now useful enough that parser quality is the next
leverage point. Wave 3 proved that row-level DOM label parsing prevents
adjacent fields from bleeding into one another on rendered adult-site pages.
That logic currently lives in the official uncensored provider, while older AV
providers still repeat simpler full-text label scanners.

This lane improves extraction quality before adding more providers. The goal is
to make future provider work cheaper, safer, and easier to verify.

## Target State

When this lane closes:

- rendered AV parsing has shared helpers for structured label rows, split field
  values, unique list accumulation, media URL filtering, and common detail
  fields where the abstraction stays small;
- migrated providers keep provider-specific URL and site quirks local but reuse
  shared parsing primitives;
- tests prove behavior through provider public seams, especially field-boundary
  cases where one label must not swallow following overview, trailer, or media
  text;
- provider docs explain the parser-quality contract and drift strategy;
- no live adult-site payloads are stored in CI fixtures.

## In Scope

- `crates/nako-metadata-scraper/src/providers/rendered_av.rs`
- AV provider parser modules and their synthetic fixtures.
- Workstream docs, evidence, and parser-quality README notes.
- Manual drift-check documentation if needed.

## Out Of Scope

- Adding new Wave 4 providers before the parser-quality foundation is stable.
- Copying selectors, fixtures, regex tables, comments, or structure from MDCx or
  other GPL/reference projects.
- Live scraping against adult websites in CI.
- Nako core refresh/locked-field/local metadata/local artwork priority.
- UI review, NFO/rename, and actor-image workflows.

## Architecture Direction

Prefer deep shared parser helpers with small interfaces. A good shared helper
encodes a behavior contract, such as "read a label from the same row before
falling back to full text", not a provider-specific page model. Provider modules
should still own selectors and source-specific mapping decisions.

The first migration target is the field-boundary parser:

- extract a shared structured label reader from the official uncensored provider
  into `rendered_av`;
- migrate official uncensored and FC2PPVDB first because their fixtures already
  cover adjacent-label fields and browser-worker rendered detail pages;
- then migrate DMM/MGStage/JavBus/JavLibrary/JavDB/FC2 only where a focused
  test proves better behavior.

## Closeout Condition

This lane can close when:

- the shared parser primitives are in `rendered_av` and used by at least two AV
  provider families;
- targeted parser-quality tests pass;
- docs and workstream evidence explain what is improved and what remains;
- full relevant package, format, JSON, and diff hygiene gates pass;
- remaining provider migrations are explicit follow-ups or completed.
