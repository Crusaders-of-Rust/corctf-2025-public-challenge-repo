use solana_program::{entrypoint, pubkey::Pubkey, account_info::AccountInfo};

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> solana_program::entrypoint::ProgramResult {
    Ok(())
}
