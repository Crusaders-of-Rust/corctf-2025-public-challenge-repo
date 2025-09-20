use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, CloseAccount, MintTo, Transfer};

pub fn token_mint_to<'info>(
    token_program: AccountInfo<'info>,
    mint: AccountInfo<'info>,
    to: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    amount: u64,
    signer_seeds: Option<&[&[&[u8]]]>,
) -> Result<()> {
    let cpi_accounts = MintTo {
        mint: mint.clone(),
        to: to.clone(),
        authority: authority.clone(),
    };

    if let Some(seeds) = signer_seeds {
        token::mint_to(
            CpiContext::new_with_signer(token_program, cpi_accounts, seeds),
            amount,
        )
    } else {
        token::mint_to(CpiContext::new(token_program, cpi_accounts), amount)
    }
}

pub fn token_transfer<'info>(
    token_program: AccountInfo<'info>,
    from: AccountInfo<'info>,
    to: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    amount: u64,
    signer_seeds: Option<&[&[&[u8]]]>,
) -> Result<()> {
    let cpi_accounts = Transfer {
        from: from.clone(),
        to: to.clone(),
        authority: authority.clone(),
    };

    if let Some(seeds) = signer_seeds {
        token::transfer(
            CpiContext::new_with_signer(token_program, cpi_accounts, seeds),
            amount,
        )
    } else {
        token::transfer(CpiContext::new(token_program, cpi_accounts), amount)
    }
}

pub fn token_burn<'info>(
    token_program: AccountInfo<'info>,
    mint: AccountInfo<'info>,
    from: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    amount: u64,
    signer_seeds: Option<&[&[&[u8]]]>,
) -> Result<()> {
    let cpi_accounts = Burn {
        mint: mint.clone(),
        from: from.clone(),
        authority: authority.clone(),
    };

    if let Some(seeds) = signer_seeds {
        token::burn(
            CpiContext::new_with_signer(token_program, cpi_accounts, seeds),
            amount,
        )
    } else {
        token::burn(CpiContext::new(token_program, cpi_accounts), amount)
    }
}

pub fn token_close_account<'info>(
    token_program: AccountInfo<'info>,
    account: AccountInfo<'info>,
    destination: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    signer_seeds: Option<&[&[&[u8]]]>,
) -> Result<()> {
    let cpi_accounts = CloseAccount {
        account: account.clone(),
        destination: destination.clone(),
        authority: authority.clone(),
    };

    if let Some(seeds) = signer_seeds {
        token::close_account(CpiContext::new_with_signer(
            token_program,
            cpi_accounts,
            seeds,
        ))
    } else {
        token::close_account(CpiContext::new(token_program, cpi_accounts))
    }
}
