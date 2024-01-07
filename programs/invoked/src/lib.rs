use anchor_lang::prelude::*;

declare_id!("DHUDQvqJmjAk83ChDeJxoddPC4TYhsNaV9UJ65adpmm8");

#[program]
pub mod invoked {
    use super::*;

    pub fn invoke_me(
        _ctx: Context<InvokeMe>,
        _data: Vec<u8>,
    ) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InvokeMe<'info> {
    #[account()]
    pub anchor_doesnt_let_me_have_zero_accounts_here_with_cpi_feature:
        UncheckedAccount<'info>,
}
