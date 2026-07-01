import { Router, type Request, type Response } from 'express';
import { getRedis } from '../cache/redis.js';

export const sseRouter = Router();

/**
 * GET /api/sse?matchId=XXX  or  ?marketPubkey=YYY
 *
 * Establishes a Server-Sent Events stream. The backend subscribes to Redis
 * Pub/Sub channels and forwards events to the connected client.
 *
 * Channels:
 *   txline:match:{matchId}       — score updates, match status changes
 *   chain:market:{marketPubkey}  — SharesBought, MarketResolved, etc.
 */
sseRouter.get('/', (req: Request, res: Response): void => {
  const { matchId, marketPubkey } = req.query as Record<string, string | undefined>;

  res.setHeader('Content-Type', 'text/event-stream');
  res.setHeader('Cache-Control', 'no-cache');
  res.setHeader('Connection', 'keep-alive');
  res.setHeader('X-Accel-Buffering', 'no');
  res.flushHeaders();

  const redis = getRedis().duplicate();
  const channels: string[] = [];
  if (matchId) channels.push(`txline:match:${matchId}`);
  if (marketPubkey) channels.push(`chain:market:${marketPubkey}`);

  if (channels.length === 0) {
    res.write('event: error\ndata: {"error":"provide matchId or marketPubkey query param"}\n\n');
    res.end();
    redis.disconnect();
    return;
  }

  redis.subscribe(...channels, (err) => {
    if (err) {
      res.write(`event: error\ndata: ${JSON.stringify({ error: err.message })}\n\n`);
      res.end();
    }
  });

  redis.on('message', (_channel: string, message: string) => {
    res.write(`event: update\ndata: ${message}\n\n`);
  });

  // Heartbeat every 20s to keep the connection alive through proxies
  const heartbeat = setInterval(() => res.write(': ping\n\n'), 20_000);

  req.on('close', () => {
    clearInterval(heartbeat);
    redis.unsubscribe();
    redis.disconnect();
  });
});
