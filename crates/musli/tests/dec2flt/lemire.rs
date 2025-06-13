use crate::dec2flt::float::RawFloat;
use crate::dec2flt::lemire::compute_float;

fn compute_float32(q: i64, w: u64) -> (i32, u64) {
    let fp = compute_float::<f32>(q, w);
    (fp.p_biased, fp.m)
}

fn compute_float64(q: i64, w: u64) -> (i32, u64) {
    let fp = compute_float::<f64>(q, w);
    (fp.p_biased, fp.m)
}

#[test]
fn compute_float_f32_rounding() {
    // the maximum integer that cna be converted to a `f32` without lost precision.
    let val = 1 << 24;
    let scale = 10_u64.pow(10);

    // These test near-halfway cases for single-precision floats.
    assert_eq!(compute_float32(0, val), (151, 0));
    assert_eq!(compute_float32(0, val + 1), (151, 0));
    assert_eq!(compute_float32(0, val + 2), (151, 1));
    assert_eq!(compute_float32(0, val + 3), (151, 2));
    assert_eq!(compute_float32(0, val + 4), (151, 2));

    // For the next power up, the two nearest representable numbers are twice as far apart.
    let val2 = 1 << 25;
    assert_eq!(compute_float32(0, val2), (152, 0));
    assert_eq!(compute_float32(0, val2 + 2), (152, 0));
    assert_eq!(compute_float32(0, val2 + 4), (152, 1));
    assert_eq!(compute_float32(0, val2 + 6), (152, 2));
    assert_eq!(compute_float32(0, val2 + 8), (152, 2));

    // These are examples of the above tests, with digits from the exponent shifted
    // to the mantissa.
    assert_eq!(compute_float32(-10, val * scale), (151, 0));
    assert_eq!(compute_float32(-10, (val + 1) * scale), (151, 0));
    assert_eq!(compute_float32(-10, (val + 2) * scale), (151, 1));
    // Let's check the lines to see if anything is different in table...
    assert_eq!(compute_float32(-10, (val + 3) * scale), (151, 2));
    assert_eq!(compute_float32(-10, (val + 4) * scale), (151, 2));

    // Check the rounding point between infinity and the next representable number down
    assert_eq!(compute_float32(38, 3), (f32::INFINITE_POWER - 1, 6402534));
    assert_eq!(compute_float32(38, 4), (f32::INFINITE_POWER, 0)); // infinity
    assert_eq!(
        compute_float32(20, 3402823470000000000),
        (f32::INFINITE_POWER - 1, 8388607)
    );
}

#[test]
fn compute_float_f64_rounding() {
    // The maximum integer that cna be converted to a `f64` without lost precision.
    let val = 1 << 53;
    let scale = 1000;

    // These test near-halfway cases for double-precision floats.
    assert_eq!(compute_float64(0, val), (1076, 0));
    assert_eq!(compute_float64(0, val + 1), (1076, 0));
    assert_eq!(compute_float64(0, val + 2), (1076, 1));
    assert_eq!(compute_float64(0, val + 3), (1076, 2));
    assert_eq!(compute_float64(0, val + 4), (1076, 2));

    // For the next power up, the two nearest representable numbers are twice as far apart.
    let val2 = 1 << 54;
    assert_eq!(compute_float64(0, val2), (1077, 0));
    assert_eq!(compute_float64(0, val2 + 2), (1077, 0));
    assert_eq!(compute_float64(0, val2 + 4), (1077, 1));
    assert_eq!(compute_float64(0, val2 + 6), (1077, 2));
    assert_eq!(compute_float64(0, val2 + 8), (1077, 2));

    // These are examples of the above tests, with digits from the exponent shifted
    // to the mantissa.
    assert_eq!(compute_float64(-3, val * scale), (1076, 0));
    assert_eq!(compute_float64(-3, (val + 1) * scale), (1076, 0));
    assert_eq!(compute_float64(-3, (val + 2) * scale), (1076, 1));
    assert_eq!(compute_float64(-3, (val + 3) * scale), (1076, 2));
    assert_eq!(compute_float64(-3, (val + 4) * scale), (1076, 2));

    // Check the rounding point between infinity and the next representable number down
    assert_eq!(
        compute_float64(308, 1),
        (f64::INFINITE_POWER - 1, 506821272651936)
    );
    assert_eq!(compute_float64(308, 2), (f64::INFINITE_POWER, 0)); // infinity
    assert_eq!(
        compute_float64(292, 17976931348623157),
        (f64::INFINITE_POWER - 1, 4503599627370495)
    );
}
