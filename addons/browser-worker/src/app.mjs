import express from 'express';

import {
  browserWorkerProxyFacts,
  extractRenderedPage,
  normalizeRenderOptions,
} from './extract.mjs';

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

function renderRequestFromBody(body) {
  const url = body?.url;
  if (typeof url !== 'string' || !url.trim()) {
    return {
      ok: false,
      status: 400,
      body: {
        status: 'error',
        error: 'invalid_request',
        safe_error_code: 'missing_url',
      },
    };
  }

  const options = normalizeRenderOptions(body);
  if (!options) {
    return {
      ok: false,
      status: 400,
      body: {
        status: 'error',
        error: 'invalid_request',
        safe_error_code: 'invalid_render_options',
      },
    };
  }

  return {
    ok: true,
    url: url.trim(),
    options,
  };
}

export function createApp({ env = process.env } = {}) {
  const app = express();

  app.use(express.json({ limit: '64kb' }));

  app.get('/health', (_request, response) => {
    response.json({
      status: 'ok',
      service: 'nako-browser-worker',
      renderer: 'playwright',
      crawler: 'crawlee',
      ...browserWorkerProxyFacts(env),
    });
  });

  app.get('/fixtures/rendered-page', (_request, response) => {
    response.type('html').send(FIXTURE_HTML);
  });

  app.post('/extract', async (request, response) => {
    const renderRequest = renderRequestFromBody(request.body);
    if (!renderRequest.ok) {
      response.status(renderRequest.status).json(renderRequest.body);
      return;
    }

    try {
      const extracted = await extractRenderedPage(renderRequest.url, renderRequest.options, env);
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

  app.post('/render', async (request, response) => {
    const renderRequest = renderRequestFromBody(request.body);
    if (!renderRequest.ok) {
      response.status(renderRequest.status).json(renderRequest.body);
      return;
    }

    try {
      const rendered = await extractRenderedPage(renderRequest.url, renderRequest.options, env);
      response.json({
        status: 'ok',
        url: rendered.url,
        title: rendered.title,
        html: rendered.html,
        text: rendered.text,
        excerpt: rendered.excerpt,
      });
    } catch (error) {
      response.status(502).json({
        status: 'error',
        error: 'render_failed',
        safe_error_code: 'rendered_page_render_failed',
      });
    }
  });

  return app;
}
