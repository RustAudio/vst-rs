use std::sync::atomic::{AtomicUsize, Ordering};

/// Simple atomic floating point variable with relaxed ordering.
///
/// Designed for the common case of sharing VST parameters between
/// multiple threads when no synchronization or change notification
/// is needed.
pub struct AtomicFloat {
    // TODO: Change atomic to AtomicU32 when stabilized (expected in 1.34).
    atomic: AtomicUsize,
}

impl AtomicFloat {
    /// New atomic float with initial value `value`.
    pub fn new(value: f32) -> AtomicFloat {
        AtomicFloat {
            atomic: AtomicUsize::new(value.to_bits() as usize),
        }
    }

    /// Get the current value of the atomic float.
    pub fn get(&self) -> f32 {
        f32::from_bits(self.atomic.load(Ordering::Relaxed) as u32)
    }

    /// Set the value of the atomic float to `value`.
    pub fn set(&self, value: f32) {
        self.atomic.store(value.to_bits() as usize, Ordering::Relaxed)
    }
}
