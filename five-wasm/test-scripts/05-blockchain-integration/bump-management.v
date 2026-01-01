    account BumpState {
        vault_bump: Option<u8>;
        token_bump: Option<u8>;
        operations_count: u64;
    }
    
    // Find and store vault bump for first time (higher CU cost)
init_vault_bump(user: pubkey, seed: u64, state: BumpState @mut) -> Result<pubkey, string> {
let (vault_addr, bump) = derive_pda(user, "vault", seed);
        state.vault_bump = Some(bump);
        state.operations_count = state.operations_count + 1;
        return Ok(vault_addr);
    }
    
    // Reuse stored bump for validation (lower CU cost)
validate_vault_fast(user: pubkey, seed: u64, state: BumpState @mut) -> Result<pubkey, string> {
        match state.vault_bump {
Some(bump) => {
                let vault_addr = derive_pda(user, "vault", seed, bump);
                state.operations_count = state.operations_count + 1;
                return Ok(vault_addr);
            }
            None => return Err("Vault bump not initialized")
        }
    }
    
    // Initialize token account bump
init_token_bump(mint: pubkey, owner: pubkey, state: BumpState @mut) -> Result<pubkey, string> {
let (token_addr, bump) = derive_pda("token", mint, owner);
        state.token_bump = Some(bump);
        state.operations_count = state.operations_count + 1;
        return Ok(token_addr);
    }
    
    // Fast token validation with stored bump
validate_token_fast(mint: pubkey, owner: pubkey, state: BumpState @mut) -> Result<pubkey, string> {
        match state.token_bump {
Some(bump) => {
                let token_addr = derive_pda("token", mint, owner, bump);
                state.operations_count = state.operations_count + 1;
                return Ok(token_addr);
            }
            None => return Err("Token bump not initialized")
        }
    }
    
    // Get stored bumps
get_vault_bump(state: BumpState) -> Option<u8> {
        return state.vault_bump;
    }
    
get_token_bump(state: BumpState) -> Option<u8> {
        return state.token_bump;
    }
    
get_operations_count(state: BumpState) -> u64 {
        return state.operations_count;
    }
