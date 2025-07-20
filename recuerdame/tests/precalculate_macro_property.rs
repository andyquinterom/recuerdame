use quickcheck_macros::quickcheck;
use recuerdame::precalculate;

#[precalculate(a = -10..=10)]
const fn identity_1(a: i32) -> i32 {
    a
}

#[precalculate(a = -10..=10, b = -10..=10)]
const fn identity_2(a: i32, b: i32) -> (i32, i32) {
    (a, b)
}

#[precalculate(a = -10..=10, b = -10..=10, c = -10..=10)]
const fn identity_3(a: i32, b: i32, c: i32) -> (i32, i32, i32) {
    (a, b, c)
}

#[precalculate(a = -10..=10, b = -10..=10, c = -10..=10, d = -10..=10)]
const fn identity_4(a: i32, b: i32, c: i32, d: i32) -> (i32, i32, i32, i32) {
    (a, b, c, d)
}

#[quickcheck]
fn keep_function_never_fails_1_argument(a: i32) -> bool {
    identity_1(a) == a
}

#[quickcheck]
fn keep_function_never_fails_2_argument(a: i32, b: i32) -> bool {
    identity_2(a, b) == (a, b)
}

#[quickcheck]
fn keep_function_never_fails_3_argument(a: i32, b: i32, c: i32) -> bool {
    identity_3(a, b, c) == (a, b, c)
}

#[quickcheck]
fn keep_function_never_fails_4_argument(a: i32, b: i32, c: i32, d: i32) -> bool {
    identity_4(a, b, c, d) == (a, b, c, d)
}
