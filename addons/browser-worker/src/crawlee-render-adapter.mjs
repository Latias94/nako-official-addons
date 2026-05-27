import { randomUUID } from 'node:crypto';
import os from 'node:os';
import path from 'node:path';

import { renderWorkerError, RenderWorkerError } from './render-errors.mjs';
import { proxyUrlsFromEnv } from './render-safety.mjs';
import {
  applyRequestHeaders,
  extractRenderedSnapshot,
  runPageActions,
  waitForPage,
} from './render-runtime.mjs';

process.env.CRAWLEE_STORAGE_DIR ??= path.join(os.tmpdir(), 'nako-browser-worker');
process.env.CRAWLEE_PURGE_ON_START ??= 'true';

const { PlaywrightCrawler, ProxyConfiguration, RequestList } = await import('crawlee');

function proxyConfigurationFor(options, env) {
  if (options.proxyPolicy === 'direct') {
    return undefined;
  }

  const proxyUrls = proxyUrlsFromEnv(env);
  if (proxyUrls.length === 0) {
    if (options.proxyPolicy === 'required') {
      throw renderWorkerError({
        message: 'Proxy policy requires a configured browser-worker proxy',
        safeErrorCode: 'proxy_required',
        failureKind: 'operator_action',
        status: 502,
      });
    }
    return undefined;
  }

  return new ProxyConfiguration({ proxyUrls });
}

export class CrawleeRenderAdapter {
  async renderPage({ url, options, env }) {
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
    const timeoutSecs = Math.ceil(options.renderTimeoutMs / 1000);

    const crawler = new PlaywrightCrawler({
      requestList,
      maxRequestsPerCrawl: 1,
      requestHandlerTimeoutSecs: timeoutSecs,
      navigationTimeoutSecs: timeoutSecs,
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
        extracted = await extractRenderedSnapshot(page, request);
      },
    });

    try {
      await crawler.run();
    } catch (error) {
      if (error instanceof RenderWorkerError) {
        throw error;
      }
      throw renderWorkerError({
        message: 'Rendered page browser execution failed',
        safeErrorCode: 'render_browser_failed',
        failureKind: error.message?.toLowerCase().includes('timeout')
          ? 'render_timeout'
          : 'browser_execution_failed',
        status: 502,
        cause: error,
      });
    }

    if (!extracted) {
      throw renderWorkerError({
        message: 'No rendered page was extracted',
        safeErrorCode: 'render_extraction_empty',
        failureKind: 'extraction_failed',
        status: 502,
      });
    }

    return extracted;
  }
}
