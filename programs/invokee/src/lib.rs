use anchor_lang::prelude::*;
use anchor_lang::solana_program::stable_layout::{
    stable_instruction::StableInstruction, stable_vec::StableVec,
};
use invoked::program::Invoked;

declare_id!("uNmABrufQJDmvBFJ8XdPNKV5rJuyhPs7hqhXuVtg3zY");

#[program]
pub mod invokee {
    use super::*;

    // Memory Crimes
    // Anchor:
    // 1) if calling multiple times, cloning data every time you call.
    // Solana:
    // 2) invoke clones Instruction (clones data: Vec<u8> AND clones metas: Vec<AccountMeta>)
    pub fn invoke_other_program<'info>(
        ctx: Context<'_, '_, '_, 'info, Invoke<'info>>,
    ) -> Result<()> {
        let data = vec![5; 1024];
        for _ in 0..20 {
            invoked::cpi::invoke_me(
                CpiContext::new(
                    ctx.accounts.invoked.to_account_info(),
                    invoked::cpi::accounts::InvokeMe {
                        anchor_doesnt_let_me_have_zero_accounts_here_with_cpi_feature: ctx.remaining_accounts[0].clone(),
                    },
                ),
                data.clone(),
            )?;
        }

        Ok(())
    }

    pub fn invoke_another_program<'info>(
        ctx: Context<'_, '_, '_, 'info, Invoke2>,
    ) -> Result<()> {
        // validate program
        assert_eq!(
            ctx.remaining_accounts[0].key.as_ref(),
            invoked2::ID.as_ref()
        );

        // Prepare for invoke syscall:
        // 1) instruction
        // 2) account infos
        // 3) seeds
        let instruction = StableInstruction {
            program_id: invoked2::ID,
            accounts: StableVec::from(
                ctx.remaining_accounts[0].to_account_metas(None),
            ),
            data: StableVec::from(vec![5; 1024]),
        };
        let infos = vec![ctx.remaining_accounts[0].clone()];
        let seeds: &[&[&[u8]]] = &[];

        #[cfg(target_os = "solana")]
        for _ in 0..63 {
            unsafe {
                solana_program::syscalls::sol_invoke_signed_rust(
                    &instruction as *const StableInstruction
                        as *const u8,
                    infos.as_ptr() as *const u8,
                    infos.len() as u64,
                    seeds.as_ptr() as *const u8,
                    seeds.len() as u64,
                );
            }
        }

        core::hint::black_box((
            &instruction,
            infos.as_ptr() as *const u8,
            infos.len() as u64,
            seeds.as_ptr() as *const u8,
            seeds.len() as u64,
        ));

        Ok(())
    }

    pub fn invoke_another_program_no_alloc<'info>(
        ctx: Context<'_, '_, '_, 'info, Invoke2>,
    ) -> Result<()> {
        // validate program
        assert_eq!(*ctx.remaining_accounts[0].key, invoked2::ID);

        #[repr(C)]
        #[derive(Debug, PartialEq)]
        pub struct StableView<T> {
            pub ptr: core::ptr::NonNull<T>,
            pub cap: usize,
            pub len: usize,
        }
        impl<T> StableView<T> {
            pub fn from_slice(slice: &[T]) -> StableView<T> {
                Self {
                    ptr: core::ptr::NonNull::new(
                        slice.as_ptr() as *mut T
                    )
                    .unwrap(),
                    cap: slice.len(),
                    len: slice.len(),
                }
            }
        }

        #[derive(Debug, PartialEq)]
        #[repr(C)]
        pub struct StableInstructionooor {
            pub accounts: StableView<AccountMeta>,
            pub data: StableView<u8>,
            pub program_id: Pubkey,
        }

        // Prepare for invoke syscall:
        // 1) instruction
        // 2) account infos
        // 3) seeds
        let instruction_accounts = [AccountMeta::new_readonly(
            *ctx.remaining_accounts[0].key,
            false,
        )];
        let instruction_data = [5; 1024];
        let instruction = StableInstructionooor {
            program_id: invoked2::ID,
            accounts: StableView::from_slice(&instruction_accounts),
            data: StableView::from_slice(&instruction_data),
        };
        let infos = [ctx.remaining_accounts[0].clone()];
        let seeds: &[&[&[u8]]] = &[];

        #[cfg(target_os = "solana")]
        unsafe {
            solana_program::syscalls::sol_invoke_signed_rust(
                &instruction as *const StableInstructionooor
                    as *const u8,
                infos.as_ptr() as *const u8,
                infos.len() as u64,
                seeds.as_ptr() as *const u8,
                seeds.len() as u64,
            );
        }

        core::hint::black_box((
            &instruction,
            infos.as_ptr() as *const u8,
            infos.len() as u64,
            seeds.as_ptr() as *const u8,
            seeds.len() as u64,
        ));

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Invoke<'info> {
    #[account()]
    pub invoked: Program<'info, Invoked>,
}

#[derive(Accounts)]
pub struct Invoke2 {}

#[cfg(test)]
mod tests {
    use anchor_lang::{system_program, InstructionData};
    use solana_program::{
        entrypoint::ProgramResult, instruction::Instruction,
    };
    use solana_program_test::{processor, ProgramTest};
    use solana_sdk::{signer::Signer, transaction::Transaction};

    use super::*;

    #[inline(always)]
    fn process_instruction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        entry(
            program_id,
            unsafe { core::mem::transmute(accounts) },
            instruction_data,
        )
    }

    #[inline(always)]
    fn process_instruction_invoked(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        invoked::entry(
            program_id,
            unsafe { core::mem::transmute(accounts) },
            instruction_data,
        )
    }

    #[tokio::test]
    async fn cpi_demo() {
        let mut pt = ProgramTest::new(
            "invokee",
            crate::ID,
            processor!(process_instruction),
        );
        pt.add_program(
            "invoked",
            invoked::ID,
            processor!(process_instruction_invoked),
        );
        pt.add_program(
            "invoked2",
            invoked2::ID,
            processor!(invoked2::process_instruction),
        );
        let (mut banks_client, payer, recent_blockhash) =
            pt.start().await;

        // Build tx
        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(invoked::ID, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data: crate::instruction::InvokeOtherProgram {}.data(),
        };
        let mut transaction = Transaction::new_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
        );
        transaction.sign(&[&payer], recent_blockhash);

        banks_client
            .process_transaction(transaction)
            .await
            .ok();

        // Build tx for invoked 2
        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(invoked2::ID, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data: crate::instruction::InvokeAnotherProgram {}.data(),
        };
        let mut transaction = Transaction::new_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
        );
        transaction.sign(&[&payer], recent_blockhash);

        banks_client
            .process_transaction(transaction)
            .await
            .ok();

        // Build tx for invoked 3 (array-backed)
        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(invoked2::ID, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data: crate::instruction::InvokeAnotherProgramNoAlloc {}
                .data(),
        };
        let mut transaction = Transaction::new_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
        );
        transaction.sign(&[&payer], recent_blockhash);

        banks_client
            .process_transaction(transaction)
            .await
            .ok();
    }
}
