import { Router, type Request, type Response, type NextFunction } from 'express';
import { getDb } from '../db/client.js';
import { trades, positions } from '../db/schema/trades.js';
import { eq, desc } from 'drizzle-orm';

export const tradesRouter = Router();

// GET /api/trades/market/:pubkey
tradesRouter.get('/market/:pubkey', async (req: Request, res: Response, next: NextFunction) => {
  try {
    const { pubkey } = req.params;
    const limit = Math.min(Number(req.query['limit'] ?? 50), 200);
    const offset = Number(req.query['offset'] ?? 0);

    const db = getDb();
    const rows = await db
      .select()
      .from(trades)
      .where(eq(trades.marketPubkey, pubkey))
      .orderBy(desc(trades.blockTime))
      .limit(limit)
      .offset(offset);

    res.json({ data: rows, limit, offset });
  } catch (err) {
    next(err);
  }
});

// GET /api/trades/portfolio/:wallet
tradesRouter.get('/portfolio/:wallet', async (req: Request, res: Response, next: NextFunction) => {
  try {
    const { wallet } = req.params;
    const db = getDb();

    const [userPositions, recentTrades] = await Promise.all([
      db.select().from(positions).where(eq(positions.ownerPubkey, wallet)),
      db
        .select()
        .from(trades)
        .where(eq(trades.trader, wallet))
        .orderBy(desc(trades.blockTime))
        .limit(100),
    ]);

    res.json({ positions: userPositions, recentTrades });
  } catch (err) {
    next(err);
  }
});
