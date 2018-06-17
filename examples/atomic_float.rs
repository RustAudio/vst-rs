use std::sync::atomic::{AtomicUsize, Ordering};

pub struct AtomicFloat {
    atomic: AtomicUsize,
}

impl AtomicFloat {
    pub fn new(value: f32) -> AtomicFloat {
        AtomicFloat {
            atomic: AtomicUsize::new(value.to_bits() as usize),
        }
    }

    pub fn get(&self) -> f32 {
        f32::from_bits(self.atomic.load(Ordering::Relaxed) as u32)
    }

    pub fn set(&self, value: f32) {
        self.atomic.store(value.to_bits() as usize, Ordering::Relaxed)
    }
}
