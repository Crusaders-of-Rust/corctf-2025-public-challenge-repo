use anchor_lang::prelude::*;
use anchor_lang::solana_program::stake::{
    self, instruction as stake_instruction,
    state::StakeStateV2,
};
use anchor_lang::solana_program::stake_history::{Epoch, StakeHistory};
use anchor_lang::solana_program::{
    program::{get_return_data, invoke, invoke_signed},
    system_instruction,
};

#[derive(Debug, PartialEq)]
pub enum StakeStatus {
    Empty,
    Uninitialized,
    Inactive,
    Activating,
    FullyActive,
    Deactivating,
}

pub fn get_stake_status(
    stake_account: &AccountInfo,
    epoch: Epoch,
    stake_history: &StakeHistory,
) -> Result<StakeStatus> {
    let stake_data = stake_account.try_borrow_data()?;

    if stake_data.len() != std::mem::size_of::<StakeStateV2>() {
        return Ok(StakeStatus::Empty);
    }

    let stake_state = StakeStateV2::deserialize(&mut stake_data.as_ref())?;
    let (_, stake) = match stake_state {
        StakeStateV2::Stake(meta, stake, _) => (meta, stake),
        StakeStateV2::Uninitialized => return Ok(StakeStatus::Uninitialized),
        _ => return Ok(StakeStatus::Inactive),
    };

    let status = stake
        .delegation
        .stake_activating_and_deactivating(epoch, stake_history, None);

    match (status.effective, status.activating, status.deactivating) {
        (0, 0, 0) => Ok(StakeStatus::Inactive),
        (0, _, _) => Ok(StakeStatus::Activating),
        (_, 0, 0) => Ok(StakeStatus::FullyActive),
        _ => Ok(StakeStatus::Deactivating),
    }
}

pub fn stake_create(
    stake_account_info: AccountInfo<'_>,
    signer: Option<&[&[&[u8]]]>,
) -> Result<()> {
    let stake_space = std::mem::size_of::<StakeStateV2>();

    if let Some(signer_seeds) = signer {
        invoke_signed(
            &system_instruction::allocate(stake_account_info.key, stake_space as u64),
            &[stake_account_info.clone()],
            signer_seeds,
        )?;
        invoke_signed(
            &system_instruction::assign(stake_account_info.key, &stake::program::id()),
            &[stake_account_info],
            signer_seeds,
        )?;
    } else {
        invoke(
            &system_instruction::allocate(stake_account_info.key, stake_space as u64),
            &[stake_account_info.clone()],
        )?;
        invoke(
            &system_instruction::assign(stake_account_info.key, &stake::program::id()),
            &[stake_account_info],
        )?;
    }
    Ok(())
}

pub fn stake_split<'info>(
    source_stake: AccountInfo<'info>,
    destination_stake: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    lamports: u64,
    seeds: Option<&[&[&[u8]]]>,
) -> Result<()> {
    let split_ixs = stake_instruction::split(
        source_stake.key,
        authority.key,
        lamports,
        destination_stake.key,
    );

    let split_ix = split_ixs
        .last()
        .ok_or(anchor_lang::error::ErrorCode::AccountNotInitialized)?;

    if let Some(signer_seeds) = seeds {
        invoke_signed(
            split_ix,
            &[
                source_stake.clone(),
                destination_stake.clone(),
                authority.clone(),
            ],
            signer_seeds,
        )?;
    } else {
        invoke(
            split_ix,
            &[
                source_stake.clone(),
                destination_stake.clone(),
                authority.clone(),
            ],
        )?;
    }

    Ok(())
}

pub fn stake_authorize<'info>(
    stake_account: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    new_authority: AccountInfo<'info>,
    stake_authorize: anchor_lang::solana_program::stake::state::StakeAuthorize,
    clock: AccountInfo<'info>,
    seeds: Option<&[&[&[u8]]]>,
) -> Result<()> {
    let authorize_ix = stake_instruction::authorize(
        stake_account.key,
        authority.key,
        new_authority.key,
        stake_authorize,
        None,
    );

    if let Some(signer_seeds) = seeds {
        invoke_signed(
            &authorize_ix,
            &[stake_account, authority, clock],
            signer_seeds,
        )?;
    } else {
        invoke(&authorize_ix, &[stake_account, authority, clock])?;
    }

    Ok(())
}

pub fn stake_deactivate<'info>(
    stake_account: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    clock: AccountInfo<'info>,
    seeds: Option<&[&[&[u8]]]>,
) -> Result<()> {
    let deactivate_ix = stake_instruction::deactivate_stake(stake_account.key, authority.key);

    if let Some(signer_seeds) = seeds {
        invoke_signed(
            &deactivate_ix,
            &[stake_account, authority, clock],
            signer_seeds,
        )?;
    } else {
        invoke(&deactivate_ix, &[stake_account, authority, clock])?;
    }

    Ok(())
}

pub fn stake_withdraw<'info>(
    stake_account: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    to: AccountInfo<'info>,
    clock: AccountInfo<'info>,
    stake_history: AccountInfo<'info>,
    lamports: u64,
    seeds: Option<&[&[&[u8]]]>,
) -> Result<()> {
    let withdraw_ix =
        stake_instruction::withdraw(stake_account.key, authority.key, to.key, lamports, None);

    if let Some(signer_seeds) = seeds {
        invoke_signed(
            &withdraw_ix,
            &[stake_account, authority, to, clock, stake_history],
            signer_seeds,
        )?;
    } else {
        invoke(
            &withdraw_ix,
            &[stake_account, authority, to, clock, stake_history],
        )?;
    }

    Ok(())
}

pub fn stake_delegate<'info>(
    stake_account: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    vote_account: AccountInfo<'info>,
    clock: AccountInfo<'info>,
    stake_history: AccountInfo<'info>,
    stake_config: AccountInfo<'info>,
    seeds: Option<&[&[&[u8]]]>,
) -> Result<()> {
    let delegate_ix =
        stake_instruction::delegate_stake(stake_account.key, authority.key, vote_account.key);

    if let Some(signer_seeds) = seeds {
        invoke_signed(
            &delegate_ix,
            &[
                stake_account,
                authority,
                vote_account,
                clock,
                stake_history,
                stake_config,
            ],
            signer_seeds,
        )?;
    } else {
        invoke(
            &delegate_ix,
            &[
                stake_account,
                authority,
                vote_account,
                clock,
                stake_history,
                stake_config,
            ],
        )?;
    }

    Ok(())
}

pub fn get_minimum_stake_amount() -> Result<u64> {
    invoke(&stake::instruction::get_minimum_delegation(), &[])?;
    let res = get_return_data()
        .ok_or(ProgramError::InvalidInstructionData)
        .and_then(|(program_id, return_data)| {
            (program_id == stake::program::id())
                .then_some(return_data)
                .ok_or(ProgramError::IncorrectProgramId)
        })
        .and_then(|return_data| {
            return_data
                .try_into()
                .or(Err(ProgramError::InvalidInstructionData))
        })
        .map(u64::from_le_bytes)?;
    Ok(res)
}
