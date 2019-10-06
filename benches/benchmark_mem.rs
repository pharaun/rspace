#[macro_use]
extern crate criterion;
extern crate rspace;

use crate::rspace::vm::mem::Mem;
use crate::rspace::vm::mem::MemIO;

use criterion::Criterion;
use criterion::black_box;


fn criterion_benchmark(c: &mut Criterion) {
    let mut mem_map = rspace::vm::mem::MemMap::new();
    let id = mem_map.add(0x0, 0x1000, 0, rspace::vm::mem::MemMapAttr::RW);
    mem_map.add(0x1000, 0x1000, 4096, rspace::vm::mem::MemMapAttr::RO);


    // Benchmark memory read/writes
    c.bench_function("mem-access-byte-rw", |b| b.iter(|| {
        mem_map.store_byte(black_box(1), black_box(0x10)).unwrap();
        mem_map.load_byte(black_box(1)).unwrap();
    }));

    c.bench_function("mem-access-half-rw", |b| b.iter(|| {
        mem_map.store_half(black_box(2), black_box(0x1020)).unwrap();
        mem_map.load_half(black_box(2)).unwrap();
    }));

    c.bench_function("mem-access-word-rw", |b| b.iter(|| {
        mem_map.store_half(black_box(4), black_box(0x10203040)).unwrap();
        mem_map.load_half(black_box(4)).unwrap();
    }));

    // Memory map block
    c.bench_function("mem-block-get", |b| b.iter(|| {
        mem_map.get(black_box(id)).unwrap()[10];
    }));

    c.bench_function("mem-block-get_mut", |b| b.iter(|| {
        mem_map.get_mut(black_box(id)).unwrap()[10] = black_box(0x30 as u8);
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
