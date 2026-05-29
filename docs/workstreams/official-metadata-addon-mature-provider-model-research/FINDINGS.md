# Official Metadata Addon Mature Provider Model Research - Findings

Status: Draft
Last updated: 2026-05-25

## Reference Inventory

| Reference | Local path | Commit | Role |
| --- | --- | --- | --- |
| Jellyfin core | `repo-ref/jellyfin` | 498d265 | Host metadata provider model, refresh, ordering, image pipeline, merge policy |
| Jellyfin TVDB plugin | `repo-ref/jellyfin-plugin-tvdb` | 5c4592f | External plugin provider model |
| Jellyfin AniDB plugin | `repo-ref/jellyfin-plugin-anidb` | 456ded6 | Anime metadata plugin model |
| Jellyfin AniList plugin | `repo-ref/jellyfin-plugin-anilist` | 0d973c4 | Anime metadata plugin model |
| Kodi TMDB Python scraper | `repo-ref/kodi-metadata-themoviedb-python` | 285bc75 | Parser/site-drift and scraper packaging reference |

## OMAPMR-020 - Jellyfin Core Provider Model

### Finding 1 - Mature systems split provider roles by responsibility

Jellyfin does not model all metadata behaviour as one generic provider. It
separates:

- remote metadata provider: `IRemoteMetadataProvider<TItemType, TLookupInfoType>`
  exposes both `GetSearchResults` and `GetMetadata`;
- local metadata provider: `ILocalMetadataProvider<TItemType>` reads local
  metadata from an item and directory service;
- image provider: `IImageProvider` only declares name and item support;
- remote image provider: `IRemoteImageProvider` exposes supported image types,
  remote image listing, and image response download.

Source anchors:

- `repo-ref/jellyfin/MediaBrowser.Controller/Providers/IRemoteMetadataProvider.cs`
- `repo-ref/jellyfin/MediaBrowser.Controller/Providers/ILocalMetadataProvider.cs`
- `repo-ref/jellyfin/MediaBrowser.Controller/Providers/IImageProvider.cs`
- `repo-ref/jellyfin/MediaBrowser.Controller/Providers/IRemoteImageProvider.cs`

Nako implication:

- Our current `MetadataProvider::suggest` is intentionally compact and works
  for an addon sidecar, but it combines lookup, suggestion, field mapping, and
  artwork proposal behind one interface.
- The next deepening should probably introduce provider roles behind the sidecar
  interface, not necessarily expose multiple protocol resources immediately.
- A likely shape is `MetadataSource`, `ArtworkSource`, and `RenderedPageSource`
  adapters assembled by a sidecar resolver.

### Finding 2 - Provider order and enablement are host policy, not provider code

Jellyfin stores metadata and image provider order in configuration. Core
`MetadataOptions` has local reader order, metadata fetcher order, image fetcher
order, and disabled provider lists. `LibraryOptions` carries per-library and
per-type settings. `ProviderManager` then orders providers by configured order
and falls back to provider default order.

Source anchors:

- `repo-ref/jellyfin/MediaBrowser.Model/Configuration/MetadataOptions.cs`
- `repo-ref/jellyfin/MediaBrowser.Model/Configuration/LibraryOptions.cs`
- `repo-ref/jellyfin/MediaBrowser.Providers/Manager/ProviderManager.cs:399`
- `repo-ref/jellyfin/MediaBrowser.Providers/Manager/ProviderManager.cs:486`
- `repo-ref/jellyfin/MediaBrowser.Providers/Manager/ProviderManager.cs:511`
- `repo-ref/jellyfin/MediaBrowser.Providers/Manager/ProviderManager.cs:597`

Nako implication:

- Provider ordering should eventually be per-library or per-request policy owned
  by Nako core, not hard-coded in this addon.
- The sidecar should expose provider descriptors, capabilities, and default
  priorities, but Nako should decide which provider order applies to a library
  or media type.

### Finding 3 - Locked fields and local metadata are host-owned invariants

Jellyfin blocks remote metadata providers when an item is locked unless a
provider is local or forced. Its merge logic checks locked fields before
updating field groups such as genres, overview, cast, runtime, studios, tags,
and production locations. It also merges provider IDs without replacing
existing IDs unless a replace-all mode is active.

Source anchors:

- `repo-ref/jellyfin/MediaBrowser.Providers/Manager/ProviderManager.cs:576`
- `repo-ref/jellyfin/MediaBrowser.Providers/Manager/MetadataService.cs:1041`
- `repo-ref/jellyfin/MediaBrowser.Providers/Manager/MetadataService.cs:1072`
- `repo-ref/jellyfin/MediaBrowser.Providers/Manager/MetadataService.cs:1085`
- `repo-ref/jellyfin/MediaBrowser.Providers/Manager/MetadataService.cs:1154`
- `repo-ref/jellyfin/MediaBrowser.Providers/Manager/MetadataService.cs:1196`

Nako implication:

- `nako-metadata-scraper` should not independently decide whether a locked
  field can be overwritten.
- The sidecar can return field-level evidence and proposed patches, but final
  apply policy should remain in Nako core or in an explicit writeback request
  contract.

### Finding 4 - Refresh mode is a first-class state machine

Jellyfin distinguishes `None`, `ValidationOnly`, `Default`, and `FullRefresh`.
`MetadataService` uses refresh mode, first-refresh state, replace-all flags,
automatic refresh age, and provider change monitors to decide whether local,
remote, and custom providers should run.

Source anchors:

- `repo-ref/jellyfin/MediaBrowser.Controller/Providers/MetadataRefreshMode.cs`
- `repo-ref/jellyfin/MediaBrowser.Controller/Providers/MetadataRefreshOptions.cs`
- `repo-ref/jellyfin/MediaBrowser.Providers/Manager/MetadataService.cs:613`
- `repo-ref/jellyfin/MediaBrowser.Providers/Manager/MetadataService.cs:620`
- `repo-ref/jellyfin/MediaBrowser.Providers/Manager/MetadataService.cs:646`
- `repo-ref/jellyfin/MediaBrowser.Providers/Manager/MetadataService.cs:683`

Nako implication:

- Our bulk metadata task is a bounded batch planner, not a mature refresh state
  machine.
- Nako core should own scheduling, refresh mode, retry, cancellation, and
  library scan semantics. The sidecar should accept enough request context to
  produce deterministic suggestions for that state.

### Finding 5 - Search-result deduplication uses provider IDs across providers

Jellyfin remote search merges results when provider IDs overlap. During
`GetRemoteSearchResults`, each result receives the search provider name, then
existing results are found by matching provider IDs; when a match exists,
provider IDs and image URL are merged.

Source anchors:

- `repo-ref/jellyfin/MediaBrowser.Model/Providers/RemoteSearchResult.cs`
- `repo-ref/jellyfin/MediaBrowser.Providers/Manager/ProviderManager.cs:959`
- `repo-ref/jellyfin/MediaBrowser.Providers/Manager/ProviderManager.cs:971`
- `repo-ref/jellyfin/MediaBrowser.Providers/Manager/ProviderManager.cs:979`

Nako implication:

- Our current orchestration deduplicates only exact `(provider, provider_id)`
  pairs after ranking.
- A mature resolver should deduplicate across providers by shared external IDs
  and then merge provider facts before ranking final candidates.

### Finding 6 - Image handling is a separate pipeline with local-first semantics

Jellyfin validates local images separately, then runs non-local image providers
according to image refresh mode and configured order. Remote image providers
declare supported image types and can download responses or save remote image
URLs/stubs depending on host policy.

Source anchors:

- `repo-ref/jellyfin/MediaBrowser.Providers/Manager/ItemImageProvider.cs:109`
- `repo-ref/jellyfin/MediaBrowser.Providers/Manager/ItemImageProvider.cs:119`
- `repo-ref/jellyfin/MediaBrowser.Providers/Manager/ItemImageProvider.cs:165`
- `repo-ref/jellyfin/MediaBrowser.Providers/Manager/ItemImageProvider.cs:276`
- `repo-ref/jellyfin/MediaBrowser.Providers/Manager/ItemImageProvider.cs:551`
- `repo-ref/jellyfin/MediaBrowser.Providers/Manager/ItemImageProvider.cs:665`

Nako implication:

- Our artwork candidates are useful but still embedded under metadata
  candidates.
- The next model should separate artwork discovery from metadata ranking enough
  to support local artwork priority, per-kind image policy, and field-level
  writeback decisions.

## Interim Architecture Reading

Jellyfin's provider model suggests a clear line for Nako:

- Nako core should own library policy: refresh mode, locked fields, per-library
  provider order, local asset preference, task scheduling, and final write
  semantics.
- The official metadata sidecar should own provider integration: external HTTP
  calls, rendered-page access, parsing, normalized facts, candidate evidence,
  provider diagnostics, and safe proposal payloads.
- The missing middle in our current implementation is a resolver that can merge
  provider facts across providers before producing a final ranked suggestion.

## OMAPMR-030 - Plugin And Scraper Model

### Finding 7 - Provider clients own external API operational policy

Jellyfin plugins put token refresh, API cache, and provider-specific rate limits
close to their external API clients:

- TVDB uses a `TvdbClientManager` with a token update lock, configurable cache
  durations, and in-memory cache keys for search, remote ID lookup, and detail
  calls.
- AniList waits between requests according to plugin configuration, retries
  after HTTP 429, and honors `Retry-After` when supplied.
- AniDB records very low upstream request limits in code, combines a request
  limiter with configurable delay, and stores downloaded series XML under a
  cache directory with a max cache age.

Source anchors:

- `repo-ref/jellyfin-plugin-tvdb/Jellyfin.Plugin.Tvdb/TvdbClientManager.cs:31`
- `repo-ref/jellyfin-plugin-tvdb/Jellyfin.Plugin.Tvdb/TvdbClientManager.cs:61`
- `repo-ref/jellyfin-plugin-tvdb/Jellyfin.Plugin.Tvdb/TvdbClientManager.cs:114`
- `repo-ref/jellyfin-plugin-tvdb/Jellyfin.Plugin.Tvdb/TvdbClientManager.cs:139`
- `repo-ref/jellyfin-plugin-anilist/Jellyfin.Plugin.AniList/Providers/AniList/AniListApi.cs:323`
- `repo-ref/jellyfin-plugin-anilist/Jellyfin.Plugin.AniList/Providers/AniList/AniListApi.cs:330`
- `repo-ref/jellyfin-plugin-anilist/Jellyfin.Plugin.AniList/Providers/AniList/AniListApi.cs:346`
- `repo-ref/jellyfin-plugin-anidb/Jellyfin.Plugin.AniDB/Providers/AniDB/Metadata/AniDbSeriesProvider.cs:32`
- `repo-ref/jellyfin-plugin-anidb/Jellyfin.Plugin.AniDB/Providers/AniDB/Metadata/AniDbSeriesProvider.cs:160`
- `repo-ref/jellyfin-plugin-anidb/Jellyfin.Plugin.AniDB/Providers/AniDB/Metadata/AniDbSeriesProvider.cs:578`

Nako implication:

- The shared `ProviderHttpRuntime` is a good baseline, but provider-specific
  cache and rate policy should be first-class provider-local config, not
  hidden ad hoc sleeps inside provider code.
- The next refactor should likely add a provider-local `ProviderNetworkPolicy`
  or `ProviderCachePolicy` shape that the shared runtime can consume without
  centralizing provider quirks.

### Finding 8 - Mature plugins split media-type providers and image providers

The TVDB plugin has separate provider modules for series, seasons, episodes,
movies, people, missing episodes, and image providers. Image providers expose
supported image kinds and map provider artwork records into host image kinds.
This mirrors Jellyfin core's provider role split.

Source anchors:

- `repo-ref/jellyfin-plugin-tvdb/Jellyfin.Plugin.Tvdb/Providers/TvdbSeriesProvider.cs:55`
- `repo-ref/jellyfin-plugin-tvdb/Jellyfin.Plugin.Tvdb/Providers/TvdbSeriesProvider.cs:66`
- `repo-ref/jellyfin-plugin-tvdb/Jellyfin.Plugin.Tvdb/Providers/TvdbSeriesImageProvider.cs:27`
- `repo-ref/jellyfin-plugin-tvdb/Jellyfin.Plugin.Tvdb/Providers/TvdbSeriesImageProvider.cs:59`
- `repo-ref/jellyfin-plugin-tvdb/Jellyfin.Plugin.Tvdb/Providers/TvdbSeriesImageProvider.cs:92`

Nako implication:

- Our current providers mostly return one metadata-candidate type with embedded
  artwork candidates. That is sufficient for movie/anime suggestion, but it
  will strain when series, season, episode, person, and multi-artwork policies
  arrive.
- Addon internals should move toward provider role adapters before the protocol
  needs to expose separate resources.

### Finding 9 - External IDs are used for cross-provider lookup, not only matching

The TVDB series provider resolves TVDB IDs from existing TVDB IDs first, then
from IMDB, Zap2It, or TMDB remote IDs. AniDB series lookup similarly prefers an
AniDB provider ID, but falls back to provider-local title matching. Search
results carry provider IDs forward into host-level deduplication and later
details.

Source anchors:

- `repo-ref/jellyfin-plugin-tvdb/Jellyfin.Plugin.Tvdb/Providers/TvdbSeriesProvider.cs:99`
- `repo-ref/jellyfin-plugin-tvdb/Jellyfin.Plugin.Tvdb/Providers/TvdbSeriesProvider.cs:117`
- `repo-ref/jellyfin-plugin-tvdb/Jellyfin.Plugin.Tvdb/Providers/TvdbSeriesProvider.cs:123`
- `repo-ref/jellyfin-plugin-anidb/Jellyfin.Plugin.AniDB/Providers/AniDB/Metadata/AniDbSeriesProvider.cs:58`
- `repo-ref/jellyfin-plugin-anidb/Jellyfin.Plugin.AniDB/Providers/AniDB/Metadata/AniDbSeriesProvider.cs:90`
- `repo-ref/jellyfin-plugin-anidb/Jellyfin.Plugin.AniDB/Providers/AniDB/Metadata/AniDbSeriesProvider.cs:120`

Nako implication:

- Moving top-level aliases into provider descriptors was correct, but mature
  lookup wants more than alias parsing.
- We likely need provider-owned external ID capabilities: which IDs a provider
  can directly consume, which IDs it can translate, which IDs it emits, and
  whether the value is numeric, URL-like, or opaque.

### Finding 10 - Scraper UX contains hard-earned matching and artwork heuristics

Kodi's TMDB scraper has practical heuristics that are easy to underestimate:

- manual search accepts TMDB and IMDB IDs;
- title search strips trailing articles;
- year search falls back to previous year, next year, and then no year;
- artwork lists are capped because large artwork sets can stress integrations;
- details can merge ratings from IMDB or Trakt and artwork from Fanart.tv;
- NFO/text parsing extracts unique IDs before lookup.

Source anchors:

- `repo-ref/kodi-metadata-themoviedb-python/README.md`
- `repo-ref/kodi-metadata-themoviedb-python/python/scraper.py:29`
- `repo-ref/kodi-metadata-themoviedb-python/python/scraper.py:34`
- `repo-ref/kodi-metadata-themoviedb-python/python/scraper.py:84`
- `repo-ref/kodi-metadata-themoviedb-python/python/scraper.py:113`
- `repo-ref/kodi-metadata-themoviedb-python/python/scraper.py:127`
- `repo-ref/kodi-metadata-themoviedb-python/python/scraper_datahelper.py:30`
- `repo-ref/kodi-metadata-themoviedb-python/python/scraper_datahelper.py:48`
- `repo-ref/kodi-metadata-themoviedb-python/python/scraper_config.py:10`

Nako implication:

- Our shared search-title variants and external ID aliases are a good start, but
  mature matching needs explicit strategy objects for title variants, year
  fallback, manual ID lookup, and per-provider confidence penalties.
- Artwork candidate limits and per-kind enablement should be modeled before
  artwork grows beyond posters.

### Finding 11 - Plugin configuration often shapes returned metadata, not only connectivity

Kodi settings and Jellyfin plugin configuration control more than credentials:
cache duration, image categories, fanart priority, language fallback, default
rating source, tag inclusion, original-title preference, and local parser
behaviour all shape output.

Source anchors:

- `repo-ref/jellyfin-plugin-tvdb/Jellyfin.Plugin.Tvdb/Configuration/PluginConfiguration.cs`
- `repo-ref/jellyfin-plugin-anidb/Jellyfin.Plugin.AniDB/Configuration/PluginConfiguration.cs`
- `repo-ref/jellyfin-plugin-anilist/Jellyfin.Plugin.AniList/Configuration/PluginConfiguration.cs`
- `repo-ref/kodi-metadata-themoviedb-python/resources/settings.xml`
- `repo-ref/kodi-metadata-themoviedb-python/python/scraper_config.py:1`
- `repo-ref/kodi-metadata-themoviedb-python/python/scraper_config.py:51`
- `repo-ref/kodi-metadata-themoviedb-python/python/scraper_config.py:71`
- `repo-ref/kodi-metadata-themoviedb-python/python/scraper_config.py:85`

Nako implication:

- Provider config should continue moving provider-local, but we need to
  distinguish network config from result-shaping policy.
- Result-shaping policy probably belongs in Nako request/library context rather
  than static sidecar environment variables.

## OMAPMR-040 - Local Architecture Comparison

### Finding 12 - The local sidecar already has provider-local integration seams

`nako-metadata-scraper` is not a single monolithic scraper anymore. It has a
catalog-backed registry, typed provider config, provider-local clients,
provider-local parsers/mappers, a shared HTTP runtime, a rendered-page runtime,
and redaction-safe diagnostics.

Source anchors:

- `crates/nako-metadata-scraper/src/providers/mod.rs:34`
- `crates/nako-metadata-scraper/src/providers/registry.rs:12`
- `crates/nako-metadata-scraper/src/providers/registry.rs:108`
- `crates/nako-metadata-scraper/src/providers/registry.rs:176`
- `crates/nako-metadata-scraper/src/config.rs:72`
- `crates/nako-metadata-scraper/src/providers/http_runtime.rs:15`
- `crates/nako-metadata-scraper/src/providers/rendered_page.rs:25`
- `crates/nako-metadata-scraper/src/routes.rs:37`

Nako implication:

- The next refactor should not restart around another generic provider trait.
  The current provider registry and provider-local module split are useful.
- The stronger opportunity is the missing middle between provider outputs and
  final suggestions: a resolver that owns cross-provider deduplication,
  provider-fact merging, and ranked proposal assembly.

### Finding 13 - Current orchestration ranks isolated candidates

The sidecar currently asks each enabled provider for suggestions, immediately
ranks every provider candidate, sorts all ranked candidates together, and then
deduplicates only exact `(provider, provider_id)` pairs.

Source anchors:

- `crates/nako-metadata-scraper/src/engine/orchestration.rs:7`
- `crates/nako-metadata-scraper/src/engine/orchestration.rs:35`
- `crates/nako-metadata-scraper/src/engine/ranking.rs:21`
- `crates/nako-metadata-scraper/src/engine/ranking.rs:104`

Nako implication:

- This is correct for a simple suggestion sidecar, but it cannot merge a TMDB
  movie and another provider result that share an IMDB ID.
- Mature systems deduplicate by shared provider IDs before final ranking. Nako
  should introduce a resolver that clusters provider facts by external IDs,
  keeps provenance, then ranks clusters instead of isolated provider rows.

### Finding 14 - External ID support is input parsing, not a capability model

Provider catalog entries expose top-level external ID aliases, and query parsing
uses those aliases to normalize payload fields into `QueryExternalId` values.
Provider-specific direct lookups are implemented inside provider search modules.

Source anchors:

- `crates/nako-metadata-scraper/src/providers/registry.rs:90`
- `crates/nako-metadata-scraper/src/providers/registry.rs:182`
- `crates/nako-metadata-scraper/src/engine/query.rs:51`
- `crates/nako-metadata-scraper/src/engine/query.rs:198`
- `crates/nako-metadata-scraper/src/providers/tmdb/search.rs:5`
- `crates/nako-metadata-scraper/src/providers/bangumi/search.rs:9`

Nako implication:

- Aliases were the right first move, but mature lookup needs descriptors for
  what each provider can consume, translate, and emit.
- This capability model should feed both provider direct lookup and resolver
  deduplication, while leaving final library/provider policy to Nako core.

### Finding 15 - Artwork is modeled as metadata-candidate decoration

Providers return artwork inside `ProviderMetadataCandidate`, and the engine
maps those into `ArtworkCandidate` values using the metadata candidate's final
confidence. Artwork writeback then selects by artwork kind, metadata
confidence, image area, and stable provider tie-breakers.

Source anchors:

- `crates/nako-metadata-scraper/src/engine/artwork.rs:14`
- `crates/nako-metadata-scraper/src/engine/artwork.rs:104`
- `crates/nako-metadata-scraper/src/engine/writeback.rs:162`
- `crates/nako-metadata-scraper/src/providers/tmdb/mapper.rs:309`
- `crates/nako-metadata-scraper/src/providers/bangumi/mapper.rs:330`
- `crates/nako-metadata-scraper/src/providers/douban/mapper.rs:21`

Nako implication:

- This is a pragmatic first model and works for poster/backdrop proposals.
- It will become too coarse when local artwork, per-kind image priority,
  language preference, and multi-image limits matter. Split artwork discovery
  internally before exposing more protocol surface.

### Finding 16 - Shared HTTP runtime lacks provider-specific operational policy

`ProviderHttpRuntime` centralizes timeout, retry attempts, response size limits,
backoff, proxy use, redacted error handling, and bounded response reads. Provider
clients pass their static config into this runtime.

Source anchors:

- `crates/nako-metadata-scraper/src/providers/http_runtime.rs:15`
- `crates/nako-metadata-scraper/src/providers/http_runtime.rs:117`
- `crates/nako-metadata-scraper/src/providers/tmdb/client.rs:20`
- `crates/nako-metadata-scraper/src/providers/bangumi/client.rs:13`
- `crates/nako-metadata-scraper/src/providers/douban/client.rs:14`

Nako implication:

- The shared runtime is valuable, but Jellyfin plugin references show that
  mature providers need cache keys, TTLs, token refresh locks, upstream-specific
  rate limits, and `Retry-After` handling.
- These policies should remain provider-local inputs to the shared runtime, not
  become one global hard-coded metadata-scraper policy.

### Finding 17 - Writeback is authority-gated but not field-policy-aware

The sidecar only submits side effects when explicit writeback payloads are
present, Nako runtime is configured, and Nako access checks allow metadata or
artwork writes. The payload sent to Nako is still the selected candidate patch;
there is no local locked-field, local-metadata, or per-field merge policy.

Source anchors:

- `crates/nako-metadata-scraper/src/engine/runtime.rs:57`
- `crates/nako-metadata-scraper/src/engine/writeback.rs:59`
- `crates/nako-metadata-scraper/src/engine/writeback.rs:97`
- `crates/nako-metadata-scraper/src/engine/writeback.rs:132`
- `crates/nako-metadata-scraper/src/engine/writeback.rs:209`
- `crates/nako-metadata-scraper/src/engine/writeback.rs:238`

Nako implication:

- This boundary is mostly correct: Nako core should remain authoritative for
  locked fields, local metadata, and final merge semantics.
- A future protocol/request policy can tell the sidecar which fields are useful
  to propose, but the sidecar should not independently become the host merge
  engine.

### Finding 18 - Bulk scraping is a batch planner, not a refresh engine

The bulk task accepts a bounded item list, cursor, and batch size, then calls the
same resource scrape path for each item. Library scanned events are accepted and
redacted, but they do not implement a provider refresh state machine.

Source anchors:

- `crates/nako-metadata-scraper/src/engine/bulk.rs:101`
- `crates/nako-metadata-scraper/src/routes.rs:113`
- `crates/nako-metadata-scraper/src/routes.rs:126`

Nako implication:

- This should stay lightweight in the sidecar.
- Mature refresh modes, local-first scans, replace-all behaviour, retries, and
  scheduling belong in Nako core. The sidecar should accept refresh context and
  produce deterministic suggestions for that context.
