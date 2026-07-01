use anchor_lang::prelude::*;
use crate::constants::MAX_OUTCOMES;

/// AMM pool state — holds the LMSR quantity vector and LP accounting
#[account]
pub struct Pool {
    /// Parent market
    pub market: Pubkey,
    /// LP token mint (one per market)
    pub lp_mint: Pubkey,
    /// LMSR liquidity parameter b (in USDC lamports × SHARE_SCALE)
    ///
    /// Controls price sensitivity: larger b = deeper market = lower slippage.
    /// Max loss for LPs ≤ b × ln(num_outcomes).
    pub b_parameter: u64,
    /// Share quantity vector q_i for each outcome.
    ///
    /// Stored as fixed-point integers scaled by SHARE_SCALE (1e9).
    /// q_i can be negative if more sells than buys (LMSR allows this).
    /// Use i128 to handle negative values safely.
    pub outcome_quantities: [i128; MAX_OUTCOMES],
    /// Total LP tokens outstanding
    pub lp_supply: u64,
    /// Total LP fees accumulated (in USDC lamports), never resets
    pub total_fees_accumulated: u64,
    /// Fees per LP token at last distribution checkpoint (scaled by 1e12 for precision)
    pub fees_per_lp_token: u128,
    /// Number of valid outcomes (mirrors Market.num_outcomes)
    pub num_outcomes: u8,
    pub bump: u8,
    pub _padding: [u8; 14],
}

impl Pool {
    pub const LEN: usize = 8         // discriminator
        + 32                         // market
        + 32                         // lp_mint
        + 8                          // b_parameter
        + (MAX_OUTCOMES * 16)        // outcome_quantities (i128 = 16 bytes each)
        + 8                          // lp_supply
        + 8                          // total_fees_accumulated
        + 16                         // fees_per_lp_token (u128)
        + 1                          // num_outcomes
        + 1                          // bump
        + 14;                        // padding
}
