import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import test from 'node:test';

import { createApp } from '../src/app.mjs';
import {
  parseRenderDriftCases,
  renderDriftCasesFromEnv,
  renderDriftSuiteExitCode,
  runRenderDriftCase,
  runRenderDriftSuite,
} from '../src/render-drift.mjs';

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

test('render drift cases default to the local fixture only', () => {
  const cases = renderDriftCasesFromEnv({}, { baseUrl: 'http://127.0.0.1:3000' });

  assert.equal(cases.length, 1);
  assert.equal(cases[0].id, 'fixture-rendered-page');
  assert.equal(cases[0].source, 'fixture');
  assert.equal(cases[0].selectorRequired, true);
});

test('render drift cases parse live JSON only when explicitly enabled', () => {
  const cases = renderDriftCasesFromEnv(
    {
      NAKO_BROWSER_WORKER_LIVE_RENDER_DRIFT: '1',
      NAKO_BROWSER_WORKER_LIVE_RENDER_DRIFT_CASES: JSON.stringify([
        {
          id: 'javbus-sample',
          url: 'https://sensitive.example/SSNI-644',
          selector: '#movie',
          selector_timeout_ms: 1500,
          proxy_policy: 'required',
          min_text_bytes: 100,
          min_html_bytes: 500,
        },
      ]),
    },
    { baseUrl: 'http://127.0.0.1:3000' },
  );

  assert.equal(cases.length, 2);
  assert.deepEqual(cases[1], {
    id: 'javbus-sample',
    source: 'live',
    requestBody: {
      url: 'https://sensitive.example/SSNI-644',
      wait_for: {
        state: 'domcontentloaded',
        selector: '#movie',
        timeout_ms: 1500,
      },
      proxy_policy: 'required',
    },
    selectorRequired: true,
    minHtmlBytes: 500,
    minTextBytes: 100,
    expectedText: [],
  });
});

test('render drift parser rejects unsafe case ids', () => {
  assert.throws(
    () => parseRenderDriftCases(JSON.stringify({ id: '../secret', url: 'https://example.test' })),
    /invalid id/,
  );
});

test('render drift failure reports never echo target URL or selector', async () => {
  const [caseDef] = parseRenderDriftCases(
    JSON.stringify({
      id: 'sensitive-case',
      url: 'https://sensitive.example/SSNI-644',
      selector: '#movie',
    }),
  );

  const report = await runRenderDriftCase(caseDef, {
    baseUrl: 'http://worker.example',
    fetchFn: async () => new Response(
      JSON.stringify({
        status: 'error',
        safe_error_code: 'render_wait_selector_timeout',
        failure_kind: 'selector_timeout',
      }),
      { status: 502, headers: { 'content-type': 'application/json' } },
    ),
  });

  assert.equal(report.status, 'error');
  assert.equal(report.failure_kind, 'selector_timeout');
  assert.equal(report.safe_error_code, 'render_wait_selector_timeout');
  const rendered = JSON.stringify(report);
  assert.doesNotMatch(rendered, /sensitive\.example/);
  assert.doesNotMatch(rendered, /SSNI-644/);
  assert.doesNotMatch(rendered, /#movie/);
});

test('render drift suite passes the default fixture through the worker app', async () => {
  const app = createApp();

  await withApp(app, async (baseUrl) => {
    const suite = await runRenderDriftSuite({ baseUrl, env: {} });

    assert.equal(suite.status, 'ok');
    assert.equal(renderDriftSuiteExitCode(suite), 0);
    assert.equal(suite.case_count, 1);
    assert.equal(suite.reports[0].case_id, 'fixture-rendered-page');
    assert.equal(suite.reports[0].status, 'ok');
    assert.equal(suite.reports[0].checks.selector_wait_satisfied, true);
    assert.equal(suite.reports[0].failure_kind, null);
    assert.ok(suite.reports[0].sizes.html_bytes > 0);
    assert.ok(suite.reports[0].sizes.text_bytes > 0);
  });
});
