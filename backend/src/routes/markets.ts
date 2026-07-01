import { Router, type Request, type Response, type NextFunction } from 'express';
import { z } from 'zod';
import { getDb } from '../db/client.js';
import { getRedis } from '../cache/redis.js';
import { CacheKeys, CacheTTL } from '../cache/keys.js';
import { markets, pools } from '../db/schema/markets.js';
import { eq, desc, and } from 'drizzle-orm';

export const marketsRouter = Router();

const ListQuerySchema = z.object({
  status: z.enum(['pending', 'open', 'closed', 'disputed', 'resolved', 'cancelled']).optional(),
  matchId: z.string().optional(),
  creator: z.string().optional(),
  limit: z.coerce.number().min(1).max(100).default(20),
  offset: z.coerce.number().min(0).default(0),
});

// GET /api/markets
marketsRouter.get('/', async (req: Request, res: Response, next: NextFunction): Promise<void> => {
  try {
    const query = ListQuerySchema.parse(req.query);
    const cacheKey = CacheKeys.marketList(JSON.stringify(query));
    const redis = getRedis();

    const cached = await redis.get(cacheKey);
    if (cached) {
      res.setHeader('X-Cache', 'HIT').json(JSON.parse(cached));
      return;
    }

    const db = getDb();
    const conditions = [];
    if (query.status) conditions.push(eq(markets.status, query.status));
    if (query.matchId) conditions.push(eq(markets.matchId, query.matchId));
    if (query.creator) conditions.push(eq(markets.creator, query.creator));

    const rows = await db
      .select()
      .from(markets)
      .where(conditions.length ? and(...conditions) : undefined)
      .orderBy(desc(markets.createdAt))
      .limit(query.limit)
      .offset(query.offset);

    const result = { data: rows, limit: query.limit, offset: query.offset };
    await redis.setex(cacheKey, CacheTTL.marketList, JSON.stringify(result));

    res.setHeader('X-Cache', 'MISS').json(result);
  } catch (err) {
    next(err);
  }
});

// GET /api/markets/:pubkey
marketsRouter.get('/:pubkey', async (req: Request, res: Response, next: NextFunction): Promise<void> => {
  try {
    const { pubkey } = req.params;
    const redis = getRedis();
    const cacheKey = CacheKeys.market(pubkey);

    const cached = await redis.get(cacheKey);
    if (cached) {
      res.setHeader('X-Cache', 'HIT').json(JSON.parse(cached));
      return;
    }

    const db = getDb();
    const [market] = await db.select().from(markets).where(eq(markets.pubkey, pubkey)).limit(1);
    if (!market) {
      res.status(404).json({ error: 'Market not found' });
      return;
    }

    const [pool] = await db.select().from(pools).where(eq(pools.marketPubkey, pubkey)).limit(1);
    const result = { market, pool: pool ?? null };

    await redis.setex(cacheKey, CacheTTL.market, JSON.stringify(result));
    res.setHeader('X-Cache', 'MISS').json(result);
  } catch (err) {
    next(err);
  }
});

// GET /api/markets/:pubkey/odds
marketsRouter.get('/:pubkey/odds', async (req: Request, res: Response, next: NextFunction): Promise<void> => {
  try {
    const { pubkey } = req.params;
    const redis = getRedis();

    const cached = await redis.get(CacheKeys.odds(pubkey));
    if (cached) {
      res.json(JSON.parse(cached));
      return;
    }

    const db = getDb();
    const [pool] = await db.select().from(pools).where(eq(pools.marketPubkey, pubkey)).limit(1);
    if (!pool) {
      res.status(404).json({ error: 'Pool not found' });
      return;
    }

    const result = {
      marketPubkey: pubkey,
      outcomeQuantities: pool.outcomeQuantities,
      bParameter: pool.bParameter,
      lpSupply: pool.lpSupply,
    };

    await redis.setex(CacheKeys.odds(pubkey), CacheTTL.odds, JSON.stringify(result));
    res.json(result);
  } catch (err) {
    next(err);
  }
});
