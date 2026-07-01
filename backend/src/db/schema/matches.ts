import {
  pgTable,
  uuid,
  varchar,
  smallint,
  numeric,
  jsonb,
  timestamp,
  index,
} from 'drizzle-orm/pg-core';
import { sql } from 'drizzle-orm';

export const matches = pgTable(
  'matches',
  {
    id: uuid('id').primaryKey().defaultRandom(),
    txlineMatchId: varchar('txline_match_id', { length: 64 }).unique().notNull(),
    homeTeam: varchar('home_team', { length: 64 }).notNull(),
    awayTeam: varchar('away_team', { length: 64 }).notNull(),
    homeScore: smallint('home_score').default(0),
    awayScore: smallint('away_score').default(0),
    status: varchar('status', { length: 16 }).notNull(),
    startTs: timestamp('start_ts', { withTimezone: true }).notNull(),
    rawData: jsonb('raw_data'),
    updatedAt: timestamp('updated_at', { withTimezone: true }).default(sql`NOW()`),
  },
  (t) => ({
    statusIdx: index('matches_status_idx').on(t.status),
    startTsIdx: index('matches_start_ts_idx').on(t.startTs),
  }),
);

export const settlements = pgTable('settlements', {
  id: uuid('id').primaryKey().defaultRandom(),
  marketPubkey: varchar('market_pubkey', { length: 44 }).unique().notNull(),
  txSignature: varchar('tx_signature', { length: 88 }).notNull(),
  merkleRoot: varchar('merkle_root', { length: 66 }).notNull(),
  winningOutcome: smallint('winning_outcome').notNull(),
  resolvedAt: timestamp('resolved_at', { withTimezone: true }).notNull(),
  keeperPubkey: varchar('keeper_pubkey', { length: 44 }).notNull(),
});

export const analyticsDaily = pgTable('analytics_daily', {
  date: varchar('date', { length: 10 }).primaryKey(),
  volume: numeric('volume', { precision: 20, scale: 6 }).notNull(),
  tvl: numeric('tvl', { precision: 20, scale: 6 }).notNull(),
  tradesCount: smallint('trades_count').notNull(),
  newMarkets: smallint('new_markets').notNull(),
  uniqueTraders: smallint('unique_traders').notNull(),
});
