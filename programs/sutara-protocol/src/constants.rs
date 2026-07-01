// ─── Seeds ───────────────────────────────────────────────────────────────────
pub const SEED_PROTOCOL: &[u8] = b"protocol";
pub const SEED_KEEPER_REGISTRY: &[u8] = b"keeper_registry";
pub const SEED_FEE_VAULT: &[u8] = b"fee_vault";
pub const SEED_MARKET: &[u8] = b"market";
pub const SEED_POOL: &[u8] = b"pool";
pub const SEED_VAULT: &[u8] = b"vault";
pub const SEED_LP_MINT: &[u8] = b"lp_mint";
pub const SEED_POSITION: &[u8] = b"position";
pub const SEED_RESOLUTION: &[u8] = b"resolution";

// ─── Account Sizes ───────────────────────────────────────────────────────────
pub const DISCRIMINATOR: usize = 8;
pub const PROTOCOL_CONFIG_SIZE: usize = DISCRIMINATOR + 256;
pub const KEEPER_REGISTRY_SIZE: usize = DISCRIMINATOR + 1024;
pub const MARKET_SIZE: usize = DISCRIMINATOR + 512;
pub const POOL_SIZE: usize = DISCRIMINATOR + 256;
pub const POSITION_SIZE: usize = DISCRIMINATOR + 256;
pub const RESOLUTION_SIZE: usize = DISCRIMINATOR + 1024;

// ─── Protocol Limits ─────────────────────────────────────────────────────────
pub const MAX_OUTCOMES: usize = 8;
pub const MAX_KEEPERS: usize = 10;
pub const MAX_MERKLE_DEPTH: usize = 20;
pub const MAX_OUTCOME_LABEL_LEN: usize = 32;
pub const MAX_MARKET_DESCRIPTION_LEN: usize = 128;

// ─── Fee Limits (basis points) ───────────────────────────────────────────────
pub const MAX_PROTOCOL_FEE_BPS: u16 = 500;   // 5%
pub const MAX_CREATOR_FEE_BPS: u16 = 300;    // 3%
pub const MAX_LP_FEE_BPS: u16 = 500;         // 5%
pub const MAX_TOTAL_FEE_BPS: u16 = 1_000;    // 10% total

// ─── Default Fees ────────────────────────────────────────────────────────────
pub const DEFAULT_PROTOCOL_FEE_BPS: u16 = 30;   // 0.30%
pub const DEFAULT_CREATOR_FEE_BPS: u16 = 20;    // 0.20%
pub const DEFAULT_LP_FEE_BPS: u16 = 50;         // 0.50%

// ─── USDC ─────────────────────────────────────────────────────────────────────
pub const USDC_DECIMALS: u8 = 6;
pub const USDC_SCALE: u64 = 1_000_000; // 1 USDC

// ─── LMSR / AMM ──────────────────────────────────────────────────────────────
// Fixed-point scale for share quantities (1 share = SHARE_SCALE units)
pub const SHARE_SCALE: u64 = 1_000_000_000; // 1e9
// Minimum b parameter (1 USDC worth of liquidity)
pub const MIN_B_PARAMETER: u64 = USDC_SCALE;
// Minimum initial liquidity to initialize a pool
pub const MIN_INITIAL_LIQUIDITY: u64 = 10 * USDC_SCALE; // 10 USDC
// Initial LP shares minted (fixed supply representation)
pub const INITIAL_LP_SHARES: u64 = 1_000_000 * USDC_SCALE;

// ─── Market Creation ─────────────────────────────────────────────────────────
pub const MARKET_CREATION_FEE: u64 = USDC_SCALE; // 1 USDC
// Minimum time before close_ts must be in the future (10 minutes)
pub const MIN_MARKET_DURATION_SECS: i64 = 600;
