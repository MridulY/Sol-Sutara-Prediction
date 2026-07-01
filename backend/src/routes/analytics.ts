import { Router, type Request, type Response, type NextFunction } from 'express';
import { getDb } from '../db/client.js';
import { getRedis } from '../cache/redis.js';
import { CacheKeys, CacheTTL } from '../cache/keys.js';
import { analyticsDaily } from '../db/schema/matches.js';
import { trades } from '../db/schema/trades.js';
import { markets, pools } from '../db/schema/markets.js';
import { sql, desc } from 'drizzle-orm';

export const analyticsRouter = Router();

// GET /api/analytics/volume  — daily volume time series
analyticsRouter.get('/volume', async (_req: Request, res: Response, next: NextFunction): Promise<void> => {
  try {
    const db = getDb();
    const rows = await db
      .select({ date: analyticsDaily.date, volume: analyticsDaily.volume })
      .from(analyticsDaily)
      .orderBy(desc(analyticsDaily.date))
      .limit(30);

    res.json({ data: rows });
  } catch (err) {
    next(err);
  }
});

// GET /api/analytics/tvl  — total value locked
analyticsRouter.get('/tvl', async (_req: Request, res: Response, next: NextFunction): Promise<void> => {
  try {
    const redis = getRedis();
    const cached = await redis.get(CacheKeys.tvl());
    if (cached) {
      res.json(JSON.parse(cached));
      return;
    }

    const db = getDb();
    const [result] = await db
      .select({ tvl: sql<string>`COALESCE(SUM(${pools.totalFees}), 0)` })
      .from(pools);

    const data = { tvl: result?.tvl ?? '0' };
    await redis.setex(CacheKeys.tvl(), CacheTTL.tvl, JSON.stringify(data));
    res.json(data);
  } catch (err) {
    next(err);
  }
});

// GET /api/analytics/summary
analyticsRouter.get('/summary', async (_req: Request, res: Response, next: NextFunction): Promise<void> => {
  try {
    const redis = getRedis();
    const cached = await redis.get(CacheKeys.volume24h());
    if (cached) {
      res.json(JSON.parse(cached));
      return;
    }

    const db = getDb();
    const [vol] = await db
      .select({ volume24h: sql<string>`COALESCE(SUM(${trades.cost}), 0)` })
      .from(trades)
      .where(sql`${trades.blockTime} > NOW() - INTERVAL '24 hours'`);

    const [mktCount] = await db
      .select({ total: sql<number>`COUNT(*)` })
      .from(markets);

    const summary = { volume24h: vol?.volume24h ?? '0', totalMarkets: mktCount?.total ?? 0 };
    await redis.setex(CacheKeys.volume24h(), CacheTTL.volume24h, JSON.stringify(summary));
    res.json(summary);
  } catch (err) {
    next(err);
  }
});
