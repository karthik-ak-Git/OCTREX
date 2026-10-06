import { test } from 'node:test';
import assert from 'node:assert/strict';
import { ErrorNormalizer } from '../core/gateway/errorNormalizer.js';

test('ErrorNormalizer converts HTTP error status codes into normalized error objects', () => {
  const err429 = ErrorNormalizer.normalize({ status: 429, message: 'Too many requests' }, 'test-provider');
  assert.equal(err429.code, 'RATE_LIMITED');
  assert.equal(err429.retryable, true);

  const err401 = ErrorNormalizer.normalize({ status: 401, message: 'Unauthorized key' }, 'test-provider');
  assert.equal(err401.code, 'AUTH_FAILURE');
  assert.equal(err401.retryable, false);

  const err413 = ErrorNormalizer.normalize({ status: 413, message: 'Request entity too large' }, 'test-provider');
  assert.equal(err413.code, 'CONTEXT_EXCEEDED');
  assert.equal(err413.retryable, false);

  const err500 = ErrorNormalizer.normalize({ status: 500, message: 'Internal server failure' }, 'test-provider');
  assert.equal(err500.code, 'PROVIDER_UNAVAILABLE');
  assert.equal(err500.retryable, true);

  const errDisconnect = ErrorNormalizer.normalize(new Error('fetch failed ECONNRESET'), 'test-provider');
  assert.equal(errDisconnect.code, 'STREAM_DISCONNECTED');
  assert.equal(errDisconnect.retryable, true);
});
