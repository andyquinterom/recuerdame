# recuerdame

[![crates.io](https://img.shields.io/crates/v/recuerdame.svg)](https://crates.io/crates/recuerdame)
[![docs.rs](https://docs.rs/recuerdame/badge.svg)](https://docs.rs/recuerdame)

*(Recuérdame: Spanish for "Remember Me")*

`recuerdame` is a Rust procedural macro that provides compile-time function memoization. It transforms a `const fn` into a blazing-fast lookup table, pre-calculating all possible return values within specified input ranges.

This is ideal for computationally expensive functions with a small, discrete input domain, trading a larger binary size and longer compile times for zero-cost runtime performance.

## Table of Contents

- [What is `recuerdame`?](#what-is-recuerdame)
- [Usage](#usage)
- [How It Works](#how-it-works)
- [Supported Types](#supported-types)
  - [Argument Types](#argument-types)
  - [Return Types (`PrecalcConst` trait)](#return-types-precalcconst-trait)
- [Examples](#examples)
- [Use Cases](#use-cases)
- [Benchmarks](#benchmarks)
- [Limitations & Caveats](#limitations--caveats)
- [License](#license)

## What is `recuerdame`?

Imagine you have a `const` function that performs a complex calculation. If you call this function repeatedly with the same arguments, you're wasting cycles re-calculating the same result.

`recuerdame` solves this by taking your `const fn` and generating a static lookup table at compile time. The new function simply performs an array lookup to get the result instantly.

**The Trade-Off:**
- **Pro:** Extremely fast (O(1)) runtime performance for function calls.
- **Con:** Increased compile times.
- **Con:** Increased binary size, proportional to the size of the lookup table.

## Usage

1.  Add `recuerdame` to your `Cargo.toml`:

    ```toml
    [dependencies]
    recuerdame = "0.2.1"
    ```

2.  Annotate your `const fn` with `#[precalculate]`, specifying the inclusive range for each argument.

    ```rust
    use recuerdame::precalculate;

    #[precalculate(a = 0..=10, b = 0..=4)]
    pub const fn add(a: i32, b: i32) -> i32 {
        // In a real scenario, this would be an expensive calculation.
        a + b
    }

    fn main() {
        // This call is now a simple, fast lookup!
        let result = add(5, 2);
        assert_eq!(result, 7);

        // This will panic at runtime!
        // add(11, 0); // 11 is outside the specified range of 0..=10
    }
    ```

## How It Works

The `#[precalculate]` macro performs the following transformation at compile time:

1.  It creates a new, private module (e.g., `_mod_precalc_add`).
2.  It moves your original function into this module and renames it (e.g., `_add_original`).
3.  Inside the module, it generates a `const` multi-dimensional array that will serve as the lookup table.
4.  It generates a `const` function that populates this table by iterating through all possible input combinations and calling your original function.
5.  Finally, it creates a new `pub const fn` with the original name (`add`). This new function simply calculates the correct index from its arguments and returns the value from the lookup table.

This allows you to test the correctness of the macro by comparing the results:
`assert_eq!(add(a, b), _mod_precalc_add::_add_original(a, b));`

## Supported Types

### Argument Types
The function arguments must be integer types (`i8`, `u8`, `i16`, `u16`, `i32`, `u32`, `i64`, `u64`, `i128`, `u128`, `isize`, `usize`) for which a range can be defined. The ranges must be inclusive, using the `..=` syntax.

You can also use `const` values to define the ranges:
```rust
const MIN_A: i16 = 0;
const MAX_A: i16 = 100;

#[precalculate(a = MIN_A..=MAX_A)]
const fn my_func(a: i16) -> i32 {
    (a * a) as i32
}
```

### Return Types (`PrecalcConst` trait)
The function's return type must implement the `recuerdame::PrecalcConst` trait. This is required to provide a default value for initializing the lookup table array before it's populated.

`recuerdame` provides out-of-the-box implementations for:
- All integer primitives (defaults to `0`).
- `Option<T>` (defaults to `None`).

You can easily implement it for your own `const`-compatible types:
```rust
// Your custom struct needs to be usable in a const context.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct MyColor {
    r: u8,
    g: u8,
    b: u8,
}

impl recuerdame::PrecalcConst for MyColor {
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
```

## Examples

**Returning an `Option<T>`**
```rust
use recuerdame::precalculate;

#[precalculate(a = 0..=5)]
const fn get_even(a: u16) -> Option<u16> {
    if a % 2 == 0 { Some(a) } else { None }
}

assert_eq!(get_even(4), Some(4));
assert_eq!(get_even(5), None);
```

**Multiple signed arguments**
```rust
use recuerdame::precalculate;

#[precalculate(a = -50..=-1, b = -127..=-100)]
const fn subtract_i8(a: i8, b: i8) -> i8 {
    a.saturating_sub(b)
}

assert_eq!(subtract_i8(-10, -110), 100);
```

## Use Cases

`recuerdame` is most effective for:

- **Digital Signal Processing (DSP):** Pre-calculating sine waves, filter coefficients, or windowing functions.
- **Game Development:** Lookup tables for things like falloff curves, experience points, or complex physics calculations with discrete steps.
- **Embedded Systems:** When CPU cycles are precious and flash memory is available, replacing math-heavy functions with a lookup table can be a huge win.
- **Cryptography:** Pre-calculating S-boxes or other fixed tables.

## Benchmarks

The core promise of `recuerdame` is trading compile time for a significant boost in runtime performance. The benchmarks below illustrate this by comparing a function that calculates a logistic regression value versus its pre-calculated equivalent using `recuerdame`.

The benchmark results are as follows:

```
logistic regression (precalculated)
                        time:   [843.09 ps 844.05 ps 845.12 ps]

logistic regression (normal)
                        time:   [12.267 ns 12.272 ns 12.277 ns]
```

#### Analysis

- **Pre-calculated (with `recuerdame`):** The function call takes approximately **844 picoseconds**. This is effectively the cost of an array lookup.
- **Normal `const fn`:** The standard function call takes about **12.2 nanoseconds** to perform the standard calculation.

In this scenario, the `recuerdame`-powered function is over **14 times faster** than the original. This performance gap widens as the computational complexity of the target function increases, making it a powerful optimization for performance-critical code paths.

## Limitations & Caveats

- **Compile Time & Binary Size:** Be mindful of your input ranges. A function like `#[precalculate(a = 0..=1000, b = 0..=1000)]` would try to create a table with over a million entries, drastically increasing compile time and binary size.

- **Panics on Out-of-Range Inputs:** The generated function performs a direct array lookup. **It will panic at runtime** if you provide an argument that is outside the range you specified in the `#[precalculate]` attribute. This is by design for maximum performance, as it avoids runtime branching. Ensure your call sites respect the defined ranges.

- **`const fn` Required:** The macro can only be applied to functions marked as `const fn`.

- **Integer Arguments Required:** The function arguments must be integer primitives.

## License

This project is licensed under the [MIT License](LICENSE).
