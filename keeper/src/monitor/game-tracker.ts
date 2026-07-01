import type { TxLINEEvent } from '../sse/client.js';
import { getLogger } from '../observability/logger.js';
import { metrics } from '../observability/metrics.js';

export interface TrackedGame {
  matchId: string;
  homeTeam: string;
  awayTeam: string;
  homeScore: number;
  awayScore: number;
  status: 'live' | 'HT' | 'FT' | 'AET' | 'PEN';
  marketPubkeys: string[];  // associated Sutara market accounts
  startTs: number;
}

export type GameCompletionCallback = (game: TrackedGame) => void | Promise<void>;

export class GameTracker {
  private readonly games = new Map<string, TrackedGame>();
  private readonly completionCallbacks: GameCompletionCallback[] = [];

  onGameCompleted(cb: GameCompletionCallback): void {
    this.completionCallbacks.push(cb);
  }

  registerMarketForMatch(matchId: string, marketPubkey: string): void {
    const game = this.games.get(matchId);
    if (game) {
      if (!game.marketPubkeys.includes(marketPubkey)) {
        game.marketPubkeys.push(marketPubkey);
      }
    } else {
      this.games.set(matchId, {
        matchId,
        homeTeam: '',
        awayTeam: '',
        homeScore: 0,
        awayScore: 0,
        status: 'live',
        marketPubkeys: [marketPubkey],
        startTs: Date.now(),
      });
    }
    metrics.activeMarketsGauge.set(this.games.size);
  }

  handleEvent(event: TxLINEEvent): void {
    const logger = getLogger();

    switch (event.type) {
      case 'match.started': {
        const existing = this.games.get(event.matchId) ?? {
          matchId: event.matchId,
          homeScore: 0,
          awayScore: 0,
          status: 'live' as const,
          marketPubkeys: [],
          startTs: event.startTs,
        };
        this.games.set(event.matchId, {
          ...existing,
          homeTeam: event.homeTeam,
          awayTeam: event.awayTeam,
        });
        break;
      }

      case 'match.score': {
        const game = this.games.get(event.matchId);
        if (game) {
          game.homeScore = event.homeScore;
          game.awayScore = event.awayScore;
        }
        break;
      }

      case 'match.status': {
        const game = this.games.get(event.matchId);
        if (!game) break;
        game.status = event.status;

        const isCompleted = event.status === 'FT' || event.status === 'AET' || event.status === 'PEN';
        if (isCompleted && game.marketPubkeys.length > 0) {
          logger.info({ matchId: event.matchId, status: event.status }, 'Game completed — triggering settlement');
          for (const cb of this.completionCallbacks) {
            Promise.resolve(cb(game)).catch((err) =>
              logger.error({ err, matchId: event.matchId }, 'Game completion callback error'),
            );
          }
          // Remove from active tracking after settlement triggered
          this.games.delete(event.matchId);
          metrics.activeMarketsGauge.set(this.games.size);
        }
        break;
      }
    }
  }

  getTrackedGames(): IterableIterator<TrackedGame> {
    return this.games.values();
  }
}
