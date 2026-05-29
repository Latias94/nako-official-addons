# Official Metadata Addon AV Fine-Grained Field Policy - Handoff

Status: Complete
Last updated: 2026-05-27

## Current State

This lane is complete. The default AV field policy is now a durable field matrix adapted to
supported providers, and score-like facts participate in provider-field fusion.

## Reference Notes

Reference config priorities to adapt to supported provider IDs:

- title: `theporndb,mgstage,dmm,javbus,jav321,javlibrary`
- outline: `theporndb,dmm,jav321`
- actor: `theporndb,javbus,javlibrary,javdb`
- thumb: `theporndb,javbus`
- poster: `theporndb,avsex,javbus` -> supported: `theporndb,javbus`
- extrafanart: `javbus,freejavbt` -> supported: `javbus`
- trailer: `freejavbt,mgstage,dmm` -> supported: `mgstage,dmm`
- tag: `javbus,freejavbt` -> supported: `javbus`
- release: `javbus,freejavbt,7mmtv` -> supported: `javbus`
- runtime: `javbus,freejavbt` -> supported: `javbus`
- score: `jav321,javlibrary,javdb`
- director: `javbus,freejavbt` -> supported: `javbus`
- series: `javbus,freejavbt` -> supported: `javbus`
- studio: `javbus,freejavbt` -> supported: `javbus`
- publisher: `javbus`
- wanted: `javlibrary,javdb`

## Completed

- Default AV provider order is explicit for title, outline, actor, thumb, poster, extra fanart,
  trailer, tag, release, runtime, score, director, series, studio, publisher, and wanted.
- Request aliases include `outline`, `actor`, `thumb`, `extrafanart`, `trailer`, `tag`, `release`,
  `runtime`, `director`, `wanted`, and `score`.
- Score/vote fusion records `community_score_milli` and `community_vote_count` field sources.
- Full package verification passed on 2026-05-27.

## Follow-Up Candidates

- Add more providers only when each provider has an explicit field-trust decision.
- Run live drift cases for score/trailer/artwork-heavy AV numbers after the next provider wave.
