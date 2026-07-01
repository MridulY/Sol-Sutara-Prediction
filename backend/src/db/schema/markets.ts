import {
  pgTable,
  uuid,
  varchar,
  jsonb,
  smallint,
  numeric,
  timestamp,
  index,
} from 'drizzle-orm/pg-core';
import { sql } from 'drizzle-orm';

export const markets = pgTable(
  'markets',
  {
    id: uuid('id').primaryKey().defaultRandom(),
    pubkey: varchar('pubkey', { length: 44 }).unique().notNull(),
    matchId: varchar('match_id', { length: 64 }).notNull(),
    marketType: varchar('market_type', { length: 32 }).notNull(),
    marketTypeData: jsonb('market_type_data'),
    outcomes: jsonb('outcomes').notNull(),
    status: varchar('status', { length: 16 }).notNull(),
    creator: varchar('creator', { length: 44 }).notNull(),
    openTs: timestamp('open_ts', { withTimezone: true }).notNull(),
    closeTs: timestamp('close_ts', { withTimezone: true }).notNull(),
    resolveTs: timestamp('resolve_ts', { withTimezone: true }),
    winningOutcome: smallint('winning_outcome'),
    poolPubkey: varchar('pool_pubkey', { length: 44 }),
    vaultPubkey: varchar('vault_pubkey', { length: 44 }),
    totalVolume: numeric('total_volume', { precision: 20, scale: 6 }).default('0'),
    createdAt: timestamp('created_at', { withTimezone: true }).default(sql`NOW()`),
    updatedAt: timestamp('updated_at', { withTimezone: true }).default(sql`NOW()`),
  },
  (t) => ({
    matchIdIdx: index('markets_match_id_idx').on(t.matchId),
    statusIdx: index('markets_status_idx').on(t.status),
    closeTsIdx: index('markets_close_ts_idx').on(t.closeTs),
    creatorIdx: index('markets_creator_idx').on(t.creator),
  }),
);

export const pools = pgTable('pools', {
  id: uuid('id').primaryKey().defaultRandom(),
  marketPubkey: varchar('market_pubkey', { length: 44 }).unique().notNull(),
  bParameter: numeric('b_parameter', { precision: 20, scale: 6 }).notNull(),
  outcomeQuantities: jsonb('outcome_quantities').notNull(),
  lpSupply: numeric('lp_supply', { precision: 20, scale: 6 }).notNull(),
  totalFees: numeric('total_fees', { precision: 20, scale: 6 }).default('0'),
  updatedAt: timestamp('updated_at', { withTimezone: true }).default(sql`NOW()`),
});
