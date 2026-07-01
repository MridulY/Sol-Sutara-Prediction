use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use crate::constants::*;
use crate::errors::SutaraError;
use crate::events::LpFeesClaimed;
use crate::state::{Market, Pool, Position};

#[derive(Accounts)]
pub struct ClaimLpFees<'info> {
    #[account(mut)]
    pub claimant: Signer<'info>,

    #[account(
        seeds = [SEED_MARKET, &market.match_id, &[market.market_type.discriminant()]],
        bump = market.bump,
    )]
    pub market: Box<Account<'info, Market>>,

    #[account(
        seeds = [SEED_POOL, market.key().as_ref()],
        bump = pool.bump,
    )]
    pub pool: Box<Account<'info, Pool>>,

    #[account(
        mut,
        seeds = [SEED_POSITION, market.key().as_ref(), claimant.key().as_ref()],
        bump = position.bump,
        constraint = position.owner == claimant.key() @ SutaraError::Unauthorized,
        constraint = position.lp_tokens > 0 @ SutaraError::InsufficientLpTokens,
    )]
    pub position: Box<Account<'info, Position>>,

    #[account(
        mut,
        seeds = [SEED_VAULT, market.key().as_ref()],
        bump,
    )]
    pub vault: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub claimant_usdc: Box<Account<'info, TokenAccount>>,

    pub usdc_mint: Box<Account<'info, Mint>>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<ClaimLpFees>) -> Result<()> {
    let pool = &ctx.accounts.pool;
    let position = &ctx.accounts.position;

    // Accrued fees = (pool.fees_per_lp_token - position.fees_per_lp_checkpoint)
    //                × position.lp_tokens / 1e12
    let delta = pool.fees_per_lp_token
        .checked_sub(position.fees_per_lp_checkpoint)
        .unwrap_or(0);

    let fees_owed = (delta as u128)
        .checked_mul(position.lp_tokens as u128)
        .ok_or(SutaraError::Overflow)?
        .checked_div(1_000_000_000_000u128) // scale factor used in accumulation
        .ok_or(SutaraError::DivisionByZero)? as u64;

    require!(fees_owed > 0, SutaraError::InvalidAmount);

    let market = &ctx.accounts.market;
    let market_seeds: &[&[u8]] = &[
        SEED_MARKET,
        market.match_id.as_ref(),
        &[market.market_type.discriminant()],
        &[market.bump],
    ];

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault.to_account_info(),
                to: ctx.accounts.claimant_usdc.to_account_info(),
                authority: ctx.accounts.market.to_account_info(),
            },
            &[market_seeds],
        ),
        fees_owed,
    )?;

    // Update checkpoint
    let position = &mut ctx.accounts.position;
    position.fees_per_lp_checkpoint = ctx.accounts.pool.fees_per_lp_token;

    emit!(LpFeesClaimed {
        market: ctx.accounts.market.key(),
        claimant: ctx.accounts.claimant.key(),
        fees_claimed: fees_owed,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
