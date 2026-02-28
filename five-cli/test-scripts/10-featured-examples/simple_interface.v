// Generic interface template example

// Declare an external program interface with discriminators
interface ExampleProgram @program("11111111111111111111111111111111") {
    do_thing @discriminator(1) (arg: u64);
}

// Demonstrate calling into the interface
pub call_example(target: account @signer, value: u64) {
    // In interfaces, account parameters are passed as pubkeys (account.key)
    ExampleProgram.do_thing(value);
}
