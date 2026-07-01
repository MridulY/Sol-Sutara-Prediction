use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::constants::*;
use crate::errors::SutaraError;
use crate::events::ProtocolInitialized;
use crate::state::{ProtocolConfig, KeeperRegistry};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeProtocolParams {
    pub txline_program: Pubkey,
    pub txline_merkle_root: [u8; 32],
    pub protocol_fee_bps: u16,
    pub creator_fee_bps: u16,
    pub lp_fee_bps: u16,
    pub market_creation_fee: u64,
}

#[derive(Accounts)]
pub struct InitializeProtocol<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = ProtocolConfig::LEN,
        seeds = [SEED_PROTOCOL],
        bump,
    )]
    pub protocol: Box<Account<'info, ProtocolConfig>>,

    #[account(
        init,
        payer = admin,
        space = KeeperRegistry::LEN,
        seeds = [SEED_KEEPER_REGISTRY],
        bump,
    )]
    pub keeper_registry: Box<Account<'info, KeeperRegistry>>,

    /// CHECK: This is the protocol treasury token account (must be a USDC ATA)
    #[account(mut)]
    pub treasury: AccountInfo<'info>,

    pub usdc_mint: Box<Account<'info, Mint>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<InitializeProtocol>, params: InitializeProtocolParams) -> Result<()> {
    require!(
        params.protocol_fee_bps <= MAX_PROTOCOL_FEE_BPS,
        SutaraError::ProtocolFeeTooHigh
    );
    require!(
        params.creator_fee_bps <= MAX_CREATOR_FEE_BPS,
        SutaraError::CreatorFeeTooHigh
    );
    require!(
        params.lp_fee_bps <= MAX_LP_FEE_BPS,
        SutaraError::LpFeeTooHigh
    );
    require!(
        params.protocol_fee_bps as u32 + params.creator_fee_bps as u32 + params.lp_fee_bps as u32
            <= MAX_TOTAL_FEE_BPS as u32,
        SutaraError::TotalFeesTooHigh
    );

    let protocol = &mut ctx.accounts.protocol;
    protocol.admin = ctx.accounts.admin.key();
    protocol.treasury = ctx.accounts.treasury.key();
    protocol.txline_program = params.txline_program;
    protocol.txline_merkle_root = params.txline_merkle_root;
    protocol.protocol_fee_bps = params.protocol_fee_bps;
    protocol.creator_fee_bps = params.creator_fee_bps;
    protocol.lp_fee_bps = params.lp_fee_bps;
    protocol.market_creation_fee = params.market_creation_fee;
    protocol.paused = false;
    protocol.total_markets = 0;
    protocol.bump = ctx.bumps.protocol;

    let registry = &mut ctx.accounts.keeper_registry;
    registry.admin = ctx.accounts.admin.key();
    registry.keeper_count = 0;
    registry.bump = ctx.bumps.keeper_registry;

    emit!(ProtocolInitialized {
        admin: ctx.accounts.admin.key(),
        treasury: ctx.accounts.treasury.key(),
        txline_program: params.txline_program,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
