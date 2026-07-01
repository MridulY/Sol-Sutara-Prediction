import { Router, type Request, type Response, type NextFunction } from 'express';
import { getDb } from '../db/client.js';
import { getRedis } from '../cache/redis.js';
import { CacheKeys, CacheTTL } from '../cache/keys.js';
import { trades } from '../db/schema/trades.js';
import { sql, desc } from 'drizzle-orm';

export const leaderboardRouter = Router();

// GET /api/leaderboard  — top traders by volume
leaderboardRouter.get('/', async (_req: Request, res: Response, next: NextFunction): Promise<void> => {
  try {
    const redis = getRedis();
    const cached = await redis.get(CacheKeys.leaderboard());
    if (cached) {
      res.setHeader('X-Cache', 'HIT').json(JSON.parse(cached));
      return;
    }

    const db = getDb();
    const rows = await db
      .select({
        wallet: trades.trader,
        totalVolume: sql<string>`SUM(${trades.cost})`,
        tradeCount: sql<number>`COUNT(*)`,
      })
      .from(trades)
      .groupBy(trades.trader)
      .orderBy(desc(sql`SUM(${trades.cost})`))
      .limit(100);

    const result = { data: rows };
    await redis.setex(CacheKeys.leaderboard(), CacheTTL.leaderboard, JSON.stringify(result));
    res.setHeader('X-Cache', 'MISS').json(result);
  } catch (err) {
    next(err);
  }
});
