import { renderWorkerError } from './render-errors.mjs';

const DEFAULT_RENDER_TIMEOUT_MS = 30_000;
const MAX_RENDER_TIMEOUT_MS = 120_000;
const DEFAULT_MAX_HTML_BYTES = 8 * 1024 * 1024;
const DEFAULT_MAX_TEXT_BYTES = 1024 * 1024;
const DEFAULT_MAX_ACTIONS = 8;
const DEFAULT_MAX_HEADERS = 16;
const DEFAULT_MAX_HEADER_VALUE_BYTES = 8 * 1024;
const ALLOWED_URL_PROTOCOLS = new Set(['http:', 'https:']);

function nonEmpty(value) {
  return typeof value === 'string' && value.trim() ? value.trim() : null;
}

function positiveIntFromEnv(value, fallback, max = Number.MAX_SAFE_INTEGER) {
  const parsed = Number.parseInt(String(value ?? ''), 10);
  if (!Number.isInteger(parsed) || parsed <= 0) {
    return fallback;
  }
  return Math.min(parsed, max);
}

export function renderSafetyPolicyFromEnv(env = process.env) {
  return {
    defaultRenderTimeoutMs: positiveIntFromEnv(
      env.NAKO_BROWSER_WORKER_RENDER_TIMEOUT_MS,
      DEFAULT_RENDER_TIMEOUT_MS,
      MAX_RENDER_TIMEOUT_MS,
    ),
    maxRenderTimeoutMs: positiveIntFromEnv(
      env.NAKO_BROWSER_WORKER_MAX_RENDER_TIMEOUT_MS,
      MAX_RENDER_TIMEOUT_MS,
      MAX_RENDER_TIMEOUT_MS,
    ),
    maxHtmlBytes: positiveIntFromEnv(
      env.NAKO_BROWSER_WORKER_MAX_HTML_BYTES,
      DEFAULT_MAX_HTML_BYTES,
    ),
    maxTextBytes: positiveIntFromEnv(
      env.NAKO_BROWSER_WORKER_MAX_TEXT_BYTES,
      DEFAULT_MAX_TEXT_BYTES,
    ),
    maxActions: positiveIntFromEnv(
      env.NAKO_BROWSER_WORKER_MAX_ACTIONS,
      DEFAULT_MAX_ACTIONS,
      DEFAULT_MAX_ACTIONS,
    ),
    maxHeaders: positiveIntFromEnv(
      env.NAKO_BROWSER_WORKER_MAX_HEADERS,
      DEFAULT_MAX_HEADERS,
      DEFAULT_MAX_HEADERS,
    ),
    maxHeaderValueBytes: positiveIntFromEnv(
      env.NAKO_BROWSER_WORKER_MAX_HEADER_VALUE_BYTES,
      DEFAULT_MAX_HEADER_VALUE_BYTES,
    ),
  };
}

export function normalizeRenderUrl(value) {
  const rawUrl = nonEmpty(value);
  if (!rawUrl) {
    throw renderWorkerError({
      message: 'Missing render URL',
      safeErrorCode: 'missing_url',
      failureKind: 'invalid_request',
      status: 400,
    });
  }

  let parsed;
  try {
    parsed = new URL(rawUrl);
  } catch {
    throw renderWorkerError({
      message: 'Invalid render URL',
      safeErrorCode: 'invalid_render_url',
      failureKind: 'invalid_request',
      status: 400,
    });
  }

  if (!ALLOWED_URL_PROTOCOLS.has(parsed.protocol)) {
    throw renderWorkerError({
      message: 'Unsupported render URL scheme',
      safeErrorCode: 'invalid_render_url_scheme',
      failureKind: 'invalid_request',
      status: 400,
    });
  }

  if (parsed.username || parsed.password) {
    throw renderWorkerError({
      message: 'Render URL must not include credentials',
      safeErrorCode: 'invalid_render_url_credentials',
      failureKind: 'invalid_request',
      status: 400,
    });
  }

  return parsed.toString();
}

export function proxyUrlsFromEnv(env = process.env) {
  const urls = [];
  const single = nonEmpty(env.NAKO_BROWSER_WORKER_PROXY_URL);
  if (single) {
    urls.push(single);
  }

  const list = nonEmpty(env.NAKO_BROWSER_WORKER_PROXY_LIST);
  if (list) {
    for (const value of list.split(/[\n,]/)) {
      const proxyUrl = nonEmpty(value);
      if (proxyUrl && !urls.includes(proxyUrl)) {
        urls.push(proxyUrl);
      }
    }
  }

  return urls;
}

export function browserWorkerProxyFacts(env = process.env) {
  const proxyUrls = proxyUrlsFromEnv(env);
  return {
    proxy_configured: proxyUrls.length > 0,
    proxy_count: proxyUrls.length,
  };
}

export function byteLength(value) {
  return Buffer.byteLength(value ?? '', 'utf8');
}

export function assertRenderedPageWithinPolicy(rendered, policy) {
  if (byteLength(rendered.html) > policy.maxHtmlBytes) {
    throw renderWorkerError({
      message: 'Rendered HTML exceeded browser-worker response budget',
      safeErrorCode: 'rendered_html_too_large',
      failureKind: 'response_too_large',
      status: 502,
    });
  }

  if (byteLength(rendered.text) > policy.maxTextBytes) {
    throw renderWorkerError({
      message: 'Rendered text exceeded browser-worker response budget',
      safeErrorCode: 'rendered_text_too_large',
      failureKind: 'response_too_large',
      status: 502,
    });
  }
}
