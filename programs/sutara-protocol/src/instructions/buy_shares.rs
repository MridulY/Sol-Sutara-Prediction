use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use crate::constants::*;
use crate::errors::SutaraError;
use crate::events::SharesBought;
use crate::state::{Market, MarketStatus, Pool, Position, ProtocolConfig, CreatorFeeVault};
use crate::amm::{lmsr, fees};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct BuySharesParams {
    /// Which outcome to buy shares of
    pub outcome_idx: u8,
    /// Number of shares to purchase (in SHARE_SCALE units)
    pub shares: u64,
    /// Maximum USDC to spend including fees (slippage protection)
    pub max_cost: u64,
}

#[derive(Accounts)]
#[instruction(params: BuySharesParams)]
pub struct BuyShares<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

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
        init_if_needed,
        payer = buyer,
        space = Position::LEN,
        seeds = [SEED_POSITION, market.key().as_ref(), buyer.key().as_ref()],
        bump,
    )]
    pub position: Box<Account<'info, Position>>,

    /// USDC vault (receives gross cost)
    #[account(
        mut,
        seeds = [SEED_VAULT, market.key().as_ref()],
        bump,
        constraint = vault.mint == usdc_mint.key() @ SutaraError::InvalidMint,
    )]
    pub vault: Box<Account<'info, TokenAccount>>,

    /// Protocol treasury (receives protocol fee)
    #[account(
        mut,
        constraint = protocol_treasury.mint == usdc_mint.key() @ SutaraError::InvalidMint,
    )]
    pub protocol_treasury: Box<Account<'info, TokenAccount>>,

    /// Creator fee vault (receives creator fee)
    #[account(
        mut,
        seeds = [b"creator_fee_vault", market.key().as_ref()],
        bump = creator_fee_vault.bump,
    )]
    pub creator_fee_vault_account: Box<Account<'info, CreatorFeeVault>>,

    /// Buyer's USDC account
    #[account(
        mut,
        constraint = buyer_usdc.owner == buyer.key() @ SutaraError::InvalidTokenOwner,
        constraint = buyer_usdc.mint == usdc_mint.key() @ SutaraError::InvalidMint,
    )]
    pub buyer_usdc: Box<Account<'info, TokenAccount>>,

    pub usdc_mint: Box<Account<'info, Mint>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
}

pub fn handler(ctx: Context<BuyShares>, params: BuySharesParams) -> Result<()> {
    let clock = &ctx.accounts.clock;

    // ── Validate timing ───────────────────────────────────────────────────
    require!(
        clock.unix_timestamp < ctx.accounts.market.close_ts,
        SutaraError::MarketTradingClosed
    );
    require!(
        clock.unix_timestamp >= ctx.accounts.market.open_ts,
        SutaraError::MarketNotOpen
    );
    require!(
        (params.outcome_idx as usize) < ctx.accounts.market.num_outcomes as usize,
        SutaraError::InvalidOutcomeIndex
    );
    require!(params.shares > 0, SutaraError::InvalidAmount);

    let pool = &ctx.accounts.pool;
    let num_outcomes = pool.num_outcomes as usize;

    // ── Compute LMSR cost ─────────────────────────────────────────────────
    let (gross_cost, new_quantities) = lmsr::cost_to_buy(
        pool.b_parameter,
        &pool.outcome_quantities,
        num_outcomes,
        params.outcome_idx as usize,
        params.shares,
    )?;

    // ── Compute fees ──────────────────────────────────────────────────────
    let fee_breakdown = fees::calculate_buy_fees(
        gross_cost,
        &ctx.accounts.protocol,
        ctx.accounts.market.creator_fee_bps,
    )?;

    // ── Slippage check ────────────────────────────────────────────────────
    require!(
        fee_breakdown.net_amount <= params.max_cost,
        SutaraError::SlippageExceeded
    );

    // ── Transfer gross cost to vault ──────────────────────────────────────
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.buyer_usdc.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
                authority: ctx.accounts.buyer.to_account_info(),
            },
        ),
        gross_cost,
    )?;

    // ── Transfer protocol fee to treasury ─────────────────────────────────
    if fee_breakdown.protocol_fee > 0 {
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.buyer_usdc.to_account_info(),
                    to: ctx.accounts.protocol_treasury.to_account_info(),
                    authority: ctx.accounts.buyer.to_account_info(),
                },
            ),
            fee_breakdown.protocol_fee,
        )?;
    }

    // ── Accrue creator fee ────────────────────────────────────────────────
    if fee_breakdown.creator_fee > 0 {
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.buyer_usdc.to_account_info(),
                    to: ctx.accounts.vault.to_account_info(), // creator claims from vault
                    authority: ctx.accounts.buyer.to_account_info(),
                },
            ),
            fee_breakdown.creator_fee,
        )?;
        ctx.accounts.creator_fee_vault_account.accumulated = ctx
            .accounts.creator_fee_vault_account.accumulated
            .checked_add(fee_breakdown.creator_fee)
            .ok_or(SutaraError::Overflow)?;
    }

    // ── Accrue LP fees ────────────────────────────────────────────────────
    let pool = &mut ctx.accounts.pool;
    if fee_breakdown.lp_fee > 0 && pool.lp_supply > 0 {
        // Increase fees_per_lp_token checkpoint
        // Scale by 1e12 for precision: delta = lp_fee * 1e12 / lp_supply
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

        // Transfer LP fee to vault (LP fees claimed separately)
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.buyer_usdc.to_account_info(),
                    to: ctx.accounts.vault.to_account_info(),
                    authority: ctx.accounts.buyer.to_account_info(),
                },
            ),
            fee_breakdown.lp_fee,
        )?;
    }

    // ── Update pool quantities ────────────────────────────────────────────
    for i in 0..num_outcomes {
        pool.outcome_quantities[i] = new_quantities[i];
    }

    // ── Update market stats ───────────────────────────────────────────────
    let market = &mut ctx.accounts.market;
    market.total_volume = market.total_volume
        .checked_add(fee_breakdown.net_amount)
        .ok_or(SutaraError::Overflow)?;
    market.version = market.version.checked_add(1).ok_or(SutaraError::Overflow)?;

    // ── Update position ───────────────────────────────────────────────────
    let position = &mut ctx.accounts.position;
    if position.market == Pubkey::default() {
        position.market = market.key();
        position.owner = ctx.accounts.buyer.key();
        position.bump = ctx.bumps.position;
    }
    position.shares[params.outcome_idx as usize] = position.shares[params.outcome_idx as usize]
        .checked_add(params.shares)
        .ok_or(SutaraError::Overflow)?;
    position.cost_basis = position.cost_basis
        .checked_add(fee_breakdown.net_amount)
        .ok_or(SutaraError::Overflow)?;

    // ── Compute new probability for event ─────────────────────────────────
    let new_probability = lmsr::probability(
        pool.b_parameter,
        &pool.outcome_quantities,
        num_outcomes,
        params.outcome_idx as usize,
    ).unwrap_or(0);

    emit!(SharesBought {
        market: market.key(),
        buyer: ctx.accounts.buyer.key(),
        outcome_idx: params.outcome_idx,
        shares: params.shares,
        cost: gross_cost,
        protocol_fee: fee_breakdown.protocol_fee,
        creator_fee: fee_breakdown.creator_fee,
        lp_fee: fee_breakdown.lp_fee,
        new_probability,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}
