import { loadConfig } from './config.js';
import { createLogger, getLogger } from './observability/logger.js';
import { startMetricsServer } from './observability/metrics.js';
import { SseClient } from './sse/client.js';
import { GameTracker } from './monitor/game-tracker.js';
import { ProofFetcher } from './proof/fetcher.js';
import { SettlementExecutor } from './settlement/executor.js';
import { withRetry } from './settlement/retry.js';

async function main(): Promise<void> {
  const config = loadConfig();
  const logger = createLogger(config);

  logger.info('Sutara keeper starting up');

  // ── Services ──────────────────────────────────────────────────────────────
  const proofFetcher = new ProofFetcher(config);
  const settlementExecutor = new SettlementExecutor(config);
  const gameTracker = new GameTracker();
  const sseClient = new SseClient(config);

  // ── Game completion → proof → settlement pipeline ─────────────────────────
  gameTracker.onGameCompleted(async (game) => {
    logger.info({ matchId: game.matchId }, 'Processing game completion');

    // Wait for TxLINE to finalize and generate proof
    await new Promise((resolve) => setTimeout(resolve, config.SETTLEMENT_DELAY_MS));

    for (const marketPubkey of game.marketPubkeys) {
      await withRetry(
        `settle:${game.matchId}:${marketPubkey}`,
        async () => {
          const proof = await proofFetcher.fetchProof(game.matchId);
          await settlementExecutor.execute({ matchId: game.matchId, marketPubkey, proof });
        },
        {
          maxAttempts: config.MAX_RETRIES,
          baseDelayMs: config.RETRY_BASE_DELAY_MS,
          maxDelayMs: config.RETRY_MAX_DELAY_MS,
        },
      ).catch((err) => {
        logger.error({ err, matchId: game.matchId, marketPubkey }, 'Settlement failed after all retries — requires manual intervention');
      });
    }
  });

  // ── SSE event routing ─────────────────────────────────────────────────────
  sseClient.onEvent((event) => gameTracker.handleEvent(event));
  sseClient.start();

  // ── Metrics server ────────────────────────────────────────────────────────
  const metricsServer = startMetricsServer(config.METRICS_PORT);
  logger.info({ port: config.METRICS_PORT }, 'Metrics server started');

  // ── Graceful shutdown ─────────────────────────────────────────────────────
  const shutdown = async (signal: string): Promise<void> => {
    logger.info({ signal }, 'Shutting down keeper');
    sseClient.stop();
    metricsServer.close();
    process.exit(0);
  };

  process.on('SIGTERM', () => shutdown('SIGTERM'));
  process.on('SIGINT', () => shutdown('SIGINT'));

  logger.info('Keeper running — watching TxLINE SSE feed');
}

main().catch((err) => {
  console.error('Fatal keeper error:', err);
  process.exit(1);
});
