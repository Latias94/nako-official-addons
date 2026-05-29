import assert from 'node:assert/strict';
import { createServer } from 'node:http';

import { createApp } from '../src/app.mjs';

async function main() {
  const app = createApp();
  const server = createServer(app);
  await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));

  try {
    const address = server.address();
    assert.ok(address && typeof address === 'object');
    const baseUrl = `http://127.0.0.1:${address.port}`;

    const health = await fetch(`${baseUrl}/health`);
    assert.equal(health.status, 200);
    const healthBody = await health.json();
    assert.equal(healthBody.status, 'ok');
    assert.equal(healthBody.renderer, 'playwright');

    const extract = await fetch(`${baseUrl}/extract`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({
        url: `${baseUrl}/fixtures/rendered-page`,
      }),
    });
    assert.equal(extract.status, 200);
    const extractBody = await extract.json();
    assert.equal(extractBody.status, 'ok');
    assert.match(extractBody.rendered_text, /rendered by JavaScript/);
    assert.match(extractBody.excerpt, /rendered by JavaScript/);

    const render = await fetch(`${baseUrl}/render`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({
        url: `${baseUrl}/fixtures/rendered-page`,
      }),
    });
    assert.equal(render.status, 200);
    const renderBody = await render.json();
    assert.equal(renderBody.status, 'ok');
    assert.match(renderBody.text, /rendered by JavaScript/);
    assert.match(renderBody.html, /rendered by JavaScript/);

    process.stdout.write('[ok] browser worker smoke passed\n');
  } finally {
    await new Promise((resolve) => server.close(resolve));
  }
}

main().catch((error) => {
  process.stderr.write(`${error.stack || error.message}\n`);
  process.exitCode = 1;
});
