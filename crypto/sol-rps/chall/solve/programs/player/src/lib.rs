use anchor_lang::prelude::*;

declare_id!("BujTCzJfF399XRtT2vwqztB8ihhhEKQkwYnyFNs2Kq7S");

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum Move {
    Rock,
    Paper,
    Scissors,
}

#[program]
pub mod player {
    use super::*;

    pub fn get_moves(_ctx: Context<GetMoves>) -> Result<[Move; 100]> {
        // TODO: solve?
        Ok([Move::Rock; 100])
    }
}

#[derive(Accounts)]
pub struct GetMoves<'info> {
    pub clock: Sysvar<'info, Clock>,
}
