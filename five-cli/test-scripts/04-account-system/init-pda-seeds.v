    account PDAState {
        vault_bump: Option<u8>;
        token_bump: Option<u8>;
    }

pub create_user_vault(user: pubkey, seed: u64, vault: account @init, state: PDAState @mut) -> pubkey {
        let (expected_vault, bump) = derive_pda(user, "vault", seed);
        require(vault.key == expected_vault);
        state.vault_bump = Some(bump);
        return vault.key;
    }

pub create_token_account(mint: pubkey, owner: pubkey, token_account: account @init, state: PDAState @mut) -> pubkey {
        let (expected_ata, bump) = derive_pda("token", mint, owner);
        require(token_account.key == expected_ata);
        state.token_bump = Some(bump);
        return token_account.key;
    }
