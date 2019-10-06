#[macro_use]
extern crate criterion;
extern crate rspace;

use byteorder::{LittleEndian, WriteBytesExt};

use criterion::Criterion;
use criterion::black_box;


fn setup_binary_code(test_asm: &str, rom: &mut [u8; 4096], full: bool) {
    let binary_code = {
        let asm = rspace::asm::parse_asm(test_asm);
        let mut wtr = vec![];

        for i in 0..asm.len() {
            let _ = wtr.write_u32::<LittleEndian>(asm[i]);
        }
        wtr
    };

    // Copy into rom
    if full {
        let len = binary_code.len();
        let times = 4096 / len;

        for i in 0..times {
            for j in 0..len {
                rom[i*len + j] = binary_code[j];
            }
        }
    } else {
        for i in 0..binary_code.len() {
            rom[i] = binary_code[i];
        }
    }
}


fn run_rom(rom: [u8; 4096]) {
    let mut vm = rspace::vm::Emul32::new_with_rom(rom);
    vm.run();
}


fn criterion_benchmark(c: &mut Criterion) {
    let test_asm = r#"
        addi x0 x0 1
        addi x1 x1 2
        slti x2 x0 0xA
        sltiu x3 x0 0x9
        andi x1 x0 0x0
        ori x2 x0 0xFF
        xori x3 x0 0x00FF
        slli x1 x1 0x0
        srli x2 x2 0x1
        srai x3 x3 0x6
        add x1 x3 x2
        slt x2 x2 x2
        sltu x3 x1 x2
        and x1 x3 x2
        or x2 x2 x2
        xor x3 x1 x2
        sll x1 x3 x2
        srl x3 x1 x2
        sub x1 x3 x2
        sra x3 x1 x2
        mul x0 x1 x2
        mulh x1 x2 x0
        mulhu x2 x0 x1
        mulhsu x0 x1 x2
        div x1 x2 x0
        divu x2 x0 x1
        rem x0 x1 x2
        remu x1 x2 x0
        rem t4 t5 t6
        lui x1 0x3412
    "#;
    let mut rom = [0; 4096];
    setup_binary_code(test_asm, &mut rom, false);

    c.bench_function("simple-inst", |b| b.iter(|| run_rom(black_box(rom)) ));

    let mut rom = [0; 4096];
    setup_binary_code(test_asm, &mut rom, true);

    c.bench_function("copy-inst", |b| b.iter(|| run_rom(black_box(rom)) ));

    // Single inst
    let test_asm = "addi x1 x1 2";
    let mut rom = [0; 4096];
    setup_binary_code(test_asm, &mut rom, false);

    c.bench_function("one-inst", |b| b.iter(|| run_rom(black_box(rom)) ));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
