pub use recuerdame_macros::precalculate;

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
