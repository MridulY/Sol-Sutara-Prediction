import { z } from 'zod';

const ConfigSchema = z.object({
  // Solana
  RPC_URL: z.string().url(),
  RPC_WS_URL: z.string().url(),
  PROGRAM_ID: z.string().length(44),
  USDC_MINT: z.string().length(44),
  KEEPER_KEYPAIR_PATH: z.string(),

  // TxLINE
  TXLINE_SSE_URL: z.string().url(),
  TXLINE_MERKLE_API_URL: z.string().url(),
  TXLINE_API_KEY: z.string().min(1),

  // Redis (for BullMQ)
  REDIS_URL: z.string().url(),

  // Tuning
  SETTLEMENT_DELAY_MS: z.coerce.number().default(30_000),
  MAX_RETRIES: z.coerce.number().default(5),
  RETRY_BASE_DELAY_MS: z.coerce.number().default(1_000),
  RETRY_MAX_DELAY_MS: z.coerce.number().default(60_000),
  COMPUTE_UNIT_PRICE: z.coerce.number().default(100_000), // micro-lamports

  // Observability
  METRICS_PORT: z.coerce.number().default(9091),
  LOG_LEVEL: z.enum(['trace', 'debug', 'info', 'warn', 'error']).default('info'),
});

export type KeeperConfig = z.infer<typeof ConfigSchema>;

export function loadConfig(): KeeperConfig {
  const result = ConfigSchema.safeParse(process.env);
  if (!result.success) {
    console.error('Invalid keeper configuration:', result.error.flatten());
    process.exit(1);
  }
  return result.data;
}
