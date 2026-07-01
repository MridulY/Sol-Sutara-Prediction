import pino from 'pino';
import type { KeeperConfig } from '../config.js';

let _logger: pino.Logger;

export function createLogger(config: KeeperConfig): pino.Logger {
  _logger = pino({
    level: config.LOG_LEVEL,
    base: { service: 'sutara-keeper' },
    timestamp: pino.stdTimeFunctions.isoTime,
  });
  return _logger;
}

export function getLogger(): pino.Logger {
  if (!_logger) throw new Error('Logger not initialized');
  return _logger;
}
