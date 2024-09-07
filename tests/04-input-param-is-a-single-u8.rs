use constable::lookup;

#[lookup]
const fn foo(_x: u16) -> bool {
    false
}

fn main() {}
