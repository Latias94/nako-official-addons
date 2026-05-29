import { createServer } from 'node:http';

import { createApp } from '../src/app.mjs';
import { renderDriftSuiteExitCode, runRenderDriftSuite } from '../src/render-drift.mjs';

async function listen(app) {
  const server = createServer(app);
  await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
  const address = server.address();
  if (!address || typeof address !== 'object') {
    throw new Error('browser worker live render drift server did not start');
  }
  return {
    server,
    baseUrl: `http://127.0.0.1:${address.port}`,
  };
}

async function main() {
  const { server, baseUrl } = await listen(createApp());
  try {
    const suite = await runRenderDriftSuite({
      baseUrl,
      env: process.env,
    });
    process.stdout.write(`${JSON.stringify(suite, null, 2)}\n`);
    process.exitCode = renderDriftSuiteExitCode(suite);
  } finally {
    await new Promise((resolve) => server.close(resolve));
  }
}

main().catch((error) => {
  process.stderr.write(`${error.stack || error.message}\n`);
  process.exitCode = 1;
});
