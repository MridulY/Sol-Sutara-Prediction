use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount, Transfer};
use anchor_spl::associated_token::AssociatedToken;
use crate::constants::*;
use crate::errors::SutaraError;
use crate::events::PoolInitialized;
use crate::state::{Market, MarketStatus, Pool, ProtocolConfig};
use crate::amm::lmsr::initial_pool_cost;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializePoolParams {
    /// Desired b parameter in USDC lamports
    /// If zero, derived automatically from initial_liquidity
    pub b_parameter: u64,
    /// Initial USDC liquidity to deposit (covers C(0) = b·ln(n))
    pub initial_liquidity: u64,
}

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

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
        constraint = market.creator == creator.key() @ SutaraError::Unauthorized,
        constraint = market.pool == Pubkey::default() @ SutaraError::PoolAlreadyInitialized,
        constraint = market.status == MarketStatus::Open || market.status == MarketStatus::Pending
            @ SutaraError::MarketAlreadyResolved,
    )]
    pub market: Box<Account<'info, Market>>,

    #[account(
        init,
        payer = creator,
        space = Pool::LEN,
        seeds = [SEED_POOL, market.key().as_ref()],
        bump,
    )]
    pub pool: Box<Account<'info, Pool>>,

    /// USDC vault for this market (PDA-controlled token account)
    #[account(
        init,
        payer = creator,
        seeds = [SEED_VAULT, market.key().as_ref()],
        bump,
        token::mint = usdc_mint,
        token::authority = market,
    )]
    pub vault: Box<Account<'info, TokenAccount>>,

    /// LP token mint (one per market)
    #[account(
        init,
        payer = creator,
        seeds = [SEED_LP_MINT, market.key().as_ref()],
        bump,
        mint::decimals = 9,
        mint::authority = market,
    )]
    pub lp_mint: Box<Account<'info, Mint>>,

    /// Creator's LP token account (receives initial LP tokens)
    #[account(
        init_if_needed,
        payer = creator,
        associated_token::mint = lp_mint,
        associated_token::authority = creator,
    )]
    pub creator_lp_account: Box<Account<'info, TokenAccount>>,

    /// Creator's USDC account (source of initial liquidity)
    #[account(
        mut,
        constraint = creator_usdc.owner == creator.key() @ SutaraError::InvalidTokenOwner,
        constraint = creator_usdc.mint == usdc_mint.key() @ SutaraError::InvalidMint,
    )]
    pub creator_usdc: Box<Account<'info, TokenAccount>>,

    pub usdc_mint: Box<Account<'info, Mint>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<InitializePool>, params: InitializePoolParams) -> Result<()> {
    let num_outcomes = ctx.accounts.market.num_outcomes as usize;

    // ── Determine b parameter ─────────────────────────────────────────────
    let b_parameter = if params.b_parameter > 0 {
        params.b_parameter
    } else {
        // Auto-derive: b = initial_liquidity / ln(n)
        // We have initial_cost = b * ln(n), so b = initial_cost / ln(n)
        // Use provided liquidity as the target initial cost
        let ln_n_scaled: u64 = match num_outcomes {
            2 => 693_147_180,
            3 => 1_098_612_289,
            4 => 1_386_294_361,
            5 => 1_609_437_912,
            6 => 1_791_759_469,
            7 => 1_945_910_149,
            8 => 2_079_441_541,
            _ => return err!(SutaraError::InvalidOutcomeCount),
        };
        // b = liquidity * 1e9 / ln_n_scaled
        (params.initial_liquidity as u128)
            .checked_mul(1_000_000_000)
            .ok_or(SutaraError::Overflow)?
            .checked_div(ln_n_scaled as u128)
            .ok_or(SutaraError::DivisionByZero)? as u64
    };

    require!(b_parameter >= MIN_B_PARAMETER, SutaraError::LiquidityTooLow);

    // ── Compute required initial deposit ──────────────────────────────────
    let required_cost = initial_pool_cost(b_parameter, num_outcomes)?;
    require!(params.initial_liquidity >= required_cost, SutaraError::LiquidityTooLow);
    require!(params.initial_liquidity >= MIN_INITIAL_LIQUIDITY, SutaraError::LiquidityTooLow);

    // ── Transfer USDC to vault ────────────────────────────────────────────
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::Transfer {
                from: ctx.accounts.creator_usdc.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
                authority: ctx.accounts.creator.to_account_info(),
            },
        ),
        params.initial_liquidity,
    )?;

    // ── Mint initial LP tokens to creator ─────────────────────────────────
    let market_key = ctx.accounts.market.key();
    let market_bump = ctx.accounts.market.bump;
    let market_seeds = &[
        SEED_MARKET,
        ctx.accounts.market.match_id.as_ref(),
        &[ctx.accounts.market.market_type.discriminant()],
        &[market_bump],
    ];

    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.lp_mint.to_account_info(),
                to: ctx.accounts.creator_lp_account.to_account_info(),
                authority: ctx.accounts.market.to_account_info(),
            },
            &[market_seeds],
        ),
        INITIAL_LP_SHARES,
    )?;

    // ── Initialise pool state ─────────────────────────────────────────────
    let pool = &mut ctx.accounts.pool;
    pool.market = market_key;
    pool.lp_mint = ctx.accounts.lp_mint.key();
    pool.b_parameter = b_parameter;
    pool.outcome_quantities = [0i128; MAX_OUTCOMES];
    pool.lp_supply = INITIAL_LP_SHARES;
    pool.total_fees_accumulated = 0;
    pool.fees_per_lp_token = 0;
    pool.num_outcomes = num_outcomes as u8;
    pool.bump = ctx.bumps.pool;

    // ── Update market to record pool and vault ────────────────────────────
    let market = &mut ctx.accounts.market;
    market.pool = pool.key();
    market.vault = ctx.accounts.vault.key();
    market.version = market.version.checked_add(1).ok_or(SutaraError::Overflow)?;

    emit!(PoolInitialized {
        market: market_key,
        pool: pool.key(),
        b_parameter,
        initial_liquidity: params.initial_liquidity,
        lp_tokens_minted: INITIAL_LP_SHARES,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
