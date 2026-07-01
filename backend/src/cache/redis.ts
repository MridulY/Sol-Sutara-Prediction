import { Redis } from 'ioredis';

let _redis: Redis;

export function createRedis(redisUrl: string): Redis {
  _redis = new Redis(redisUrl, {
    maxRetriesPerRequest: 3,
    enableReadyCheck: true,
    lazyConnect: false,
  });

  _redis.on('error', (err) => {
    console.error('Redis connection error:', err);
  });

  return _redis;
}

export function getRedis(): Redis {
  if (!_redis) throw new Error('Redis not initialized');
  return _redis;
}
