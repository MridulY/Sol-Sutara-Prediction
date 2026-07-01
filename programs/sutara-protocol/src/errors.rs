use anchor_lang::prelude::*;

#[error_code]
pub enum SutaraError {
    // ── Protocol ──────────────────────────────────────────────────────────
    #[msg("Protocol is currently paused")]
    ProtocolPaused,
    #[msg("Unauthorized: signer is not admin")]
    Unauthorized,
    #[msg("Protocol fee exceeds maximum allowed (5%)")]
    ProtocolFeeTooHigh,
    #[msg("Creator fee exceeds maximum allowed (3%)")]
    CreatorFeeTooHigh,
    #[msg("LP fee exceeds maximum allowed (5%)")]
    LpFeeTooHigh,
    #[msg("Total fees exceed maximum allowed (10%)")]
    TotalFeesTooHigh,

    // ── Market ────────────────────────────────────────────────────────────
    #[msg("Market is not in Open status")]
    MarketNotOpen,
    #[msg("Market trading period has closed")]
    MarketTradingClosed,
    #[msg("Market is already resolved")]
    MarketAlreadyResolved,
    #[msg("Market is not yet closed for settlement")]
    MarketNotYetClosed,
    #[msg("Market is cancelled")]
    MarketCancelled,
    #[msg("Invalid market type parameters")]
    InvalidMarketType,
    #[msg("Market close time must be at least 10 minutes in the future")]
    InvalidCloseTime,
    #[msg("Number of outcomes must be between 2 and 8")]
    InvalidOutcomeCount,
    #[msg("Outcome index is out of range")]
    InvalidOutcomeIndex,
    #[msg("Outcome label exceeds maximum length (32 chars)")]
    OutcomeLabelTooLong,
    #[msg("Market description exceeds maximum length (128 chars)")]
    DescriptionTooLong,
    #[msg("Match ID is invalid or empty")]
    InvalidMatchId,

    // ── Trading ───────────────────────────────────────────────────────────
    #[msg("Actual cost exceeds max_cost slippage tolerance")]
    SlippageExceeded,
    #[msg("Actual proceeds are below min_proceeds slippage tolerance")]
    ProceedsBelowMinimum,
    #[msg("Insufficient shares in position")]
    InsufficientShares,
    #[msg("Amount must be greater than zero")]
    InvalidAmount,
    #[msg("Invalid token account owner")]
    InvalidTokenOwner,
    #[msg("Invalid token mint")]
    InvalidMint,

    // ── Liquidity ─────────────────────────────────────────────────────────
    #[msg("Insufficient LP tokens")]
    InsufficientLpTokens,
    #[msg("Initial liquidity is below minimum (10 USDC)")]
    LiquidityTooLow,
    #[msg("Pool is already initialized")]
    PoolAlreadyInitialized,

    // ── Settlement ────────────────────────────────────────────────────────
    #[msg("Proof has already been submitted for this market")]
    ProofAlreadySubmitted,
    #[msg("No proof has been submitted yet")]
    ProofNotSubmitted,
    #[msg("Merkle proof verification failed")]
    InvalidMerkleProof,
    #[msg("Market result has not been validated yet")]
    ResultNotValidated,
    #[msg("Signer is not an authorized keeper")]
    UnauthorizedKeeper,
    #[msg("CPI to TxLINE validation program failed")]
    ValidationCpiFailed,
    #[msg("TxLINE program ID does not match protocol config")]
    InvalidTxlineProgram,

    // ── Claims ────────────────────────────────────────────────────────────
    #[msg("Rewards have already been claimed for this position")]
    AlreadyClaimed,
    #[msg("Position holds no shares of the winning outcome")]
    NoWinningShares,

    // ── Math ─────────────────────────────────────────────────────────────
    #[msg("Arithmetic overflow in calculation")]
    Overflow,
    #[msg("Arithmetic underflow in calculation")]
    Underflow,
    #[msg("Division by zero")]
    DivisionByZero,
    #[msg("LMSR calculation produced invalid result")]
    InvalidLmsrResult,
}
