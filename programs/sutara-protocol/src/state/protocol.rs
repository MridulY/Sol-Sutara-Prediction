use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct ProtocolConfig {
    /// Admin authority (should be a multisig in production)
    pub admin: Pubkey,
    /// Treasury that receives protocol fees
    pub treasury: Pubkey,
    /// TxLINE on-chain validation program ID
    pub txline_program: Pubkey,
    /// Latest Merkle root published by TxLINE (updated via update_config)
    pub txline_merkle_root: [u8; 32],
    /// Protocol fee in basis points (0.01% = 1 bps)
    pub protocol_fee_bps: u16,
    /// Default creator fee in basis points (market creator overrides per market)
    pub creator_fee_bps: u16,
    /// LP fee in basis points
    pub lp_fee_bps: u16,
    /// USDC lamports required to create a market
    pub market_creation_fee: u64,
    /// Whether the entire protocol is paused
    pub paused: bool,
    /// Total markets ever created (monotonic counter)
    pub total_markets: u64,
    /// PDA bump
    pub bump: u8,
    // Padding for future fields without realloc
    pub _padding: [u8; 61],
}

impl ProtocolConfig {
    pub const LEN: usize = 8 + // discriminator
        32 + // admin
        32 + // treasury
        32 + // txline_program
        32 + // txline_merkle_root
        2  + // protocol_fee_bps
        2  + // creator_fee_bps
        2  + // lp_fee_bps
        8  + // market_creation_fee
        1  + // paused
        8  + // total_markets
        1  + // bump
        61;  // padding
}
