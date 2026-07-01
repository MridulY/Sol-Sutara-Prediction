use anchor_lang::prelude::*;
use crate::constants::*;

// ─── Market Type ─────────────────────────────────────────────────────────────

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum MarketType {
    /// Home / Draw / Away (or binary YES/NO for head-to-head)
    WinnerMarket,
    /// Over/Under — threshold stored as integer × 10 (e.g. 25 = 2.5 goals)
    OverUnder { threshold_x10: u16 },
    /// Will both teams score?
    BothTeamsScore,
    /// First goal scorer (player identified by TxLINE player_id)
    FirstGoalScorer { player_id: u32 },
    /// First team to receive a yellow card
    FirstYellowCard { team: TeamSide },
    /// Total corners over/under
    Corners { threshold_x10: u16 },
    /// Completely custom market — outcome definition stored off-chain, hash on-chain
    Custom { description_hash: [u8; 32] },
}

impl MarketType {
    pub fn discriminant(&self) -> u8 {
        match self {
            MarketType::WinnerMarket => 0,
            MarketType::OverUnder { .. } => 1,
            MarketType::BothTeamsScore => 2,
            MarketType::FirstGoalScorer { .. } => 3,
            MarketType::FirstYellowCard { .. } => 4,
            MarketType::Corners { .. } => 5,
            MarketType::Custom { .. } => 6,
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum TeamSide {
    Home,
    Away,
}

// ─── Market Status ────────────────────────────────────────────────────────────

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum MarketStatus {
    /// Created but not yet open for trading
    Pending,
    /// Actively trading
    Open,
    /// Trading closed, awaiting proof submission
    Closed,
    /// Proof submitted, CPI validation pending
    Disputed,
    /// Outcome finalised — claims are open
    Resolved,
    /// Emergency cancelled — full refund available
    Cancelled,
}

// ─── Outcome Label ────────────────────────────────────────────────────────────

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct OutcomeLabel {
    /// UTF-8 label, max MAX_OUTCOME_LABEL_LEN bytes
    pub label: [u8; MAX_OUTCOME_LABEL_LEN],
    pub label_len: u8,
}

impl OutcomeLabel {
    pub fn from_str(s: &str) -> Self {
        let bytes = s.as_bytes();
        let len = bytes.len().min(MAX_OUTCOME_LABEL_LEN);
        let mut label = [0u8; MAX_OUTCOME_LABEL_LEN];
        label[..len].copy_from_slice(&bytes[..len]);
        Self { label, label_len: len as u8 }
    }

    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.label[..self.label_len as usize]).unwrap_or("")
    }
}

// ─── Market Account ───────────────────────────────────────────────────────────

#[account]
pub struct Market {
    /// Wallet that created this market
    pub creator: Pubkey,
    /// TxLINE match identifier (UTF-8 padded to 32 bytes)
    pub match_id: [u8; 32],
    /// What this market resolves on
    pub market_type: MarketType,
    /// Outcome labels (fixed array of MAX_OUTCOMES, valid up to num_outcomes)
    pub outcomes: [OutcomeLabel; MAX_OUTCOMES],
    /// Number of valid outcomes (2–8)
    pub num_outcomes: u8,
    /// Current lifecycle status
    pub status: MarketStatus,
    /// Unix timestamp when trading opens (0 = immediately)
    pub open_ts: i64,
    /// Unix timestamp when trading closes
    pub close_ts: i64,
    /// Unix timestamp when market was resolved (None until resolved)
    pub resolve_ts: Option<i64>,
    /// Index of winning outcome (None until resolved)
    pub winning_outcome: Option<u8>,
    /// AMM pool account
    pub pool: Pubkey,
    /// USDC vault token account (PDA-controlled)
    pub vault: Pubkey,
    /// Resolution proof account
    pub resolution: Pubkey,
    /// Fee vault for creator earnings
    pub creator_fee_vault: Pubkey,
    /// Creator fee override (0 = use protocol default)
    pub creator_fee_bps: u16,
    /// Total USDC volume traded through this market (in USDC lamports)
    pub total_volume: u64,
    /// Monotonic version for client cache-busting
    pub version: u32,
    /// PDA bump
    pub bump: u8,
    pub _padding: [u8; 30],
}

impl Market {
    pub const LEN: usize = 8         // discriminator
        + 32                         // creator
        + 32                         // match_id
        + 64                         // market_type (generous upper bound for enum + variants)
        + (MAX_OUTCOMES * (MAX_OUTCOME_LABEL_LEN + 1)) // outcomes
        + 1                          // num_outcomes
        + 2                          // status (enum)
        + 8                          // open_ts
        + 8                          // close_ts
        + 9                          // resolve_ts (Option<i64>)
        + 2                          // winning_outcome (Option<u8>)
        + 32                         // pool
        + 32                         // vault
        + 32                         // resolution
        + 32                         // creator_fee_vault
        + 2                          // creator_fee_bps
        + 8                          // total_volume
        + 4                          // version
        + 1                          // bump
        + 30;                        // padding
}
