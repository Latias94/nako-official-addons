# Nako Browser Worker

Internal browser automation worker for anti-bot metadata sources.

## Endpoints

- `GET /health`
- `GET /fixtures/rendered-page`
- `POST /extract`
- `POST /render`

`POST /render` is the stable browser contract for metadata providers that need
rendered HTML. It accepts `{ "url": "https://example.test/page" }` and returns
`status`, final `url`, `title`, rendered `html`, normalized body `text`, and a
short `excerpt`.

This worker is the Crawlee/Playwright execution boundary. It owns page loading,
browser lifecycle, and future session/wait/proxy mechanics. Site-specific
metadata search, detail parsing, field mapping, provider routing, and
multi-source source policy stay in `nako-metadata-scraper`.

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
