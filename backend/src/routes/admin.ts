import { Router, type Request, type Response, type NextFunction } from 'express';

export const adminRouter = Router();

/**
 * Admin routes are gated by a static API key supplied in the Authorization header.
 * Bearer token must match ADMIN_API_KEY env variable.
 *
 * This backend NEVER touches on-chain funds — it is informational only.
 * On-chain admin actions are executed directly via wallet in the frontend.
 */
function adminOnly(req: Request, res: Response, next: NextFunction): void {
  const apiKey = process.env['ADMIN_API_KEY'];
  const auth = req.headers['authorization'] ?? '';
  const provided = auth.startsWith('Bearer ') ? auth.slice(7) : '';

  if (!apiKey || !provided || provided !== apiKey) {
    res.status(401).json({ error: 'Unauthorized' });
    return;
  }
  next();
}

// GET /api/admin/status
adminRouter.get('/status', (_req: Request, res: Response) => {
  res.json({ status: 'ok', timestamp: new Date().toISOString() });
});

// POST /api/admin/cache/flush
adminRouter.post('/cache/flush', adminOnly, async (_req: Request, res: Response, next: NextFunction): Promise<void> => {
  try {
    const { getRedis } = await import('../cache/redis.js');
    const redis = getRedis();
    await redis.flushdb();
    res.json({ flushed: true });
  } catch (err) {
    next(err);
  }
});

// POST /api/admin/cache/invalidate/:key
adminRouter.post('/cache/invalidate/:key', adminOnly, async (req: Request, res: Response, next: NextFunction): Promise<void> => {
  try {
    const { getRedis } = await import('../cache/redis.js');
    const redis = getRedis();
    const deleted = await redis.del(`sutara:${req.params['key']}`);
    res.json({ deleted });
  } catch (err) {
    next(err);
  }
});
