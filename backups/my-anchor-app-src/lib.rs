use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{
    self, Mint, Token, TokenAccount, Transfer,
};

declare_id!("CVgxvZ7TwkyAuE8XY8ac35HkUNyAgBjgC5gyN2NA3TTA"); // we'll sync this later

#[program]
pub mod my_anchor_app {
    use super::*;

    /// One-time setup for a given mint:
    /// - creates a PDA "Vault" account (metadata)
    /// - creates the PDA's ATA to hold the mint's tokens
    pub fn initialize_vault(ctx: Context<InitializeVault>) -> Result<()> {
        // Nothing else to do; account initializations are handled by the macros
        Ok(())
    }

    /// Move `amount` of tokens from the user’s ATA into the vault’s ATA.
    /// The user must be the signer; Phantom will handle approval client-side.
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        // Perform a CPI to the SPL Token program to transfer from user to vault
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_token.to_account_info(),
            to: ctx.accounts.vault_token.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;
        Ok(())
    }
}

/// PDA that represents your vault config (one per mint).
#[account]
pub struct Vault {
    /// The authority who initialized (optional for future admin logic).
    pub authority: Pubkey,
    /// The mint this vault is for (e.g., USDC).
    pub mint: Pubkey,
    /// Bump for the PDA.
    pub bump: u8,
}

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    /// Payer/initializer (you, or an admin). Funds the account creations.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The mint we’re vaulting (pass USDC mint here).
    pub mint: Account<'info, Mint>,

    /// PDA that stores vault metadata.
    /// Seeds: ["vault", mint]
    #[account(
        init,
        payer = payer,
        space = 8 + 32 + 32 + 1,
        seeds = [b"vault", mint.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, Vault>,

    /// The vault’s token account (ATA) for `mint`, owned by the vault PDA.
    /// It will be auto-created if missing.
    #[account(
        init,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = vault
    )]
    pub vault_token: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    /// The user who is depositing (must sign).
    #[account(mut)]
    pub user: Signer<'info>,

    /// Same mint as used in InitializeVault.
    pub mint: Account<'info, Mint>,

    /// The vault PDA derived from the mint.
    /// We don’t mutate it here, so no `mut`.
    #[account(
        seeds = [b"vault", mint.key().as_ref()],
        bump = vault.bump
    )]
    pub vault: Account<'info, Vault>,

    /// The vault’s token account (ATA) for `mint`.
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = vault
    )]
    pub vault_token: Account<'info, TokenAccount>,

    /// The user’s token account holding `mint`.
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = user
    )]
    pub user_token: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
