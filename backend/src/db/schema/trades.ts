import {
  pgTable,
  uuid,
  varchar,
  smallint,
  numeric,
  bigint,
  jsonb,
  timestamp,
  index,
} from 'drizzle-orm/pg-core';

export const trades = pgTable(
  'trades',
  {
    id: uuid('id').primaryKey().defaultRandom(),
    txSignature: varchar('tx_signature', { length: 88 }).unique().notNull(),
    marketPubkey: varchar('market_pubkey', { length: 44 }).notNull(),
    trader: varchar('trader', { length: 44 }).notNull(),
    action: varchar('action', { length: 8 }).notNull(),
    outcomeIdx: smallint('outcome_idx').notNull(),
    shares: numeric('shares', { precision: 20, scale: 9 }).notNull(),
    cost: numeric('cost', { precision: 20, scale: 6 }).notNull(),
    price: numeric('price', { precision: 10, scale: 8 }).notNull(),
    protocolFee: numeric('protocol_fee', { precision: 20, scale: 6 }).notNull(),
    creatorFee: numeric('creator_fee', { precision: 20, scale: 6 }).notNull(),
    lpFee: numeric('lp_fee', { precision: 20, scale: 6 }).notNull(),
    slot: bigint('slot', { mode: 'number' }).notNull(),
    blockTime: timestamp('block_time', { withTimezone: true }).notNull(),
  },
  (t) => ({
    marketTimeIdx: index('trades_market_time_idx').on(t.marketPubkey, t.blockTime),
    traderTimeIdx: index('trades_trader_time_idx').on(t.trader, t.blockTime),
    slotIdx: index('trades_slot_idx').on(t.slot),
  }),
);

export const positions = pgTable(
  'positions',
  {
    id: uuid('id').primaryKey().defaultRandom(),
    marketPubkey: varchar('market_pubkey', { length: 44 }).notNull(),
    ownerPubkey: varchar('owner_pubkey', { length: 44 }).notNull(),
    shares: jsonb('shares').notNull(),
    lpTokens: numeric('lp_tokens', { precision: 20, scale: 6 }).default('0'),
    costBasis: numeric('cost_basis', { precision: 20, scale: 6 }).default('0'),
    claimed: varchar('claimed', { length: 5 }).default('false'),
    updatedAt: timestamp('updated_at', { withTimezone: true }),
  },
  (t) => ({
    ownerIdx: index('positions_owner_idx').on(t.ownerPubkey),
    marketOwnerIdx: index('positions_market_owner_idx').on(t.marketPubkey, t.ownerPubkey),
  }),
);
