use anchor_lang::prelude::*;
use crate::constants::*;
use crate::errors::SutaraError;
use crate::events::MarketResolved;
use crate::state::{Market, MarketStatus, Resolution};

#[derive(Accounts)]
pub struct ResolveMarket<'info> {
    /// Permissionless: anyone can trigger once validation is complete
    pub resolver: Signer<'info>,

    #[account(
        mut,
        seeds = [SEED_MARKET, &market.match_id, &[market.market_type.discriminant()]],
        bump = market.bump,
        constraint = market.status == MarketStatus::Disputed @ SutaraError::ProofNotSubmitted,
    )]
    pub market: Box<Account<'info, Market>>,

    #[account(
        seeds = [SEED_RESOLUTION, market.key().as_ref()],
        bump = resolution.bump,
        constraint = resolution.validated @ SutaraError::ResultNotValidated,
        constraint = resolution.winning_outcome.is_some() @ SutaraError::ResultNotValidated,
    )]
    pub resolution: Box<Account<'info, Resolution>>,

    pub clock: Sysvar<'info, Clock>,
}

pub fn handler(ctx: Context<ResolveMarket>) -> Result<()> {
    let winning_outcome = ctx.accounts.resolution.winning_outcome.unwrap();
    let clock = &ctx.accounts.clock;

    let market = &mut ctx.accounts.market;
    market.status = MarketStatus::Resolved;
    market.winning_outcome = Some(winning_outcome);
    market.resolve_ts = Some(clock.unix_timestamp);
    market.version = market.version.checked_add(1).ok_or(SutaraError::Overflow)?;

    emit!(MarketResolved {
        market: market.key(),
        winning_outcome,
        resolved_by: ctx.accounts.resolver.key(),
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}
