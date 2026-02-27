// Generic local interface template example.
// Use dot-call syntax for locally declared interfaces.

interface ExampleProgram @program("11111111111111111111111111111111") @serializer(raw) {
    do_thing @discriminator_bytes([1]) (
        authority: account,
        value: u64
    );
}

pub call_example(authority: account @signer, value: u64) {
    ExampleProgram.do_thing(authority, value);
}
