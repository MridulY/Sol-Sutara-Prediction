use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount, Transfer};
use crate::constants::*;
use crate::errors::SutaraError;
use crate::events::LiquidityWithdrawn;
use crate::state::{Market, MarketStatus, Pool, Position, ProtocolConfig};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct WithdrawLiquidityParams {
    pub lp_tokens: u64,
    pub min_usdc_out: u64,
}

#[derive(Accounts)]
pub struct WithdrawLiquidity<'info> {
    #[account(mut)]
    pub provider: Signer<'info>,

    #[account(
        seeds = [SEED_PROTOCOL],
        bump = protocol.bump,
        constraint = !protocol.paused @ SutaraError::ProtocolPaused,
    )]
    pub protocol: Box<Account<'info, ProtocolConfig>>,

    #[account(
        seeds = [SEED_MARKET, &market.match_id, &[market.market_type.discriminant()]],
        bump = market.bump,
        // Cannot withdraw if market is cancelled (use emergency_withdraw instead)
        constraint = market.status != MarketStatus::Cancelled @ SutaraError::MarketCancelled,
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
        seeds = [SEED_POSITION, market.key().as_ref(), provider.key().as_ref()],
        bump = position.bump,
        constraint = position.owner == provider.key() @ SutaraError::Unauthorized,
        constraint = position.lp_tokens >= params.lp_tokens @ SutaraError::InsufficientLpTokens,
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
        seeds = [SEED_LP_MINT, market.key().as_ref()],
        bump,
    )]
    pub lp_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = lp_mint,
        associated_token::authority = provider,
    )]
    pub provider_lp_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = provider_usdc.owner == provider.key() @ SutaraError::InvalidTokenOwner,
    )]
    pub provider_usdc: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<WithdrawLiquidity>, params: WithdrawLiquidityParams) -> Result<()> {
    require!(params.lp_tokens > 0, SutaraError::InvalidAmount);

    let vault_balance = ctx.accounts.vault.amount;
    let lp_supply = ctx.accounts.pool.lp_supply;

    // Pro-rata USDC: usdc_out = (lp_tokens / lp_supply) × vault_balance
    let usdc_out = (params.lp_tokens as u128)
        .checked_mul(vault_balance as u128)
        .ok_or(SutaraError::Overflow)?
        .checked_div(lp_supply as u128)
        .ok_or(SutaraError::DivisionByZero)? as u64;

    require!(usdc_out >= params.min_usdc_out, SutaraError::ProceedsBelowMinimum);

    // Burn LP tokens
    token::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.lp_mint.to_account_info(),
                from: ctx.accounts.provider_lp_account.to_account_info(),
                authority: ctx.accounts.provider.to_account_info(),
            },
        ),
        params.lp_tokens,
    )?;

    // Transfer USDC from vault to provider
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
                to: ctx.accounts.provider_usdc.to_account_info(),
                authority: ctx.accounts.market.to_account_info(),
            },
            &[market_seeds],
        ),
        usdc_out,
    )?;

    // Update pool and position
    let pool = &mut ctx.accounts.pool;
    pool.lp_supply = pool.lp_supply.checked_sub(params.lp_tokens).ok_or(SutaraError::Underflow)?;

    let position = &mut ctx.accounts.position;
    position.lp_tokens = position.lp_tokens.checked_sub(params.lp_tokens).ok_or(SutaraError::Underflow)?;

    emit!(LiquidityWithdrawn {
        market: market.key(),
        pool: pool.key(),
        provider: ctx.accounts.provider.key(),
        lp_tokens_burned: params.lp_tokens,
        usdc_returned: usdc_out,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
