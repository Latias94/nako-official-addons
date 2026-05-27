import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import test from 'node:test';

import { createApp } from '../src/app.mjs';
import { normalizeRenderOptions } from '../src/extract.mjs';
import { RenderRuntime } from '../src/render-runtime.mjs';
import { RenderWorkerError } from '../src/render-errors.mjs';

async function withApp(app, callback) {
  const server = createServer(app);
  await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
  try {
    const address = server.address();
    assert.ok(address && typeof address === 'object');
    await callback(`http://127.0.0.1:${address.port}`);
  } finally {
    await new Promise((resolve) => server.close(resolve));
  }
}

test('health reports redaction-safe proxy facts', async () => {
  const app = createApp({
    env: {
      NAKO_BROWSER_WORKER_PROXY_URL: 'http://user:pass@proxy.example:8080',
      NAKO_BROWSER_WORKER_PROXY_LIST: 'http://proxy-two.example:8080',
    },
  });

  await withApp(app, async (baseUrl) => {
    const response = await fetch(`${baseUrl}/health`);
    assert.equal(response.status, 200);
    const body = await response.json();

    assert.equal(body.status, 'ok');
    assert.equal(body.proxy_configured, true);
    assert.equal(body.proxy_count, 2);
    const text = JSON.stringify(body);
    assert.doesNotMatch(text, /user:pass/);
    assert.doesNotMatch(text, /proxy\.example/);
  });
});

test('render endpoint rejects invalid render options before browser work', async () => {
  const app = createApp();

  await withApp(app, async (baseUrl) => {
    const response = await fetch(`${baseUrl}/render`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({
        url: `${baseUrl}/fixtures/rendered-page`,
        proxy_policy: 'unknown',
      }),
    });

    assert.equal(response.status, 400);
    const body = await response.json();
    assert.equal(body.safe_error_code, 'invalid_render_options');
    assert.equal(body.failure_kind, 'invalid_options');
  });
});

test('render endpoint rejects non-http URLs before browser work', async () => {
  const app = createApp();

  await withApp(app, async (baseUrl) => {
    const response = await fetch(`${baseUrl}/render`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({
        url: 'file:///etc/passwd',
      }),
    });

    assert.equal(response.status, 400);
    const body = await response.json();
    assert.equal(body.safe_error_code, 'invalid_render_url_scheme');
    assert.equal(body.failure_kind, 'invalid_request');
  });
});

test('normalizeRenderOptions accepts wait, session, and proxy policy aliases', () => {
  const options = normalizeRenderOptions({
    wait_for: { state: 'domcontentloaded', selector: '#status', timeout_ms: 1000 },
    session_key: 'javdb:ssni-644',
    proxy_policy: 'required',
    headers: {
      cookie: 'age=verified',
    },
    actions: [
      { type: 'check', selector: '#ageVerify input[type="checkbox"]', optional: true },
      {
        type: 'click',
        selector: '#ageVerify #submit',
        optional: true,
        wait_for: { state: 'domcontentloaded', timeout_ms: 2000 },
      },
    ],
  });

  assert.deepEqual(options, {
    waitFor: {
      state: 'domcontentloaded',
      selector: '#status',
      timeoutMs: 1000,
    },
    proxyPolicy: 'required',
    renderTimeoutMs: 30000,
    sessionKey: 'javdb:ssni-644',
    headers: {
      cookie: 'age=verified',
    },
    actions: [
      { type: 'check', selector: '#ageVerify input[type="checkbox"]', optional: true },
      {
        type: 'click',
        selector: '#ageVerify #submit',
        optional: true,
        waitFor: {
          state: 'domcontentloaded',
          timeoutMs: 2000,
        },
      },
    ],
  });
});

test('normalizeRenderOptions rejects over-budget actions', () => {
  const actions = Array.from({ length: 9 }, () => ({
    type: 'click',
    selector: '#submit',
  }));

  assert.equal(normalizeRenderOptions({ actions }), null);
});

test('RenderRuntime enforces rendered response size policy behind the runtime seam', async () => {
  const runtime = new RenderRuntime({
    adapter: {
      async renderPage() {
        return {
          url: 'https://example.test/page',
          title: 'Oversized',
          html: '<html></html>',
          text: 'too large',
          rendered_text: 'too large',
          excerpt: 'too large',
        };
      },
    },
    policy: {
      maxHtmlBytes: 1024,
      maxTextBytes: 4,
    },
  });

  await assert.rejects(
    runtime.render({
      url: 'https://example.test/page',
      options: {
        waitFor: { state: 'networkidle' },
        proxyPolicy: 'default',
        renderTimeoutMs: 30000,
      },
    }),
    (error) => {
      assert.ok(error instanceof RenderWorkerError);
      assert.equal(error.safeErrorCode, 'rendered_text_too_large');
      assert.equal(error.failureKind, 'response_too_large');
      return true;
    },
  );
});
