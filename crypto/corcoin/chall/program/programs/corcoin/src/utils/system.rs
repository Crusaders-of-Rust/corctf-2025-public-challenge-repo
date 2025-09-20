use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke_signed, system_instruction};

pub fn system_transfer<'info>(
    from: AccountInfo<'info>,
    to: AccountInfo<'info>,
    system_program: AccountInfo<'info>,
    amount: u64,
    signer_seeds: Option<&[&[&[u8]]]>,
) -> Result<()> {
    let transfer_ix = system_instruction::transfer(from.key, to.key, amount);

    if let Some(seeds) = signer_seeds {
        invoke_signed(&transfer_ix, &[from, to, system_program], seeds)?;
    } else {
        anchor_lang::solana_program::program::invoke(&transfer_ix, &[from, to, system_program])?;
    }

    Ok(())
}

pub fn system_create_account<'info>(
    payer: AccountInfo<'info>,
    new_account: AccountInfo<'info>,
    system_program: AccountInfo<'info>,
    lamports: u64,
    space: u64,
    owner: &Pubkey,
    signer_seeds: Option<&[&[&[u8]]]>,
) -> Result<()> {
    let create_ix =
        system_instruction::create_account(payer.key, new_account.key, lamports, space, owner);

    if let Some(seeds) = signer_seeds {
        invoke_signed(&create_ix, &[payer, new_account, system_program], seeds)?;
    } else {
        anchor_lang::solana_program::program::invoke(
            &create_ix,
            &[payer, new_account, system_program],
        )?;
    }

    Ok(())
}
