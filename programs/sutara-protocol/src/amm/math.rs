/// Fixed-point arithmetic helpers for LMSR computations.
///
/// All LMSR quantities use PRECISION = 1_000_000_000 (1e9) as the base scale.
/// USDC values are in USDC lamports (6 decimal places, 1 USDC = 1_000_000).
///
/// exp() and ln() are approximated with sufficient precision for prediction
/// market applications while staying within BPF compute limits.

use anchor_lang::prelude::*;
use crate::errors::SutaraError;

/// Scale factor for fixed-point arithmetic (1e9)
pub const PRECISION: i128 = 1_000_000_000;

/// ln(2) scaled by PRECISION
pub const LN2_SCALED: i128 = 693_147_180; // ln(2) × 1e9

/// ln(3) scaled by PRECISION
pub const LN3_SCALED: i128 = 1_098_612_289;

/// Natural logarithm via Padé approximant.
///
/// Input:  x > 0, scaled by PRECISION
/// Output: ln(x), scaled by PRECISION
pub fn ln_scaled(x: i128) -> Result<i128> {
    require!(x > 0, SutaraError::InvalidLmsrResult);

    // Reduce to [0.5, 1.5] using: ln(x) = ln(x/2^k) + k*ln(2)
    let mut val = x;
    let mut k: i128 = 0;

    while val > PRECISION + PRECISION / 2 {
        val = val.checked_div(2).ok_or(SutaraError::Overflow)?;
        k = k.checked_add(1).ok_or(SutaraError::Overflow)?;
    }
    while val < PRECISION / 2 {
        val = val.checked_mul(2).ok_or(SutaraError::Overflow)?;
        k = k.checked_sub(1).ok_or(SutaraError::Underflow)?;
    }

    // Padé approximant for ln(1+u) where u = (val - PRECISION) / PRECISION ∈ (-0.5, 0.5)
    let u = val.checked_sub(PRECISION).ok_or(SutaraError::Underflow)?;

    // ln(1+u) ≈ u(6 + u²) / (6 + 4u + u²/PRECISION)  (Padé [2/2])
    // Higher precision variant:
    // ln(1+u) ≈ u - u²/2 + u³/3 - u⁴/4 (Taylor, truncated at degree 6)
    let u2 = u.checked_mul(u).ok_or(SutaraError::Overflow)?.checked_div(PRECISION).ok_or(SutaraError::Overflow)?;
    let u3 = u2.checked_mul(u).ok_or(SutaraError::Overflow)?.checked_div(PRECISION).ok_or(SutaraError::Overflow)?;
    let u4 = u3.checked_mul(u).ok_or(SutaraError::Overflow)?.checked_div(PRECISION).ok_or(SutaraError::Overflow)?;
    let u5 = u4.checked_mul(u).ok_or(SutaraError::Overflow)?.checked_div(PRECISION).ok_or(SutaraError::Overflow)?;
    let u6 = u5.checked_mul(u).ok_or(SutaraError::Overflow)?.checked_div(PRECISION).ok_or(SutaraError::Overflow)?;

    let ln_1_plus_u = u
        .checked_sub(u2.checked_div(2).ok_or(SutaraError::Overflow)?).ok_or(SutaraError::Overflow)?
        .checked_add(u3.checked_div(3).ok_or(SutaraError::Overflow)?).ok_or(SutaraError::Overflow)?
        .checked_sub(u4.checked_div(4).ok_or(SutaraError::Overflow)?).ok_or(SutaraError::Overflow)?
        .checked_add(u5.checked_div(5).ok_or(SutaraError::Overflow)?).ok_or(SutaraError::Overflow)?
        .checked_sub(u6.checked_div(6).ok_or(SutaraError::Overflow)?).ok_or(SutaraError::Overflow)?;

    let result = ln_1_plus_u
        .checked_add(k.checked_mul(LN2_SCALED).ok_or(SutaraError::Overflow)?)
        .ok_or(SutaraError::Overflow)?;

    Ok(result)
}

/// Natural exponentiation via Taylor series.
///
/// Input:  x scaled by PRECISION (i.e. actual exponent = x / PRECISION)
/// Output: exp(x/PRECISION) scaled by PRECISION
///
/// Valid range: x / PRECISION ∈ [-20, 20] (covers all realistic probability values)
pub fn exp_scaled(x: i128) -> Result<i128> {
    // Handle large inputs
    if x > 20 * PRECISION {
        return Ok(i128::MAX / 2);
    }
    if x < -20 * PRECISION {
        return Ok(0);
    }

    // exp(x) = exp(k*ln2 + r) = 2^k * exp(r), where r ∈ [-ln2/2, ln2/2]
    let k = x.checked_div(LN2_SCALED).ok_or(SutaraError::Overflow)?;
    let r = x.checked_sub(k.checked_mul(LN2_SCALED).ok_or(SutaraError::Overflow)?)
             .ok_or(SutaraError::Underflow)?;

    // Taylor series: exp(r) = 1 + r + r²/2! + r³/3! + ... (r is small)
    let mut term = PRECISION; // term_0 = 1
    let mut sum = PRECISION;  // sum starts at 1

    for i in 1..=12i128 {
        term = term
            .checked_mul(r).ok_or(SutaraError::Overflow)?
            .checked_div(PRECISION).ok_or(SutaraError::Overflow)?
            .checked_div(i).ok_or(SutaraError::Overflow)?;
        sum = sum.checked_add(term).ok_or(SutaraError::Overflow)?;
        if term.abs() < 1 { break; }
    }

    // Multiply by 2^k via bit shifts
    let result = if k >= 0 {
        sum.checked_shl(k as u32).ok_or(SutaraError::Overflow)?
    } else {
        sum.checked_shr((-k) as u32).ok_or(SutaraError::Overflow)?
    };

    Ok(result)
}

/// Safe checked multiplication of two scaled values, returning a scaled result.
/// (a_scaled × b_scaled) / PRECISION
pub fn mul_scaled(a: i128, b: i128) -> Result<i128> {
    a.checked_mul(b)
     .ok_or(error!(SutaraError::Overflow))?
     .checked_div(PRECISION)
     .ok_or(error!(SutaraError::Overflow))
}
