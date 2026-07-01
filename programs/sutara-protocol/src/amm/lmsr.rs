/// LMSR (Logarithmic Market Scoring Rule) implementation.
///
/// Cost function:
///   C(q) = b · ln( Σᵢ exp(qᵢ / b) )
///
/// Cost to buy Δq shares of outcome k:
///   cost(Δq) = C(..., qₖ + Δq, ...) − C(..., qₖ, ...)
///
/// Share probability (instantaneous price):
///   pₖ = exp(qₖ / b) / Σᵢ exp(qᵢ / b)
///
/// All quantities are stored scaled by PRECISION (1e9).
/// b_parameter is stored in USDC lamports (6 decimals).

use anchor_lang::prelude::*;
use crate::errors::SutaraError;
use super::math::{ln_scaled, exp_scaled, PRECISION};

/// Compute the LMSR cost function value C(q) scaled by PRECISION.
///
/// b_parameter: USDC lamports (will be scaled internally)
/// quantities: i128 array scaled by PRECISION
/// num_outcomes: number of valid outcomes
fn cost_function(
    b_parameter: u64,
    quantities: &[i128],
    num_outcomes: usize,
) -> Result<i128> {
    // b scaled to PRECISION units
    let b = (b_parameter as i128)
        .checked_mul(PRECISION)
        .ok_or(SutaraError::Overflow)?;

    // Compute Σ exp(qᵢ / b)
    let mut sum_exp: i128 = 0;
    for i in 0..num_outcomes {
        // qᵢ / b = quantities[i] / b_scaled
        // Since both are scaled by PRECISION, the ratio is unscaled.
        // We then pass it as a scaled value to exp_scaled.
        let q_over_b = quantities[i]
            .checked_mul(PRECISION)
            .ok_or(SutaraError::Overflow)?
            .checked_div(b)
            .ok_or(SutaraError::DivisionByZero)?;

        let exp_val = exp_scaled(q_over_b)?;
        sum_exp = sum_exp.checked_add(exp_val).ok_or(SutaraError::Overflow)?;
    }

    require!(sum_exp > 0, SutaraError::InvalidLmsrResult);

    // C(q) = b · ln(sum_exp)
    // ln_scaled expects its input scaled by PRECISION and returns scaled by PRECISION
    let ln_sum = ln_scaled(sum_exp)?;

    // b (in USDC lamports) × ln_sum (scaled by 1e9) → result scaled by 1e9
    // We want result in USDC lamports (unscaled), so divide by PRECISION
    let cost = (b_parameter as i128)
        .checked_mul(ln_sum)
        .ok_or(SutaraError::Overflow)?
        .checked_div(PRECISION)
        .ok_or(SutaraError::Overflow)?;

    Ok(cost)
}

/// Compute the USDC cost (in lamports) to buy `delta_shares` of outcome `outcome_idx`.
///
/// Returns: (cost_usdc_lamports, new_quantities)
pub fn cost_to_buy(
    b_parameter: u64,
    quantities: &[i128],
    num_outcomes: usize,
    outcome_idx: usize,
    delta_shares: u64,
) -> Result<(u64, Vec<i128>)> {
    require!(outcome_idx < num_outcomes, SutaraError::InvalidOutcomeIndex);
    require!(delta_shares > 0, SutaraError::InvalidAmount);

    let cost_before = cost_function(b_parameter, quantities, num_outcomes)?;

    let mut new_quantities = quantities[..num_outcomes].to_vec();
    new_quantities[outcome_idx] = new_quantities[outcome_idx]
        .checked_add(delta_shares as i128)
        .ok_or(SutaraError::Overflow)?;

    let cost_after = cost_function(b_parameter, &new_quantities, num_outcomes)?;

    let cost_delta = cost_after
        .checked_sub(cost_before)
        .ok_or(SutaraError::Underflow)?;

    require!(cost_delta >= 0, SutaraError::InvalidLmsrResult);

    Ok((cost_delta as u64, new_quantities))
}

/// Compute the USDC proceeds (in lamports) from selling `delta_shares` of outcome `outcome_idx`.
///
/// Returns: (proceeds_usdc_lamports, new_quantities)
pub fn proceeds_from_sell(
    b_parameter: u64,
    quantities: &[i128],
    num_outcomes: usize,
    outcome_idx: usize,
    delta_shares: u64,
) -> Result<(u64, Vec<i128>)> {
    require!(outcome_idx < num_outcomes, SutaraError::InvalidOutcomeIndex);
    require!(delta_shares > 0, SutaraError::InvalidAmount);

    let cost_before = cost_function(b_parameter, quantities, num_outcomes)?;

    let mut new_quantities = quantities[..num_outcomes].to_vec();
    new_quantities[outcome_idx] = new_quantities[outcome_idx]
        .checked_sub(delta_shares as i128)
        .ok_or(SutaraError::Underflow)?;

    let cost_after = cost_function(b_parameter, &new_quantities, num_outcomes)?;

    let proceeds_delta = cost_before
        .checked_sub(cost_after)
        .ok_or(SutaraError::Underflow)?;

    require!(proceeds_delta >= 0, SutaraError::InvalidLmsrResult);

    Ok((proceeds_delta as u64, new_quantities))
}

/// Compute instantaneous probability of outcome `outcome_idx`.
///
/// pₖ = exp(qₖ / b) / Σᵢ exp(qᵢ / b)
///
/// Returns probability scaled by PRECISION (e.g. 500_000_000 = 50%)
pub fn probability(
    b_parameter: u64,
    quantities: &[i128],
    num_outcomes: usize,
    outcome_idx: usize,
) -> Result<u64> {
    require!(outcome_idx < num_outcomes, SutaraError::InvalidOutcomeIndex);

    let b = (b_parameter as i128)
        .checked_mul(PRECISION)
        .ok_or(SutaraError::Overflow)?;

    let mut sum_exp: i128 = 0;
    let mut exp_k: i128 = 0;

    for i in 0..num_outcomes {
        let q_over_b = quantities[i]
            .checked_mul(PRECISION)
            .ok_or(SutaraError::Overflow)?
            .checked_div(b)
            .ok_or(SutaraError::DivisionByZero)?;

        let exp_val = exp_scaled(q_over_b)?;
        sum_exp = sum_exp.checked_add(exp_val).ok_or(SutaraError::Overflow)?;
        if i == outcome_idx {
            exp_k = exp_val;
        }
    }

    require!(sum_exp > 0, SutaraError::InvalidLmsrResult);

    // pₖ = exp_k / sum_exp (both scaled by PRECISION)
    let prob = exp_k
        .checked_mul(PRECISION)
        .ok_or(SutaraError::Overflow)?
        .checked_div(sum_exp)
        .ok_or(SutaraError::DivisionByZero)?;

    Ok(prob as u64)
}

/// Compute all outcome probabilities in one pass.
///
/// Returns array of probabilities scaled by PRECISION, summing to PRECISION.
pub fn all_probabilities(
    b_parameter: u64,
    quantities: &[i128],
    num_outcomes: usize,
) -> Result<Vec<u64>> {
    let b = (b_parameter as i128)
        .checked_mul(PRECISION)
        .ok_or(SutaraError::Overflow)?;

    let mut exp_vals = Vec::with_capacity(num_outcomes);
    let mut sum_exp: i128 = 0;

    for i in 0..num_outcomes {
        let q_over_b = quantities[i]
            .checked_mul(PRECISION)
            .ok_or(SutaraError::Overflow)?
            .checked_div(b)
            .ok_or(SutaraError::DivisionByZero)?;

        let exp_val = exp_scaled(q_over_b)?;
        sum_exp = sum_exp.checked_add(exp_val).ok_or(SutaraError::Overflow)?;
        exp_vals.push(exp_val);
    }

    require!(sum_exp > 0, SutaraError::InvalidLmsrResult);

    let probs = exp_vals
        .iter()
        .map(|&e| {
            e.checked_mul(PRECISION)
             .ok_or(error!(SutaraError::Overflow))
             .and_then(|v| v.checked_div(sum_exp).ok_or(error!(SutaraError::DivisionByZero)))
             .map(|v| v as u64)
        })
        .collect::<Result<Vec<u64>>>()?;

    Ok(probs)
}

/// Initial cost to bootstrap a pool with b_parameter.
///
/// An empty LMSR market (all qᵢ = 0) has:
///   C(0) = b · ln(n)
///
/// This is the amount of USDC the first LP must deposit to initialise the pool.
pub fn initial_pool_cost(b_parameter: u64, num_outcomes: usize) -> Result<u64> {
    require!(num_outcomes >= 2, SutaraError::InvalidOutcomeCount);

    // ln(n) scaled by PRECISION
    let ln_n = match num_outcomes {
        2 => 693_147_180i128,                 // ln(2)
        3 => 1_098_612_289i128,               // ln(3)
        4 => 1_386_294_361i128,               // ln(4)
        5 => 1_609_437_912i128,               // ln(5)
        6 => 1_791_759_469i128,               // ln(6)
        7 => 1_945_910_149i128,               // ln(7)
        8 => 2_079_441_541i128,               // ln(8)
        _ => return err!(SutaraError::InvalidOutcomeCount),
    };

    let cost = (b_parameter as i128)
        .checked_mul(ln_n)
        .ok_or(SutaraError::Overflow)?
        .checked_div(PRECISION)
        .ok_or(SutaraError::Overflow)?;

    Ok(cost as u64)
}
