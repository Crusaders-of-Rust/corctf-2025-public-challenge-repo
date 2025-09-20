use anchor_lang::prelude::*;

declare_id!("63BgSDqQVJdfX9HDsxFUKJfjt45pnEPpT2CoWMwPNsho");

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum Move {
    Rock,
    Paper,
    Scissors,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub struct GameResult {
    pub player1_wins: u32,
    pub player2_wins: u32,
    pub ties: u32,
}

#[program]
pub mod rps {
    use super::*;

    pub fn create_game(ctx: Context<CreateGame>) -> Result<()> {
        let game = &mut ctx.accounts.game;
        game.player1_moves = [Move::Rock; 100];
        game.player2_moves = [Move::Rock; 100];
        game.authority = ctx.accounts.authority.key();
        Ok(())
    }

    pub fn set_player1_moves(ctx: Context<SetMoves>, moves: [Move; 100]) -> Result<()> {
        ctx.accounts.game.player1_moves = moves;
        Ok(())
    }

    pub fn set_player2_moves(ctx: Context<SetMoves>, moves: [Move; 100]) -> Result<()> {
        ctx.accounts.game.player2_moves = moves;
        Ok(())
    }

    pub fn compute_winner(ctx: Context<ComputeWinner>) -> Result<GameResult> {
        let game = &ctx.accounts.game;
        let mut player1_wins = 0u32;
        let mut player2_wins = 0u32;
        let mut ties = 0u32;

        for i in 0..100 {
            let p1_move = game.player1_moves[i];
            let p2_move = game.player2_moves[i];

            match (p1_move, p2_move) {
                (Move::Rock, Move::Scissors) | (Move::Paper, Move::Rock) | (Move::Scissors, Move::Paper) => player1_wins += 1,
                (Move::Scissors, Move::Rock) | (Move::Rock, Move::Paper) | (Move::Paper, Move::Scissors) => player2_wins += 1,
                _ => ties += 1,
            }
        }

        Ok(GameResult {
            player1_wins,
            player2_wins,
            ties,
        })
    }
}

#[derive(Accounts)]
pub struct CreateGame<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init, 
        payer = authority,
        space = 8 + Game::INIT_SPACE,
        seeds = [b"game", authority.key().as_ref()],
        bump
    )]
    pub game: Account<'info, Game>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetMoves<'info> {
    pub authority: Signer<'info>,
    #[account(
        mut,
        has_one = authority @ ErrorCode::UnauthorizedAuthority
    )]
    pub game: Account<'info, Game>,
}

#[derive(Accounts)]
pub struct ComputeWinner<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut,
        close = authority,
        has_one = authority @ ErrorCode::UnauthorizedAuthority
    )]
    pub game: Account<'info, Game>,
}

#[account]
#[derive(InitSpace)]
pub struct Game {
    pub authority: Pubkey,
    pub player1_moves: [Move; 100],
    pub player2_moves: [Move; 100],
}

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized authority")]
    UnauthorizedAuthority,
}