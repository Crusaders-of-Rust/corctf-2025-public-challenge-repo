use anchor_lang::prelude::*;

use crate::state::Config;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account(init, payer = initializer, space = 8 + Config::INIT_SPACE, seeds = [b"config"], bump)]
    pub config: Account<'info, Config>,
    /// CHECK: Derived by seeds
    #[account(mut, seeds = [b"gcp"], bump)]
    pub gcp: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Initialize>, admin: Pubkey, amount: u64) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.admin = admin;
    config.gcp_bump = ctx.bumps.gcp;

    crate::utils::system::system_transfer(
        ctx.accounts.initializer.to_account_info(),
        ctx.accounts.gcp.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        amount,
        None,
    )?;

    Ok(())
}
