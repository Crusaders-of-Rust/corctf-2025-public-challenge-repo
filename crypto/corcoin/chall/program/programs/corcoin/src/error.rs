use anchor_lang::prelude::*;

#[error_code]
pub enum BondError {
    #[msg("Maturity timestamp must be in the future")]
    MaturityInPast,
    #[msg("Amount must be greater than 1 SOL (to account for fees)")]
    InvalidAmount,
    #[msg("Bond is not active")]
    BondNotActive,
    #[msg("Bond has not yet matured")]
    BondNotMatured,
    #[msg("Insufficient tokens")]
    InsufficientTokens,
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Stake account is not in the correct state")]
    InvalidStakeState,
    #[msg("Stake redemption amount below minimum delegation. Use lamport redemption instead.")]
    InsufficientStakeAmount,
}
