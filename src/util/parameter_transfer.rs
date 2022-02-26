use std::mem::size_of;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

const USIZE_BITS: usize = size_of::<usize>() * 8;

fn word_and_bit(index: usize) -> (usize, usize) {
    (index / USIZE_BITS, 1usize << (index & (USIZE_BITS - 1)))
}

/// A set of parameters that can be shared between threads.
///
/// Supports efficient iteration over parameters that changed since last iteration.
#[derive(Default)]
pub struct ParameterTransfer {
    values: Vec<AtomicU32>,
    changed: Vec<AtomicUsize>,
}

impl ParameterTransfer {
    /// Create a new parameter set with `parameter_count` parameters.
    pub fn new(parameter_count: usize) -> Self {
        let bit_words = (parameter_count + USIZE_BITS - 1) / USIZE_BITS;
        ParameterTransfer {
            values: (0..parameter_count).map(|_| AtomicU32::new(0)).collect(),
            changed: (0..bit_words).map(|_| AtomicUsize::new(0)).collect(),
        }
    }

    /// Set the value of the parameter with index `index` to `value` and mark
    /// it as changed.
    pub fn set_parameter(&self, index: usize, value: f32) {
        let (word, bit) = word_and_bit(index);
        self.values[index].store(value.to_bits(), Ordering::Relaxed);
        self.changed[word].fetch_or(bit, Ordering::AcqRel);
    }

    /// Get the current value of the parameter with index `index`.
    pub fn get_parameter(&self, index: usize) -> f32 {
        f32::from_bits(self.values[index].load(Ordering::Relaxed))
    }

    /// Iterate over all parameters marked as changed. If `acquire` is `true`,
    /// mark all returned parameters as no longer changed.
    ///
    /// The iterator returns a pair of `(index, value)` for each changed parameter.
    ///
    /// When parameters have been changed on the current thread, the iterator is
    /// precise: it reports all changed parameters with the values they were last
    /// changed to.
    ///
    /// When parameters are changed on a different thread, the iterator is
    /// conservative, in the sense that it is guaranteed to report changed
    /// parameters eventually, but if a parameter is changed multiple times in
    /// a short period of time, it may skip some of the changes (but never the
    /// last) and may report an extra, spurious change at the end.
    ///
    /// The changed parameters are reported in increasing index order, and the same
    /// parameter is never reported more than once in the same iteration.
    pub fn iterate(&self, acquire: bool) -> ParameterTransferIterator {
        ParameterTransferIterator {
            pt: self,
            word: 0,
            bit: 1,
            acquire,
        }
    }
}

/// An iterator over changed parameters.
/// Returned by [`iterate`](struct.ParameterTransfer.html#method.iterate).
pub struct ParameterTransferIterator<'pt> {
    pt: &'pt ParameterTransfer,
    word: usize,
    bit: usize,
    acquire: bool,
}

impl<'pt> Iterator for ParameterTransferIterator<'pt> {
    type Item = (usize, f32);

    fn next(&mut self) -> Option<(usize, f32)> {
        let bits = loop {
            if self.word == self.pt.changed.len() {
                return None;
            }
            let bits = self.pt.changed[self.word].load(Ordering::Acquire) & self.bit.wrapping_neg();
            if bits != 0 {
                break bits;
            }
            self.word += 1;
            self.bit = 1;
        };

        let bit_index = bits.trailing_zeros() as usize;
        let bit = 1usize << bit_index;
        let index = self.word * USIZE_BITS + bit_index;

        if self.acquire {
            self.pt.changed[self.word].fetch_and(!bit, Ordering::AcqRel);
        }

        let next_bit = bit << 1;
        if next_bit == 0 {
            self.word += 1;
            self.bit = 1;
        } else {
            self.bit = next_bit;
        }

        Some((index, self.pt.get_parameter(index)))
    }
}

#[cfg(test)]
mod tests {
    extern crate rand;

    use crate::util::ParameterTransfer;

    use std::sync::mpsc::channel;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    use self::rand::rngs::StdRng;
    use self::rand::{Rng, SeedableRng};

    const THREADS: usize = 3;
    const PARAMETERS: usize = 1000;
    const UPDATES: usize = 1_000_000;

    #[test]
    fn parameter_transfer() {
        let transfer = Arc::new(ParameterTransfer::new(PARAMETERS));
        let (tx, rx) = channel();

        // Launch threads that change parameters
        for t in 0..THREADS {
            let t_transfer = Arc::clone(&transfer);
            let t_tx = tx.clone();
            let mut t_rng = StdRng::seed_from_u64(t as u64);
            thread::spawn(move || {
                let mut values = vec![0f32; PARAMETERS];
                for _ in 0..UPDATES {
                    let p: usize = t_rng.gen_range(0..PARAMETERS);
                    let v: f32 = t_rng.gen_range(0.0..1.0);
                    values[p] = v;
                    t_transfer.set_parameter(p, v);
                }
                t_tx.send(values).unwrap();
            });
        }

        // Continually receive updates from threads
        let mut values = vec![0f32; PARAMETERS];
        let mut results = vec![];
        let mut acquire_rng = StdRng::seed_from_u64(42);
        while results.len() < THREADS {
            let mut last_p = -1;
            for (p, v) in transfer.iterate(acquire_rng.gen_bool(0.9)) {
                assert!(p as isize > last_p);
                last_p = p as isize;
                values[p] = v;
            }
            thread::sleep(Duration::from_micros(100));
            while let Ok(result) = rx.try_recv() {
                results.push(result);
            }
        }

        // One last iteration to pick up all updates
        let mut last_p = -1;
        for (p, v) in transfer.iterate(true) {
            assert!(p as isize > last_p);
            last_p = p as isize;
            values[p] = v;
        }

        // Now there should be no more updates
        assert!(transfer.iterate(true).next().is_none());

        // Verify final values
        for p in 0..PARAMETERS {
            assert!((0..THREADS).any(|t| results[t][p] == values[p]));
        }
    }
}
