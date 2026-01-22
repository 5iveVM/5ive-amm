use anchor_lang::prelude::*;

declare_id!("EXYTTMwHkRziMdQ1guGGrThxzX6dJDvhJBzz57JGKmsw");

/// Mint account - equivalent to Five DSL Mint
/// Size: 8 (discriminator) + 32 + 32 + 8 + 1 + 32 + 32 + 32 = 177 bytes
#[account]
#[derive(InitSpace)]
pub struct Mint {
    pub authority: Pubkey,
    pub freeze_authority: Pubkey,
    pub supply: u64,
    pub decimals: u8,
    #[max_len(32)]
    pub name: String,
    #[max_len(32)]
    pub symbol: String,
    #[max_len(32)]
    pub uri: String,
}

/// TokenAccount - equivalent to Five DSL TokenAccount
/// Size: 8 (discriminator) + 32 + 32 + 8 + 1 + 8 + 32 + 1 = 122 bytes
#[account]
#[derive(InitSpace)]
pub struct TokenAccount {
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub balance: u64,
    pub is_frozen: bool,
    pub delegated_amount: u64,
    pub delegate: Pubkey,
    pub initialized: bool,
}

#[program]
pub mod anchor_token_comparison {
    use super::*;

    /// Initialize a new mint account
    pub fn init_mint(
        ctx: Context<InitMint>,
        freeze_authority: Pubkey,
        decimals: u8,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<Pubkey> {
        require!(decimals <= 20, TokenError::InvalidDecimals);

        let mint = &mut ctx.accounts.mint_account;
        mint.authority = ctx.accounts.authority.key();
        mint.freeze_authority = freeze_authority;
        mint.supply = 0;
        mint.decimals = decimals;
        mint.name = name;
        mint.symbol = symbol;
        mint.uri = uri;

        Ok(mint.key())
    }

    /// Initialize a new token account
    pub fn init_token_account(
        ctx: Context<InitTokenAccount>,
        mint: Pubkey,
    ) -> Result<Pubkey> {
        let token_account = &mut ctx.accounts.token_account;
        token_account.owner = ctx.accounts.owner.key();
        token_account.mint = mint;
        token_account.balance = 0;
        token_account.is_frozen = false;
        token_account.delegated_amount = 0;
        token_account.delegate = Pubkey::default();
        token_account.initialized = true;

        Ok(token_account.key())
    }

    /// Mint tokens to a destination account
    pub fn mint_to(ctx: Context<MintTo>, amount: u64) -> Result<()> {
        let mint_state = &mut ctx.accounts.mint_state;
        let destination = &mut ctx.accounts.destination_account;

        require!(
            mint_state.authority == ctx.accounts.mint_authority.key(),
            TokenError::InvalidAuthority
        );
        require!(
            destination.mint == mint_state.key(),
            TokenError::MintMismatch
        );
        require!(!destination.is_frozen, TokenError::AccountFrozen);
        require!(amount > 0, TokenError::InvalidAmount);

        mint_state.supply = mint_state.supply.checked_add(amount).unwrap();
        destination.balance = destination.balance.checked_add(amount).unwrap();

        Ok(())
    }

    /// Transfer tokens between accounts
    pub fn transfer(ctx: Context<Transfer>, amount: u64) -> Result<()> {
        let source = &mut ctx.accounts.source_account;
        let destination = &mut ctx.accounts.destination_account;

        require!(
            source.owner == ctx.accounts.owner.key(),
            TokenError::InvalidOwner
        );
        require!(source.balance >= amount, TokenError::InsufficientBalance);
        require!(source.mint == destination.mint, TokenError::MintMismatch);
        require!(!source.is_frozen, TokenError::AccountFrozen);
        require!(!destination.is_frozen, TokenError::AccountFrozen);
        require!(amount > 0, TokenError::InvalidAmount);

        source.balance = source.balance.checked_sub(amount).unwrap();
        destination.balance = destination.balance.checked_add(amount).unwrap();

        Ok(())
    }

    /// Transfer tokens using delegate authority
    pub fn transfer_from(ctx: Context<TransferFrom>, amount: u64) -> Result<()> {
        let source = &mut ctx.accounts.source_account;
        let destination = &mut ctx.accounts.destination_account;
        let authority_key = ctx.accounts.authority.key();

        let is_owner = source.owner == authority_key;

        if !is_owner {
            require!(
                source.delegate == authority_key,
                TokenError::InvalidDelegate
            );
            require!(
                source.delegated_amount >= amount,
                TokenError::InsufficientDelegatedAmount
            );
        }

        require!(source.balance >= amount, TokenError::InsufficientBalance);
        require!(source.mint == destination.mint, TokenError::MintMismatch);
        require!(!source.is_frozen, TokenError::AccountFrozen);
        require!(!destination.is_frozen, TokenError::AccountFrozen);
        require!(amount > 0, TokenError::InvalidAmount);

        if !is_owner {
            source.delegated_amount = source.delegated_amount.checked_sub(amount).unwrap();
        }

        source.balance = source.balance.checked_sub(amount).unwrap();
        destination.balance = destination.balance.checked_add(amount).unwrap();

        Ok(())
    }

    /// Approve a delegate to transfer tokens
    pub fn approve(ctx: Context<Approve>, delegate: Pubkey, amount: u64) -> Result<()> {
        let source = &mut ctx.accounts.source_account;

        require!(
            source.owner == ctx.accounts.owner.key(),
            TokenError::InvalidOwner
        );

        source.delegate = delegate;
        source.delegated_amount = amount;

        Ok(())
    }

    /// Revoke delegate authority
    pub fn revoke(ctx: Context<Revoke>) -> Result<()> {
        let source = &mut ctx.accounts.source_account;

        require!(
            source.owner == ctx.accounts.owner.key(),
            TokenError::InvalidOwner
        );

        source.delegate = Pubkey::default();
        source.delegated_amount = 0;

        Ok(())
    }

    /// Burn tokens from an account
    pub fn burn(ctx: Context<Burn>, amount: u64) -> Result<()> {
        let mint_state = &mut ctx.accounts.mint_state;
        let source = &mut ctx.accounts.source_account;

        require!(
            source.owner == ctx.accounts.owner.key(),
            TokenError::InvalidOwner
        );
        require!(source.balance >= amount, TokenError::InsufficientBalance);
        require!(source.mint == mint_state.key(), TokenError::MintMismatch);
        require!(!source.is_frozen, TokenError::AccountFrozen);
        require!(amount > 0, TokenError::InvalidAmount);

        mint_state.supply = mint_state.supply.checked_sub(amount).unwrap();
        source.balance = source.balance.checked_sub(amount).unwrap();

        Ok(())
    }

    /// Freeze a token account
    pub fn freeze_account(ctx: Context<FreezeAccount>) -> Result<()> {
        let mint_state = &ctx.accounts.mint_state;
        let account_to_freeze = &mut ctx.accounts.account_to_freeze;

        require!(
            mint_state.freeze_authority == ctx.accounts.freeze_authority.key(),
            TokenError::InvalidFreezeAuthority
        );
        require!(
            account_to_freeze.mint == mint_state.key(),
            TokenError::MintMismatch
        );

        account_to_freeze.is_frozen = true;

        Ok(())
    }

    /// Thaw a frozen token account
    pub fn thaw_account(ctx: Context<ThawAccount>) -> Result<()> {
        let mint_state = &ctx.accounts.mint_state;
        let account_to_thaw = &mut ctx.accounts.account_to_thaw;

        require!(
            mint_state.freeze_authority == ctx.accounts.freeze_authority.key(),
            TokenError::InvalidFreezeAuthority
        );
        require!(
            account_to_thaw.mint == mint_state.key(),
            TokenError::MintMismatch
        );

        account_to_thaw.is_frozen = false;

        Ok(())
    }

    /// Set a new mint authority
    pub fn set_mint_authority(
        ctx: Context<SetMintAuthority>,
        new_authority: Pubkey,
    ) -> Result<()> {
        let mint_state = &mut ctx.accounts.mint_state;

        require!(
            mint_state.authority == ctx.accounts.current_authority.key(),
            TokenError::InvalidAuthority
        );

        mint_state.authority = new_authority;

        Ok(())
    }

    /// Set a new freeze authority
    pub fn set_freeze_authority(
        ctx: Context<SetFreezeAuthority>,
        new_freeze_authority: Pubkey,
    ) -> Result<()> {
        let mint_state = &mut ctx.accounts.mint_state;

        require!(
            mint_state.freeze_authority == ctx.accounts.current_freeze_authority.key(),
            TokenError::InvalidFreezeAuthority
        );

        mint_state.freeze_authority = new_freeze_authority;

        Ok(())
    }

    /// Disable minting permanently
    pub fn disable_mint(ctx: Context<DisableMint>) -> Result<()> {
        let mint_state = &mut ctx.accounts.mint_state;

        require!(
            mint_state.authority == ctx.accounts.current_authority.key(),
            TokenError::InvalidAuthority
        );

        mint_state.authority = Pubkey::default();

        Ok(())
    }

    /// Disable freeze authority permanently
    pub fn disable_freeze(ctx: Context<DisableFreeze>) -> Result<()> {
        let mint_state = &mut ctx.accounts.mint_state;

        require!(
            mint_state.freeze_authority == ctx.accounts.current_freeze_authority.key(),
            TokenError::InvalidFreezeAuthority
        );

        mint_state.freeze_authority = Pubkey::default();

        Ok(())
    }
}

// ============================================================================
// Account Contexts
// ============================================================================

#[derive(Accounts)]
pub struct InitMint<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + Mint::INIT_SPACE
    )]
    pub mint_account: Account<'info, Mint>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitTokenAccount<'info> {
    #[account(
        init,
        payer = owner,
        space = 8 + TokenAccount::INIT_SPACE
    )]
    pub token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MintTo<'info> {
    #[account(mut)]
    pub mint_state: Account<'info, Mint>,
    #[account(mut)]
    pub destination_account: Account<'info, TokenAccount>,
    pub mint_authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct Transfer<'info> {
    #[account(mut)]
    pub source_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub destination_account: Account<'info, TokenAccount>,
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct TransferFrom<'info> {
    #[account(mut)]
    pub source_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub destination_account: Account<'info, TokenAccount>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct Approve<'info> {
    #[account(mut)]
    pub source_account: Account<'info, TokenAccount>,
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct Revoke<'info> {
    #[account(mut)]
    pub source_account: Account<'info, TokenAccount>,
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct Burn<'info> {
    #[account(mut)]
    pub mint_state: Account<'info, Mint>,
    #[account(mut)]
    pub source_account: Account<'info, TokenAccount>,
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct FreezeAccount<'info> {
    pub mint_state: Account<'info, Mint>,
    #[account(mut)]
    pub account_to_freeze: Account<'info, TokenAccount>,
    pub freeze_authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct ThawAccount<'info> {
    pub mint_state: Account<'info, Mint>,
    #[account(mut)]
    pub account_to_thaw: Account<'info, TokenAccount>,
    pub freeze_authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct SetMintAuthority<'info> {
    #[account(mut)]
    pub mint_state: Account<'info, Mint>,
    pub current_authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct SetFreezeAuthority<'info> {
    #[account(mut)]
    pub mint_state: Account<'info, Mint>,
    pub current_freeze_authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct DisableMint<'info> {
    #[account(mut)]
    pub mint_state: Account<'info, Mint>,
    pub current_authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct DisableFreeze<'info> {
    #[account(mut)]
    pub mint_state: Account<'info, Mint>,
    pub current_freeze_authority: Signer<'info>,
}

// ============================================================================
// Error Types
// ============================================================================

#[error_code]
pub enum TokenError {
    #[msg("Invalid decimals: must be <= 20")]
    InvalidDecimals,
    #[msg("Invalid authority")]
    InvalidAuthority,
    #[msg("Invalid owner")]
    InvalidOwner,
    #[msg("Invalid delegate")]
    InvalidDelegate,
    #[msg("Invalid freeze authority")]
    InvalidFreezeAuthority,
    #[msg("Mint mismatch")]
    MintMismatch,
    #[msg("Account is frozen")]
    AccountFrozen,
    #[msg("Insufficient balance")]
    InsufficientBalance,
    #[msg("Insufficient delegated amount")]
    InsufficientDelegatedAmount,
    #[msg("Invalid amount: must be > 0")]
    InvalidAmount,
}
