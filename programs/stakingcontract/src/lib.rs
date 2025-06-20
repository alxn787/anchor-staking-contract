use anchor_lang::prelude::*;

declare_id!("ERoFVBGqdz2xsgSvuGqaZbjni4swEjo4ByKXHwpDDP5U");

#[program]
pub mod stakingcontract {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

#[error_code]
pub enum ErrorMessages {
    msg!("amount must be greater than 0")
}
