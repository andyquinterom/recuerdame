use recuerdame::{PrecalcConst, precalculate};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct MyColor {
    r: u8,
    g: u8,
    b: u8,
}

impl PrecalcConst for MyColor {
    const DEFAULT: Self = MyColor { r: 0, g: 0, b: 0 };
}

#[precalculate(val = 0..=2)]
const fn get_primary_color(val: u8) -> MyColor {
    match val {
        0 => MyColor { r: 255, g: 0, b: 0 },
        1 => MyColor { r: 0, g: 255, b: 0 },
        _ => MyColor { r: 0, g: 0, b: 255 },
    }
}

#[test]
fn using_custom_types_works() {
    assert_eq!(get_primary_color(0), MyColor { r: 255, g: 0, b: 0 });
    assert_eq!(get_primary_color(1), MyColor { r: 0, g: 255, b: 0 });
    assert_eq!(get_primary_color(2), MyColor { r: 0, g: 0, b: 255 });
}
