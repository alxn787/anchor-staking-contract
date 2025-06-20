use anchor_lang::prelude::*;

declare_id!("ERoFVBGqdz2xsgSvuGqaZbjni4swEjo4ByKXHwpDDP5U");

#[program]
pub mod stakingcontract {
    use anchor_lang::system_program;

    use super::*;

    pub fn create_pda_account(ctx: Context<CreatePdaAccount>) -> Result<()> {
        let pda_acc = &mut ctx.accounts.pda_acc;
        pda_acc.owner = ctx.accounts.payer.key();
        pda_acc.bump = ctx.bumps.pda_acc;
        pda_acc.points = 0;
        pda_acc.stakeamount = 0;
        let clock = Clock::get()?;
        pda_acc.lasttimestamp = clock.unix_timestamp;
        msg!("Pda account created Succesfully");
        Ok(())
    }

    pub fn stake(ctx: Context<Stake>,amount:u64)->Result<()>{
        require!(amount>0, ErrorMessages::InvalidAmount);
        let pda_acc = &mut ctx.accounts.pda_acc;
        let clock = Clock::get()?;

        update_stake(pda_acc,clock);

        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer{
                from:ctx.accounts.payer.to_account_info(),
                to:ctx.accounts.pda_acc.to_account_info()
        });

        system_program::transfer(cpi_context, amount)?;
        pda_acc.stakeamount.checked_add(amount).ok_or(ErrorMessages::Overflow);
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
        space = 8 + 32 + 8 + 8 + 1,
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

#[derive(Accounts)]
pub struct Unstake<'info>{
    #[account(mut)]
    pub payer:Signer<'info>,
    #[account(
        mut,
        seeds = [b"stake1",payer.key().as_ref()],
        bump = pda_acc.bump,
        constraint = pda_acc.owner == payer.key() @ErrorMessages::Unautharized
    )]
    pub pda_acc: Account<'info,StakeAccount>,
    pub system_program: Program<'info,System>
}

#[derive(Accounts)]
pub struct ClaimPoints<'info>{
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"stake1",payer.key().as_ref()],
        bump = pda_acc.bump,
        constraint = pda_acc.owner == payer.key() @ErrorMessages::Unautharized
    )]
    pub pda_acc:Account<'info,StakeAccount>,
    pub  system_program: Program<'info,System>
}

#[derive(Accounts)]
pub struct GetPoints<'info> {
    pub user: Signer<'info>,
    
    #[account(
        seeds = [b"client1", user.key().as_ref()],
        bump = pda_account.bump,
        constraint = pda_account.owner == user.key() @ErrorMessages::Unautharized
    )]
    pub pda_account: Account<'info, StakeAccount>,
}



#[account]
pub struct StakeAccount{
    owner:Pubkey,
    stakeamount: u64,
    points:u64,
    lasttimestamp:i64,
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

