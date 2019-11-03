//! Structures for easing the implementation of VST plugins.

mod atomic_float;
mod parameter_transfer;

pub use self::atomic_float::AtomicFloat;
pub use self::parameter_transfer::{ParameterTransfer, ParameterTransferIterator};
