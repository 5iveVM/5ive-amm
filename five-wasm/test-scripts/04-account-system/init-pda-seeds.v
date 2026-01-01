    account PDAState {
        vault_count: u64;
        vault_bump: Option<u8>;
        token_bump: Option<u8>;
    }
    
create_user_vault(payer: account @signer, user: pubkey, seed: u64, vault: account @init, state: PDAState @mut) -> pubkey {
        // Find PDA and get bump seed for future validation
let (expected_vault, bump) = derive_pda(user, "vault", seed);
require(vault.key == expected_vault);
        
        // Store bump for future CU optimization
        state.vault_bump = Some(bump);
        state.vault_count = state.vault_count + 1;
        
        return vault.key;
    }
    
create_token_account(payer: account @signer, mint: pubkey, owner: pubkey, token_account: account @init, state: PDAState @mut) -> pubkey {
        // Create associated token account at PDA with bump tracking
let (expected_ata, bump) = derive_pda("token", mint, owner);
require(token_account.key == expected_ata);
        
        // Store bump for reuse
        state.token_bump = Some(bump);
        state.vault_count = state.vault_count + 1;
        
        return token_account.key;
    }
    
create_metadata_account(payer: account @signer, mint: pubkey, metadata: account @init) -> pubkey {
        // Create metadata account with bump-aware derivation
let (expected_metadata, bump) = derive_pda("metadata", mint);
require(metadata.key == expected_metadata);
        
        return metadata.key;
    }
    
    // Validate vault using stored bump (lower CU cost)
validate_vault_with_bump(user: pubkey, seed: u64, state: PDAState) -> Result<pubkey, string> {
        match state.vault_bump {
Some(bump) => {
                let vault_addr = derive_pda(user, "vault", seed, bump);
                return Ok(vault_addr);
            }
            None => return Err("Vault bump not stored")
        }
    }
    
    // Validate token using stored bump
validate_token_with_bump(mint: pubkey, owner: pubkey, state: PDAState) -> Result<pubkey, string> {
        match state.token_bump {
Some(bump) => {
                let token_addr = derive_pda("token", mint, owner, bump);
                return Ok(token_addr);
            }
            None => return Err("Token bump not stored")
        }
    }
    
get_vault_count(state: PDAState) -> u64 {
        return state.vault_count;
    }
    
get_vault_bump(state: PDAState) -> Option<u8> {
        return state.vault_bump;
    }
    
get_token_bump(state: PDAState) -> Option<u8> {
        return state.token_bump;
    }
