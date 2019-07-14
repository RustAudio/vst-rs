//! Utility functions that are useful for tests.

#[inline(always)]
/// Compare two `f32`s, and assert equality.
pub fn assert_f32_equal(a: f32, b: f32) {
    assert!((a - b).abs() < std::f32::EPSILON);
}

#[inline(always)]
/// Assert that an `f32` is zero.
pub fn assert_f32_zero(a: f32) {
    assert!(a.abs() < std::f32::EPSILON);
}
