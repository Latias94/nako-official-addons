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
process.env.CRAWLEE_LOG_LEVEL ??= 'OFF';

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

function safeRenderErrorFromCrawlerError(error) {
  if (error instanceof RenderWorkerError) {
    return error;
  }

  const message = String(error?.message ?? '').toLowerCase();
  if (message.includes('403') || message.includes('blocked')) {
    return renderWorkerError({
      message: 'Rendered page request was blocked',
      safeErrorCode: 'render_request_blocked',
      failureKind: 'auth_or_forbidden',
      status: 502,
      cause: error,
    });
  }
  if (message.includes('timeout')) {
    return renderWorkerError({
      message: 'Rendered page request timed out',
      safeErrorCode: 'render_timeout',
      failureKind: 'render_timeout',
      status: 502,
      cause: error,
    });
  }
  if (
    message.includes('err_connection_closed')
    || message.includes('err_tunnel_connection_failed')
    || message.includes('err_proxy')
    || message.includes('net::')
  ) {
    return renderWorkerError({
      message: 'Rendered page network request failed',
      safeErrorCode: 'render_network_failed',
      failureKind: 'provider_error',
      status: 502,
      cause: error,
    });
  }

  return renderWorkerError({
    message: 'Rendered page browser execution failed',
    safeErrorCode: 'render_browser_failed',
    failureKind: 'browser_execution_failed',
    status: 502,
    cause: error,
  });
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
    let failedCrawlerError = null;

    const crawler = new PlaywrightCrawler({
      requestList,
      maxRequestsPerCrawl: 1,
      maxRequestRetries: 0,
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
      failedRequestHandler(_context, error) {
        failedCrawlerError = error;
      },
    });

    try {
      await crawler.run();
    } catch (error) {
      if (error instanceof RenderWorkerError) {
        throw error;
      }
      throw safeRenderErrorFromCrawlerError(error);
    }

    if (!extracted) {
      if (failedCrawlerError) {
        throw safeRenderErrorFromCrawlerError(failedCrawlerError);
      }
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
