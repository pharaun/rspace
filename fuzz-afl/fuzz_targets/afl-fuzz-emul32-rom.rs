#[macro_use]
extern crate afl;
extern crate rspace;

// Build with: RUSTFLAGS="-Clink-arg=-fuse-ld=gold" otherwise link error
fn main() {
    fuzz!(|data: &[u8]| {
        let rom = {
            let mut rom: [u8; 4096] = [0; 4096];
            for i in 0..data.len() {
                rom[i] = data[i];
            }
            rom
        };

        // fuzzed code goes here
        let mut vm = rspace::vm::Emul32::new_with_rom(rom);

        // Can't infinite run, so do a thousand (?) steps then call it quits
        // exit early if Trap fires
        for _ in 0..1024 {
            match vm.step() {
                Ok(_)   => (),
                Err(_)  => break,
            }
        }
    });
}
