use anchor_lang::prelude::*;

declare_id!("ERoFVBGqdz2xsgSvuGqaZbjni4swEjo4ByKXHwpDDP5U");

#[program]
pub mod stakingcontract {
    use super::*;

    pub fn create_pda_account(ctx: Context<CreatePdaAccount>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreatePdaAccount<'info> {
    #[account(mut)]
    pub payer : Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = 8 + 32 + 4 + 8 + 1,
        seeds = [b"stake1",payer.key().as_ref()],
        bump
    )]

    pub pda_acc : Account<'info, StakeAccount>,

    pub system_program: Program<'info,System>
}

#[derive(Accounts)]
pub struct Stake <'info>{
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"stake1", payer.key().as_ref()],
        bump = pda_acc.bump,
        constraint = pda_acc.owner == payer.key() @ErrorMessages::Unautharized
    )]
    pub pda_acc : Account<'info, StakeAccount>,
    pub system_program: Program<'info,System>
}



#[account]
pub struct StakeAccount{
    owner:Pubkey,
    stakeamount: u32,
    points:u32,
    lasttimestamp:u64,
    bump:u8
}

#[error_code]
pub enum ErrorMessages {
    #[msg("amount must be greater than 0")]
    InvalidAmount,
    #[msg("Insufficient staked amount")]
    InsufficientStake,
    #[msg("Unautharized access")]
    Unautharized,
    #[msg("Arithmetic Overflow")]
    Overflow,
    #[msg("Arithmetic Underflow")]
    Underflow,
    #[msg("Invalid Timestamp")]
    InvalidTimestamp
}

