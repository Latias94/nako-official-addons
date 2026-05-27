import { renderWorkerError } from './render-errors.mjs';
import {
  assertRenderedPageWithinPolicy,
  renderSafetyPolicyFromEnv,
} from './render-safety.mjs';

export function normalizeWhitespace(value) {
  return value.replace(/\s+/g, ' ').trim();
}

export async function waitForPage(page, waitFor) {
  await page.waitForLoadState(waitFor.state).catch(() => {});
  if (waitFor.selector) {
    try {
      await page.waitForSelector(waitFor.selector, {
        timeout: waitFor.timeoutMs ?? 5000,
      });
    } catch (error) {
      throw renderWorkerError({
        message: 'Rendered page wait selector timed out',
        safeErrorCode: 'render_wait_selector_timeout',
        failureKind: 'selector_timeout',
        status: 502,
        cause: error,
      });
    }
  }
}

export async function runPageActions(page, actions = []) {
  for (const action of actions) {
    const locator = page.locator(action.selector).first();
    if (action.optional && (await locator.count()) === 0) {
      continue;
    }

    const timeout = action.waitFor?.timeoutMs ?? 5000;
    try {
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
    } catch (error) {
      throw renderWorkerError({
        message: 'Rendered page action failed',
        safeErrorCode: 'render_action_failed',
        failureKind: 'action_failed',
        status: 502,
        cause: error,
      });
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

export async function applyRequestHeaders(page, request) {
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

export async function extractRenderedSnapshot(page, request) {
  try {
    const title = normalizeWhitespace(await page.title());
    const renderedText = normalizeWhitespace(
      await page.locator('body').innerText({ timeout: 5000 }),
    );
    const html = await page.content();

    return {
      url: page.url() || request.url,
      title,
      html,
      text: renderedText,
      rendered_text: renderedText,
      excerpt: renderedText.slice(0, 240),
    };
  } catch (error) {
    throw renderWorkerError({
      message: 'Rendered page extraction failed',
      safeErrorCode: 'render_extraction_failed',
      failureKind: 'extraction_failed',
      status: 502,
      cause: error,
    });
  }
}

export class RenderRuntime {
  constructor({ adapter, env = process.env, policy = renderSafetyPolicyFromEnv(env) }) {
    this.adapter = adapter;
    this.env = env;
    this.policy = policy;
  }

  async render({ url, options }) {
    const rendered = await this.adapter.renderPage({
      url,
      options,
      env: this.env,
      policy: this.policy,
    });
    assertRenderedPageWithinPolicy(rendered, this.policy);
    return rendered;
  }
}
