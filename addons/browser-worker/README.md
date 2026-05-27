# Nako Browser Worker

Internal browser automation worker for anti-bot metadata sources.

## Endpoints

- `GET /health`
- `GET /fixtures/rendered-page`
- `POST /extract`
- `POST /render`

`POST /render` is the stable browser contract for metadata providers that need
rendered HTML. It accepts `{ "url": "https://example.test/page" }` plus optional
`wait_for`, `session_key`, `proxy_policy`, `headers`, `actions`, and
`render_timeout_ms` controls. It returns `status`, final `url`, `title`,
rendered `html`, normalized body `text`, and a short `excerpt`.

This worker is the Crawlee/Playwright execution boundary. It owns page loading,
browser lifecycle, session intent, wait behavior, and proxy mechanics.
Site-specific metadata search, detail parsing, field mapping, provider routing,
and multi-source source policy stay in `nako-metadata-scraper`.

The worker code is split by interface:

- `render-contract.mjs` owns Render Intent parsing, aliases, defaults, and
  invalid-request errors.
- `render-safety.mjs` owns URL validation, proxy facts, timeout defaults, and
  response/action/header budgets.
- `render-runtime.mjs` owns page-level execution helpers and response budget
  enforcement behind a small runtime interface.
- `crawlee-render-adapter.mjs` owns the concrete Crawlee/Playwright adapter.
- `extract.mjs` remains a compatibility facade for existing callers.

Proxy configuration is redaction-safe. Set `NAKO_BROWSER_WORKER_PROXY_URL` for
one proxy or `NAKO_BROWSER_WORKER_PROXY_LIST` for a comma/newline-separated
pool. `/health` reports only `proxy_configured` and `proxy_count`, never proxy
URLs or credentials. `proxy_policy` may be `default`, `direct`, or `required`;
the default uses configured worker proxies when present.

Render safety defaults are configurable:

- `NAKO_BROWSER_WORKER_RENDER_TIMEOUT_MS=30000`
- `NAKO_BROWSER_WORKER_MAX_RENDER_TIMEOUT_MS=120000`
- `NAKO_BROWSER_WORKER_MAX_HTML_BYTES=8388608`
- `NAKO_BROWSER_WORKER_MAX_TEXT_BYTES=1048576`
- `NAKO_BROWSER_WORKER_MAX_ACTIONS=8`
- `NAKO_BROWSER_WORKER_MAX_HEADERS=16`
- `NAKO_BROWSER_WORKER_MAX_HEADER_VALUE_BYTES=8192`

Only `http` and `https` URLs are accepted, and URLs with embedded credentials
are rejected before browser work starts.

`wait_for` may be a load-state string (`load`, `domcontentloaded`,
`networkidle`) or an object such as:

```json
{
  "state": "domcontentloaded",
  "selector": "#movie",
  "timeout_ms": 5000
}
```

`headers` is an optional object of request headers to apply before navigation.
It exists for provider-owned operational needs such as a site cookie and is not
reported by `/health`. Header count and header value size are bounded by the
render safety policy.

`actions` is an optional bounded list of page actions executed after the initial
wait and before extraction. Supported actions are `check` and `click`; each
action has a CSS `selector`, optional `optional: true`, and optional `wait_for`
for the post-action page state:

```json
{
  "actions": [
    {
      "type": "check",
      "selector": "#ageVerify input[type=\"checkbox\"]",
      "optional": true
    },
    {
      "type": "click",
      "selector": "#ageVerify #submit",
      "optional": true,
      "wait_for": { "state": "domcontentloaded", "timeout_ms": 10000 }
    }
  ]
}
```

Error responses are redaction-safe. They keep the existing `error` and
`safe_error_code` fields and also return `failure_kind`, for example
`invalid_request`, `invalid_options`, `selector_timeout`, `action_failed`,
`operator_action`, `render_timeout`, `browser_execution_failed`,
`extraction_failed`, or `response_too_large`. They never include target URLs,
selectors, cookies, proxy URLs, or credentials.

## Local run

```bash
npm install
npm start
```

## Smoke

```bash
npm run smoke
```

## Test

```bash
npm test
```
