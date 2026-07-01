use anchor_lang::prelude::*;
use crate::errors::SutaraError;
use crate::state::ProtocolConfig;

pub struct FeeBreakdown {
    pub protocol_fee: u64,
    pub creator_fee: u64,
    pub lp_fee: u64,
    pub total_fee: u64,
    /// Net amount paid by user = gross_cost + total_fee (buy) or gross - total (sell)
    pub net_amount: u64,
}

/// Calculate fee breakdown for a buy trade.
///
/// user pays: gross_cost + total_fee
/// gross_cost goes into vault, fees routed separately
pub fn calculate_buy_fees(
    gross_cost: u64,
    protocol: &ProtocolConfig,
    creator_fee_bps: u16,
) -> Result<FeeBreakdown> {
    let protocol_fee = bps_of(gross_cost, protocol.protocol_fee_bps)?;
    let creator_fee  = bps_of(gross_cost, creator_fee_bps)?;
    let lp_fee       = bps_of(gross_cost, protocol.lp_fee_bps)?;

    let total_fee = protocol_fee
        .checked_add(creator_fee).ok_or(SutaraError::Overflow)?
        .checked_add(lp_fee).ok_or(SutaraError::Overflow)?;

    let net_amount = gross_cost
        .checked_add(total_fee).ok_or(SutaraError::Overflow)?;

    Ok(FeeBreakdown { protocol_fee, creator_fee, lp_fee, total_fee, net_amount })
}

/// Calculate fee breakdown for a sell trade.
///
/// user receives: gross_proceeds − total_fee
pub fn calculate_sell_fees(
    gross_proceeds: u64,
    protocol: &ProtocolConfig,
    creator_fee_bps: u16,
) -> Result<FeeBreakdown> {
    let protocol_fee = bps_of(gross_proceeds, protocol.protocol_fee_bps)?;
    let creator_fee  = bps_of(gross_proceeds, creator_fee_bps)?;
    let lp_fee       = bps_of(gross_proceeds, protocol.lp_fee_bps)?;

    let total_fee = protocol_fee
        .checked_add(creator_fee).ok_or(SutaraError::Overflow)?
        .checked_add(lp_fee).ok_or(SutaraError::Overflow)?;

    let net_amount = gross_proceeds
        .checked_sub(total_fee).ok_or(SutaraError::Underflow)?;

    Ok(FeeBreakdown { protocol_fee, creator_fee, lp_fee, total_fee, net_amount })
}

/// amount * bps / 10_000  with overflow protection
fn bps_of(amount: u64, bps: u16) -> Result<u64> {
    (amount as u128)
        .checked_mul(bps as u128)
        .ok_or(error!(SutaraError::Overflow))?
        .checked_div(10_000)
        .ok_or(error!(SutaraError::DivisionByZero))
        .map(|v| v as u64)
}
