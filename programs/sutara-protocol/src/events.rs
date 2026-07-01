use anchor_lang::prelude::*;

// ─── Protocol Events ─────────────────────────────────────────────────────────

#[event]
pub struct ProtocolInitialized {
    pub admin: Pubkey,
    pub treasury: Pubkey,
    pub txline_program: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct ProtocolPaused {
    pub admin: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct ProtocolUnpaused {
    pub admin: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct FeesUpdated {
    pub protocol_fee_bps: u16,
    pub creator_fee_bps: u16,
    pub lp_fee_bps: u16,
    pub timestamp: i64,
}

#[event]
pub struct ConfigUpdated {
    pub txline_program: Pubkey,
    pub txline_merkle_root: [u8; 32],
    pub timestamp: i64,
}

// ─── Market Events ────────────────────────────────────────────────────────────

#[event]
pub struct MarketCreated {
    pub market: Pubkey,
    pub creator: Pubkey,
    pub match_id: [u8; 32],
    pub market_type_discriminant: u8,
    pub num_outcomes: u8,
    pub close_ts: i64,
    pub timestamp: i64,
}

#[event]
pub struct MarketClosed {
    pub market: Pubkey,
    pub closed_by: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct MarketResolved {
    pub market: Pubkey,
    pub winning_outcome: u8,
    pub resolved_by: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct MarketCancelled {
    pub market: Pubkey,
    pub cancelled_by: Pubkey,
    pub timestamp: i64,
}

// ─── Pool / Liquidity Events ──────────────────────────────────────────────────

#[event]
pub struct PoolInitialized {
    pub market: Pubkey,
    pub pool: Pubkey,
    pub b_parameter: u64,
    pub initial_liquidity: u64,
    pub lp_tokens_minted: u64,
    pub timestamp: i64,
}

#[event]
pub struct LiquidityDeposited {
    pub market: Pubkey,
    pub pool: Pubkey,
    pub provider: Pubkey,
    pub usdc_amount: u64,
    pub lp_tokens_minted: u64,
    pub timestamp: i64,
}

#[event]
pub struct LiquidityWithdrawn {
    pub market: Pubkey,
    pub pool: Pubkey,
    pub provider: Pubkey,
    pub lp_tokens_burned: u64,
    pub usdc_returned: u64,
    pub timestamp: i64,
}

#[event]
pub struct LpFeesClaimed {
    pub market: Pubkey,
    pub claimant: Pubkey,
    pub fees_claimed: u64,
    pub timestamp: i64,
}

// ─── Trading Events ───────────────────────────────────────────────────────────

#[event]
pub struct SharesBought {
    pub market: Pubkey,
    pub buyer: Pubkey,
    pub outcome_idx: u8,
    pub shares: u64,
    pub cost: u64,
    pub protocol_fee: u64,
    pub creator_fee: u64,
    pub lp_fee: u64,
    /// Probability of outcome after trade (scaled by 1e9)
    pub new_probability: u64,
    pub timestamp: i64,
}

#[event]
pub struct SharesSold {
    pub market: Pubkey,
    pub seller: Pubkey,
    pub outcome_idx: u8,
    pub shares: u64,
    pub proceeds: u64,
    pub protocol_fee: u64,
    pub creator_fee: u64,
    pub lp_fee: u64,
    /// Probability of outcome after trade (scaled by 1e9)
    pub new_probability: u64,
    pub timestamp: i64,
}

// ─── Settlement Events ────────────────────────────────────────────────────────

#[event]
pub struct ProofSubmitted {
    pub market: Pubkey,
    pub submitted_by: Pubkey,
    pub merkle_root: [u8; 32],
    pub timestamp: i64,
}

#[event]
pub struct ResultValidated {
    pub market: Pubkey,
    pub validated_by: Pubkey,
    pub winning_outcome: u8,
    pub timestamp: i64,
}

// ─── Claim Events ─────────────────────────────────────────────────────────────

#[event]
pub struct RewardsClaimed {
    pub market: Pubkey,
    pub claimant: Pubkey,
    pub winning_shares: u64,
    pub usdc_payout: u64,
    pub timestamp: i64,
}

#[event]
pub struct EmergencyWithdrawn {
    pub market: Pubkey,
    pub recipient: Pubkey,
    pub usdc_amount: u64,
    pub timestamp: i64,
}
