use anchor_lang::prelude::*;

use crate::state::Config;

#[derive(Accounts)]
pub struct AdminWithdraw<'info> {
    #[account(seeds = [b"config"], bump)]
    pub config: Account<'info, Config>,
    #[account(mut, constraint = admin.key() == config.admin)]
    pub admin: Signer<'info>,
    /// CHECK: Derived by seeds
    #[account(mut, seeds = [b"gcp"], bump = config.gcp_bump)]
    pub gcp: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<AdminWithdraw>, amount: u64) -> Result<()> {
    let gcp_seeds: &[&[u8]] = &[b"gcp", &[ctx.accounts.config.gcp_bump]];

    crate::utils::system::system_transfer(
        ctx.accounts.gcp.to_account_info(),
        ctx.accounts.admin.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        amount,
        Some(&[gcp_seeds]),
    )?;

    Ok(())
}
