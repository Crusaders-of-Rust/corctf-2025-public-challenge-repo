use anchor_lang::prelude::*;

pub mod error;
pub mod instructions;
pub mod state;
pub mod utils;

use instructions::*;

declare_id!("AHtNDB1hvKSNbtb3PYo1Eh8gvHuH1F1c5HcLeAvNPa8d");

#[program]
pub mod corcoin {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, admin: Pubkey, amount: u64) -> Result<()> {
        instructions::initialize::handler(ctx, admin, amount)
    }

    pub fn create_bond(
        ctx: Context<CreateBond>,
        amount: u64,
        maturity_timestamp: i64,
    ) -> Result<()> {
        instructions::create_bond::handler(ctx, amount, maturity_timestamp)
    }

    pub fn redeem_lamports(ctx: Context<RedeemLamports>, corcoin_amount: u64) -> Result<()> {
        instructions::redeem_lamports::handler(ctx, corcoin_amount)
    }

    pub fn redeem_stake(ctx: Context<RedeemStake>, corcoin_amount: u64) -> Result<()> {
        instructions::redeem_stake::handler(ctx, corcoin_amount)
    }

    pub fn admin_withdraw(ctx: Context<AdminWithdraw>, amount: u64) -> Result<()> {
        instructions::admin_withdraw::handler(ctx, amount)
    }

    pub fn update_bond_reserves(ctx: Context<UpdateBondReserves>) -> Result<()> {
        instructions::update_bond_reserves::handler(ctx)
    }

    pub fn counter_party_redeem(
        ctx: Context<CounterPartyRedeem>,
        corcoin_amount: u64,
    ) -> Result<()> {
        instructions::counter_party_redeem::handler(ctx, corcoin_amount)
    }
}
