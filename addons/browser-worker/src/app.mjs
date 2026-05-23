import express from 'express';

import { extractRenderedPage } from './extract.mjs';

const FIXTURE_HTML = `<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Nako Browser Worker Fixture</title>
</head>
<body>
  <main>
    <h1>Browser worker fixture</h1>
    <p id="status">initial</p>
  </main>
  <script>
    document.getElementById('status').textContent = 'rendered by JavaScript';
  </script>
</body>
</html>`;

export function createApp() {
  const app = express();

  app.use(express.json({ limit: '64kb' }));

  app.get('/health', (_request, response) => {
    response.json({
      status: 'ok',
      service: 'nako-browser-worker',
      renderer: 'playwright',
      crawler: 'crawlee',
    });
  });

  app.get('/fixtures/rendered-page', (_request, response) => {
    response.type('html').send(FIXTURE_HTML);
  });

  app.post('/extract', async (request, response) => {
    const url = request.body?.url;
    if (typeof url !== 'string' || !url.trim()) {
      response.status(400).json({
        status: 'error',
        error: 'invalid_request',
        safe_error_code: 'missing_url',
      });
      return;
    }

    try {
      const extracted = await extractRenderedPage(url.trim());
      response.json({
        status: 'ok',
        ...extracted,
      });
    } catch (error) {
      response.status(502).json({
        status: 'error',
        error: 'extract_failed',
        safe_error_code: 'rendered_page_extraction_failed',
      });
    }
  });

  return app;
}
