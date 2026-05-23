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
short `excerpt`. Site-specific metadata parsing stays in
`nako-metadata-scraper`; this worker owns browser execution, not provider
semantics.

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
