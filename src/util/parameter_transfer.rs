use std::mem::size_of;
use std::sync::atomic::{AtomicUsize, Ordering};

const USIZE_BITS: usize = size_of::<usize>() * 8;

fn word_and_bit(index: usize) -> (usize, usize) {
	(index / USIZE_BITS, 1usize << (index & (USIZE_BITS - 1)))
}

/// A set of parameters that can be shared between threads.
///
/// Supports efficient iteration over parameters that changed since last iteration.
#[derive(Default)]
pub struct ParameterTransfer {
	// TODO: Change values to AtomicU32 when stabilized (expected in 1.34).
	values: Vec<AtomicUsize>,
	changed: Vec<AtomicUsize>,
}

impl ParameterTransfer {
	/// Create a new parameter set with `parameter_count` parameters.
	pub fn new(parameter_count: usize) -> Self {
		let bit_words = (parameter_count + USIZE_BITS - 1) / USIZE_BITS;
		ParameterTransfer {
			values: (0..parameter_count).map(|_| AtomicUsize::new(0)).collect(),
			changed: (0..bit_words).map(|_| AtomicUsize::new(0)).collect(),
		}
	}

	/// Set the value of the parameter with index `index` to `value` and mark
	/// it as changed.
	pub fn set_parameter(&self, index: usize, value: f32) {
		let (word, bit) = word_and_bit(index);
		self.values[index].store(value.to_bits() as usize, Ordering::Relaxed);
		self.changed[word].fetch_or(bit, Ordering::AcqRel);
	}

	/// Get the current value of the parameter with index `index`.
	pub fn get_parameter(&self, index: usize) -> f32 {
		f32::from_bits(self.values[index].load(Ordering::Relaxed) as u32)
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
	pub fn iterate<'pt>(&'pt self, acquire: bool) -> ParameterTransferIterator<'pt> {
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
				return None
			}
			let bits = self.pt.changed[self.word].load(Ordering::Acquire) & self.bit.wrapping_neg();
			if bits != 0 { break bits; }
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
