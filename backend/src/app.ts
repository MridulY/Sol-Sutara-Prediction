import express from 'express';
import cors from 'cors';
import helmet from 'helmet';
import compression from 'compression';
import rateLimit from 'express-rate-limit';
import { pinoHttp } from 'pino-http';
import pino from 'pino';

import { marketsRouter } from './routes/markets.js';
import { tradesRouter } from './routes/trades.js';
import { matchesRouter } from './routes/matches.js';
import { leaderboardRouter } from './routes/leaderboard.js';
import { analyticsRouter } from './routes/analytics.js';
import { sseRouter } from './routes/sse.js';
import { adminRouter } from './routes/admin.js';

export function createApp(): express.Application {
  const app = express();

  // ── Security ────────────────────────────────────────────────────────────────
  app.use(helmet());
  app.use(cors({
    origin: process.env['FRONTEND_URL'] ?? 'http://localhost:3000',
    credentials: true,
  }));

  // ── Compression ─────────────────────────────────────────────────────────────
  app.use(compression());

  // ── Logging ──────────────────────────────────────────────────────────────────
  const logger = pino({ level: process.env['LOG_LEVEL'] ?? 'info' });
  app.use(pinoHttp({ logger }));

  // ── Body parsing ─────────────────────────────────────────────────────────────
  app.use(express.json({ limit: '1mb' }));
  app.use(express.urlencoded({ extended: true }));

  // ── Rate limiting ─────────────────────────────────────────────────────────────
  const apiLimiter = rateLimit({
    windowMs: 60 * 1000,   // 1 minute
    max: 120,              // 120 req/min per IP
    standardHeaders: true,
    legacyHeaders: false,
    message: { error: 'Too many requests, please slow down' },
  });

  const tradingLimiter = rateLimit({
    windowMs: 60 * 1000,
    max: 60,
    standardHeaders: true,
    legacyHeaders: false,
  });

  // ── Health ───────────────────────────────────────────────────────────────────
  app.get('/health', (_req, res) => {
    res.json({ status: 'ok', timestamp: new Date().toISOString() });
  });

  // ── Routes ───────────────────────────────────────────────────────────────────
  app.use('/api', apiLimiter);
  app.use('/api/markets', marketsRouter);
  app.use('/api/trades', tradingLimiter, tradesRouter);
  app.use('/api/matches', matchesRouter);
  app.use('/api/leaderboard', leaderboardRouter);
  app.use('/api/analytics', analyticsRouter);
  app.use('/api/sse', sseRouter);
  app.use('/api/admin', adminRouter);

  // ── 404 handler ──────────────────────────────────────────────────────────────
  app.use((_req, res) => {
    res.status(404).json({ error: 'Not found' });
  });

  // ── Global error handler ─────────────────────────────────────────────────────
  app.use((err: Error, _req: express.Request, res: express.Response, _next: express.NextFunction) => {
    logger.error({ err }, 'Unhandled error');
    res.status(500).json({ error: 'Internal server error' });
  });

  return app;
}
