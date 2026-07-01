use anchor_lang::prelude::*;

/// Per-market creator fee accumulator
/// Protocol fees go directly to protocol treasury token account.
/// Creator fees accumulate here and are claimable by the market creator.
#[account]
pub struct CreatorFeeVault {
    pub market: Pubkey,
    pub creator: Pubkey,
    /// Total USDC fees accumulated (in lamports)
    pub accumulated: u64,
    /// Total USDC fees ever claimed
    pub total_claimed: u64,
    pub bump: u8,
    pub _padding: [u8; 7],
}

impl CreatorFeeVault {
    pub const LEN: usize = 8    // discriminator
        + 32                    // market
        + 32                    // creator
        + 8                     // accumulated
        + 8                     // total_claimed
        + 1                     // bump
        + 7;                    // padding
}
