#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/01-parse.rs");
    t.compile_fail("tests/02-non-const-fn.rs");
    t.compile_fail("tests/03-non-bool-return-type.rs");
    t.compile_fail("tests/04-input-param-is-a-single-u8.rs");
}

#[constable::lookup]
const fn is_valid(packed: u8) -> bool {
    let u0 = packed & 0b11;
    let u1 = (packed >> 2) & 0b11;
    let u2 = (packed >> 4) & 0b11;
    let u3 = (packed >> 6) & 0b11;
    u0 != u1
        && u0 != u2
        && u1 != u2
        && (u0 != u3 && u1 != u3 && u2 != u3 || u3 == 0 && u0 != 3 && u1 != 3 && u2 != 3)
}

const fn is_valid2(packed: u8) -> bool {
    let u0 = packed & 0b11;
    let u1 = (packed >> 2) & 0b11;
    let u2 = (packed >> 4) & 0b11;
    let u3 = (packed >> 6) & 0b11;
    u0 != u1
        && u0 != u2
        && u1 != u2
        && (u0 != u3 && u1 != u3 && u2 != u3 || u3 == 0 && u0 != 3 && u1 != 3 && u2 != 3)
}

#[test]
fn all_valid() {
    for i in 0..=255 {
        assert_eq!(is_valid(i), is_valid2(i), "{i}");
    }
}
