use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use crate::constants::*;
use crate::errors::SutaraError;
use crate::events::MarketCreated;
use crate::state::{
    Market, MarketStatus, MarketType, OutcomeLabel,
    ProtocolConfig, Resolution, CreatorFeeVault,
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreateMarketParams {
    pub match_id: [u8; 32],
    pub market_type: MarketType,
    pub outcome_labels: Vec<String>,
    pub close_ts: i64,
    pub open_ts: i64,
    /// Override creator fee (0 = use protocol default)
    pub creator_fee_bps: u16,
}

#[derive(Accounts)]
#[instruction(params: CreateMarketParams)]
pub struct CreateMarket<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        mut,
        seeds = [SEED_PROTOCOL],
        bump = protocol.bump,
        constraint = !protocol.paused @ SutaraError::ProtocolPaused,
    )]
    pub protocol: Box<Account<'info, ProtocolConfig>>,

    #[account(
        init,
        payer = creator,
        space = Market::LEN,
        seeds = [
            SEED_MARKET,
            &params.match_id,
            &[params.market_type.discriminant()],
        ],
        bump,
    )]
    pub market: Box<Account<'info, Market>>,

    #[account(
        init,
        payer = creator,
        space = Resolution::LEN,
        seeds = [SEED_RESOLUTION, market.key().as_ref()],
        bump,
    )]
    pub resolution: Box<Account<'info, Resolution>>,

    #[account(
        init,
        payer = creator,
        space = CreatorFeeVault::LEN,
        seeds = [b"creator_fee_vault", market.key().as_ref()],
        bump,
    )]
    pub creator_fee_vault: Box<Account<'info, CreatorFeeVault>>,

    /// Creator's USDC account (pays market creation fee)
    #[account(
        mut,
        constraint = creator_usdc.owner == creator.key() @ SutaraError::InvalidTokenOwner,
        constraint = creator_usdc.mint == usdc_mint.key() @ SutaraError::InvalidMint,
    )]
    pub creator_usdc: Box<Account<'info, TokenAccount>>,

    /// Protocol treasury USDC account (receives creation fee)
    #[account(
        mut,
        constraint = treasury.mint == usdc_mint.key() @ SutaraError::InvalidMint,
    )]
    pub treasury: Box<Account<'info, TokenAccount>>,

    pub usdc_mint: Box<Account<'info, Mint>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<CreateMarket>, params: CreateMarketParams) -> Result<()> {
    let clock = Clock::get()?;

    // ── Validations ──────────────────────────────────────────────────────────
    require!(!params.match_id.iter().all(|&b| b == 0), SutaraError::InvalidMatchId);

    require!(
        params.close_ts > clock.unix_timestamp + MIN_MARKET_DURATION_SECS,
        SutaraError::InvalidCloseTime
    );
    require!(
        params.outcome_labels.len() >= 2 && params.outcome_labels.len() <= MAX_OUTCOMES,
        SutaraError::InvalidOutcomeCount
    );
    for label in &params.outcome_labels {
        require!(label.len() <= MAX_OUTCOME_LABEL_LEN, SutaraError::OutcomeLabelTooLong);
    }
    if params.creator_fee_bps > 0 {
        require!(
            params.creator_fee_bps <= MAX_CREATOR_FEE_BPS,
            SutaraError::CreatorFeeTooHigh
        );
    }

    // ── Transfer creation fee ─────────────────────────────────────────────
    if ctx.accounts.protocol.market_creation_fee > 0 {
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.creator_usdc.to_account_info(),
                    to: ctx.accounts.treasury.to_account_info(),
                    authority: ctx.accounts.creator.to_account_info(),
                },
            ),
            ctx.accounts.protocol.market_creation_fee,
        )?;
    }

    // ── Build outcome labels ──────────────────────────────────────────────
    let mut outcomes = [OutcomeLabel { label: [0u8; MAX_OUTCOME_LABEL_LEN], label_len: 0 }; MAX_OUTCOMES];
    for (i, label_str) in params.outcome_labels.iter().enumerate() {
        outcomes[i] = OutcomeLabel::from_str(label_str);
    }

    // ── Initialise market ─────────────────────────────────────────────────
    let market = &mut ctx.accounts.market;
    market.creator = ctx.accounts.creator.key();
    market.match_id = params.match_id;
    market.market_type = params.market_type;
    market.outcomes = outcomes;
    market.num_outcomes = params.outcome_labels.len() as u8;
    market.status = if params.open_ts <= clock.unix_timestamp {
        MarketStatus::Open
    } else {
        MarketStatus::Pending
    };
    market.open_ts = params.open_ts;
    market.close_ts = params.close_ts;
    market.resolve_ts = None;
    market.winning_outcome = None;
    market.pool = Pubkey::default(); // set when pool is initialised
    market.vault = Pubkey::default(); // set when pool is initialised
    market.resolution = ctx.accounts.resolution.key();
    market.creator_fee_vault = ctx.accounts.creator_fee_vault.key();
    market.creator_fee_bps = if params.creator_fee_bps == 0 {
        ctx.accounts.protocol.creator_fee_bps
    } else {
        params.creator_fee_bps
    };
    market.total_volume = 0;
    market.version = 0;
    market.bump = ctx.bumps.market;

    // ── Initialise resolution account ─────────────────────────────────────
    let resolution = &mut ctx.accounts.resolution;
    resolution.market = market.key();
    resolution.match_id = params.match_id;
    resolution.validated = false;
    resolution.winning_outcome = None;
    resolution.bump = ctx.bumps.resolution;

    // ── Initialise creator fee vault ──────────────────────────────────────
    let fee_vault = &mut ctx.accounts.creator_fee_vault;
    fee_vault.market = market.key();
    fee_vault.creator = ctx.accounts.creator.key();
    fee_vault.accumulated = 0;
    fee_vault.total_claimed = 0;

    // ── Increment protocol counter ────────────────────────────────────────
    ctx.accounts.protocol.total_markets = ctx.accounts.protocol
        .total_markets
        .checked_add(1)
        .ok_or(SutaraError::Overflow)?;

    emit!(MarketCreated {
        market: market.key(),
        creator: ctx.accounts.creator.key(),
        match_id: params.match_id,
        market_type_discriminant: market.market_type.discriminant(),
        num_outcomes: market.num_outcomes,
        close_ts: params.close_ts,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}
