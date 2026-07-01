use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use crate::constants::*;
use crate::errors::SutaraError;
use crate::events::RewardsClaimed;
use crate::state::{Market, MarketStatus, Pool, Position};

#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    #[account(mut)]
    pub claimant: Signer<'info>,

    #[account(
        seeds = [SEED_MARKET, &market.match_id, &[market.market_type.discriminant()]],
        bump = market.bump,
        constraint = market.status == MarketStatus::Resolved @ SutaraError::ResultNotValidated,
        constraint = market.winning_outcome.is_some() @ SutaraError::ResultNotValidated,
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
        constraint = !position.claimed @ SutaraError::AlreadyClaimed,
    )]
    pub position: Box<Account<'info, Position>>,

    #[account(
        mut,
        seeds = [SEED_VAULT, market.key().as_ref()],
        bump,
    )]
    pub vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = claimant_usdc.owner == claimant.key() @ SutaraError::InvalidTokenOwner,
    )]
    pub claimant_usdc: Box<Account<'info, TokenAccount>>,

    pub usdc_mint: Box<Account<'info, Mint>>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<ClaimRewards>) -> Result<()> {
    let winning_outcome = ctx.accounts.market.winning_outcome.unwrap() as usize;
    let winning_shares = ctx.accounts.position.shares[winning_outcome];

    require!(winning_shares > 0, SutaraError::NoWinningShares);

    // Compute payout: each winning share redeems proportionally from vault
    // payout = winning_shares × vault_balance / total_winning_shares_in_pool
    //
    // total_winning_shares = pool.outcome_quantities[winning_outcome] (total bought)
    // This is the total shares outstanding for the winning outcome.
    let total_winning = ctx.accounts.pool.outcome_quantities[winning_outcome];
    require!(total_winning > 0, SutaraError::DivisionByZero);

    let vault_balance = ctx.accounts.vault.amount;

    let payout = (winning_shares as u128)
        .checked_mul(vault_balance as u128)
        .ok_or(SutaraError::Overflow)?
        .checked_div(total_winning as u128)
        .ok_or(SutaraError::DivisionByZero)? as u64;

    require!(payout > 0, SutaraError::NoWinningShares);

    // Transfer payout from vault to claimant
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
        payout,
    )?;

    // Mark claimed
    let position = &mut ctx.accounts.position;
    position.claimed = true;

    emit!(RewardsClaimed {
        market: market.key(),
        claimant: ctx.accounts.claimant.key(),
        winning_shares,
        usdc_payout: payout,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
