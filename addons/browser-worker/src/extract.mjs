import os from 'node:os';
import path from 'node:path';
import { randomUUID } from 'node:crypto';

process.env.CRAWLEE_STORAGE_DIR ??= path.join(os.tmpdir(), 'nako-browser-worker');
process.env.CRAWLEE_PURGE_ON_START ??= 'true';

const { PlaywrightCrawler, ProxyConfiguration, RequestList } = await import('crawlee');

const DEFAULT_WAIT_STATE = 'networkidle';
const ALLOWED_WAIT_STATES = new Set(['load', 'domcontentloaded', 'networkidle']);
const ALLOWED_PROXY_POLICIES = new Set(['default', 'direct', 'required']);

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

export function normalizeRenderOptions(input = {}) {
  const waitFor = normalizeWaitFor(input.wait_for ?? input.waitFor);
  const proxyPolicy = normalizeProxyPolicy(input.proxy_policy ?? input.proxyPolicy);
  const sessionKey = nonEmpty(input.session_key ?? input.sessionKey);

  if (!waitFor || !proxyPolicy) {
    return null;
  }

  return {
    waitFor,
    proxyPolicy,
    ...(sessionKey ? { sessionKey } : {}),
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
    async requestHandler({ page, request }) {
      await waitForPage(page, options.waitFor);
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
