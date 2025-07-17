pub use recuerdame_macros::precalculate;

/// This trait is needed for the return types of precalculated functions.
/// This tells the crate how to pre-populate the look-up table at compile
/// time.
///
/// Example:
/// ```rust
// use recuerdame::{PrecalcConst, precalculate};
//
// #[derive(Debug, PartialEq, Eq, Clone, Copy)]
// struct MyColor {
//     r: u8,
//     g: u8,
//     b: u8,
// }
//
// impl PrecalcConst for MyColor {
//     const DEFAULT: Self = MyColor { r: 0, g: 0, b: 0 };
// }
//
// #[precalculate(val = 0..=2)]
// const fn get_primary_color(val: u8) -> MyColor {
//     match val {
//         0 => MyColor { r: 255, g: 0, b: 0 },
//         1 => MyColor { r: 0, g: 255, b: 0 },
//         _ => MyColor { r: 0, g: 0, b: 255 },
//     }
// }
//
// #[test]
// fn using_custom_types_works() {
//     assert_eq!(get_primary_color(0), MyColor { r: 255, g: 0, b: 0 });
//     assert_eq!(get_primary_color(1), MyColor { r: 0, g: 255, b: 0 });
//     assert_eq!(get_primary_color(2), MyColor { r: 0, g: 0, b: 255 });
// }
/// ```
pub trait PrecalcConst {
    const DEFAULT: Self;
}

impl<T> PrecalcConst for Option<T> {
    const DEFAULT: Self = None;
}

macro_rules! impl_precalc_const_int {
    ($int_ty:ty) => {
        impl PrecalcConst for $int_ty {
            const DEFAULT: Self = 0;
        }
    };
}

macro_rules! impl_precalc_const_float {
    ($f_ty:ty) => {
        impl PrecalcConst for $f_ty {
            const DEFAULT: Self = 0.0;
        }
    };
}

impl_precalc_const_int!(usize);

impl_precalc_const_int!(u8);
impl_precalc_const_int!(i8);

impl_precalc_const_int!(u16);
impl_precalc_const_int!(i16);

impl_precalc_const_int!(u32);
impl_precalc_const_int!(i32);

impl_precalc_const_int!(u64);
impl_precalc_const_int!(i64);

impl_precalc_const_int!(u128);
impl_precalc_const_int!(i128);

impl_precalc_const_float!(f32);
impl_precalc_const_float!(f64);
