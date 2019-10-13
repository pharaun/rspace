#[macro_use]
extern crate criterion;
extern crate rspace;

use crate::rspace::vm::mem::Mem;
use crate::rspace::vm::mem::MemIO;

use criterion::Criterion;
use criterion::black_box;


fn criterion_benchmark(c: &mut Criterion) {
    let mut mem_map: rspace::vm::mem::MemMap = Default::default();
    let id = mem_map.add(0x0, 0x1000, 0, rspace::vm::mem::MemMapAttr::RW);
    mem_map.add(0x1000, 0x1000, 0x1000, rspace::vm::mem::MemMapAttr::RW); // Slow path check
    mem_map.add(0x2000, 0x1000, 0x2000, rspace::vm::mem::MemMapAttr::RO);


    // Benchmark memory read/writes
    c.bench_function("mem-access-byte-read", |b| b.iter(|| {
        for i in 0..10 {
            mem_map.load_byte(black_box(i)).unwrap();
        }
    }));

    c.bench_function("mem-access-byte-write", |b| b.iter(|| {
        for i in 0..10 {
            mem_map.store_byte(black_box(i), black_box(0x10)).unwrap();
        }
    }));

    c.bench_function("mem-access-fast-half-read", |b| b.iter(|| {
        for i in 0..10 {
            mem_map.load_half(black_box(i)).unwrap();
        }
    }));

    c.bench_function("mem-access-slow-half-read", |b| b.iter(|| {
        for _ in 0..10 {
            mem_map.load_half(black_box(0xFFF)).unwrap();
        }
    }));

    c.bench_function("mem-access-fast-half-write", |b| b.iter(|| {
        for i in 0..10 {
            mem_map.store_half(black_box(i), black_box(0x1020)).unwrap();
        }
    }));

    c.bench_function("mem-access-slow-half-write", |b| b.iter(|| {
        for _ in 0..10 {
            mem_map.store_half(black_box(0xFFF), black_box(0x1020)).unwrap();
        }
    }));

    // Should be entirely within the region thus fast path
    c.bench_function("mem-access-fast-word-read", |b| b.iter(|| {
        for i in 0..10 {
            mem_map.load_word(black_box(i)).unwrap();
        }
    }));

    // Should be half in region, half out, thus fast half read
    c.bench_function("mem-access-slow-word-read", |b| b.iter(|| {
        for _ in 0..10 {
            mem_map.load_word(black_box(0xFFE)).unwrap();
        }
    }));

    // should be 1/4 in region 3/4 out, thus byte read
    c.bench_function("mem-access-slow-again-word-read", |b| b.iter(|| {
        for _ in 0..10 {
            mem_map.load_word(black_box(0xFFF)).unwrap();
        }
    }));

    c.bench_function("mem-access-fast-word-write", |b| b.iter(|| {
        for i in 0..10 {
            mem_map.store_word(black_box(i), black_box(0x10203040)).unwrap();
        }
    }));

    c.bench_function("mem-access-slow-word-write", |b| b.iter(|| {
        for _ in 0..10 {
            mem_map.store_word(black_box(0xFFE), black_box(0x10203040)).unwrap();
        }
    }));

    c.bench_function("mem-access-slow-again-word-write", |b| b.iter(|| {
        for _ in 0..10 {
            mem_map.store_word(black_box(0xFFF), black_box(0x10203040)).unwrap();
        }
    }));

    // Memory map block
    c.bench_function("mem-block-get", |b| b.iter(|| {
        for i in 0..10 {
            mem_map.get(black_box(id)).unwrap()[i];
        }
    }));

    c.bench_function("mem-block-get_mut", |b| b.iter(|| {
        for i in 0..10 {
            mem_map.get_mut(black_box(id)).unwrap()[i] = black_box(0x30 as u8);
        }
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
