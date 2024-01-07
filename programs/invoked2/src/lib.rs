use solana_program::{
    account_info::AccountInfo, declare_id, entrypoint::ProgramResult,
    program_error::ProgramError, pubkey::Pubkey,
};

declare_id!("7Lo52siXF692aPaTDhwdkkeaDvKUqMWNx98RACVMrA7d");

mod entrypoint {
    use super::*;

    #[no_mangle]
    #[inline(always)]
    pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
        let (program_id, accounts, instruction_data) =
            unsafe { solana_program::entrypoint::deserialize(input) };
        match process_instruction(
            &program_id,
            &accounts,
            &instruction_data,
        ) {
            Ok(()) => solana_program::entrypoint::SUCCESS,
            Err(error) => error.into(),
        }
    }
    #[cfg(not(feature = "no-entrypoint"))]
    solana_program::custom_heap_default!();
    #[cfg(not(feature = "no-entrypoint"))]
    solana_program::custom_panic_default!();
}

#[inline(always)]
pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    if accounts.is_empty() {
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    let (_first_account, _remaining_accounts) = accounts.split_at(1);
    assert_eq!(data.len(), 1024);

    Ok(())
}
