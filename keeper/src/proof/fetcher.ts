import type { KeeperConfig } from '../config.js';
import { getLogger } from '../observability/logger.js';
import { metrics } from '../observability/metrics.js';

export interface MerkleProofResponse {
  matchId: string;
  outcomeIdx: number;
  scoreHome: number;
  scoreAway: number;
  finalizedTs: number;
  merkleRoot: string;        // hex-encoded 32 bytes
  leafIndex: number;
  proof: string[];           // hex-encoded 32-byte hashes
  leaf: string;              // hex-encoded 32-byte leaf hash
}

export class ProofFetcher {
  constructor(private readonly config: KeeperConfig) {}

  async fetchProof(matchId: string): Promise<MerkleProofResponse> {
    const logger = getLogger();
    const end = metrics.proofFetchDuration.startTimer();

    try {
      const url = `${this.config.TXLINE_MERKLE_API_URL}/proof?matchId=${encodeURIComponent(matchId)}`;
      const response = await fetch(url, {
        headers: {
          Authorization: `Bearer ${this.config.TXLINE_API_KEY}`,
          Accept: 'application/json',
        },
        signal: AbortSignal.timeout(15_000),
      });

      if (!response.ok) {
        throw new Error(`TxLINE Merkle API returned ${response.status}: ${await response.text()}`);
      }

      const data = (await response.json()) as MerkleProofResponse;
      logger.debug({ matchId, outcomeIdx: data.outcomeIdx }, 'Merkle proof fetched');
      return data;
    } finally {
      end();
    }
  }
}
