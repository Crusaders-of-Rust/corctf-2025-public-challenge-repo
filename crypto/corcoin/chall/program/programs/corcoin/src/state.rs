use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub admin: Pubkey,
    pub gcp_bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Bond {
    pub authority: Pubkey,
    pub validator: Pubkey,
    pub stake_account: Pubkey,
    pub maturity_timestamp: i64,
    pub creation_timestamp: i64,
    pub corcoin_mint: Pubkey,
    pub bump: u8,
}
