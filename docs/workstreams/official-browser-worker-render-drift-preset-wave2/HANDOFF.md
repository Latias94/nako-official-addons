# Official Browser Worker Render Drift Preset Wave2 - Handoff

Status: Complete
Last updated: 2026-05-27

## Current Phase

OBWRDW2-010 through OBWRDW2-030 are complete. Wave2 provider-owned Browser
Worker render drift presets are implemented, documented, validated, and ready
to commit.

## Current Shape

- `dmm::render_drift_case` owns DMM search URL and `cid=` selector.
- `mgstage::render_drift_case` owns MGStage detail URL and detail selectors.
- `rendered_search_av::render_drift_case` owns generic rendered-search AV
  search URL/selector generation.
- `providers::render_drift` now emits cases for Douban, DMM, JavBus,
  JavLibrary, XCity, AirAV, AVSox, and MGStage when enabled.

## Next Action

Commit this lane. Follow-ups should cover remaining rendered providers or an
explicit IMDb provider/recipe before adding IMDb presets.
