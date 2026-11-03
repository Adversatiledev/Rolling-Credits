// Rolling Credits: splits program (devnet proof of concept, SOL only)
use anchor_lang::prelude::*;

declare_id!("EsYnYU2o5tzeWR78Z48jRgbAqWQHPubokY33ecBZctUN");

#[program]
pub mod rolling_splits {
    use super::*;

    pub fn create_split(ctx: Context<CreateSplit>, id: u32) -> Result<()> {
        let split = &mut ctx.accounts.split;
        split.authority = ctx.accounts.authority.key();
        split.id = id;
        split.recipient_count = 0;
        split.total_bps = 0;
        split.locked = false;
        split.total_claimed = 0;
        split.bump = ctx.bumps.split;
        Ok(())
    }

    pub fn add_recipient(ctx: Context<AddRecipient>, bps: u16) -> Result<()> {
        let split = &mut ctx.accounts.split;
        require!(!split.locked, SplitsError::AlreadyLocked);
        require!(bps > 0, SplitsError::ZeroShare);
        let new_total = split
            .total_bps
            .checked_add(bps)
            .ok_or(SplitsError::Overflow)?;
        require!(new_total <= 10_000, SplitsError::BpsExceeded);

        let recipient = &mut ctx.accounts.recipient;
        recipient.split = split.key();
        recipient.wallet = ctx.accounts.wallet.key();
        recipient.bps = bps;
        recipient.claimed = 0;
        recipient.bump = ctx.bumps.recipient;

        split.total_bps = new_total;
        split.recipient_count = split
            .recipient_count
            .checked_add(1)
            .ok_or(SplitsError::Overflow)?;
        Ok(())
    }

    pub fn lock(ctx: Context<LockSplit>) -> Result<()> {
        let split = &mut ctx.accounts.split;
        require!(!split.locked, SplitsError::AlreadyLocked);
        require!(split.total_bps == 10_000, SplitsError::BpsIncomplete);
        split.locked = true;
        Ok(())
    }

    pub fn claim(ctx: Context<ClaimShare>) -> Result<()> {
        let split = &mut ctx.accounts.split;
        require!(split.locked, SplitsError::NotLocked);

        let split_info = split.to_account_info();
        let rent_min = Rent::get()?.minimum_balance(split_info.data_len());
        let available = split_info.lamports().saturating_sub(rent_min);
        let total_received = (available as u128)
            .checked_add(split.total_claimed as u128)
            .ok_or(SplitsError::Overflow)?;

        let recipient = &mut ctx.accounts.recipient;
        let entitled = total_received
            .checked_mul(recipient.bps as u128)
            .ok_or(SplitsError::Overflow)?
            / 10_000u128;
        let owed = (entitled as u64)
            .checked_sub(recipient.claimed)
            .ok_or(SplitsError::Overflow)?;
        require!(owed > 0, SplitsError::NothingToClaim);

        let wallet_info = ctx.accounts.wallet.to_account_info();
        **split_info.try_borrow_mut_lamports()? -= owed;
        **wallet_info.try_borrow_mut_lamports()? += owed;

        recipient.claimed = recipient
            .claimed
            .checked_add(owed)
            .ok_or(SplitsError::Overflow)?;
        split.total_claimed = split
            .total_claimed
            .checked_add(owed)
            .ok_or(SplitsError::Overflow)?;
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(id: u32)]
pub struct CreateSplit<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + SplitConfig::INIT_SPACE,
        seeds = [b"split", authority.key().as_ref(), &id.to_le_bytes()],
        bump
    )]
    pub split: Account<'info, SplitConfig>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddRecipient<'info> {
    #[account(mut, has_one = authority)]
    pub split: Account<'info, SplitConfig>,
    #[account(
        init,
        payer = authority,
        space = 8 + Recipient::INIT_SPACE,
        seeds = [b"recipient", split.key().as_ref(), wallet.key().as_ref()],
        bump
    )]
    pub recipient: Account<'info, Recipient>,
    /// CHECK: address only, the recipient signs later in claim
    pub wallet: UncheckedAccount<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct LockSplit<'info> {
    #[account(mut, has_one = authority)]
    pub split: Account<'info, SplitConfig>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct ClaimShare<'info> {
    #[account(mut)]
    pub split: Account<'info, SplitConfig>,
    #[account(
        mut,
        seeds = [b"recipient", split.key().as_ref(), wallet.key().as_ref()],
        bump = recipient.bump,
        has_one = split
    )]
    pub recipient: Account<'info, Recipient>,
    #[account(mut)]
    pub wallet: Signer<'info>,
}

#[account]
#[derive(InitSpace)]
pub struct SplitConfig {
    pub authority: Pubkey,
    pub id: u32,
    pub recipient_count: u16,
    pub total_bps: u16,
    pub locked: bool,
    pub total_claimed: u64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Recipient {
    pub split: Pubkey,
    pub wallet: Pubkey,
    pub bps: u16,
    pub claimed: u64,
    pub bump: u8,
}

#[error_code]
pub enum SplitsError {
    #[msg("Shares are locked")]
    AlreadyLocked,
    #[msg("Split must be locked before claiming")]
    NotLocked,
    #[msg("Share must be above zero")]
    ZeroShare,
    #[msg("Shares can't add up to more than 10,000 basis points")]
    BpsExceeded,
    #[msg("Shares must add up to exactly 10,000 basis points to lock")]
    BpsIncomplete,
    #[msg("Nothing to claim yet")]
    NothingToClaim,
    #[msg("Arithmetic overflow")]
    Overflow,
}
