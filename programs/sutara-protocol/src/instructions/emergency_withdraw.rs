use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use crate::constants::*;
use crate::errors::SutaraError;
use crate::events::EmergencyWithdrawn;
use crate::state::{Market, MarketStatus, ProtocolConfig};

#[derive(Accounts)]
pub struct EmergencyWithdraw<'info> {
    pub admin: Signer<'info>,

    #[account(
        seeds = [SEED_PROTOCOL],
        bump = protocol.bump,
        constraint = admin.key() == protocol.admin @ SutaraError::Unauthorized,
    )]
    pub protocol: Box<Account<'info, ProtocolConfig>>,

    #[account(
        mut,
        seeds = [SEED_MARKET, &market.match_id, &[market.market_type.discriminant()]],
        bump = market.bump,
        constraint = market.status == MarketStatus::Cancelled @ SutaraError::MarketNotOpen,
    )]
    pub market: Box<Account<'info, Market>>,

    #[account(
        mut,
        seeds = [SEED_VAULT, market.key().as_ref()],
        bump,
    )]
    pub vault: Box<Account<'info, TokenAccount>>,

    /// Admin-controlled recipient account
    #[account(mut)]
    pub recipient: Box<Account<'info, TokenAccount>>,

    pub usdc_mint: Box<Account<'info, Mint>>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<EmergencyWithdraw>) -> Result<()> {
    let amount = ctx.accounts.vault.amount;
    require!(amount > 0, SutaraError::InvalidAmount);

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
                to: ctx.accounts.recipient.to_account_info(),
                authority: ctx.accounts.market.to_account_info(),
            },
            &[market_seeds],
        ),
        amount,
    )?;

    emit!(EmergencyWithdrawn {
        market: market.key(),
        recipient: ctx.accounts.recipient.owner,
        usdc_amount: amount,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
