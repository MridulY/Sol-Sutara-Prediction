import { Registry, Counter, Histogram, Gauge } from 'prom-client';
import http from 'http';

export const registry = new Registry();
registry.setDefaultLabels({ service: 'sutara-keeper' });

export const metrics = {
  sseEventsTotal: new Counter({
    name: 'sutara_keeper_sse_events_total',
    help: 'Total SSE events received',
    labelNames: ['type'],
    registers: [registry],
  }),

  sseReconnectsTotal: new Counter({
    name: 'sutara_keeper_sse_reconnects_total',
    help: 'Total SSE reconnections',
    registers: [registry],
  }),

  proofFetchDuration: new Histogram({
    name: 'sutara_keeper_proof_fetch_duration_ms',
    help: 'Duration of Merkle proof fetch requests',
    buckets: [100, 500, 1000, 3000, 5000, 10000, 30000],
    registers: [registry],
  }),

  settlementSuccessTotal: new Counter({
    name: 'sutara_keeper_settlement_success_total',
    help: 'Successful market settlements',
    registers: [registry],
  }),

  settlementFailureTotal: new Counter({
    name: 'sutara_keeper_settlement_failure_total',
    help: 'Failed market settlements after all retries',
    labelNames: ['reason'],
    registers: [registry],
  }),

  retryAttemptsTotal: new Counter({
    name: 'sutara_keeper_retry_attempts_total',
    help: 'Total retry attempts',
    registers: [registry],
  }),

  activeMarketsGauge: new Gauge({
    name: 'sutara_keeper_active_markets',
    help: 'Number of markets currently being watched',
    registers: [registry],
  }),

  pendingSettlementsGauge: new Gauge({
    name: 'sutara_keeper_pending_settlements',
    help: 'Number of settlements pending in queue',
    registers: [registry],
  }),
};

export function startMetricsServer(port: number): http.Server {
  const server = http.createServer(async (req, res) => {
    if (req.url === '/metrics') {
      res.setHeader('Content-Type', registry.contentType);
      res.end(await registry.metrics());
    } else if (req.url === '/health') {
      res.writeHead(200);
      res.end('OK');
    } else {
      res.writeHead(404);
      res.end();
    }
  });
  server.listen(port);
  return server;
}
