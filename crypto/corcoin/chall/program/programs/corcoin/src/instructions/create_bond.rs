use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::error::BondError;
use crate::state::{Bond, Config};

const FEE_BPS: u64 = 100; // 1%

#[derive(Accounts)]
#[instruction(amount: u64, maturity_timestamp: i64)]
pub struct CreateBond<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    /// CHECK: Validator validated via delegate_stake CPI
    pub validator_vote: AccountInfo<'info>,
    /// CHECK: Checked by PDA seeds
    #[account(
        mut,
        seeds = [b"stake", bond.key().as_ref()],
        bump
    )]
    pub stake_account: AccountInfo<'info>,
    #[account(
        init,
        payer = user,
        space = 8 + Bond::INIT_SPACE,
        seeds = [b"bond", user.key().as_ref(), validator_vote.key().as_ref(), maturity_timestamp.to_le_bytes().as_ref()],
        bump
    )]
    pub bond: Account<'info, Bond>,
    #[account(
        init,
        payer = user,
        mint::decimals = 6,
        mint::authority = bond,
        seeds = [b"corcoin_mint", bond.key().as_ref()],
        bump
    )]
    pub corcoin_mint: Account<'info, Mint>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = corcoin_mint,
        associated_token::authority = user
    )]
    pub user_corcoin_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    /// CHECK: Checked by address constaint
    #[account(address = anchor_lang::solana_program::stake::program::ID)]
    pub stake_program: AccountInfo<'info>,
    pub clock: Sysvar<'info, Clock>,
    pub stake_history: Sysvar<'info, StakeHistory>,
    /// CHECK: Checked by address constaint
    #[account(
        address = anchor_lang::solana_program::stake::config::ID,
    )]
    pub stake_config: AccountInfo<'info>,
    /// CHECK: Derived by seeds
    #[account(mut, seeds = [b"gcp"], bump = config.gcp_bump)]
    pub gcp: AccountInfo<'info>,
    #[account(seeds = [b"config"], bump)]
    pub config: Account<'info, Config>,
}

pub fn handler(mut ctx: Context<CreateBond>, amount: u64, maturity_timestamp: i64) -> Result<()> {
    let clock = Clock::get()?;

    require!(
        maturity_timestamp > clock.unix_timestamp,
        BondError::MaturityInPast
    );
    require!(amount > 1_000_000_000, BondError::InvalidAmount);

    let fee_amount = calculate_fee(amount)?;
    let bond_amount = amount
        .checked_sub(fee_amount)
        .ok_or(BondError::MathOverflow)?;

    transfer_fee_to_gcp(&ctx, fee_amount)?;
    initialize_bond_state(&mut ctx, maturity_timestamp, clock.unix_timestamp)?;

    create_and_initialize_stake_account(&ctx, bond_amount)?;
    delegate_stake_to_validator(&ctx)?;
    mint_corcoin_tokens(&ctx, bond_amount)?;

    Ok(())
}

fn initialize_bond_state(
    ctx: &mut Context<CreateBond>,
    maturity_timestamp: i64,
    creation_timestamp: i64,
) -> Result<()> {
    let bond = &mut ctx.accounts.bond;
    bond.authority = ctx.accounts.user.key();
    bond.validator = ctx.accounts.validator_vote.key();
    bond.maturity_timestamp = maturity_timestamp;
    bond.creation_timestamp = creation_timestamp;
    bond.corcoin_mint = ctx.accounts.corcoin_mint.key();
    bond.bump = ctx.bumps.bond;
    bond.stake_account = ctx.accounts.stake_account.key();
    Ok(())
}

fn create_and_initialize_stake_account(ctx: &Context<CreateBond>, amount: u64) -> Result<()> {
    let stake_space =
        std::mem::size_of::<anchor_lang::solana_program::stake::state::StakeStateV2>();
    let rent = Rent::get()?;
    let rent_exempt = rent.minimum_balance(stake_space);
    let total_lamports = amount + rent_exempt;

    let bond_key = ctx.accounts.bond.key();
    let stake_seeds: &[&[u8]] = &[b"stake", bond_key.as_ref(), &[ctx.bumps.stake_account]];

    let create_ixs = anchor_lang::solana_program::stake::instruction::create_account(
        &ctx.accounts.user.key(),
        &ctx.accounts.stake_account.key(),
        &anchor_lang::solana_program::stake::state::Authorized {
            staker: ctx.accounts.bond.key(),
            withdrawer: ctx.accounts.bond.key(),
        },
        &anchor_lang::solana_program::stake::state::Lockup::default(),
        total_lamports,
    );

    for create_ix in create_ixs.iter() {
        anchor_lang::solana_program::program::invoke_signed(
            create_ix,
            &[
                ctx.accounts.stake_account.to_account_info(),
                ctx.accounts.user.to_account_info(),
                ctx.accounts.clock.to_account_info(),
                ctx.accounts.rent.to_account_info(),
            ],
            &[stake_seeds],
        )?;
    }

    Ok(())
}

fn delegate_stake_to_validator(ctx: &Context<CreateBond>) -> Result<()> {
    let user_key = ctx.accounts.user.key();
    let validator_key = ctx.accounts.validator_vote.key();
    let maturity_bytes = ctx.accounts.bond.maturity_timestamp.to_le_bytes();
    let bond_seeds: &[&[u8]] = &[
        b"bond",
        user_key.as_ref(),
        validator_key.as_ref(),
        maturity_bytes.as_ref(),
        &[ctx.bumps.bond],
    ];

    let delegate_ix = anchor_lang::solana_program::stake::instruction::delegate_stake(
        ctx.accounts.stake_account.key,
        &ctx.accounts.bond.key(),
        ctx.accounts.validator_vote.key,
    );

    anchor_lang::solana_program::program::invoke_signed(
        &delegate_ix,
        &[
            ctx.accounts.stake_account.to_account_info(),
            ctx.accounts.validator_vote.to_account_info(),
            ctx.accounts.clock.to_account_info(),
            ctx.accounts.stake_history.to_account_info(),
            ctx.accounts.stake_config.to_account_info(),
            ctx.accounts.bond.to_account_info(),
        ],
        &[bond_seeds],
    )?;

    Ok(())
}

fn mint_corcoin_tokens(ctx: &Context<CreateBond>, amount: u64) -> Result<()> {
    let bump = ctx.accounts.bond.bump;
    let authority_key = ctx.accounts.bond.authority;
    let validator_key = ctx.accounts.bond.validator;
    let maturity_bytes = ctx.accounts.bond.maturity_timestamp.to_le_bytes();
    let bond_seeds: &[&[u8]] = &[
        b"bond",
        authority_key.as_ref(),
        validator_key.as_ref(),
        maturity_bytes.as_ref(),
        &[bump],
    ];

    crate::utils::token::token_mint_to(
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.corcoin_mint.to_account_info(),
        ctx.accounts.user_corcoin_account.to_account_info(),
        ctx.accounts.bond.to_account_info(),
        amount,
        Some(&[bond_seeds]),
    )?;

    Ok(())
}

fn calculate_fee(amount: u64) -> Result<u64> {
    amount
        .checked_mul(FEE_BPS)
        .and_then(|x| x.checked_add(9999))
        .and_then(|x| x.checked_div(10000))
        .ok_or(BondError::MathOverflow.into())
}

fn transfer_fee_to_gcp(ctx: &Context<CreateBond>, fee_amount: u64) -> Result<()> {
    crate::utils::system::system_transfer(
        ctx.accounts.user.to_account_info(),
        ctx.accounts.gcp.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        fee_amount,
        None,
    )
}
