# Official Resource Link Check Provider - Closeout

Status: Closed
Closed: 2026-05-28

## Delivered

- Added first-class `resource_link_check` to the official resource-search addon
  manifest and checked-in example manifest.
- Added `/resource-link-check`.
- Added typed protocol decode/encode for
  `nako.addon.resource_link_check.request.v1` and
  `nako.addon.resource_link_check.response.v1`.
- Added a `ResourceLinkCheckProvider` boundary.
- Added a conservative checker provider:
  - fixture cloud links return `reachable`;
  - ordinary cloud/web links return `unknown` without live network;
  - magnet/ed2k links return `unsupported`.
- Kept response payloads redaction-safe: no raw URL, password/code, note, token,
  or provider raw body is returned.
- Updated local smoke and docs.

## Verification

- `cargo nextest run -p nako-resource-search resource_link_check --no-fail-fast`
- `cargo nextest run -p nako-resource-search manifest --no-fail-fast`
- `cargo nextest run -p nako-resource-search link_check conservative_checker runtime_check_link manifest --no-fail-fast`
- `cargo fmt --all -- --check`
- `cargo check -p nako-resource-search --tests`
- `git diff --check`

## Follow-Ons

- Site-specific live checker providers.
- Admin UI call site.
- Downloader/external runner contracts.
- Cloud-drive transfer contracts.
- Password/code reference policy.
