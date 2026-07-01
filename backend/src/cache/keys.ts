export const CacheKeys = {
  market: (pubkey: string) => `sutara:market:${pubkey}`,
  pool: (pubkey: string) => `sutara:pool:${pubkey}`,
  odds: (marketPubkey: string) => `sutara:odds:${marketPubkey}`,
  matchesLive: () => `sutara:matches:live`,
  match: (txlineId: string) => `sutara:match:${txlineId}`,
  leaderboard: () => `sutara:leaderboard`,
  volume24h: () => `sutara:volume:24h`,
  tvl: () => `sutara:tvl`,
  portfolio: (wallet: string) => `sutara:portfolio:${wallet}`,
  marketList: (filter: string) => `sutara:markets:list:${filter}`,
} as const;

export const CacheTTL = {
  market: 30,
  pool: 10,
  odds: 5,
  matchesLive: 10,
  match: 30,
  leaderboard: 60,
  volume24h: 60,
  tvl: 30,
  portfolio: 15,
  marketList: 10,
} as const;
