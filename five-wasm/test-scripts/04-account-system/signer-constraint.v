    account SignerState {
        authorized_users: u64;
        last_signer: pubkey;
    }
    
authorize_user(authority: account @signer, state: SignerState @mut, user: pubkey) -> bool {
        // Only the authority can authorize new users
require(authority.key == state.last_signer);
        
        state.authorized_users = state.authorized_users + 1;
        state.last_signer = user;
        
        return true;
    }
    
secure_operation(caller: account @signer, amount: u64) -> u64 {
        // Caller must sign this transaction
require(amount <= 1000);
        
        return amount * 2;
    }
    
get_last_signer(state: SignerState) -> pubkey {
        return state.last_signer;
    }
