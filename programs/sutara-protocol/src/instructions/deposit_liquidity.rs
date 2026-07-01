use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount, Transfer};
use anchor_spl::associated_token::AssociatedToken;
use crate::constants::*;
use crate::errors::SutaraError;
use crate::events::LiquidityDeposited;
use crate::state::{Market, MarketStatus, Pool, Position, ProtocolConfig};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct DepositLiquidityParams {
    /// USDC amount to deposit (lamports)
    pub usdc_amount: u64,
    /// Minimum LP tokens to receive (slippage protection)
    pub min_lp_tokens: u64,
}

#[derive(Accounts)]
pub struct DepositLiquidity<'info> {
    #[account(mut)]
    pub provider: Signer<'info>,

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
        payer = provider,
        space = Position::LEN,
        seeds = [SEED_POSITION, market.key().as_ref(), provider.key().as_ref()],
        bump,
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
        init_if_needed,
        payer = provider,
        associated_token::mint = lp_mint,
        associated_token::authority = provider,
    )]
    pub provider_lp_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = provider_usdc.owner == provider.key() @ SutaraError::InvalidTokenOwner,
        constraint = provider_usdc.mint == vault.mint @ SutaraError::InvalidMint,
    )]
    pub provider_usdc: Box<Account<'info, TokenAccount>>,

    pub usdc_mint: Box<Account<'info, Mint>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn handler(ctx: Context<DepositLiquidity>, params: DepositLiquidityParams) -> Result<()> {
    require!(params.usdc_amount >= MIN_INITIAL_LIQUIDITY, SutaraError::LiquidityTooLow);

    let pool = &ctx.accounts.pool;
    let vault_balance = ctx.accounts.vault.amount;

    // Compute LP tokens to mint proportionally to vault balance
    // lp_to_mint = (usdc_amount / vault_balance) * lp_supply
    let lp_to_mint = if vault_balance == 0 || pool.lp_supply == 0 {
        INITIAL_LP_SHARES
    } else {
        (params.usdc_amount as u128)
            .checked_mul(pool.lp_supply as u128)
            .ok_or(SutaraError::Overflow)?
            .checked_div(vault_balance as u128)
            .ok_or(SutaraError::DivisionByZero)? as u64
    };

    require!(lp_to_mint >= params.min_lp_tokens, SutaraError::SlippageExceeded);

    // Transfer USDC to vault
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.provider_usdc.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
                authority: ctx.accounts.provider.to_account_info(),
            },
        ),
        params.usdc_amount,
    )?;

    // Mint LP tokens
    let market = &ctx.accounts.market;
    let market_seeds: &[&[u8]] = &[
        SEED_MARKET,
        market.match_id.as_ref(),
        &[market.market_type.discriminant()],
        &[market.bump],
    ];

    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.lp_mint.to_account_info(),
                to: ctx.accounts.provider_lp_account.to_account_info(),
                authority: ctx.accounts.market.to_account_info(),
            },
            &[market_seeds],
        ),
        lp_to_mint,
    )?;

    // Update pool
    let pool = &mut ctx.accounts.pool;
    pool.lp_supply = pool.lp_supply.checked_add(lp_to_mint).ok_or(SutaraError::Overflow)?;

    // Update position
    let position = &mut ctx.accounts.position;
    if position.market == Pubkey::default() {
        position.market = market.key();
        position.owner = ctx.accounts.provider.key();
        position.bump = ctx.bumps.position;
    }
    position.lp_tokens = position.lp_tokens.checked_add(lp_to_mint).ok_or(SutaraError::Overflow)?;
    // Snapshot fees checkpoint so provider only earns fees from this point forward
    position.fees_per_lp_checkpoint = pool.fees_per_lp_token;

    emit!(LiquidityDeposited {
        market: market.key(),
        pool: pool.key(),
        provider: ctx.accounts.provider.key(),
        usdc_amount: params.usdc_amount,
        lp_tokens_minted: lp_to_mint,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
