use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    program::invoke, stake, stake::state::StakeAuthorize, system_instruction,
};
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::error::BondError;
use crate::state::Bond;
use crate::utils::system::system_transfer;

#[derive(Accounts)]
pub struct RedeemStake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut, constraint = bond.authority == user.key())]
    pub bond: Account<'info, Bond>,
    #[account(mut, constraint = corcoin_mint.key() == bond.corcoin_mint)]
    pub corcoin_mint: Account<'info, Mint>,
    #[account(mut, constraint = user_corcoin_account.owner == user.key() && user_corcoin_account.mint == corcoin_mint.key())]
    pub user_corcoin_account: Account<'info, TokenAccount>,
    /// CHECK: Must match the bond's stake account
    #[account(mut, constraint = stake_account.key() == bond.stake_account)]
    pub stake_account: AccountInfo<'info>,
    /// CHECK: Derived by seeds
    #[account(seeds = [b"deactivating_stake", bond.key().as_ref()], bump)]
    pub deactivating_stake_account: AccountInfo<'info>,
    /// CHECK: Destination stake account for split, can be any account owned by user
    #[account(mut, signer)]
    pub destination_stake_account: AccountInfo<'info>,
    pub clock: Sysvar<'info, Clock>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    /// CHECK: Checked by address constraint
    #[account(address = anchor_lang::solana_program::stake::program::ID)]
    pub stake_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<RedeemStake>, corcoin_amount: u64) -> Result<()> {
    let clock = Clock::get()?;
    let bond = &ctx.accounts.bond;
    let corcoin_balance = ctx.accounts.user_corcoin_account.amount;

    validate_redemption_conditions(bond, &clock, corcoin_balance, corcoin_amount)?;

    let active_stake_balance = ctx.accounts.stake_account.lamports();
    let total_bond_value = crate::utils::bond::calculate_total_bond_value(
        &ctx.accounts.stake_account,
        &ctx.accounts.deactivating_stake_account,
        &ctx.accounts.bond,
    )?;
    let total_supply = ctx.accounts.corcoin_mint.supply;

    let user_share_lamports =
        crate::utils::bond::calculate_user_share(total_bond_value, total_supply, corcoin_amount)?;

    require!(
        user_share_lamports <= active_stake_balance,
        BondError::InsufficientTokens
    );

    validate_stake_split_amounts(user_share_lamports, active_stake_balance)?;
    create_destination_stake_account(&ctx)?;

    let (seed1, seed2, seed3, seed4, bump) = get_bond_seeds(bond);
    let bond_seeds: &[&[u8]] = &[seed1, seed2, seed3, seed4.as_ref(), &[bump]];
    split_stake_account(&ctx, user_share_lamports, bond_seeds)?;

    burn_corcoin_tokens(&ctx, corcoin_amount)?;
    transfer_stake_authorities(&ctx, bond_seeds)?;

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

fn validate_stake_split_amounts(user_share: u64, stake_balance: u64) -> Result<()> {
    let minimum_delegation = crate::utils::stake::get_minimum_stake_amount()?;
    let remaining_stake = stake_balance
        .checked_sub(user_share)
        .ok_or(BondError::MathOverflow)?;

    require!(
        user_share >= minimum_delegation
            && (remaining_stake >= minimum_delegation || remaining_stake == 0),
        BondError::InsufficientStakeAmount
    );

    Ok(())
}

fn get_bond_seeds(bond: &Bond) -> (&[u8], &[u8], &[u8], [u8; 8], u8) {
    let maturity_bytes = bond.maturity_timestamp.to_le_bytes();
    (
        b"bond",
        bond.authority.as_ref(),
        bond.validator.as_ref(),
        maturity_bytes,
        bond.bump,
    )
}

fn split_stake_account(
    ctx: &Context<RedeemStake>,
    user_share_lamports: u64,
    bond_seeds: &[&[u8]],
) -> Result<()> {
    crate::utils::stake::stake_split(
        ctx.accounts.stake_account.to_account_info(),
        ctx.accounts.destination_stake_account.to_account_info(),
        ctx.accounts.bond.to_account_info(),
        user_share_lamports,
        Some(&[bond_seeds]),
    )
}

fn burn_corcoin_tokens(ctx: &Context<RedeemStake>, corcoin_amount: u64) -> Result<()> {
    crate::utils::token::token_burn(
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.corcoin_mint.to_account_info(),
        ctx.accounts.user_corcoin_account.to_account_info(),
        ctx.accounts.user.to_account_info(),
        corcoin_amount,
        None,
    )
}

fn create_destination_stake_account(ctx: &Context<RedeemStake>) -> Result<()> {
    let stake_space =
        std::mem::size_of::<anchor_lang::solana_program::stake::state::StakeStateV2>();
    let rent = Rent::get()?;
    let rent_exempt = rent.minimum_balance(stake_space);

    invoke(
        &system_instruction::allocate(
            ctx.accounts.destination_stake_account.key,
            stake_space as u64,
        ),
        &[ctx.accounts.destination_stake_account.clone()],
    )?;

    system_transfer(
        ctx.accounts.user.to_account_info(),
        ctx.accounts.destination_stake_account.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        rent_exempt,
        None,
    )?;

    invoke(
        &system_instruction::assign(
            ctx.accounts.destination_stake_account.key,
            &stake::program::id(),
        ),
        &[ctx.accounts.destination_stake_account.clone()],
    )?;

    Ok(())
}

fn transfer_stake_authorities(ctx: &Context<RedeemStake>, bond_seeds: &[&[u8]]) -> Result<()> {
    crate::utils::stake::stake_authorize(
        ctx.accounts.destination_stake_account.to_account_info(),
        ctx.accounts.bond.to_account_info(),
        ctx.accounts.user.to_account_info(),
        StakeAuthorize::Staker,
        ctx.accounts.clock.to_account_info(),
        Some(&[bond_seeds]),
    )?;

    crate::utils::stake::stake_authorize(
        ctx.accounts.destination_stake_account.to_account_info(),
        ctx.accounts.bond.to_account_info(),
        ctx.accounts.user.to_account_info(),
        StakeAuthorize::Withdrawer,
        ctx.accounts.clock.to_account_info(),
        Some(&[bond_seeds]),
    )?;

    Ok(())
}
