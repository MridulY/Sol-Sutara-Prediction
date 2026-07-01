import { createApp } from './app.js';
import { createDb } from './db/client.js';
import { createRedis } from './cache/redis.js';

const PORT = parseInt(process.env['PORT'] ?? '3001', 10);
const DATABASE_URL = process.env['DATABASE_URL'];
const REDIS_URL = process.env['REDIS_URL'];

if (!DATABASE_URL) {
  console.error('Missing DATABASE_URL');
  process.exit(1);
}
if (!REDIS_URL) {
  console.error('Missing REDIS_URL');
  process.exit(1);
}

// Initialise global singletons before app is created
createDb(DATABASE_URL);
createRedis(REDIS_URL);

const app = createApp();

const server = app.listen(PORT, () => {
  console.log(`Sutara backend running on http://localhost:${PORT}`);
});

const shutdown = (): void => {
  console.log('Shutting down backend…');
  server.close(() => process.exit(0));
};

process.on('SIGTERM', shutdown);
process.on('SIGINT', shutdown);
