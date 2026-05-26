# Nako Browser Worker

Internal browser automation worker for anti-bot metadata sources.

## Endpoints

- `GET /health`
- `GET /fixtures/rendered-page`
- `POST /extract`
- `POST /render`

`POST /render` is the stable browser contract for metadata providers that need
rendered HTML. It accepts `{ "url": "https://example.test/page" }` plus optional
`wait_for`, `session_key`, and `proxy_policy` controls. It returns `status`,
final `url`, `title`, rendered `html`, normalized body `text`, and a short
`excerpt`.

This worker is the Crawlee/Playwright execution boundary. It owns page loading,
browser lifecycle, session intent, wait behavior, and proxy mechanics.
Site-specific metadata search, detail parsing, field mapping, provider routing,
and multi-source source policy stay in `nako-metadata-scraper`.

Proxy configuration is redaction-safe. Set `NAKO_BROWSER_WORKER_PROXY_URL` for
one proxy or `NAKO_BROWSER_WORKER_PROXY_LIST` for a comma/newline-separated
pool. `/health` reports only `proxy_configured` and `proxy_count`, never proxy
URLs or credentials. `proxy_policy` may be `default`, `direct`, or `required`;
the default uses configured worker proxies when present.

`wait_for` may be a load-state string (`load`, `domcontentloaded`,
`networkidle`) or an object such as:

```json
{
  "state": "domcontentloaded",
  "selector": "#movie",
  "timeout_ms": 5000
}
```

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
