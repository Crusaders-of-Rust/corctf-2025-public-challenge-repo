use crate::error::BondError;
use crate::state::Bond;
use anchor_lang::prelude::*;

pub fn calculate_total_bond_value(
    stake_account: &AccountInfo,
    deactivating_stake_account: &AccountInfo,
    bond: &Account<Bond>,
) -> Result<u64> {
    let active_balance = stake_account.lamports();

    let deactivating_balance = deactivating_stake_account.lamports();
    let bond_rent = Rent::get()?.minimum_balance(8 + crate::state::Bond::INIT_SPACE);
    let liquid_reserves = bond.to_account_info().lamports().saturating_sub(bond_rent);

    active_balance
        .checked_add(deactivating_balance)
        .and_then(|x| x.checked_add(liquid_reserves))
        .ok_or(BondError::MathOverflow.into())
}

pub fn calculate_user_share(
    total_bond_value: u64,
    total_corcoin_supply: u64,
    corcoin_amount: u64,
) -> Result<u64> {
    if total_corcoin_supply == 0 {
        return Ok(0);
    }

    total_bond_value
        .checked_mul(corcoin_amount)
        .and_then(|x| x.checked_div(total_corcoin_supply))
        .ok_or(BondError::MathOverflow.into())
}

pub fn calculate_required_coverage(
    total_bond_value: u64,
    corcoin_tokens: u64,
    total_supply: u64,
) -> Result<u64> {
    if total_supply == 0 {
        return Ok(0);
    }

    total_bond_value
        .checked_mul(corcoin_tokens)
        .and_then(|x| x.checked_div(total_supply))
        .ok_or(BondError::MathOverflow.into())
}
