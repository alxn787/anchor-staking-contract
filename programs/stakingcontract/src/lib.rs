use anchor_lang::prelude::*;

declare_id!("ERoFVBGqdz2xsgSvuGqaZbjni4swEjo4ByKXHwpDDP5U");

#[program]
pub mod stakingcontract {
    use anchor_lang::{solana_program::clock, system_program};

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

        update_points(pda_acc,clock.unix_timestamp);

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

    pub fn unstake(ctx: Context<Unstake>, amount:u64)->Result<()>{
        require!(amount>0, ErrorMessages::InvalidAmount);
        let pda_acc = ctx.accounts.pda_acc;
        let clock = Clock::get()?;
        require!(pda_acc.stakeamount >amount, ErrorMessages::InsufficientStake);

        update_points(pda_acc,clock.unix_timestamp);
        let seeds = &[
        b"client1",
        ctx.accounts.payer.key().as_ref(),
        &[pda_acc.bump],
        ];
        let signer = &[&seeds[..]];
        
        let cpi_context = CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),

            system_program::Transfer{
                from:ctx.accounts.pda_acc.to_account_info(),
                to:ctx.accounts.payer.to_account_info()
            }, signer
        );

        system_program::transfer(cpi_context, amount)?;

        pda_acc.stakeamount = pda_acc.stakeamount.checked_sub(amount).ok_or(ErrorMessages::Underflow)?;
        
        Ok(())
    }

}

pub fn update_points(pda_acc: &mut StakeAccount, current_time:i64)->Result<()>{
    let time_elapsed = current_time.checked_sub(pda_acc.lasttimestamp).ok_or(ErrorMessages::InvalidTimestamp)? as u64;
    if time_elapsed>0 && pda_acc.stakeamount>0{
        let new_points = calculate_points_earned(pda_acc.stakeamount, time_elapsed)?;
        pda_acc.points = pda_acc.points.checked_add(new_points).ok_or(ErrorMessages::Overflow)?
    }
Ok(())
}

fn calculate_points_earned(staked_amount: u64, time_elapsed_seconds: u64) -> Result<u64> {
    // Points = (staked_amount_in_sol * time_in_days * points_per_sol_per_day)
    // Using micro-points for precision to avoid floating point
    let points = (staked_amount as u128)
        .checked_mul(time_elapsed_seconds as u128)
        .ok_or(StakeError::Overflow)?
        .checked_mul(POINTS_PER_SOL_PER_DAY as u128)
        .ok_or(StakeError::Overflow)?
        .checked_div(LAMPORTS_PER_SOL as u128)
        .ok_or(StakeError::Overflow)?
        .checked_div(SECONDS_PER_DAY as u128)
        .ok_or(StakeError::Overflow)?;
    
    Ok(points as u64)
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

