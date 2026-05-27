export class RenderWorkerError extends Error {
  constructor({
    message,
    safeErrorCode,
    failureKind,
    status = 502,
    cause,
  }) {
    super(message, { cause });
    this.name = 'RenderWorkerError';
    this.safeErrorCode = safeErrorCode;
    this.failureKind = failureKind;
    this.status = status;
  }
}

export function renderWorkerError(input) {
  return new RenderWorkerError(input);
}

export function errorResponseFromError(error, {
  errorCode = 'render_failed',
  fallbackSafeErrorCode = 'rendered_page_render_failed',
  fallbackFailureKind = 'render_failed',
} = {}) {
  if (error instanceof RenderWorkerError) {
    return {
      status: error.status,
      body: {
        status: 'error',
        error: errorCode,
        safe_error_code: error.safeErrorCode,
        failure_kind: error.failureKind,
      },
    };
  }

  return {
    status: 502,
    body: {
      status: 'error',
      error: errorCode,
      safe_error_code: fallbackSafeErrorCode,
      failure_kind: fallbackFailureKind,
    },
  };
}
