import { getLogger } from '../observability/logger.js';
import { metrics } from '../observability/metrics.js';

export interface RetryOptions {
  maxAttempts: number;
  baseDelayMs: number;
  maxDelayMs: number;
  jitterMs?: number;
}

export async function withRetry<T>(
  label: string,
  fn: () => Promise<T>,
  opts: RetryOptions,
): Promise<T> {
  const logger = getLogger();
  let lastError: unknown;

  for (let attempt = 1; attempt <= opts.maxAttempts; attempt++) {
    try {
      return await fn();
    } catch (err) {
      lastError = err;
      metrics.retryAttemptsTotal.inc();

      if (attempt === opts.maxAttempts) break;

      const jitter = Math.random() * (opts.jitterMs ?? 500);
      const delay = Math.min(opts.baseDelayMs * 2 ** (attempt - 1) + jitter, opts.maxDelayMs);

      logger.warn(
        { label, attempt, maxAttempts: opts.maxAttempts, delayMs: Math.round(delay), err },
        'Retrying after failure',
      );

      await sleep(delay);
    }
  }

  logger.error({ label, lastError }, 'All retry attempts exhausted');
  throw lastError;
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
