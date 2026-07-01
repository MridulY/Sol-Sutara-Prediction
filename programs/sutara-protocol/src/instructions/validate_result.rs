use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::program::invoke;
use crate::constants::*;
use crate::errors::SutaraError;
use crate::events::ResultValidated;
use crate::state::{Market, MarketStatus, Resolution, ProtocolConfig};

/// Calls the TxLINE on-chain validation program via CPI.
///
/// The TxLINE program is an external Solana program that:
///  1. Accepts: (root, proof[], leaf, leaf_index)
///  2. Verifies the Merkle proof against its stored root
///  3. Returns a boolean result via account data
///
/// We use a checked invoke (not invoke_signed) because the TxLINE program
/// doesn't require a PDA signer from us.

#[derive(Accounts)]
pub struct ValidateResult<'info> {
    /// Anyone can trigger validation once proof is submitted
    pub validator: Signer<'info>,

    #[account(
        seeds = [SEED_PROTOCOL],
        bump = protocol.bump,
    )]
    pub protocol: Box<Account<'info, ProtocolConfig>>,

    #[account(
        mut,
        seeds = [SEED_MARKET, &market.match_id, &[market.market_type.discriminant()]],
        bump = market.bump,
        constraint = market.status == MarketStatus::Disputed @ SutaraError::ProofNotSubmitted,
    )]
    pub market: Box<Account<'info, Market>>,

    #[account(
        mut,
        seeds = [SEED_RESOLUTION, market.key().as_ref()],
        bump = resolution.bump,
        constraint = !resolution.validated @ SutaraError::ProofAlreadySubmitted,
        constraint = resolution.winning_outcome.is_some() @ SutaraError::ProofNotSubmitted,
    )]
    pub resolution: Box<Account<'info, Resolution>>,

    /// CHECK: TxLINE validation program (ID verified against protocol.txline_program)
    #[account(
        constraint = txline_program.key() == protocol.txline_program @ SutaraError::InvalidTxlineProgram,
    )]
    pub txline_program: AccountInfo<'info>,

    /// CHECK: TxLINE state account that holds the current Merkle root
    pub txline_state: AccountInfo<'info>,
}

pub fn handler(ctx: Context<ValidateResult>) -> Result<()> {
    let resolution = &ctx.accounts.resolution;

    // ── Build CPI instruction to TxLINE validation program ───────────────
    //
    // TxLINE program interface (assumed ABI):
    // Instruction discriminator: [0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x00, 0x00, 0x01]
    // Accounts: [txline_state (read)]
    // Data: { root: [u8;32], proof: Vec<[u8;32]>, leaf: [u8;32], leaf_index: u64 }
    //
    // The program writes a bool result to a dedicated result account, which we
    // can read after the CPI. For this integration, we assume the TxLINE program
    // returns success (Ok(())) on valid proof and error on invalid.

    let proof_slice = &resolution.proof[..resolution.proof_len as usize];

    let mut cpi_data = vec![0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x00, 0x00, 0x01]; // discriminator
    cpi_data.extend_from_slice(&resolution.merkle_root);
    cpi_data.extend_from_slice(&(proof_slice.len() as u32).to_le_bytes());
    for hash in proof_slice {
        cpi_data.extend_from_slice(hash);
    }
    cpi_data.extend_from_slice(&resolution.leaf);
    cpi_data.extend_from_slice(&resolution.leaf_index.to_le_bytes());

    let cpi_instruction = Instruction {
        program_id: ctx.accounts.txline_program.key(),
        accounts: vec![
            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                ctx.accounts.txline_state.key(),
                false,
            ),
        ],
        data: cpi_data,
    };

    invoke(
        &cpi_instruction,
        &[ctx.accounts.txline_state.to_account_info()],
    ).map_err(|_| error!(SutaraError::ValidationCpiFailed))?;

    // ── Mark as validated ─────────────────────────────────────────────────
    let resolution = &mut ctx.accounts.resolution;
    resolution.validated = true;

    let market = &mut ctx.accounts.market;
    market.version = market.version.checked_add(1).ok_or(SutaraError::Overflow)?;

    let winning_outcome = resolution.winning_outcome.unwrap();

    emit!(ResultValidated {
        market: market.key(),
        validated_by: ctx.accounts.validator.key(),
        winning_outcome,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
