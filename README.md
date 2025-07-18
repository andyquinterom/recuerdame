# recuerdame

[![crates.io](https://img.shields.io/crates/v/recuerdame.svg)](https://crates.io/crates/recuerdame)
[![docs.rs](https://docs.rs/recuerdame/badge.svg)](https://docs.rs/recuerdame)

*(Recuérdame: Spanish for "Remember Me")*

`recuerdame` is a Rust procedural macro that provides compile-time function memoization. It transforms a `const fn` into a blazing-fast lookup table, pre-calculating all possible return values within specified input ranges.

This is ideal for computationally expensive functions with a small, discrete input domain, trading a larger binary size and longer compile times for zero-cost runtime performance.

## Table of Contents

- [What is `recuerdame`?](#what-is-recuerdame)
- [Usage & Operating Modes](#usage--operating-modes)
  - [Default Mode (Panic)](#default-mode-panic)
  - [`option` Mode](#option-mode)
  - [`keep` Mode](#keep-mode)
- [How It Works](#how-it-works)
- [Supported Types](#supported-types)
  - [Argument Types](#argument-types)
  - [Return Types (`PrecalcConst` trait)](#return-types-precalcconst-trait)
- [Examples](#examples)
  - [Comparing Modes](#comparing-modes)
  - [Using Custom Types](#using-custom-types)
- [Use Cases](#use-cases)
- [Benchmarks](#benchmarks)
- [Limitations & Caveats](#limitations--caveats)
- [License](#license)

## What is `recuerdame`?

Imagine you have a `const` function that performs a complex calculation. If you call this function repeatedly with the same arguments, you're wasting cycles re-calculating the same result.

`recuerdame` solves this by taking your `const fn` and generating a static lookup table at compile time. At runtime, the new function simply performs an array lookup to get the result instantly.

**The Trade-Off:**
- **Pro:** Extremely fast (O(1)) runtime performance for in-range function calls.
- **Con:** Increased compile times.
- **Con:** Increased binary size, proportional to the size of the lookup table.

## Usage & Operating Modes

1.  Add `recuerdame` to your `Cargo.toml`:

    ```bash
    cargo add recuerdame
    ```

2.  Annotate your `const fn` with `#[precalculate]` and choose an operating mode. The macro gives you three ways to handle inputs that are outside the pre-calculated range.

### Default Mode (Panic)

This is the fastest mode. If an input is outside the specified range, it panics. Use this when you can guarantee at the call site that inputs will always be in range.

```rust
use recuerdame::precalculate;

#[precalculate(a = 0..=10, b = 0..=4)]
pub const fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Works:
assert_eq!(add(5, 2), 7);
// Panics: 11 is outside the specified range of 0..=10
// add(11, 0);
```

### `option` Mode

This mode wraps the function's return type in an `Option`. If the inputs are within the pre-calculated range, it returns `Some(value)`. If they are out of range, it returns `None`. This adds a small runtime cost for the bounds check.

```rust
use recuerdame::precalculate;

#[precalculate(a = 0..=10, b = 0..=4, option)]
pub const fn add_option(a: i32, b: i32) -> i32 {
    a + b
}

// Works:
assert_eq!(add_option(5, 2), Some(7));
// Returns None:
assert_eq!(add_option(11, 0), None);
```

### `keep` Mode

This "fallback" mode keeps the original function alongside the lookup table. If the inputs are in range, it uses the fast lookup table. If they are out of range, it calls the original function to compute the result on the fly. This is useful when you want fast lookups for a common "hot path" but still need to handle all other cases. This also adds a small runtime cost for the bounds check.

```rust
use recuerdame::precalculate;

#[precalculate(a = 0..=10, b = 0..=4, keep)]
pub const fn add_keep(a: i32, b: i32) -> i32 {
    a + b
}

// In-range uses the lookup table:
assert_eq!(add_keep(5, 2), 7);
// Out-of-range calls the original function:
assert_eq!(add_keep(11, 0), 11);
```

## How It Works

The `#[precalculate]` macro performs the following transformation at compile time:

1.  It creates a new, private module (e.g., `_mod_precalc_add`).
2.  It moves your original function into this module and renames it (e.g., `_add_original`).
3.  Inside the module, it generates a `const` multi-dimensional array that will serve as the lookup table.
4.  It generates a `const` function that populates this table by iterating through all possible input combinations and calling your original function.
5.  Finally, it creates a new `pub const fn` with the original name (`add`). Depending on the mode, this new function either looks up the value directly, performs a bounds check before looking up, or falls back to the original function.

This allows you to test the correctness of the macro by comparing the results:
`assert_eq!(add(a, b), _mod_precalc_add::_add_original(a, b));`

## Supported Types

### Argument Types
The function arguments must be integer types (`i8`, `u8`, `i16`, `u16`, `i32`, `u32`, `i64`, `u64`, `i128`, `u128`, `isize`, `usize`) for which a range can be defined. The ranges must be inclusive, using the `..=` syntax.

You can also use `const` values to define the ranges:

```rust
use recuerdame::precalculate;

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
- All integer and float primitives (defaults to `0` or `0.0`).
- Tuples of types that implement `PrecalcConst`.
- `Option<T>` (defaults to `None`).

You can easily implement it for your own `const`-compatible types:

```rust
use recuerdame::PrecalcConst;

// Your custom struct needs to be usable in a const context.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct MyColor {
    r: u8,
    g: u8,
    b: u8,
}

impl PrecalcConst for MyColor {
    const DEFAULT: Self = MyColor { r: 0, g: 0, b: 0 };
}
```

## Examples

### Comparing Modes

Here is a side-by-side comparison of how each mode behaves.

```rust
use recuerdame::precalculate;

// 1. Default (Panic) Mode
#[precalculate(a = -10..=10)]
const fn identity(a: i32) -> i32 { a }

// 2. Option Mode
#[precalculate(a = -10..=10, option)]
const fn identity_opt(a: i32) -> i32 { a }

// 3. Keep Mode
#[precalculate(a = -10..=10, keep)]
const fn identity_keep(a: i32) -> i32 { a }

fn main() {
    // In-range behavior is the same (except for Option's wrapper)
    assert_eq!(identity(5), 5);
    assert_eq!(identity_opt(5), Some(5));
    assert_eq!(identity_keep(5), 5);

    // Out-of-range behavior differs
    // identity(20) would panic!
    assert_eq!(identity_opt(20), None);
    assert_eq!(identity_keep(20), 20); // falls back to original function
}
```

### Using Custom Types

This example uses the custom `MyColor` struct defined in the [Return Types](#return-types-precalcconst-trait) section.

```rust
use recuerdame::{precalculate, PrecalcConst};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct MyColor { r: u8, g: u8, b: u8 }

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

// The lookup works perfectly with custom types
assert_eq!(get_primary_color(0), MyColor { r: 255, g: 0, b: 0 });
```

## Use Cases

`recuerdame` is most effective for:

- **Digital Signal Processing (DSP):** Pre-calculating sine waves, filter coefficients, or windowing functions.
- **Game Development:** Lookup tables for things like falloff curves, experience points, or complex physics calculations with discrete steps.
- **Embedded Systems:** When CPU cycles are precious and flash memory is available, replacing math-heavy functions with a lookup table can be a huge win.
- **Cryptography:** Pre-calculating S-boxes or other fixed tables.

## Benchmarks

The core promise of `recuerdame` is trading compile time for a significant boost in runtime performance. The benchmarks below illustrate this by comparing a function that calculates a logistic regression value versus its pre-calculated equivalent. The benchmark measures an in-range lookup.

```
logistic regression (precalculated)
                        time:   [843.09 ps 844.05 ps 845.12 ps]

logistic regression (normal)
                        time:   [12.267 ns 12.272 ns 12.277 ns]
```

#### Analysis

- **Pre-calculated (with `recuerdame`):** The function call takes approximately **844 picoseconds**. This is effectively the cost of an array lookup.
- **Normal `const fn`:** The standard function call takes about **12.2 nanoseconds** to perform the standard calculation.

In this scenario, the `recuerdame`-powered function is over **14 times faster** than the original. This performance gap widens as the computational complexity of the target function increases.

## Limitations & Caveats

- **Handling Out-of-Range Inputs:** Choose your operating mode carefully. The default mode **will panic** on out-of-range inputs for maximum speed. If you need to handle such cases, use the `option` or `keep` modes, which introduce a small, but measurable, runtime check.

- **Compile Time & Binary Size:** Be mindful of your input ranges. A function like `#[precalculate(a = 0..=1000, b = 0..=1000)]` would try to create a table with over a million entries, drastically increasing compile time and binary size.

- **`const fn` Required:** The macro can only be applied to functions marked as `const fn`.

- **Integer Arguments Required:** The function arguments must be integer primitives.

## License

This project is licensed under the [MIT License](LICENSE).
