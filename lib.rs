use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount, MintTo, TransferChecked, transfer_checked};
use anchor_spl::token;

declare_id!("6ZFtauozJcpn6fSYwispi9toTvsSKkS1GEPv7a77SSvo");

#[program]
pub mod token_vesting {
    use super::*;

    pub fn init_token_vesting(ctx: Context<InitTokenVesting>, tokens_to_vest: u64, vest_start_time: i64, vest_end_time: i64, vest_cliff_period: i64) -> Result<()> {
        let vesting_account = &mut ctx.accounts.vesting_account;
        **vesting_account = TokenVestingInfo {
            total_tokens: tokens_to_vest,
            claimed_tokens: 0,
            start_time: vest_start_time,
            end_time: vest_end_time,
            cliff_period: vest_cliff_period,
        };
        msg!("Token Vesting Initialized Successfully!");
        msg!("Token Vesting Data : ");
        msg!("{:?}", vesting_account);

        msg!("Transfering tokens from mint account to token vault account...");

        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.token_vault_account.to_account_info(),
            authority: ctx.accounts.employeer.to_account_info(),
        };

        let cpi_context = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);

        token::mint_to(cpi_context, tokens_to_vest)?;

        msg!("Tokens successfully transfered from mint account to token vault account!");

        Ok(())
    }

    pub fn close_vesting_account(_ctx: Context<CloseVestingAccount>) -> Result<()> {
        msg!("Vesting account closed successfully!");
        msg!("The remaining amount is successfully transfered to the signer!");
        Ok(())
    }

    pub fn initialize_employee_account(_ctx: Context<InitializeEmployeeAccount>) -> Result<()> {
        msg!("Employee account initialized successfully!");
        Ok(())
    }

    pub fn claim_tokens(ctx: Context<ClaimTokens>) -> Result<()> {
        let vesting_account = &mut ctx.accounts.vesting_account;
        let current_time = Clock::get()?.unix_timestamp;

        require!(
            current_time >= vesting_account.start_time + vesting_account.cliff_period,
            VestingError::CliffPeriodNotReached
        );

        let elapsed_time = current_time.saturating_sub(vesting_account.start_time);
        let total_vesting_duration = vesting_account.end_time.saturating_sub(vesting_account.start_time);

        let vested_tokens = if current_time >= vesting_account.end_time {
            vesting_account.total_tokens 
        } else {
            (vesting_account.total_tokens as u128)
                .checked_mul(elapsed_time as u128)
                .unwrap_or(0)
                .checked_div(total_vesting_duration as u128)
                .unwrap_or(0) as u64
        };

        let claimable_tokens = vested_tokens.saturating_sub(vesting_account.claimed_tokens);

        require!(claimable_tokens > 0, VestingError::NoTokensToClaim);

        vesting_account.claimed_tokens += claimable_tokens;

        let cpi_accounts = TransferChecked {
            from: ctx.accounts.token_vault_account.to_account_info(),
            to: ctx.accounts.employee_token_account.to_account_info(),
            authority: ctx.accounts.employeer.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
        };

        let cpi_context = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        token::transfer_checked(cpi_context, claimable_tokens, vesting_account.total_tokens as u8)?;

        msg!("Successfully claimed {} tokens!", claimable_tokens);

        Ok(())
    }

}

#[derive(Accounts)]
pub struct InitTokenVesting<'info> {
    #[account(mut)]
    pub employeer: Signer<'info>,

    #[account(
        init,
        payer = employeer,
        space = 8 + TokenVestingInfo::INIT_SPACE,
        seeds = [b"token_vesting", employeer.key().as_ref()],
        bump
    )]
    pub vesting_account: Account<'info, TokenVestingInfo>,

    #[account(
        init,
        payer = employeer,
        seeds = [b"token_vault_account", employeer.key().as_ref()],
        bump,
        token::mint = mint,
        token::authority = employeer,
    )]
    pub token_vault_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct CloseVestingAccount<'info> {
    #[account(mut)]
    pub employeer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"token_vault_account", employeer.key().as_ref()],
        bump,
        close = employeer,
    )]
    pub token_vault_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(InitSpace, Debug)]
pub struct TokenVestingInfo {
    pub total_tokens: u64,
    pub claimed_tokens: u64,
    pub start_time: i64,
    pub end_time: i64,
    pub cliff_period: i64,
}

#[derive(Accounts)]
pub struct InitializeEmployeeAccount<'info> {
    #[account(mut)]
    pub employee: Signer<'info>,

    #[account(
        init,
        payer = employee,
        seeds = [b"employee_token_account", employee.key().as_ref()],
        bump,
        token::mint = mint,
        token::authority = employee,
    )]
    pub employee_token_account: Account<'info, TokenAccount>,

    pub mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ClaimTokens<'info> {
    #[account(mut)]
    pub employeer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"token_vesting", employeer.key().as_ref()],
        bump
    )]
    pub vesting_account: Account<'info, TokenVestingInfo>,
    #[account(
        mut,
        seeds = [b"token_vault_account", employeer.key().as_ref()],
        bump
    )]
    pub token_vault_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub employee_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
}

#[error_code]
pub enum VestingError {
    #[msg("The cliff period has not been reached yet.")]
    CliffPeriodNotReached,
    #[msg("No tokens are available to claim.")]
    NoTokensToClaim,
}
