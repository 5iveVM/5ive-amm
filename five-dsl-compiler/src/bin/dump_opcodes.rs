use five_protocol::opcodes::*;

fn main() {
    println!("JUMP: {}", JUMP);
    println!("PUSH_U64: {}", PUSH_U64);
    println!("PUSH_ZERO: {}", PUSH_ZERO);
    println!("PUSH_ONE: {}", PUSH_ONE);
    println!("HALT: {}", HALT);
    println!("POP: {}", POP);
    println!("RETURN: {}", RETURN);
    println!("RETURN_VALUE: {}", RETURN_VALUE);
    println!("LOAD_PARAM (implied): check range");
}
