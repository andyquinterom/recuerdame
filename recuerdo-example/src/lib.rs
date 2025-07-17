use recuerdo_macros::precalculate;

#[precalculate(a = 0..=10, b = (-1..=4))]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
