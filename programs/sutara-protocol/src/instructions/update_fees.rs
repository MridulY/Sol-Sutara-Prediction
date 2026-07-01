use anchor_lang::prelude::*;
use crate::constants::*;
use crate::errors::SutaraError;
use crate::events::FeesUpdated;
use crate::state::ProtocolConfig;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct UpdateFeesParams {
    pub protocol_fee_bps: u16,
    pub creator_fee_bps: u16,
    pub lp_fee_bps: u16,
    pub market_creation_fee: u64,
}

#[derive(Accounts)]
pub struct UpdateFees<'info> {
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [SEED_PROTOCOL],
        bump = protocol.bump,
        constraint = admin.key() == protocol.admin @ SutaraError::Unauthorized,
    )]
    pub protocol: Box<Account<'info, ProtocolConfig>>,
}

pub fn handler(ctx: Context<UpdateFees>, params: UpdateFeesParams) -> Result<()> {
    require!(params.protocol_fee_bps <= MAX_PROTOCOL_FEE_BPS, SutaraError::ProtocolFeeTooHigh);
    require!(params.creator_fee_bps <= MAX_CREATOR_FEE_BPS, SutaraError::CreatorFeeTooHigh);
    require!(params.lp_fee_bps <= MAX_LP_FEE_BPS, SutaraError::LpFeeTooHigh);
    require!(
        params.protocol_fee_bps as u32 + params.creator_fee_bps as u32 + params.lp_fee_bps as u32
            <= MAX_TOTAL_FEE_BPS as u32,
        SutaraError::TotalFeesTooHigh
    );

    let protocol = &mut ctx.accounts.protocol;
    protocol.protocol_fee_bps = params.protocol_fee_bps;
    protocol.creator_fee_bps = params.creator_fee_bps;
    protocol.lp_fee_bps = params.lp_fee_bps;
    protocol.market_creation_fee = params.market_creation_fee;

    emit!(FeesUpdated {
        protocol_fee_bps: params.protocol_fee_bps,
        creator_fee_bps: params.creator_fee_bps,
        lp_fee_bps: params.lp_fee_bps,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
