use anchor_lang::prelude::*;
use crate::constants::SEED_PROTOCOL;
use crate::errors::SutaraError;
use crate::events::ConfigUpdated;
use crate::state::ProtocolConfig;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct UpdateConfigParams {
    pub txline_program: Option<Pubkey>,
    pub txline_merkle_root: Option<[u8; 32]>,
    pub treasury: Option<Pubkey>,
    pub new_admin: Option<Pubkey>,
}

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [SEED_PROTOCOL],
        bump = protocol.bump,
        constraint = admin.key() == protocol.admin @ SutaraError::Unauthorized,
    )]
    pub protocol: Box<Account<'info, ProtocolConfig>>,
}

pub fn handler(ctx: Context<UpdateConfig>, params: UpdateConfigParams) -> Result<()> {
    let protocol = &mut ctx.accounts.protocol;

    if let Some(txline_program) = params.txline_program {
        protocol.txline_program = txline_program;
    }
    if let Some(root) = params.txline_merkle_root {
        protocol.txline_merkle_root = root;
    }
    if let Some(treasury) = params.treasury {
        protocol.treasury = treasury;
    }
    if let Some(new_admin) = params.new_admin {
        protocol.admin = new_admin;
    }

    emit!(ConfigUpdated {
        txline_program: protocol.txline_program,
        txline_merkle_root: protocol.txline_merkle_root,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
