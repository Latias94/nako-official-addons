import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import test from 'node:test';

import { createApp } from '../src/app.mjs';
import { normalizeRenderOptions } from '../src/extract.mjs';

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
