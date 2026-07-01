use anchor_lang::prelude::*;
use crate::constants::MAX_OUTCOMES;

/// Per-user, per-market position — tracks shares held and LP tokens
#[account]
pub struct Position {
    /// The market this position belongs to
    pub market: Pubkey,
    /// Position owner
    pub owner: Pubkey,
    /// Shares held per outcome (in SHARE_SCALE units)
    pub shares: [u64; MAX_OUTCOMES],
    /// LP tokens held in this market's pool
    pub lp_tokens: u64,
    /// Total USDC spent buying shares (for P&L display, not enforced on-chain)
    pub cost_basis: u64,
    /// Snapshot of pool.fees_per_lp_token at time of last LP deposit/claim
    /// Used to calculate accrued LP fees: (current - checkpoint) × lp_tokens
    pub fees_per_lp_checkpoint: u128,
    /// Whether share rewards have been claimed (prevents double claim)
    pub claimed: bool,
    pub bump: u8,
    pub _padding: [u8; 14],
}

impl Position {
    pub const LEN: usize = 8         // discriminator
        + 32                         // market
        + 32                         // owner
        + (MAX_OUTCOMES * 8)         // shares
        + 8                          // lp_tokens
        + 8                          // cost_basis
        + 16                         // fees_per_lp_checkpoint (u128)
        + 1                          // claimed
        + 1                          // bump
        + 14;                        // padding
}
