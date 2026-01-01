// Vault template: lamport custody via System Program CPI

// System Program interface for lamports transfer
interface SystemProgram @program("11111111111111111111111111111112") {
    create_account @discriminator(0) (from: pubkey, to: pubkey, amount: u64, space: u64, owner: pubkey);
    assign @discriminator(1) (acc: pubkey, owner: pubkey);
    transfer_lamports @discriminator(2) (from: pubkey, to: pubkey, amount: u64);
}

account VaultState {
    balance: u64;
    authorized_user: pubkey;
}

// Initialize vault state (sets authority)
pub init_vault(state: VaultState @mut, authority: account @signer) {
    state.balance = 0;
    state.authorized_user = authority.key;
}

// Deposit lamports into the vault: transfer from payer to vault_account
// - payer: signer funding the deposit
// - vault_account: the on-chain account holding lamports for the vault
// Updates internal balance for accounting
pub deposit(state: VaultState @mut, payer: account @signer @mut, vault_account: account @mut, amount: u64) {
    require(amount > 0);
    SystemProgram.transfer_lamports(payer, vault_account, amount);
    state.balance = state.balance + amount;
}

// Withdraw lamports from the vault to a recipient (requires authority)
// - authority: must match configured authorized_user
// - vault_account: source of lamports (vault's account)
// - recipient: destination account to receive lamports
pub withdraw(state: VaultState @mut, authority: account @signer, vault_account: account @mut, recipient: account @mut, amount: u64) {
    require(state.authorized_user == authority.key);
    require(state.balance >= amount);
    SystemProgram.transfer_lamports(vault_account, recipient, amount);
    state.balance = state.balance - amount;
}
