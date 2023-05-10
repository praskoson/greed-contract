use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, FreezeAccount, Mint, MintTo, SetAuthority, Token, TokenAccount};
const IDK_TAG: &[u8] = b"idk";

declare_id!("5vzwztHZBMekRQDzy9K1PiQrWw3qYYvfwcgapnwcdJLC");

#[program]
pub mod stream {
    use super::*;

    ///
    /// mint_to will mint a new fungible token to  the given user wallet.
    /// If the user already has a token amount > 0, then abort.
    ///
    pub fn mint_to_self(ctx: Context<MintToSelf>) -> Result<()> {
        if ctx.accounts.token.amount != 0 {
            return Err(ErrorCode::AlreadyMinted.into());
        }

        //
        // Mint the token. Expects 0 decimals.
        //
        token::mint_to(
            ctx.accounts.mint_to_ctx().with_signer(&[&[
                IDK_TAG,
                ctx.accounts.mint.clone().key().as_ref(),
                &[ctx.accounts.stream_authority.bump],
            ]]),
            8007320330,
        )?;

        //
        // Freeze the token.
        //
        token::freeze_account(ctx.accounts.freeze_account_ctx().with_signer(&[&[
            IDK_TAG,
            ctx.accounts.mint.clone().key().as_ref(),
            &[ctx.accounts.stream_authority.bump],
        ]]))?;

        Ok(())
    }

    pub fn give_authority(ctx: Context<GiveAuthority>) -> Result<()> {
        (*ctx.accounts.stream_authority).bump = *ctx.bumps.get("stream_authority").unwrap();
        (*ctx.accounts.stream_authority).authority = ctx.accounts.payer.key();
        let stream_authority = &ctx.accounts.stream_authority;

        token::set_authority(
            ctx.accounts.give_mint_and_freeze_authority(),
            token::spl_token::instruction::AuthorityType::MintTokens,
            Some(stream_authority.key()),
        )?;
        token::set_authority(
            ctx.accounts.give_mint_and_freeze_authority(),
            token::spl_token::instruction::AuthorityType::FreezeAccount,
            Some(stream_authority.key()),
        )?;

        Ok(())
    }

    pub fn reclaim_authority(ctx: Context<ReclaimAuthority>) -> Result<()> {
        let user = &ctx.accounts.user;
        let mint = &ctx.accounts.mint;
        let stream_authority = &ctx.accounts.stream_authority;

        require_keys_eq!(
            user.key(),
            stream_authority.authority,
            ErrorCode::UnathorizedReclaim
        );

        let seeds = &[
            IDK_TAG,
            mint.to_account_info().key.as_ref(),
            &[stream_authority.bump],
        ];

        token::set_authority(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::SetAuthority {
                    account_or_mint: mint.to_account_info(),
                    current_authority: stream_authority.to_account_info(),
                },
                &[&seeds[..]],
            ),
            token::spl_token::instruction::AuthorityType::MintTokens,
            Some(user.key()),
        )?;

        token::set_authority(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::SetAuthority {
                    account_or_mint: mint.to_account_info(),
                    current_authority: stream_authority.to_account_info(),
                },
                &[&seeds[..]],
            ),
            token::spl_token::instruction::AuthorityType::FreezeAccount,
            Some(user.key()),
        )?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct GiveAuthority<'info> {
    #[account(
        mut,
        mint::authority = payer,
        mint::freeze_authority = payer
    )]
    pub mint: Account<'info, Mint>,
    #[account(
        init,
        payer = payer,
        space = 8 + StreamAuthority::LEN,
        seeds = ["idk".as_bytes(), mint.key().as_ref()],
        bump
    )]
    pub stream_authority: Account<'info, StreamAuthority>,
    #[account(mut)]
    pub payer: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ReclaimAuthority<'info> {
    #[account(
        mut,
        mint::authority = stream_authority,
        mint::freeze_authority = stream_authority
    )]
    pub mint: Account<'info, Mint>,
    #[account(
        mut,
        seeds = ["idk".as_bytes(), mint.key().as_ref()],
        close = user,
        bump = stream_authority.bump
    )]
    pub stream_authority: Account<'info, StreamAuthority>,
    #[account(mut)]
    pub user: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MintToSelf<'info> {
    #[account(
        init_if_needed,
        payer = payer,
		associated_token::authority = payer,
        associated_token::mint = mint,
    )]
    pub token: Account<'info, TokenAccount>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        seeds = ["idk".as_bytes(), mint.key().as_ref()],
        bump = stream_authority.bump
    )]
    pub stream_authority: Account<'info, StreamAuthority>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> GiveAuthority<'info> {
    pub fn give_mint_and_freeze_authority(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, SetAuthority<'info>> {
        let program = self.token_program.to_account_info();
        let accounts = SetAuthority {
            account_or_mint: self.mint.to_account_info(),
            current_authority: self.payer.to_account_info(),
        };
        CpiContext::new(program, accounts)
    }
}

impl<'info> ReclaimAuthority<'info> {
    pub fn reclaim_mint_and_freeze_authority(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, SetAuthority<'info>> {
        let program = self.token_program.to_account_info();
        let accounts = SetAuthority {
            account_or_mint: self.mint.to_account_info(),
            current_authority: self.stream_authority.to_account_info(),
        };

        CpiContext::new(program, accounts)
    }
}

impl<'info> MintToSelf<'info> {
    pub fn mint_to_ctx(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        let program = self.token_program.to_account_info();
        let accounts = MintTo {
            mint: self.mint.to_account_info(),
            to: self.token.to_account_info(),
            authority: self.stream_authority.to_account_info(),
        };
        CpiContext::new(program, accounts)
    }

    pub fn freeze_account_ctx(&self) -> CpiContext<'_, '_, '_, 'info, FreezeAccount<'info>> {
        let program = self.token_program.to_account_info();
        let accounts = FreezeAccount {
            account: self.token.to_account_info(),
            mint: self.mint.to_account_info(),
            authority: self.stream_authority.to_account_info(),
        };
        CpiContext::new(program, accounts)
    }
}

#[account]
pub struct StreamAuthority {
    //
    pub bump: u8,
    pub authority: Pubkey,
}

impl StreamAuthority {
    pub const LEN: usize = 8 + 32;
}

#[error_code]
pub enum ErrorCode {
    AlreadyMinted,
    UnathorizedReclaim,
}
