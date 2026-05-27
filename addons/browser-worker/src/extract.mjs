import { CrawleeRenderAdapter } from './crawlee-render-adapter.mjs';
import { parseRenderRequestBody, normalizeRenderOptions } from './render-contract.mjs';
import { RenderRuntime } from './render-runtime.mjs';
import { browserWorkerProxyFacts } from './render-safety.mjs';

export { browserWorkerProxyFacts, normalizeRenderOptions };

export async function extractRenderedPage(url, rawOptions = {}, env = process.env) {
  const renderRequest = parseRenderRequestBody({ ...rawOptions, url }, env);
  const runtime = new RenderRuntime({
    adapter: new CrawleeRenderAdapter(),
    env,
  });
  return runtime.render(renderRequest);
}
