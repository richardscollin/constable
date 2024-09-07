use rand::Rng;
use rand::SeedableRng as _;
use std::hint::black_box;

use divan::main;

// divide an 8-bit integer into 4 2-bit values
// return true if the xor of the first 2 is equal
// to the xor of the second 2

#[constable::lookup]
const fn is_valid_table(packed: u8) -> bool {
    let u0 = packed & 0b11;
    let u1 = (packed >> 2) & 0b11;
    let u2 = (packed >> 4) & 0b11;
    let u3 = (packed >> 6) & 0b11;
    (u0 ^ u1) == (u2 ^ u3)
}

#[no_mangle]
const fn is_valid_computed(packed: u8) -> bool {
    let u0 = packed & 0b11;
    let u1 = (packed >> 2) & 0b11;
    let u2 = (packed >> 4) & 0b11;
    let u3 = (packed >> 6) & 0b11;
    (u0 ^ u1) == (u2 ^ u3)
}

#[divan::bench]
fn computed(bencher: divan::Bencher) {
    //let values = rand::
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let values: Vec<u8> = (0..1000).map(|_| rng.gen()).collect();

    bencher.bench(|| {
        for &e in values.iter() {
            black_box(is_valid_computed(black_box(e)));
        }
    })
}

#[divan::bench]
fn lookup_table(bencher: divan::Bencher) {
    //let values = rand::
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let values: Vec<u8> = (0..1000).map(|_| rng.gen()).collect();

    bencher.bench(|| {
        for &e in values.iter() {
            black_box(is_valid_table(black_box(e)));
        }
    })
}
