use std::sync::atomic::{AtomicU32, Ordering};

/// Simple atomic floating point variable with relaxed ordering.
///
/// Designed for the common case of sharing VST parameters between
/// multiple threads when no synchronization or change notification
/// is needed.
pub struct AtomicFloat {
    atomic: AtomicU32,
}

impl AtomicFloat {
    /// New atomic float with initial value `value`.
    pub fn new(value: f32) -> AtomicFloat {
        AtomicFloat {
            atomic: AtomicU32::new(value.to_bits()),
        }
    }

    /// Get the current value of the atomic float.
    pub fn get(&self) -> f32 {
        f32::from_bits(self.atomic.load(Ordering::Relaxed))
    }

    /// Set the value of the atomic float to `value`.
    pub fn set(&self, value: f32) {
        self.atomic.store(value.to_bits(), Ordering::Relaxed)
    }
}

impl Default for AtomicFloat {
    fn default() -> Self {
        AtomicFloat::new(0.0)
    }
}

impl std::fmt::Debug for AtomicFloat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.get(), f)
    }
}

impl std::fmt::Display for AtomicFloat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.get(), f)
    }
}

impl From<f32> for AtomicFloat {
    fn from(value: f32) -> Self {
        AtomicFloat::new(value)
    }
}

impl From<AtomicFloat> for f32 {
    fn from(value: AtomicFloat) -> Self {
        value.get()
    }
}
