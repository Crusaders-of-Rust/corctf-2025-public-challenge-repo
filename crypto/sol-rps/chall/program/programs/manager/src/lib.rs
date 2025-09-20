use anchor_lang::prelude::*;

declare_id!("GdWp89pQC5n36HAznAYxqE6ev9dGvrH2cPMn5BQjJ6nW");

#[program]
pub mod manager {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, admin: Pubkey) -> Result<()> {
        ctx.accounts.config.admin = admin;
        ctx.accounts.config.won = false;
        Ok(())
    }

    pub fn run_game(ctx: Context<RunGame>, admin_moves: [rps::Move; 100]) -> Result<()> {
        rps::cpi::create_game(CpiContext::new(
            ctx.accounts.rps_program.to_account_info(),
            rps::cpi::accounts::CreateGame {
                authority: ctx.accounts.admin.to_account_info(),
                game: ctx.accounts.game.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
            },
        ))?;

        rps::cpi::set_player1_moves(CpiContext::new(
            ctx.accounts.rps_program.to_account_info(),
            rps::cpi::accounts::SetMoves {
                authority: ctx.accounts.admin.to_account_info(),
                game: ctx.accounts.game.to_account_info(),
            },
        ), admin_moves)?;

        let player_moves = player::cpi::get_moves(CpiContext::new(
            ctx.accounts.player_program.to_account_info(),
            player::cpi::accounts::GetMoves {
                clock: ctx.accounts.clock.to_account_info(),
            },
        ))?;

        let mut rps_moves = [rps::Move::Rock; 100];
        for (i, player_move) in player_moves.get().iter().enumerate() {
            rps_moves[i] = match player_move {
                player::Move::Rock => rps::Move::Rock,
                player::Move::Paper => rps::Move::Paper,
                player::Move::Scissors => rps::Move::Scissors,
            };
        }

        rps::cpi::set_player2_moves(CpiContext::new(
            ctx.accounts.rps_program.to_account_info(),
            rps::cpi::accounts::SetMoves {
                authority: ctx.accounts.admin.to_account_info(),
                game: ctx.accounts.game.to_account_info(),
            },
        ), rps_moves)?;

        let game_result = rps::cpi::compute_winner(CpiContext::new(
            ctx.accounts.rps_program.to_account_info(),
            rps::cpi::accounts::ComputeWinner {
                authority: ctx.accounts.admin.to_account_info(),
                game: ctx.accounts.game.to_account_info(),
            },
        ))?.get();

        if game_result.player2_wins == 100 {
            ctx.accounts.config.won = true;
        }

        Ok(())
    }
}

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub admin: Pubkey,
    pub won: bool
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        seeds = [b"config"],
        bump,
        space = 8 + Config::INIT_SPACE
    )]
    pub config: Account<'info, Config>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RunGame<'info> {
    #[account(constraint = admin.key() == config.admin @ ErrorCode::UnauthorizedAdmin)]
    pub admin: Signer<'info>,
    /// CHECK: PDA checked by RPS program
    #[account(mut)]
    pub game: AccountInfo<'info>,
    pub player_program: Program<'info, player::program::Player>,
    #[account(
        mut,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, Config>,
    pub rps_program: Program<'info, rps::program::Rps>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized admin")]
    UnauthorizedAdmin,
}