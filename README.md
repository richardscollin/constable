# Constable

Generate lookup tables at compile time using attribute macros on const functions.

## Example

```rust
#[constable::lookup]
const fn foo(packed: u8) -> bool {
   // divide an 8-bit integer into 4 2-bit values
   // return true if the xor of the first 2 is equal
   // to the xor of the second 2
   let u0 = packed & 0b11;
   let u1 = (packed >> 2) & 0b11;
   let u2 = (packed >> 4) & 0b11;
   let u3 = (packed >> 6) & 0b11;
   (u0 ^ u1) == (u2 ^ u3)
}

fn main() {
    let x = foo(5);
}
```

The code gets expanded to an inner function definition and building a const lookup table. (note the example below is simplified and not valid const code)

```rust
const fn foo(value: u8) -> bool {
  const fn foo_orig(value: u8) -> { .. }

  const LOOKUP_TABLE: [bool; 256] = const {
    let mut table = [false; 256];
    for i in 0..256 {
      table[i] = foo_orig(i);
    }
  };

  LOOKUP_TABLE[value]
}
```

Also in cases where the return value is a bool, the table is bit-packed so that the lookup table takes up less memory (and therefore more cache efficient).

```
$ cargo bench --bench bench
    Finished `bench` profile [optimized] target(s) in 0.03s
     Running benches/bench.rs (target/release/deps/bench-ad4b147a8cee6c74)
Timer precision: 12 ns
bench            fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ computed      1.037 µs      │ 10.51 µs      │ 1.202 µs      │ 1.656 µs      │ 100     │ 200
╰─ lookup_table  672.7 ns      │ 10.1 µs       │ 772.7 ns      │ 1.191 µs      │ 100     │ 200
```

Note that the fastest for computed and lookup_table are different units.
