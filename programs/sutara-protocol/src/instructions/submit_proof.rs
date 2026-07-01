use anchor_lang::prelude::*;
use crate::constants::*;
use crate::errors::SutaraError;
use crate::events::ProofSubmitted;
use crate::state::{Market, MarketStatus, Resolution, KeeperRegistry, ProtocolConfig};
use crate::merkle::verify::{verify_merkle_proof, build_leaf};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct SubmitProofParams {
    pub merkle_root: [u8; 32],
    pub proof: Vec<[u8; 32]>,
    pub leaf_index: u64,
    /// Encoded: outcome_idx, score_home, score_away
    pub outcome_idx: u8,
    pub score_home: u8,
    pub score_away: u8,
    pub finalized_ts: i64,
}

#[derive(Accounts)]
pub struct SubmitProof<'info> {
    /// Must be an authorized keeper
    pub keeper: Signer<'info>,

    #[account(
        seeds = [SEED_PROTOCOL],
        bump = protocol.bump,
        constraint = !protocol.paused @ SutaraError::ProtocolPaused,
    )]
    pub protocol: Box<Account<'info, ProtocolConfig>>,

    #[account(
        seeds = [SEED_KEEPER_REGISTRY],
        bump = keeper_registry.bump,
    )]
    pub keeper_registry: Box<Account<'info, KeeperRegistry>>,

    #[account(
        mut,
        seeds = [SEED_MARKET, &market.match_id, &[market.market_type.discriminant()]],
        bump = market.bump,
        constraint = market.status == MarketStatus::Closed @ SutaraError::MarketNotYetClosed,
    )]
    pub market: Box<Account<'info, Market>>,

    #[account(
        mut,
        seeds = [SEED_RESOLUTION, market.key().as_ref()],
        bump = resolution.bump,
        constraint = !resolution.validated @ SutaraError::ProofAlreadySubmitted,
    )]
    pub resolution: Box<Account<'info, Resolution>>,
}

pub fn handler(ctx: Context<SubmitProof>, params: SubmitProofParams) -> Result<()> {
    // ── Keeper authorization ──────────────────────────────────────────────
    require!(
        ctx.accounts.keeper_registry.is_authorized(&ctx.accounts.keeper.key()),
        SutaraError::UnauthorizedKeeper
    );

    // ── Proof length bounds ───────────────────────────────────────────────
    require!(
        params.proof.len() <= MAX_MERKLE_DEPTH,
        SutaraError::InvalidMerkleProof
    );

    // ── Build and verify leaf locally ─────────────────────────────────────
    let leaf = build_leaf(
        &ctx.accounts.market.match_id,
        params.outcome_idx,
        params.score_home,
        params.score_away,
        params.finalized_ts,
    );

    let is_valid = verify_merkle_proof(
        &params.proof,
        &params.merkle_root,
        &leaf,
        params.leaf_index,
        params.proof.len() as u8,
    )?;

    require!(is_valid, SutaraError::InvalidMerkleProof);

    // ── Verify root matches protocol-stored root ──────────────────────────
    require!(
        params.merkle_root == ctx.accounts.protocol.txline_merkle_root,
        SutaraError::InvalidMerkleProof
    );

    // ── Store proof in resolution account ─────────────────────────────────
    let resolution = &mut ctx.accounts.resolution;
    resolution.merkle_root = params.merkle_root;
    resolution.leaf = leaf;
    resolution.leaf_index = params.leaf_index;
    resolution.proof_len = params.proof.len() as u8;
    for (i, hash) in params.proof.iter().enumerate() {
        resolution.proof[i] = *hash;
    }
    resolution.submitted_by = ctx.accounts.keeper.key();
    resolution.submitted_at = Clock::get()?.unix_timestamp;
    // winning_outcome is set after CPI validation in validate_result
    resolution.winning_outcome = Some(params.outcome_idx);

    // Update market status to Disputed (awaiting CPI validation)
    let market = &mut ctx.accounts.market;
    market.status = MarketStatus::Disputed;
    market.version = market.version.checked_add(1).ok_or(SutaraError::Overflow)?;

    emit!(ProofSubmitted {
        market: market.key(),
        submitted_by: ctx.accounts.keeper.key(),
        merkle_root: params.merkle_root,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
