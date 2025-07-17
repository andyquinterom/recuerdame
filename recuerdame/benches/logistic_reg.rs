use criterion::{Criterion, criterion_group, criterion_main};
use recuerdame::precalculate;
use std::hint::black_box;

const fn const_exp(x: f32) -> f32 {
    // For negative inputs, use the identity exp(x) = 1 / exp(-x)
    // to maintain precision of the Taylor series.
    if x < 0.0 {
        return 1.0 / const_exp(-x);
    }

    let mut result = 1.0;
    let mut term = 1.0;
    let mut i = 1.0;

    // The number of iterations in the Taylor series expansion.
    // More iterations lead to higher precision but longer compile times.
    // 20 iterations is a reasonable trade-off.
    while i < 20.0 {
        term *= x / i;
        result += term;
        i += 1.0;
    }

    result
}

const fn logistic_regression_inner(x1: i32, x2: i32) -> f32 {
    // Arbitrary parameters for the logistic regression model.
    // These would be the learned parameters from a training process.
    const WEIGHT_1: f32 = 0.5;
    const WEIGHT_2: f32 = -0.8;
    const BIAS: f32 = 0.1;

    // Calculate the linear combination of the inputs and parameters.
    let z = (x1 as f32 * WEIGHT_1) + (x2 as f32 * WEIGHT_2) + BIAS;

    // Apply the logistic (sigmoid) function.
    1.0 / (1.0 + const_exp(-z))
}

#[precalculate(x1 = -5..=5, x2 = -20..=20)]
pub const fn logistic_regression(x1: i32, x2: i32) -> f32 {
    logistic_regression_inner(x1, x2)
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("logistic regression (precalculated)", |b| {
        b.iter(|| logistic_regression(black_box(-1), black_box(15)))
    });
    c.bench_function("logistic regression (normal)", |b| {
        b.iter(|| logistic_regression_inner(black_box(-1), black_box(15)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
