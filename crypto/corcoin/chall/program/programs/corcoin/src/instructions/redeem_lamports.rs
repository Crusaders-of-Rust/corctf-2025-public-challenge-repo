use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::error::BondError;
use crate::state::{Bond, Config};

const FEE_BPS: u64 = 100; // 1%

#[derive(Accounts)]
pub struct RedeemLamports<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut, constraint = bond.authority == user.key())]
    pub bond: Account<'info, Bond>,
    #[account(mut, constraint = corcoin_mint.key() == bond.corcoin_mint)]
    pub corcoin_mint: Account<'info, Mint>,
    #[account(mut, constraint = user_corcoin_account.owner == user.key() && user_corcoin_account.mint == corcoin_mint.key())]
    pub user_corcoin_account: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = corcoin_mint,
        associated_token::authority = gcp
    )]
    pub gcp_corcoin_account: Account<'info, TokenAccount>,
    /// CHECK: Must match the bond's stake account
    #[account(mut, constraint = stake_account.key() == bond.stake_account)]
    pub stake_account: AccountInfo<'info>,
    /// CHECK: Derived by seeds
    #[account(seeds = [b"deactivating_stake", bond.key().as_ref()], bump)]
    pub deactivating_stake_account: AccountInfo<'info>,
    /// CHECK: Derived by seeds
    #[account(mut, seeds = [b"gcp"], bump = config.gcp_bump)]
    pub gcp: AccountInfo<'info>,
    #[account(seeds = [b"config"], bump)]
    pub config: Account<'info, Config>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<RedeemLamports>, corcoin_amount: u64) -> Result<()> {
    let clock = Clock::get()?;
    let bond = &ctx.accounts.bond;
    let corcoin_balance = ctx.accounts.user_corcoin_account.amount;

    validate_redemption_conditions(bond, &clock, corcoin_balance, corcoin_amount)?;

    let total_bond_value = crate::utils::bond::calculate_total_bond_value(
        &ctx.accounts.stake_account,
        &ctx.accounts.deactivating_stake_account,
        &ctx.accounts.bond,
    )?;

    let user_share = crate::utils::bond::calculate_user_share(
        total_bond_value,
        ctx.accounts.corcoin_mint.supply,
        corcoin_amount,
    )?;

    let amount_to_user = calculate_amount_after_fee(user_share)?;

    transfer_corcoin_tokens_to_gcp(&ctx, corcoin_amount)?;
    transfer_sol_to_user(&ctx, amount_to_user)?;

    Ok(())
}

fn validate_redemption_conditions(
    bond: &Bond,
    clock: &Clock,
    corcoin_balance: u64,
    corcoin_amount: u64,
) -> Result<()> {
    require!(
        clock.unix_timestamp >= bond.maturity_timestamp,
        BondError::BondNotMatured
    );
    require!(corcoin_balance > 0, BondError::InsufficientTokens);
    require!(corcoin_amount > 0, BondError::InsufficientTokens);
    require!(
        corcoin_amount <= corcoin_balance,
        BondError::InsufficientTokens
    );
    Ok(())
}

fn calculate_amount_after_fee(user_share: u64) -> Result<u64> {
    let fee = user_share
        .checked_mul(FEE_BPS)
        .and_then(|x| x.checked_add(9999))
        .and_then(|x| x.checked_div(10000))
        .ok_or(BondError::MathOverflow)?;
    user_share
        .checked_sub(fee)
        .ok_or(BondError::MathOverflow.into())
}

fn transfer_corcoin_tokens_to_gcp(
    ctx: &Context<RedeemLamports>,
    corcoin_amount: u64,
) -> Result<()> {
    crate::utils::token::token_transfer(
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.user_corcoin_account.to_account_info(),
        ctx.accounts.gcp_corcoin_account.to_account_info(),
        ctx.accounts.user.to_account_info(),
        corcoin_amount,
        None,
    )
}

fn transfer_sol_to_user(ctx: &Context<RedeemLamports>, amount: u64) -> Result<()> {
    let gcp_seeds: &[&[u8]] = &[b"gcp", &[ctx.accounts.config.gcp_bump]];
    crate::utils::system::system_transfer(
        ctx.accounts.gcp.to_account_info(),
        ctx.accounts.user.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        amount,
        Some(&[gcp_seeds]),
    )
}
