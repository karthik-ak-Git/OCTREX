import { NormalizedError, NormalizedErrorCode } from '../types/gateway.js';

export class ErrorNormalizer {
  public static normalize(error: any, providerId: string, modelId?: string): NormalizedError {
    if (!error) {
      return {
        code: 'UNKNOWN_ERROR',
        message: 'An unknown error occurred',
        providerId,
        modelId,
        retryable: false,
      };
    }

    // Standardized NormalizedError check
    if (error.code && error.providerId && typeof error.retryable === 'boolean') {
      return error as NormalizedError;
    }

    const status = error.status || error.httpStatus || error.statusCode || error.response?.status;
    const rawMessage = error.message || error.statusText || String(error);

    let code: NormalizedErrorCode = 'UNKNOWN_ERROR';
    let retryable = false;

    if (status === 400) {
      code = 'INVALID_REQUEST';
      retryable = false;
    } else if (status === 401) {
      code = 'AUTH_FAILURE';
      retryable = false;
    } else if (status === 403) {
      code = 'PERMISSION_DENIED';
      retryable = false;
    } else if (status === 404) {
      code = 'MODEL_NOT_FOUND';
      retryable = false;
    } else if (status === 408 || rawMessage.toLowerCase().includes('timeout')) {
      code = 'TIMEOUT';
      retryable = true;
    } else if (status === 413 || rawMessage.toLowerCase().includes('context') || rawMessage.toLowerCase().includes('too large')) {
      code = 'CONTEXT_EXCEEDED';
      retryable = false;
    } else if (status === 429 || rawMessage.toLowerCase().includes('rate limit') || rawMessage.toLowerCase().includes('quota')) {
      code = 'RATE_LIMITED';
      retryable = true;
    } else if (status >= 500 && status <= 504) {
      code = 'PROVIDER_UNAVAILABLE';
      retryable = true;
    } else if (rawMessage.toLowerCase().includes('disconnect') || rawMessage.toLowerCase().includes('econnreset') || rawMessage.toLowerCase().includes('fetch failed')) {
      code = 'STREAM_DISCONNECTED';
      retryable = true;
    }

    return {
      code,
      message: rawMessage,
      httpStatus: status,
      providerId,
      modelId,
      rawError: error,
      retryable,
    };
  }
}
