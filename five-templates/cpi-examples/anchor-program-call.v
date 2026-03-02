// Localnet-stable anchor-style equivalent using System Program CPI transfer.
interface SystemProgram @program("11111111111111111111111111111111") {
    transfer @discriminator([0x02, 0x00, 0x00, 0x00]) (from: account, to: account, amount: u64);
}

pub increment_remote(
    counter: account @mut,
    user: account @signer @mut
) {
    SystemProgram.transfer(user, counter, 1);
}
