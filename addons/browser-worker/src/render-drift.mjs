const SUITE_SCHEMA = 'nako.browser-worker.render-drift.suite.v1';
const CASE_SCHEMA = 'nako.browser-worker.render-drift.case-health.v1';
const CASE_ID_PATTERN = /^[A-Za-z0-9_.:-]{1,80}$/;

function nonEmpty(value) {
  return typeof value === 'string' && value.trim() ? value.trim() : null;
}

function truthyEnv(value) {
  return ['1', 'true', 'yes', 'on'].includes(String(value ?? '').trim().toLowerCase());
}

function envFlagEnabled(value, defaultValue) {
  const normalized = String(value ?? '').trim().toLowerCase();
  if (!normalized) {
    return defaultValue;
  }
  return ['1', 'true', 'yes', 'on'].includes(normalized);
}

function byteLength(value) {
  return Buffer.byteLength(value ?? '', 'utf8');
}

function positiveInt(value, fallback) {
  if (value === undefined || value === null || value === '') {
    return fallback;
  }
  const parsed = Number(value);
  return Number.isInteger(parsed) && parsed >= 0 ? parsed : fallback;
}

function stringList(value) {
  if (value === undefined || value === null) {
    return [];
  }
  const raw = Array.isArray(value) ? value : [value];
  return raw.map(nonEmpty).filter(Boolean);
}

function parseJson(value, label) {
  try {
    return JSON.parse(value);
  } catch {
    throw new Error(`${label} must be valid JSON`);
  }
}

function responseJsonOrNull(value) {
  try {
    return JSON.parse(value);
  } catch {
    return null;
  }
}

function safeString(value, fallback) {
  return typeof value === 'string' && value.trim() ? value.trim() : fallback;
}

function safeProxyPolicy(value) {
  const policy = safeString(value, 'default').toLowerCase();
  return ['default', 'direct', 'required'].includes(policy) ? policy : 'default';
}

export function liveRenderDriftEnabled(env = process.env) {
  return truthyEnv(env.NAKO_BROWSER_WORKER_LIVE_RENDER_DRIFT);
}

export function defaultFixtureRenderDriftCase(baseUrl) {
  return {
    id: 'fixture-rendered-page',
    source: 'fixture',
    requestBody: {
      url: new URL('/fixtures/rendered-page', baseUrl).toString(),
      wait_for: {
        state: 'domcontentloaded',
        selector: '#status',
        timeout_ms: 5000,
      },
    },
    selectorRequired: true,
    minHtmlBytes: 100,
    minTextBytes: 10,
    expectedText: ['rendered by JavaScript'],
  };
}

function normalizeWaitFor(value, selector, timeoutMs) {
  if (value === undefined || value === null) {
    if (!selector) {
      return undefined;
    }
    return {
      state: 'domcontentloaded',
      selector,
      ...(timeoutMs ? { timeout_ms: timeoutMs } : {}),
    };
  }

  if (typeof value === 'string') {
    return selector
      ? { state: value, selector, ...(timeoutMs ? { timeout_ms: timeoutMs } : {}) }
      : value;
  }

  if (!value || typeof value !== 'object' || Array.isArray(value)) {
    throw new Error('render drift wait_for must be a string or object');
  }

  return {
    ...value,
    ...(selector && !value.selector ? { selector } : {}),
    ...(timeoutMs && !value.timeout_ms && !value.timeoutMs ? { timeout_ms: timeoutMs } : {}),
  };
}

function normalizeRenderDriftCase(value, index) {
  if (!value || typeof value !== 'object' || Array.isArray(value)) {
    throw new Error(`render drift case ${index} must be an object`);
  }

  const id = nonEmpty(value.id ?? value.case_id);
  if (!id || !CASE_ID_PATTERN.test(id)) {
    throw new Error(`render drift case ${index} has an invalid id`);
  }

  const url = nonEmpty(value.url);
  if (!url) {
    throw new Error(`render drift case ${index} is missing url`);
  }

  const selector = nonEmpty(value.selector ?? value.wait_selector);
  const selectorTimeoutMs = positiveInt(
    value.selector_timeout_ms ?? value.selectorTimeoutMs,
    undefined,
  );
  const waitFor = normalizeWaitFor(value.wait_for ?? value.waitFor, selector, selectorTimeoutMs);

  const requestBody = {
    url,
    ...(waitFor !== undefined ? { wait_for: waitFor } : {}),
    ...(value.proxy_policy !== undefined
      ? { proxy_policy: value.proxy_policy }
      : value.proxyPolicy !== undefined
        ? { proxy_policy: value.proxyPolicy }
        : {}),
    ...(value.render_timeout_ms !== undefined
      ? { render_timeout_ms: value.render_timeout_ms }
      : value.renderTimeoutMs !== undefined
        ? { render_timeout_ms: value.renderTimeoutMs }
        : {}),
    ...(value.session_key !== undefined
      ? { session_key: value.session_key }
      : value.sessionKey !== undefined
        ? { session_key: value.sessionKey }
        : {}),
    ...(value.headers !== undefined ? { headers: value.headers } : {}),
    ...(value.actions !== undefined ? { actions: value.actions } : {}),
  };

  return {
    id,
    source: 'live',
    requestBody,
    selectorRequired: Boolean(
      selector || (waitFor && typeof waitFor === 'object' && nonEmpty(waitFor.selector)),
    ),
    minHtmlBytes: positiveInt(value.min_html_bytes ?? value.minHtmlBytes, 1),
    minTextBytes: positiveInt(value.min_text_bytes ?? value.minTextBytes, 1),
    expectedText: stringList(value.text_contains ?? value.textContains),
  };
}

export function parseRenderDriftCases(value) {
  if (!nonEmpty(value)) {
    return [];
  }

  const parsed = parseJson(value, 'NAKO_BROWSER_WORKER_LIVE_RENDER_DRIFT_CASES');
  const cases = Array.isArray(parsed) ? parsed : [parsed];
  return cases.map(normalizeRenderDriftCase);
}

export function renderDriftCasesFromEnv(env = process.env, { baseUrl }) {
  const cases = [];
  const includeFixture = envFlagEnabled(env.NAKO_BROWSER_WORKER_RENDER_DRIFT_INCLUDE_FIXTURE, true);
  if (includeFixture) {
    cases.push(defaultFixtureRenderDriftCase(baseUrl));
  }

  if (liveRenderDriftEnabled(env)) {
    cases.push(...parseRenderDriftCases(env.NAKO_BROWSER_WORKER_LIVE_RENDER_DRIFT_CASES));
  }

  return cases;
}

function okCaseReport(caseDef, responseStatus, body, durationMs) {
  const htmlBytes = byteLength(body.html);
  const textBytes = byteLength(body.text);
  const excerptBytes = byteLength(body.excerpt);
  const expectedTextPresent = caseDef.expectedText.every(
    (value) => String(body.text ?? '').includes(value) || String(body.html ?? '').includes(value),
  );
  const checks = {
    http_ok: true,
    html_nonempty: htmlBytes > 0,
    text_nonempty: textBytes > 0,
    min_html_bytes_met: htmlBytes >= caseDef.minHtmlBytes,
    min_text_bytes_met: textBytes >= caseDef.minTextBytes,
    expected_text_present: expectedTextPresent,
    selector_wait_satisfied: caseDef.selectorRequired ? true : null,
  };
  const status = Object.values(checks).every((value) => value !== false) ? 'ok' : 'drift';

  return {
    schema: CASE_SCHEMA,
    case_id: caseDef.id,
    source: caseDef.source,
    status,
    http_status: responseStatus,
    failure_kind: null,
    safe_error_code: null,
    proxy_policy: safeProxyPolicy(caseDef.requestBody.proxy_policy),
    selector_required: caseDef.selectorRequired,
    title_present: Boolean(nonEmpty(body.title)),
    sizes: {
      html_bytes: htmlBytes,
      text_bytes: textBytes,
      excerpt_bytes: excerptBytes,
    },
    checks,
    duration_ms: durationMs,
  };
}

function failedCaseReport(caseDef, responseStatus, body, durationMs) {
  return {
    schema: CASE_SCHEMA,
    case_id: caseDef.id,
    source: caseDef.source,
    status: 'error',
    http_status: responseStatus,
    failure_kind: safeString(body?.failure_kind, responseStatus ? 'http_error' : 'transport_error'),
    safe_error_code: safeString(
      body?.safe_error_code,
      responseStatus ? 'render_drift_http_error' : 'render_drift_transport_error',
    ),
    proxy_policy: safeProxyPolicy(caseDef.requestBody.proxy_policy),
    selector_required: caseDef.selectorRequired,
    title_present: false,
    sizes: {
      html_bytes: 0,
      text_bytes: 0,
      excerpt_bytes: 0,
    },
    checks: {
      http_ok: false,
      html_nonempty: false,
      text_nonempty: false,
      min_html_bytes_met: false,
      min_text_bytes_met: false,
      expected_text_present: false,
      selector_wait_satisfied: caseDef.selectorRequired ? false : null,
    },
    duration_ms: durationMs,
  };
}

export async function runRenderDriftCase(caseDef, { baseUrl, fetchFn = fetch } = {}) {
  const startedAt = Date.now();
  try {
    const response = await fetchFn(new URL('/render', baseUrl), {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify(caseDef.requestBody),
    });
    const responseText = await response.text();
    const body = responseJsonOrNull(responseText);
    const durationMs = Date.now() - startedAt;

    if (!response.ok || !body || body.status !== 'ok') {
      return failedCaseReport(caseDef, response.status, body, durationMs);
    }

    return okCaseReport(caseDef, response.status, body, durationMs);
  } catch {
    return failedCaseReport(caseDef, null, null, Date.now() - startedAt);
  }
}

export async function runRenderDriftSuite({
  baseUrl,
  env = process.env,
  fetchFn = fetch,
} = {}) {
  const cases = renderDriftCasesFromEnv(env, { baseUrl });
  const reports = [];
  for (const caseDef of cases) {
    reports.push(await runRenderDriftCase(caseDef, { baseUrl, fetchFn }));
  }
  const failedCaseCount = reports.filter((report) => report.status !== 'ok').length;

  return {
    schema: SUITE_SCHEMA,
    status: failedCaseCount === 0 ? 'ok' : 'drift',
    live_enabled: liveRenderDriftEnabled(env),
    case_count: reports.length,
    failed_case_count: failedCaseCount,
    reports,
  };
}

export function renderDriftSuiteExitCode(suite) {
  return suite.status === 'ok' ? 0 : 1;
}
