# Official Browser Worker Live AV Provider Tuning - Design

Status: Completed
Last updated: 2026-05-27

## Problem

The rendered AV drift preset wave now covers the intended Browser Worker-backed
providers, but the first live sample showed that several real sites need more
mature waiting and timeout policy than the initial generated presets:

- DMM, JavBus, and 1Pondo reached the browser worker but failed while waiting
  for provider selectors.
- FC2 and Caribbean exhausted the render timeout.
- JavLibrary, MGStage, AVSOX, and FC2PPVDB failed for access or network reasons
  that should remain redaction-safe and operator-actionable.

## Target State

Generated AV render-drift cases should model realistic live scraping conditions:

- slow AV sites receive explicit selector wait budgets;
- slow detail pages receive a larger live-drift render budget;
- provider secrets, target URLs, sample numbers, and proxy URLs remain absent
  from drift output;
- access-bound failures are classified and documented instead of hidden behind
  generic extraction failures.

## Scope

- Rust drift case contract and generated provider presets.
- Targeted Browser Worker live drift evidence.
- Operator notes for proxy/cookie/network-bound provider failures.

## Non-Goals

- No hardcoded bypass for forbidden, CAPTCHA, regional, or account-gated pages.
- No raw HTML, full target URLs, sample numbers, cookies, or proxy URLs in
  committed evidence.
- No expansion to new providers in this lane.

## Architecture Direction

Keep Browser Worker as the execution boundary and make provider-owned drift
presets richer. Rust should emit provider-specific intent: target URL,
selector, proxy policy, render timeout, selector timeout, byte thresholds, and
optional actions. Browser Worker should continue normalizing and enforcing that
contract without knowing provider internals.

The important split is:

- provider modules know which pages are slow and which selectors prove page
  health;
- Browser Worker knows how to render safely, classify failures, and redact
  diagnostics;
- live drift docs record operational evidence and follow-up decisions.

## Closeout

This lane is complete. The generated live drift contract now covers selector
wait budgets, safe cookie/header env references, and slow render budgets. The
second live run validates the intended improvements and leaves only
access/network-bound provider failures for a separate follow-up lane.
