import { Router, type Request, type Response, type NextFunction } from 'express';
import { getDb } from '../db/client.js';
import { getRedis } from '../cache/redis.js';
import { CacheKeys, CacheTTL } from '../cache/keys.js';
import { matches } from '../db/schema/matches.js';
import { eq, desc } from 'drizzle-orm';

export const matchesRouter = Router();

// GET /api/matches  — live + upcoming
matchesRouter.get('/', async (_req: Request, res: Response, next: NextFunction): Promise<void> => {
  try {
    const redis = getRedis();
    const cached = await redis.get(CacheKeys.matchesLive());
    if (cached) {
      res.setHeader('X-Cache', 'HIT').json(JSON.parse(cached));
      return;
    }

    const db = getDb();
    const rows = await db
      .select()
      .from(matches)
      .orderBy(desc(matches.startTs))
      .limit(50);

    await redis.setex(CacheKeys.matchesLive(), CacheTTL.matchesLive, JSON.stringify({ data: rows }));
    res.setHeader('X-Cache', 'MISS').json({ data: rows });
  } catch (err) {
    next(err);
  }
});

// GET /api/matches/:txlineMatchId
matchesRouter.get('/:txlineMatchId', async (req: Request, res: Response, next: NextFunction): Promise<void> => {
  try {
    const { txlineMatchId } = req.params;
    const redis = getRedis();
    const cacheKey = CacheKeys.match(txlineMatchId);

    const cached = await redis.get(cacheKey);
    if (cached) {
      res.setHeader('X-Cache', 'HIT').json(JSON.parse(cached));
      return;
    }

    const db = getDb();
    const [match] = await db.select().from(matches).where(eq(matches.txlineMatchId, txlineMatchId)).limit(1);
    if (!match) {
      res.status(404).json({ error: 'Match not found' });
      return;
    }

    await redis.setex(cacheKey, CacheTTL.match, JSON.stringify(match));
    res.setHeader('X-Cache', 'MISS').json(match);
  } catch (err) {
    next(err);
  }
});
