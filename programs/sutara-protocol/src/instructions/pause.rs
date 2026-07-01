use anchor_lang::prelude::*;
use crate::constants::SEED_PROTOCOL;
use crate::errors::SutaraError;
use crate::events::ProtocolPaused;
use crate::state::ProtocolConfig;

#[derive(Accounts)]
pub struct Pause<'info> {
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [SEED_PROTOCOL],
        bump = protocol.bump,
        constraint = admin.key() == protocol.admin @ SutaraError::Unauthorized,
        constraint = !protocol.paused @ SutaraError::ProtocolPaused,
    )]
    pub protocol: Box<Account<'info, ProtocolConfig>>,
}

pub fn handler(ctx: Context<Pause>) -> Result<()> {
    ctx.accounts.protocol.paused = true;
    emit!(ProtocolPaused {
        admin: ctx.accounts.admin.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
}
