import EventSource from 'eventsource';
import type { KeeperConfig } from '../config.js';
import { getLogger } from '../observability/logger.js';
import { metrics } from '../observability/metrics.js';

export type TxLINEEvent =
  | { type: 'match.started'; matchId: string; homeTeam: string; awayTeam: string; startTs: number }
  | { type: 'match.score'; matchId: string; homeScore: number; awayScore: number; minute: number }
  | { type: 'match.event'; matchId: string; eventType: string; player?: string; minute: number; team: 'home' | 'away' }
  | { type: 'match.status'; matchId: string; status: 'HT' | 'FT' | 'AET' | 'PEN' | 'live' }
  | { type: 'odds.update'; matchId: string; marketType: string; odds: number[] }
  | { type: 'merkle.root'; rootHash: string; blockNumber: number; timestamp: number };

export type EventHandler = (event: TxLINEEvent) => void | Promise<void>;

export class SseClient {
  private es: EventSource | null = null;
  private reconnectDelay = 1000;
  private readonly maxReconnectDelay = 30_000;
  private handlers: EventHandler[] = [];
  private lastEventId: string | null = null;
  private stopped = false;

  constructor(private readonly config: KeeperConfig) {}

  onEvent(handler: EventHandler): void {
    this.handlers.push(handler);
  }

  start(): void {
    this.stopped = false;
    this.connect();
  }

  stop(): void {
    this.stopped = true;
    this.es?.close();
    this.es = null;
  }

  private connect(): void {
    const logger = getLogger();
    const url = this.lastEventId
      ? `${this.config.TXLINE_SSE_URL}?lastEventId=${this.lastEventId}`
      : this.config.TXLINE_SSE_URL;

    this.es = new EventSource(url, {
      headers: { Authorization: `Bearer ${this.config.TXLINE_API_KEY}` },
    });

    this.es.onopen = () => {
      logger.info('SSE connected to TxLINE feed');
      this.reconnectDelay = 1000;
    };

    this.es.onmessage = (e) => {
      if (e.lastEventId) this.lastEventId = e.lastEventId;
      this.dispatch(e.data);
    };

    this.es.onerror = (err) => {
      logger.warn({ err }, 'SSE connection error, scheduling reconnect');
      metrics.sseReconnectsTotal.inc();
      this.es?.close();
      this.es = null;
      if (!this.stopped) {
        setTimeout(() => this.connect(), this.reconnectDelay);
        this.reconnectDelay = Math.min(this.reconnectDelay * 2, this.maxReconnectDelay);
      }
    };

    // Heartbeat watchdog — reconnect if server stops sending pings
    let heartbeatTimer: ReturnType<typeof setTimeout>;
    const resetHeartbeat = () => {
      clearTimeout(heartbeatTimer);
      heartbeatTimer = setTimeout(() => {
        logger.warn('SSE heartbeat timeout — forcing reconnect');
        metrics.sseReconnectsTotal.inc();
        this.es?.close();
        this.es = null;
        if (!this.stopped) this.connect();
      }, 60_000);
    };

    this.es.addEventListener('ping', () => resetHeartbeat());
    resetHeartbeat();
  }

  private dispatch(raw: string): void {
    const logger = getLogger();
    try {
      const event = JSON.parse(raw) as TxLINEEvent;
      metrics.sseEventsTotal.inc({ type: event.type });
      for (const handler of this.handlers) {
        Promise.resolve(handler(event)).catch((err) =>
          logger.error({ err, eventType: event.type }, 'Event handler error'),
        );
      }
    } catch (err) {
      logger.warn({ err, raw }, 'Failed to parse SSE event');
    }
  }
}
