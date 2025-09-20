use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::error::BondError;
use crate::state::{Bond, Config};

#[derive(Accounts)]
pub struct CounterPartyRedeem<'info> {
    #[account(seeds = [b"config"], bump)]
    pub config: Account<'info, Config>,
    #[account(mut)]
    pub bond: Account<'info, Bond>,
    #[account(mut, constraint = corcoin_mint.key() == bond.corcoin_mint)]
    pub corcoin_mint: Account<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = corcoin_mint,
        associated_token::authority = gcp
    )]
    pub gcp_corcoin_account: Account<'info, TokenAccount>,
    /// CHECK: Must match the bond's stake account
    #[account(constraint = stake_account.key() == bond.stake_account)]
    pub stake_account: AccountInfo<'info>,
    /// CHECK: Derived by seeds
    #[account(seeds = [b"deactivating_stake", bond.key().as_ref()], bump)]
    pub deactivating_stake_account: AccountInfo<'info>,
    /// CHECK: Derived by seeds
    #[account(mut, seeds = [b"gcp"], bump = config.gcp_bump)]
    pub gcp: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<CounterPartyRedeem>, corcoin_amount: u64) -> Result<()> {
    let clock = Clock::get()?;
    let bond = &ctx.accounts.bond;

    require!(
        clock.unix_timestamp >= bond.maturity_timestamp,
        BondError::BondNotMatured
    );

    let gcp_corcoin_balance = ctx.accounts.gcp_corcoin_account.amount;
    require!(
        gcp_corcoin_balance >= corcoin_amount,
        BondError::InsufficientTokens
    );
    require!(corcoin_amount > 0, BondError::InsufficientTokens);

    let total_bond_value = crate::utils::bond::calculate_total_bond_value(
        &ctx.accounts.stake_account,
        &ctx.accounts.deactivating_stake_account,
        &ctx.accounts.bond,
    )?;
    let total_supply = ctx.accounts.corcoin_mint.supply;

    let lamports_for_gcp =
        crate::utils::bond::calculate_user_share(total_bond_value, total_supply, corcoin_amount)?;

    let bond_rent = Rent::get()?.minimum_balance(8 + crate::state::Bond::INIT_SPACE);
    let liquid_reserves = ctx
        .accounts
        .bond
        .to_account_info()
        .lamports()
        .saturating_sub(bond_rent);

    require!(
        liquid_reserves >= lamports_for_gcp,
        BondError::InsufficientTokens
    );

    let gcp_seeds: &[&[u8]] = &[b"gcp", &[ctx.accounts.config.gcp_bump]];
    crate::utils::token::token_burn(
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.corcoin_mint.to_account_info(),
        ctx.accounts.gcp_corcoin_account.to_account_info(),
        ctx.accounts.gcp.to_account_info(),
        corcoin_amount,
        Some(&[gcp_seeds]),
    )?;

    ctx.accounts.bond.sub_lamports(lamports_for_gcp)?;
    ctx.accounts.gcp.add_lamports(lamports_for_gcp)?;

    Ok(())
}
