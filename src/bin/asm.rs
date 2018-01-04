extern crate rspace;

use rspace::asm;

fn main() {
    // TODO: ingest asm from file or stdin, emit to file or stdout the binary code
    let binary_code = rspace::asm::parse_asm("add x0 x0 x0");
}
