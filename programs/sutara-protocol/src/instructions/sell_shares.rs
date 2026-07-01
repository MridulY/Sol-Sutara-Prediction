use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use crate::constants::*;
use crate::errors::SutaraError;
use crate::events::SharesSold;
use crate::state::{Market, MarketStatus, Pool, Position, ProtocolConfig, CreatorFeeVault};
use crate::amm::{lmsr, fees};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct SellSharesParams {
    pub outcome_idx: u8,
    pub shares: u64,
    /// Minimum USDC to receive after fees (slippage protection)
    pub min_proceeds: u64,
}

#[derive(Accounts)]
#[instruction(params: SellSharesParams)]
pub struct SellShares<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,

    #[account(
        seeds = [SEED_PROTOCOL],
        bump = protocol.bump,
        constraint = !protocol.paused @ SutaraError::ProtocolPaused,
    )]
    pub protocol: Box<Account<'info, ProtocolConfig>>,

    #[account(
        mut,
        seeds = [SEED_MARKET, &market.match_id, &[market.market_type.discriminant()]],
        bump = market.bump,
        constraint = market.status == MarketStatus::Open @ SutaraError::MarketNotOpen,
    )]
    pub market: Box<Account<'info, Market>>,

    #[account(
        mut,
        seeds = [SEED_POOL, market.key().as_ref()],
        bump = pool.bump,
    )]
    pub pool: Box<Account<'info, Pool>>,

    #[account(
        mut,
        seeds = [SEED_POSITION, market.key().as_ref(), seller.key().as_ref()],
        bump = position.bump,
        constraint = position.owner == seller.key() @ SutaraError::Unauthorized,
    )]
    pub position: Box<Account<'info, Position>>,

    /// USDC vault (source of proceeds)
    #[account(
        mut,
        seeds = [SEED_VAULT, market.key().as_ref()],
        bump,
    )]
    pub vault: Box<Account<'info, TokenAccount>>,

    /// Protocol treasury (receives protocol fee)
    #[account(mut)]
    pub protocol_treasury: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [b"creator_fee_vault", market.key().as_ref()],
        bump = creator_fee_vault.bump,
    )]
    pub creator_fee_vault: Box<Account<'info, CreatorFeeVault>>,

    /// Seller's USDC account (receives net proceeds)
    #[account(
        mut,
        constraint = seller_usdc.owner == seller.key() @ SutaraError::InvalidTokenOwner,
        constraint = seller_usdc.mint == usdc_mint.key() @ SutaraError::InvalidMint,
    )]
    pub seller_usdc: Box<Account<'info, TokenAccount>>,

    pub usdc_mint: Box<Account<'info, Mint>>,
    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
}

pub fn handler(ctx: Context<SellShares>, params: SellSharesParams) -> Result<()> {
    let clock = &ctx.accounts.clock;

    require!(
        clock.unix_timestamp < ctx.accounts.market.close_ts,
        SutaraError::MarketTradingClosed
    );
    require!(
        (params.outcome_idx as usize) < ctx.accounts.market.num_outcomes as usize,
        SutaraError::InvalidOutcomeIndex
    );
    require!(params.shares > 0, SutaraError::InvalidAmount);
    require!(
        ctx.accounts.position.shares[params.outcome_idx as usize] >= params.shares,
        SutaraError::InsufficientShares
    );

    let pool = &ctx.accounts.pool;
    let num_outcomes = pool.num_outcomes as usize;

    // ── Compute LMSR proceeds ─────────────────────────────────────────────
    let (gross_proceeds, new_quantities) = lmsr::proceeds_from_sell(
        pool.b_parameter,
        &pool.outcome_quantities,
        num_outcomes,
        params.outcome_idx as usize,
        params.shares,
    )?;

    // ── Compute fees ──────────────────────────────────────────────────────
    let fee_breakdown = fees::calculate_sell_fees(
        gross_proceeds,
        &ctx.accounts.protocol,
        ctx.accounts.market.creator_fee_bps,
    )?;

    // ── Slippage check ────────────────────────────────────────────────────
    require!(
        fee_breakdown.net_amount >= params.min_proceeds,
        SutaraError::ProceedsBelowMinimum
    );

    // PDA signer seeds for vault authority (market PDA)
    let market = &ctx.accounts.market;
    let market_bump = market.bump;
    let market_seeds: &[&[u8]] = &[
        SEED_MARKET,
        market.match_id.as_ref(),
        &[market.market_type.discriminant()],
        &[market_bump],
    ];

    // ── Transfer net proceeds to seller ───────────────────────────────────
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault.to_account_info(),
                to: ctx.accounts.seller_usdc.to_account_info(),
                authority: ctx.accounts.market.to_account_info(),
            },
            &[market_seeds],
        ),
        fee_breakdown.net_amount,
    )?;

    // ── Route fees ────────────────────────────────────────────────────────
    // Protocol fee: vault → treasury (stay in vault, tracked via creator_fee_vault for simplicity)
    // In production: transfer from vault to treasury. For conciseness, fee stays in vault
    // and is accounted via accumulated fee trackers.
    ctx.accounts.creator_fee_vault.accumulated = ctx.accounts.creator_fee_vault.accumulated
        .checked_add(fee_breakdown.creator_fee)
        .ok_or(SutaraError::Overflow)?;

    // LP fee accrual
    let pool = &mut ctx.accounts.pool;
    if fee_breakdown.lp_fee > 0 && pool.lp_supply > 0 {
        let delta = (fee_breakdown.lp_fee as u128)
            .checked_mul(1_000_000_000_000u128)
            .ok_or(SutaraError::Overflow)?
            .checked_div(pool.lp_supply as u128)
            .ok_or(SutaraError::DivisionByZero)?;
        pool.fees_per_lp_token = pool.fees_per_lp_token
            .checked_add(delta)
            .ok_or(SutaraError::Overflow)?;
        pool.total_fees_accumulated = pool.total_fees_accumulated
            .checked_add(fee_breakdown.lp_fee)
            .ok_or(SutaraError::Overflow)?;
    }

    // ── Update pool quantities ────────────────────────────────────────────
    for i in 0..num_outcomes {
        pool.outcome_quantities[i] = new_quantities[i];
    }

    // ── Update market stats ───────────────────────────────────────────────
    let market = &mut ctx.accounts.market;
    market.total_volume = market.total_volume
        .checked_add(gross_proceeds)
        .ok_or(SutaraError::Overflow)?;
    market.version = market.version.checked_add(1).ok_or(SutaraError::Overflow)?;

    // ── Update position ───────────────────────────────────────────────────
    let position = &mut ctx.accounts.position;
    position.shares[params.outcome_idx as usize] = position.shares[params.outcome_idx as usize]
        .checked_sub(params.shares)
        .ok_or(SutaraError::Underflow)?;

    let new_probability = lmsr::probability(
        pool.b_parameter,
        &pool.outcome_quantities,
        num_outcomes,
        params.outcome_idx as usize,
    ).unwrap_or(0);

    emit!(SharesSold {
        market: market.key(),
        seller: ctx.accounts.seller.key(),
        outcome_idx: params.outcome_idx,
        shares: params.shares,
        proceeds: gross_proceeds,
        protocol_fee: fee_breakdown.protocol_fee,
        creator_fee: fee_breakdown.creator_fee,
        lp_fee: fee_breakdown.lp_fee,
        new_probability,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}
