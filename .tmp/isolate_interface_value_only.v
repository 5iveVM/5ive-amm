interface ExampleProgram @program("11111111111111111111111111111111") {
    do_thing @discriminator(1) (arg: u64);
}

pub call_example(value: u64) {
    ExampleProgram.do_thing(value);
}
