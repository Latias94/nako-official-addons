import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import test from 'node:test';

import { extractRenderedPage } from '../src/extract.mjs';

test('extractRenderedPage captures rendered DOM text from a local page', async (t) => {
  const server = createServer((_request, response) => {
    response
      .statusCode = 200;
    response.setHeader('content-type', 'text/html; charset=utf-8');
    response.end(`<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Local Rendered Page</title>
</head>
<body>
  <main>
    <h1>Local Rendered Page</h1>
    <p id="status">initial</p>
  </main>
  <script>
    document.getElementById('status').textContent = 'rendered by JavaScript';
  </script>
</body>
</html>`);
  });

  await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
  t.after(async () => {
    await new Promise((resolve) => server.close(resolve));
  });

  const address = server.address();
  assert.ok(address && typeof address === 'object');
  const result = await extractRenderedPage(`http://127.0.0.1:${address.port}/page`);

  assert.equal(result.title, 'Local Rendered Page');
  assert.match(result.rendered_text, /rendered by JavaScript/);
  assert.match(result.excerpt, /rendered by JavaScript/);
});
