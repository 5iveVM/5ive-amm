// Localnet-stable anchor-style equivalent using System Program CPI transfer.
interface SystemProgram @program("11111111111111111111111111111111") {
    transfer @discriminator(2) (from: pubkey, to: pubkey, amount: u64);
}

pub increment_remote(
    counter: account @mut,
    user: account @signer @mut
) {
    SystemProgram.transfer(user, counter, 1);
}
