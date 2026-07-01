import { drizzle } from 'drizzle-orm/node-postgres';
import { Pool } from 'pg';
import * as marketSchema from './schema/markets.js';
import * as tradeSchema from './schema/trades.js';
import * as matchSchema from './schema/matches.js';

export const schema = { ...marketSchema, ...tradeSchema, ...matchSchema };

let _db: ReturnType<typeof drizzle<typeof schema>>;

export function createDb(databaseUrl: string) {
  const pool = new Pool({ connectionString: databaseUrl, max: 20 });
  _db = drizzle(pool, { schema });
  return _db;
}

export function getDb() {
  if (!_db) throw new Error('Database not initialized');
  return _db;
}

export type Db = ReturnType<typeof getDb>;
