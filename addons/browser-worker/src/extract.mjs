import os from 'node:os';
import path from 'node:path';
import { randomUUID } from 'node:crypto';

process.env.CRAWLEE_STORAGE_DIR ??= path.join(os.tmpdir(), 'nako-browser-worker');
process.env.CRAWLEE_PURGE_ON_START ??= 'true';

const { PlaywrightCrawler, RequestList } = await import('crawlee');

function normalizeWhitespace(value) {
  return value.replace(/\s+/g, ' ').trim();
}

export async function extractRenderedPage(url) {
  const requestList = await RequestList.open(`nako-browser-worker-${randomUUID()}`, [
    { url },
  ]);
  let extracted = null;

  const crawler = new PlaywrightCrawler({
    requestList,
    maxRequestsPerCrawl: 1,
    async requestHandler({ page, request }) {
      await page.waitForLoadState('networkidle').catch(() => {});
      const title = normalizeWhitespace(await page.title());
      const renderedText = normalizeWhitespace(
        await page.locator('body').innerText({ timeout: 5000 }),
      );

      extracted = {
        url: page.url() || request.url,
        title,
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
