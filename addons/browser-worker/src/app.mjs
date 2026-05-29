import express from 'express';

import {
  browserWorkerProxyFacts,
  extractRenderedPage,
} from './extract.mjs';
import { parseRenderRequestBody } from './render-contract.mjs';
import { errorResponseFromError } from './render-errors.mjs';

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
  try {
    return {
      ok: true,
      ...parseRenderRequestBody(body),
    };
  } catch (error) {
    const response = errorResponseFromError(error, {
      errorCode: 'invalid_request',
      fallbackSafeErrorCode: 'invalid_render_options',
      fallbackFailureKind: 'invalid_options',
    });
    return {
      ok: false,
      status: response.status,
      body: response.body,
    };
  }
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
      const renderedError = errorResponseFromError(error, {
        errorCode: 'extract_failed',
        fallbackSafeErrorCode: 'rendered_page_extraction_failed',
        fallbackFailureKind: 'extraction_failed',
      });
      response.status(renderedError.status).json(renderedError.body);
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
      const renderedError = errorResponseFromError(error, {
        errorCode: 'render_failed',
        fallbackSafeErrorCode: 'rendered_page_render_failed',
        fallbackFailureKind: 'render_failed',
      });
      response.status(renderedError.status).json(renderedError.body);
    }
  });

  return app;
}
