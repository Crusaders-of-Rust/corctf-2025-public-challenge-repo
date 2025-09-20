use anchor_lang::prelude::*;
use anchor_lang::solana_program::stake::state::StakeStateV2;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::error::BondError;
use crate::state::{Bond, Config};
use crate::utils::stake::{get_stake_status, StakeStatus};

#[derive(Accounts)]
pub struct UpdateBondReserves<'info> {
    #[account(seeds = [b"config"], bump)]
    pub config: Account<'info, Config>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub bond: Account<'info, Bond>,
    #[account(mut, constraint = corcoin_mint.key() == bond.corcoin_mint)]
    pub corcoin_mint: Account<'info, Mint>,
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = corcoin_mint,
        associated_token::authority = gcp
    )]
    pub gcp_corcoin_account: Account<'info, TokenAccount>,
    /// CHECK: Must match bond stake account
    #[account(mut, constraint = stake_account.key() == bond.stake_account)]
    pub stake_account: AccountInfo<'info>,
    /// CHECK: Derived by seeds
    #[account(mut,
        seeds = [b"deactivating_stake", bond.key().as_ref()],
        bump
    )]
    pub deactivating_stake: AccountInfo<'info>,
    /// CHECK: Derived by seeds
    #[account(mut, seeds = [b"gcp"], bump = config.gcp_bump)]
    pub gcp: AccountInfo<'info>,
    pub clock: Sysvar<'info, Clock>,
    /// CHECK: The stake history sysvar
    pub stake_history: Sysvar<'info, StakeHistory>,
    /// CHECK: Checked by address constraint
    #[account(address = anchor_lang::solana_program::stake::program::ID)]
    pub stake_program: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<UpdateBondReserves>) -> Result<()> {
    let clock = Clock::get()?;
    let bond = &ctx.accounts.bond;

    require!(
        clock.unix_timestamp >= bond.maturity_timestamp,
        BondError::BondNotMatured
    );

    let gcp_corcoin_balance = ctx.accounts.gcp_corcoin_account.amount;

    if gcp_corcoin_balance == 0 {
        return Ok(());
    }

    let bond_rent = Rent::get()?.minimum_balance(8 + crate::state::Bond::INIT_SPACE);
    let liquid_reserve_lamports = ctx
        .accounts
        .bond
        .to_account_info()
        .lamports()
        .saturating_sub(bond_rent);

    let total_bond_value = crate::utils::bond::calculate_total_bond_value(
        &ctx.accounts.stake_account,
        &ctx.accounts.deactivating_stake,
        &ctx.accounts.bond,
    )?;
    let total_corcoin_supply = ctx.accounts.corcoin_mint.supply;
    let lamports_needed_for_counter_party = crate::utils::bond::calculate_user_share(
        total_bond_value,
        total_corcoin_supply,
        gcp_corcoin_balance,
    )?;

    let lamports_to_unstake =
        lamports_needed_for_counter_party.saturating_sub(liquid_reserve_lamports);

    if lamports_to_unstake == 0 {
        return Ok(());
    }

    let minimum_stake_account_amt = crate::utils::stake::get_minimum_stake_amount()?;
    let lamports_to_unstake = std::cmp::max(lamports_to_unstake, minimum_stake_account_amt);

    let stake_to_split = lamports_to_unstake;

    let stake_history = ctx.accounts.stake_history.to_account_info();
    let stake_history_data = stake_history.try_borrow_data()?;
    let stake_history = bincode::deserialize::<
        anchor_lang::solana_program::stake_history::StakeHistory,
    >(&stake_history_data)
    .map_err(|_| BondError::InvalidStakeState)?;

    let deactivating_stake_status = get_stake_status(
        &ctx.accounts.deactivating_stake,
        clock.epoch,
        &stake_history,
    )?;

    let bond_key = bond.key();

    let deactivating_stake_seeds: &[&[u8]] = &[
        b"deactivating_stake",
        bond_key.as_ref(),
        &[ctx.bumps.deactivating_stake],
    ];
    let bond_seeds: &[&[u8]] = &[
        b"bond",
        bond.authority.as_ref(),
        bond.validator.as_ref(),
        &bond.maturity_timestamp.to_le_bytes(),
        &[bond.bump],
    ];
    let gcp_seeds: &[&[u8]] = &[b"gcp", &[ctx.accounts.config.gcp_bump]];

    match deactivating_stake_status {
        StakeStatus::Empty => {
            crate::utils::stake::stake_create(
                ctx.accounts.deactivating_stake.to_account_info(),
                Some(&[&deactivating_stake_seeds[..]]),
            )?;

            let minimum_delegation = crate::utils::stake::get_minimum_stake_amount()?;
            let active_stake_balance = ctx.accounts.stake_account.lamports();
            let remaining_after_split = active_stake_balance
                .checked_sub(stake_to_split)
                .ok_or(BondError::MathOverflow)?;

            let gcp_share =
                if remaining_after_split < minimum_delegation && remaining_after_split > 0 {
                    active_stake_balance
                } else {
                    stake_to_split
                };

            let rent_exempt = ctx.accounts.rent.minimum_balance(StakeStateV2::size_of());
            crate::utils::system::system_transfer(
                ctx.accounts.gcp.to_account_info(),
                ctx.accounts.deactivating_stake.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
                rent_exempt,
                Some(&[gcp_seeds]),
            )?;

            crate::utils::stake::stake_split(
                ctx.accounts.stake_account.to_account_info(),
                ctx.accounts.deactivating_stake.to_account_info(),
                ctx.accounts.bond.to_account_info(),
                gcp_share,
                Some(&[bond_seeds]),
            )?;

            crate::utils::stake::stake_deactivate(
                ctx.accounts.deactivating_stake.to_account_info(),
                ctx.accounts.bond.to_account_info(),
                ctx.accounts.clock.to_account_info(),
                Some(&[bond_seeds]),
            )?;
        }
        StakeStatus::Uninitialized => {
            return Err(BondError::InvalidStakeState.into());
        }
        StakeStatus::Inactive => {
            crate::utils::stake::stake_withdraw(
                ctx.accounts.deactivating_stake.to_account_info(),
                ctx.accounts.bond.to_account_info(),
                ctx.accounts.bond.to_account_info(),
                ctx.accounts.clock.to_account_info(),
                ctx.accounts.stake_history.to_account_info(),
                ctx.accounts.deactivating_stake.lamports(),
                Some(&[&bond_seeds[..]]),
            )?;
        }
        StakeStatus::Activating | StakeStatus::FullyActive | StakeStatus::Deactivating => {
            return Err(BondError::InvalidStakeState.into());
        }
    }

    Ok(())
}
