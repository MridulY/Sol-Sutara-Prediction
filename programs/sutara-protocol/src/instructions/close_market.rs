use anchor_lang::prelude::*;
use crate::constants::*;
use crate::errors::SutaraError;
use crate::events::MarketClosed;
use crate::state::{Market, MarketStatus, ProtocolConfig};

#[derive(Accounts)]
pub struct CloseMarket<'info> {
    pub closer: Signer<'info>,

    #[account(
        seeds = [SEED_PROTOCOL],
        bump = protocol.bump,
    )]
    pub protocol: Box<Account<'info, ProtocolConfig>>,

    #[account(
        mut,
        seeds = [SEED_MARKET, &market.match_id, &[market.market_type.discriminant()]],
        bump = market.bump,
        constraint = market.status == MarketStatus::Open || market.status == MarketStatus::Pending
            @ SutaraError::MarketAlreadyResolved,
    )]
    pub market: Box<Account<'info, Market>>,

    pub clock: Sysvar<'info, Clock>,
}

pub fn handler(ctx: Context<CloseMarket>) -> Result<()> {
    let clock = &ctx.accounts.clock;
    let market = &mut ctx.accounts.market;
    let protocol = &ctx.accounts.protocol;

    // Anyone can close a market once its close_ts has passed
    // Admin can force-close at any time (emergency)
    let is_admin = ctx.accounts.closer.key() == protocol.admin;
    let past_close_time = clock.unix_timestamp >= market.close_ts;

    require!(
        past_close_time || is_admin,
        SutaraError::MarketNotYetClosed
    );

    market.status = MarketStatus::Closed;
    market.version = market.version.checked_add(1).ok_or(SutaraError::Overflow)?;

    emit!(MarketClosed {
        market: market.key(),
        closed_by: ctx.accounts.closer.key(),
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}
