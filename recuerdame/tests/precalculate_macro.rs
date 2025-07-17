use recuerdame::precalculate;

#[precalculate(a = 0..=10, b = 0..=4)]
const fn add(a: i32, b: i32) -> i32 {
    a + b
}

const MIN_A: i16 = 0;
const MAX_A: i16 = 100;

#[precalculate(a = MIN_A..=MAX_A, b = 0..=4, c = -10..=10)]
const fn add_3(a: i16, b: i16, c: i16) -> i32 {
    ((0.3 * a as f64) + (0.4 * b as f64) + (0.9 * c as f64)) as i32
}

#[precalculate(val = 0..=255)]
const fn identity_u8(val: u8) -> u8 {
    val
}

#[precalculate(a = -50..=-1, b = -127..=-100)]
const fn subtract_i8(a: i8, b: i8) -> i8 {
    a.saturating_sub(b)
}

#[precalculate(x = 0..=10, y = 1..=5)]
const fn multiply_usize(x: usize, y: usize) -> usize {
    x * y
}

#[precalculate(a = 5..=5, b = 0..=10)]
const fn single_value_range_test(a: i32, b: i32) -> i32 {
    a - b
}

#[precalculate(a = 0..=5)]
const fn return_option(a: u16) -> Option<u16> {
    if a % 2 == 0 { Some(a) } else { None }
}

const START: u32 = 10;
const END: u32 = 20;
#[precalculate(i = START..=END)]
const fn const_range_test(i: u32) -> u32 {
    i * i
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn equivalence_add() {
        (0..=10).for_each(|a| {
            (0..=4).for_each(|b| assert_eq!(add(a, b), _mod_precalc_add::_add_original(a, b)))
        });
    }

    #[test]
    fn equivalence_add_3() {
        (MIN_A..=MAX_A).for_each(|a| {
            (0..=4).for_each(|b| {
                (-10..=10).for_each(|c| {
                    assert_eq!(add_3(a, b, c), _mod_precalc_add_3::_add_3_original(a, b, c))
                })
            })
        });
    }

    #[test]
    fn equivalence_identity_u8() {
        (0..=255).for_each(|val| {
            assert_eq!(
                identity_u8(val),
                _mod_precalc_identity_u8::_identity_u8_original(val)
            )
        });
    }

    #[test]
    fn equivalence_subtract_i8() {
        (-50..=-1).for_each(|a| {
            (-127..=-100).for_each(|b| {
                assert_eq!(
                    subtract_i8(a, b),
                    _mod_precalc_subtract_i8::_subtract_i8_original(a, b)
                )
            })
        });
    }

    #[test]
    fn equivalence_multiply_usize() {
        (0..=10).for_each(|x| {
            (1..=5).for_each(|y| {
                assert_eq!(
                    multiply_usize(x, y),
                    _mod_precalc_multiply_usize::_multiply_usize_original(x, y)
                )
            })
        });
    }

    #[test]
    fn equivalence_single_value_range_test() {
        (5..=5).for_each(|a| {
            (0..=10).for_each(|b| {
                assert_eq!(
                    single_value_range_test(a, b),
                    _mod_precalc_single_value_range_test::_single_value_range_test_original(a, b)
                )
            })
        });
    }

    #[test]
    fn equivalence_return_option() {
        (0..=5).for_each(|a| {
            assert_eq!(
                return_option(a),
                _mod_precalc_return_option::_return_option_original(a)
            )
        });
    }

    #[test]
    fn equivalence_const_range_test() {
        (START..=END).for_each(|i| {
            assert_eq!(
                const_range_test(i),
                _mod_precalc_const_range_test::_const_range_test_original(i)
            )
        });
    }

    #[test]
    #[should_panic]
    fn add_should_panic_when_first_arg_is_out_of_bounds_upper() {
        add(11, 0);
    }

    #[test]
    #[should_panic]
    fn add_should_panic_when_first_arg_is_out_of_bounds_lower() {
        add(-1, 0);
    }

    #[test]
    #[should_panic]
    fn add_should_panic_when_second_arg_is_out_of_bounds_upper() {
        add(0, 5);
    }

    #[test]
    #[should_panic]
    fn add_should_panic_when_second_arg_is_out_of_bounds_lower() {
        add(0, -1);
    }

    #[test]
    #[should_panic]
    fn subtract_i8_should_panic_when_first_arg_is_out_of_bounds_upper() {
        subtract_i8(0, -120);
    }

    #[test]
    #[should_panic]
    fn subtract_i8_should_panic_when_first_arg_is_out_of_bounds_lower() {
        subtract_i8(-51, -120);
    }

    #[test]
    #[should_panic]
    fn subtract_i8_should_panic_when_second_arg_is_out_of_bounds_upper() {
        subtract_i8(-5, -99);
    }

    #[test]
    #[should_panic]
    fn subtract_i8_should_panic_when_second_arg_is_out_of_bounds_lower() {
        subtract_i8(-5, -128);
    }
}
