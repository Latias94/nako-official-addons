import os from 'node:os';
import path from 'node:path';
import { randomUUID } from 'node:crypto';

process.env.CRAWLEE_STORAGE_DIR ??= path.join(os.tmpdir(), 'nako-browser-worker');
process.env.CRAWLEE_PURGE_ON_START ??= 'true';

const { PlaywrightCrawler, ProxyConfiguration, RequestList } = await import('crawlee');

const DEFAULT_WAIT_STATE = 'networkidle';
const ALLOWED_WAIT_STATES = new Set(['load', 'domcontentloaded', 'networkidle']);
const ALLOWED_PROXY_POLICIES = new Set(['default', 'direct', 'required']);
const ALLOWED_ACTION_TYPES = new Set(['check', 'click']);
const HEADER_NAME_PATTERN = /^[!#$%&'*+.^_`|~0-9A-Za-z-]+$/;

function nonEmpty(value) {
  return typeof value === 'string' && value.trim() ? value.trim() : null;
}

function proxyUrlsFromEnv(env = process.env) {
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

function normalizeWaitFor(value) {
  if (typeof value === 'string') {
    const state = value.trim().toLowerCase();
    return ALLOWED_WAIT_STATES.has(state) ? { state } : null;
  }

  if (!value || typeof value !== 'object') {
    return { state: DEFAULT_WAIT_STATE };
  }

  const state = nonEmpty(value.state)?.toLowerCase() ?? DEFAULT_WAIT_STATE;
  if (!ALLOWED_WAIT_STATES.has(state)) {
    return null;
  }

  const selector = nonEmpty(value.selector);
  const timeoutMs = Number.isInteger(value.timeout_ms) && value.timeout_ms > 0
    ? value.timeout_ms
    : undefined;

  return {
    state,
    ...(selector ? { selector } : {}),
    ...(timeoutMs ? { timeoutMs } : {}),
  };
}

function normalizeProxyPolicy(value) {
  const policy = nonEmpty(value)?.toLowerCase() ?? 'default';
  return ALLOWED_PROXY_POLICIES.has(policy) ? policy : null;
}

function normalizeAction(value) {
  if (!value || typeof value !== 'object') {
    return null;
  }

  const type = nonEmpty(value.type)?.toLowerCase();
  const selector = nonEmpty(value.selector);
  if (!type || !ALLOWED_ACTION_TYPES.has(type) || !selector) {
    return null;
  }

  const waitForInput = value.wait_for ?? value.waitFor;
  const waitFor = waitForInput === undefined ? undefined : normalizeWaitFor(waitForInput);
  if (waitForInput !== undefined && !waitFor) {
    return null;
  }

  return {
    type,
    selector,
    ...(value.optional === true ? { optional: true } : {}),
    ...(waitFor ? { waitFor } : {}),
  };
}

function normalizeActions(value) {
  if (value === undefined) {
    return [];
  }
  if (!Array.isArray(value) || value.length > 8) {
    return null;
  }

  const actions = [];
  for (const action of value) {
    const normalized = normalizeAction(action);
    if (!normalized) {
      return null;
    }
    actions.push(normalized);
  }
  return actions;
}

function normalizeHeaders(value) {
  if (value === undefined) {
    return {};
  }
  if (!value || typeof value !== 'object' || Array.isArray(value)) {
    return null;
  }

  const entries = Object.entries(value);
  if (entries.length > 16) {
    return null;
  }

  const headers = {};
  for (const [rawName, rawValue] of entries) {
    const name = nonEmpty(rawName)?.toLowerCase();
    if (!name || !HEADER_NAME_PATTERN.test(name) || typeof rawValue !== 'string') {
      return null;
    }
    const headerValue = nonEmpty(rawValue);
    if (!headerValue) {
      continue;
    }
    headers[name] = headerValue;
  }
  return headers;
}

export function normalizeRenderOptions(input = {}) {
  const waitFor = normalizeWaitFor(input.wait_for ?? input.waitFor);
  const proxyPolicy = normalizeProxyPolicy(input.proxy_policy ?? input.proxyPolicy);
  const sessionKey = nonEmpty(input.session_key ?? input.sessionKey);
  const actions = normalizeActions(input.actions);
  const headers = normalizeHeaders(input.headers);

  if (!waitFor || !proxyPolicy || !actions || !headers) {
    return null;
  }

  return {
    waitFor,
    proxyPolicy,
    ...(sessionKey ? { sessionKey } : {}),
    ...(Object.keys(headers).length ? { headers } : {}),
    ...(actions.length ? { actions } : {}),
  };
}

export function browserWorkerProxyFacts(env = process.env) {
  const proxyUrls = proxyUrlsFromEnv(env);
  return {
    proxy_configured: proxyUrls.length > 0,
    proxy_count: proxyUrls.length,
  };
}

function normalizeWhitespace(value) {
  return value.replace(/\s+/g, ' ').trim();
}

function proxyConfigurationFor(options, env) {
  if (options.proxyPolicy === 'direct') {
    return undefined;
  }

  const proxyUrls = proxyUrlsFromEnv(env);
  if (proxyUrls.length === 0) {
    if (options.proxyPolicy === 'required') {
      throw new Error('Proxy policy requires a configured browser-worker proxy');
    }
    return undefined;
  }

  return new ProxyConfiguration({ proxyUrls });
}

async function waitForPage(page, waitFor) {
  await page.waitForLoadState(waitFor.state).catch(() => {});
  if (waitFor.selector) {
    await page.waitForSelector(waitFor.selector, {
      timeout: waitFor.timeoutMs ?? 5000,
    });
  }
}

async function runPageActions(page, actions = []) {
  for (const action of actions) {
    const locator = page.locator(action.selector).first();
    if (action.optional && (await locator.count()) === 0) {
      continue;
    }

    const timeout = action.waitFor?.timeoutMs ?? 5000;
    if (!action.optional) {
      await locator.waitFor({ timeout });
    }

    if (action.type === 'check') {
      await locator.check({ timeout });
    } else if (action.type === 'click') {
      await locator.click({ timeout });
    }

    if (action.waitFor) {
      await waitForPage(page, action.waitFor);
    }
  }
}

function cookiesFromHeader(cookieHeader, url) {
  return cookieHeader
    .split(';')
    .map((part) => part.trim())
    .filter(Boolean)
    .map((part) => {
      const separator = part.indexOf('=');
      if (separator <= 0) {
        return null;
      }
      return {
        name: part.slice(0, separator).trim(),
        value: part.slice(separator + 1).trim(),
        url,
      };
    })
    .filter((cookie) => cookie && cookie.name);
}

async function applyRequestHeaders(page, request) {
  if (!request.userData.headers) {
    return;
  }

  const headers = { ...request.userData.headers };
  const cookieHeader = headers.cookie;
  delete headers.cookie;

  if (cookieHeader) {
    const cookies = cookiesFromHeader(cookieHeader, request.url);
    if (cookies.length) {
      await page.context().addCookies(cookies);
    }
  }

  if (Object.keys(headers).length) {
    await page.setExtraHTTPHeaders(headers);
  }
}

export async function extractRenderedPage(url, rawOptions = {}, env = process.env) {
  const options = normalizeRenderOptions(rawOptions);
  if (!options) {
    throw new Error('Invalid browser worker render options');
  }

  const requestList = await RequestList.open(`nako-browser-worker-${randomUUID()}`, [
    {
      url,
      userData: {
        ...(options.sessionKey ? { sessionKey: options.sessionKey } : {}),
        ...(options.headers ? { headers: options.headers } : {}),
      },
    },
  ]);
  let extracted = null;
  const proxyConfiguration = proxyConfigurationFor(options, env);
  const useSessionPool = Boolean(options.sessionKey);

  const crawler = new PlaywrightCrawler({
    requestList,
    maxRequestsPerCrawl: 1,
    ...(proxyConfiguration ? { proxyConfiguration } : {}),
    ...(useSessionPool ? { useSessionPool: true, persistCookiesPerSession: true } : {}),
    preNavigationHooks: [
      async ({ page, request }) => {
        await applyRequestHeaders(page, request);
      },
    ],
    async requestHandler({ page, request }) {
      await waitForPage(page, options.waitFor);
      await runPageActions(page, options.actions);
      const title = normalizeWhitespace(await page.title());
      const renderedText = normalizeWhitespace(
        await page.locator('body').innerText({ timeout: 5000 }),
      );
      const html = await page.content();

      extracted = {
        url: page.url() || request.url,
        title,
        html,
        text: renderedText,
        rendered_text: renderedText,
        excerpt: renderedText.slice(0, 240),
      };
    },
  });

  await crawler.run();

  if (!extracted) {
    throw new Error(`No rendered page was extracted from ${url}`);
  }

  return extracted;
}
