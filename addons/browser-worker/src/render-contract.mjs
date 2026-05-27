import { renderWorkerError } from './render-errors.mjs';
import { normalizeRenderUrl, renderSafetyPolicyFromEnv, byteLength } from './render-safety.mjs';

const DEFAULT_WAIT_STATE = 'networkidle';
const ALLOWED_WAIT_STATES = new Set(['load', 'domcontentloaded', 'networkidle']);
const DEFAULT_SELECTOR_STATE = 'attached';
const ALLOWED_SELECTOR_STATES = new Set(['attached', 'visible']);
const ALLOWED_PROXY_POLICIES = new Set(['default', 'direct', 'required']);
const ALLOWED_ACTION_TYPES = new Set(['check', 'click']);
const HEADER_NAME_PATTERN = /^[!#$%&'*+.^_`|~0-9A-Za-z-]+$/;

function nonEmpty(value) {
  return typeof value === 'string' && value.trim() ? value.trim() : null;
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
  const timeoutInput = value.timeout_ms ?? value.timeoutMs;
  const timeoutMs = Number.isInteger(timeoutInput) && timeoutInput > 0
    ? timeoutInput
    : undefined;
  const selectorState = nonEmpty(value.selector_state ?? value.selectorState)?.toLowerCase()
    ?? DEFAULT_SELECTOR_STATE;
  if (selector && !ALLOWED_SELECTOR_STATES.has(selectorState)) {
    return null;
  }

  return {
    state,
    ...(selector ? { selector, selectorState } : {}),
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

function normalizeActions(value, policy) {
  if (value === undefined) {
    return [];
  }
  if (!Array.isArray(value) || value.length > policy.maxActions) {
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

function normalizeHeaders(value, policy) {
  if (value === undefined) {
    return {};
  }
  if (!value || typeof value !== 'object' || Array.isArray(value)) {
    return null;
  }

  const entries = Object.entries(value);
  if (entries.length > policy.maxHeaders) {
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
    if (byteLength(headerValue) > policy.maxHeaderValueBytes) {
      return null;
    }
    headers[name] = headerValue;
  }
  return headers;
}

function normalizeRenderTimeoutMs(input, policy) {
  const value = input.render_timeout_ms ?? input.renderTimeoutMs;
  if (value === undefined) {
    return policy.defaultRenderTimeoutMs;
  }
  if (!Number.isInteger(value) || value <= 0 || value > policy.maxRenderTimeoutMs) {
    return null;
  }
  return value;
}

export function normalizeRenderOptions(input = {}, env = process.env) {
  const policy = renderSafetyPolicyFromEnv(env);
  const waitFor = normalizeWaitFor(input.wait_for ?? input.waitFor);
  const proxyPolicy = normalizeProxyPolicy(input.proxy_policy ?? input.proxyPolicy);
  const sessionKey = nonEmpty(input.session_key ?? input.sessionKey);
  const actions = normalizeActions(input.actions, policy);
  const headers = normalizeHeaders(input.headers, policy);
  const renderTimeoutMs = normalizeRenderTimeoutMs(input, policy);

  if (!waitFor || !proxyPolicy || !actions || !headers || !renderTimeoutMs) {
    return null;
  }

  return {
    waitFor,
    proxyPolicy,
    renderTimeoutMs,
    ...(sessionKey ? { sessionKey } : {}),
    ...(Object.keys(headers).length ? { headers } : {}),
    ...(actions.length ? { actions } : {}),
  };
}

export function parseRenderRequestBody(body, env = process.env) {
  const url = normalizeRenderUrl(body?.url);
  const options = normalizeRenderOptions(body, env);
  if (!options) {
    throw renderWorkerError({
      message: 'Invalid browser worker render options',
      safeErrorCode: 'invalid_render_options',
      failureKind: 'invalid_options',
      status: 400,
    });
  }

  return {
    url,
    options,
  };
}
