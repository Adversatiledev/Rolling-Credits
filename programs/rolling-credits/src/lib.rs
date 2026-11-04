use anchor_lang::prelude::*;

declare_id!("EEXBANe3p9Y6CRZ1PENKkdxrEBHsGhYi3VoqxPHHFuEk");

#[program]
pub mod rolling_credits {
    use super::*;

    pub fn create_production(
        ctx: Context<CreateProduction>,
        slug: String,
        title: String,
    ) -> Result<()> {
        require!(slug.len() <= 32, CreditsError::SlugTooLong);
        require!(title.len() <= 64, CreditsError::TitleTooLong);

        let production = &mut ctx.accounts.production;
        production.owner = ctx.accounts.owner.key();
        production.slug = slug;
        production.title = title;
        production.credit_count = 0;
        production.created_at = Clock::get()?.unix_timestamp;
        production.bump = ctx.bumps.production;

        emit!(ProductionCreated {
            production: production.key(),
            owner: production.owner,
        });
        Ok(())
    }

    pub fn propose_credit(ctx: Context<ProposeCredit>, role: String) -> Result<()> {
        require!(!role.is_empty(), CreditsError::RoleEmpty);
        require!(role.len() <= 48, CreditsError::RoleTooLong);

        let credit = &mut ctx.accounts.credit;
        credit.production = ctx.accounts.production.key();
        credit.contributor = ctx.accounts.contributor.key();
        credit.role = role;
        credit.status = CreditStatus::Proposed;
        credit.proposed_at = Clock::get()?.unix_timestamp;
        credit.verified_at = 0;
        credit.owner_revoke = false;
        credit.contributor_revoke = false;
        credit.bump = ctx.bumps.credit;

        let production = &mut ctx.accounts.production;
        production.credit_count = production
            .credit_count
            .checked_add(1)
            .ok_or(CreditsError::Overflow)?;

        emit!(CreditProposed {
            production: credit.production,
            credit: credit.key(),
            contributor: credit.contributor,
        });
        Ok(())
    }

    pub fn accept_credit(ctx: Context<AcceptCredit>) -> Result<()> {
        let credit = &mut ctx.accounts.credit;
        require!(
            credit.status == CreditStatus::Proposed,
            CreditsError::NotProposed
        );

        credit.status = CreditStatus::Verified;
        credit.verified_at = Clock::get()?.unix_timestamp;

        emit!(CreditVerified {
            production: credit.production,
            credit: credit.key(),
            contributor: credit.contributor,
        });
        Ok(())
    }

    pub fn decline_credit(ctx: Context<DeclineCredit>) -> Result<()> {
        let credit = &mut ctx.accounts.credit;
        require!(
            credit.status == CreditStatus::Proposed,
            CreditsError::NotProposed
        );

        credit.status = CreditStatus::Declined;

        emit!(CreditDeclined {
            production: credit.production,
            credit: credit.key(),
            contributor: credit.contributor,
        });
        Ok(())
    }

    pub fn request_revoke(ctx: Context<ChangeRevoke>) -> Result<()> {
        let signer = ctx.accounts.signer.key();
        let owner = ctx.accounts.production.owner;
        let credit = &mut ctx.accounts.credit;
        require!(
            credit.status == CreditStatus::Verified,
            CreditsError::NotVerified
        );
        let is_owner = signer == owner;
        let is_contributor = signer == credit.contributor;
        require!(is_owner || is_contributor, CreditsError::Unauthorised);

        if is_owner {
            credit.owner_revoke = true;
        }
        if is_contributor {
            credit.contributor_revoke = true;
        }
        if credit.owner_revoke && credit.contributor_revoke {
            credit.status = CreditStatus::Revoked;
            emit!(CreditRevoked {
                production: credit.production,
                credit: credit.key(),
                contributor: credit.contributor,
            });
        }
        Ok(())
    }

    pub fn cancel_revoke(ctx: Context<ChangeRevoke>) -> Result<()> {
        let signer = ctx.accounts.signer.key();
        let owner = ctx.accounts.production.owner;
        let credit = &mut ctx.accounts.credit;
        require!(
            credit.status == CreditStatus::Verified,
            CreditsError::NotVerified
        );
        let is_owner = signer == owner;
        let is_contributor = signer == credit.contributor;
        require!(is_owner || is_contributor, CreditsError::Unauthorised);

        if is_owner {
            credit.owner_revoke = false;
        }
        if is_contributor {
            credit.contributor_revoke = false;
        }
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(slug: String)]
pub struct CreateProduction<'info> {
    #[account(
        init,
        payer = owner,
        space = 8 + Production::INIT_SPACE,
        seeds = [b"production", owner.key().as_ref(), slug.as_bytes()],
        bump
    )]
    pub production: Account<'info, Production>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ProposeCredit<'info> {
    #[account(mut, has_one = owner)]
    pub production: Account<'info, Production>,
    #[account(
        init,
        payer = owner,
        space = 8 + Credit::INIT_SPACE,
        seeds = [b"credit", production.key().as_ref(), contributor.key().as_ref()],
        bump
    )]
    pub credit: Account<'info, Credit>,
    /// CHECK: address only, the contributor signs later in accept_credit
    pub contributor: UncheckedAccount<'info>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AcceptCredit<'info> {
    #[account(mut, has_one = contributor)]
    pub credit: Account<'info, Credit>,
    pub contributor: Signer<'info>,
}

#[derive(Accounts)]
pub struct DeclineCredit<'info> {
    #[account(mut, has_one = contributor)]
    pub credit: Account<'info, Credit>,
    pub contributor: Signer<'info>,
}

#[derive(Accounts)]
pub struct ChangeRevoke<'info> {
    #[account(mut, has_one = production)]
    pub credit: Account<'info, Credit>,
    pub production: Account<'info, Production>,
    pub signer: Signer<'info>,
}

#[account]
#[derive(InitSpace)]
pub struct Production {
    pub owner: Pubkey,
    #[max_len(32)]
    pub slug: String,
    #[max_len(64)]
    pub title: String,
    pub credit_count: u32,
    pub created_at: i64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Credit {
    pub production: Pubkey,
    pub contributor: Pubkey,
    #[max_len(48)]
    pub role: String,
    pub status: CreditStatus,
    pub proposed_at: i64,
    pub verified_at: i64,
    pub owner_revoke: bool,
    pub contributor_revoke: bool,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum CreditStatus {
    Proposed,
    Verified,
    Declined,
    Revoked,
}

#[event]
pub struct ProductionCreated {
    pub production: Pubkey,
    pub owner: Pubkey,
}

#[event]
pub struct CreditProposed {
    pub production: Pubkey,
    pub credit: Pubkey,
    pub contributor: Pubkey,
}

#[event]
pub struct CreditVerified {
    pub production: Pubkey,
    pub credit: Pubkey,
    pub contributor: Pubkey,
}

#[event]
pub struct CreditDeclined {
    pub production: Pubkey,
    pub credit: Pubkey,
    pub contributor: Pubkey,
}

#[event]
pub struct CreditRevoked {
    pub production: Pubkey,
    pub credit: Pubkey,
    pub contributor: Pubkey,
}

#[error_code]
pub enum CreditsError {
    #[msg("Slug must be 32 characters or fewer")]
    SlugTooLong,
    #[msg("Title must be 64 characters or fewer")]
    TitleTooLong,
    #[msg("Role can't be empty")]
    RoleEmpty,
    #[msg("Role must be 48 characters or fewer")]
    RoleTooLong,
    #[msg("Only a proposed credit can be accepted or declined")]
    NotProposed,
    #[msg("Only a verified credit can be revoked")]
    NotVerified,
    #[msg("Only the production owner or the contributor can do this")]
    Unauthorised,
    #[msg("Counter overflow")]
    Overflow,
}
