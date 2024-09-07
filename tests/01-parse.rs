use constable::lookup;

#[lookup]
const fn foo(x: u8) -> bool {
    x & 1 == 1
}

fn main() {}
