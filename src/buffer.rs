//! Buffers to safely work with audio samples.

use std::iter::{Zip, IntoIterator};
use std::vec::IntoIter;
use std::slice;

use num::Float;

/// A buffer containing `ChannelBuffer` buffers for each input/output.
#[allow(dead_code)]
pub struct AudioBuffer<'a, T: 'a + Float> {
    inputs: Vec<ChannelBuffer<'a, T>>,
    outputs: Vec<ChannelBuffer<'a, T>>
}

/// Iterator over channel buffers for either inputs or outputs.
pub type ChannelBufferIter<'a, T> = IntoIter<ChannelBuffer<'a, T>>;

#[allow(dead_code)]
impl<'a, T: 'a + Float> AudioBuffer<'a, T> {
    /// Create an `AudioBuffer` from vectors of slices. Each vector represents either an input or
    /// output, and contains an array of samples.
    /// Eg if inputs was a vector of size 2 containing slices of size 512, it would hold 2 inputs
    /// where each input holds 512 samples.
    pub fn new(inputs: Vec<&'a mut [T]>, outputs: Vec<&'a mut [T]>) -> AudioBuffer<'a, T> {
        // Creats an input / output for each array in respective Vec
        AudioBuffer {
            inputs: inputs.into_iter().map(|input| ChannelBuffer::new(input)).collect(),
            outputs: outputs.into_iter().map(|output| ChannelBuffer::new(output)).collect()
        }
    }

    /// Create an `AudioBuffer` from raw pointers. Only really useful for interacting with the VST
    /// API.
    pub unsafe fn from_raw(inputs_raw: *mut *mut T, outputs_raw: *mut *mut T, num_inputs: usize, num_outputs: usize, samples: usize) -> AudioBuffer<'a, T> {
        // Allocate an array size of vst input count
        let mut inputs: Vec<&mut [T]> = Vec::with_capacity(num_inputs);
        for i in 0 .. inputs.capacity() {
            // Push samples for each input to `inputs` array
            inputs.push(slice::from_raw_parts_mut(*inputs_raw.offset(i as isize), samples));
        }

        // Allocate an array size of vst output count
        let mut outputs: Vec<&mut [T]> = Vec::with_capacity(num_outputs);
        for i in 0 .. outputs.capacity() {
            // Push samples for each output to `outputs` array
            outputs.push(slice::from_raw_parts_mut(*outputs_raw.offset(i as isize), samples));
        }

        // Call constructor with slices
        AudioBuffer::new(inputs, outputs)
    }

    /// Retreieve input at specified index.
    pub fn input(&'a mut self, index: usize) -> Option<&'a mut ChannelBuffer<'a, T>> {
        self.inputs.get_mut(index)
    }
 
    /// Retreieve output at specified index.
    pub fn output(&'a mut self, index: usize) -> Option<&'a mut ChannelBuffer<'a, T>> {
        self.outputs.get_mut(index)
    }

    /// Create an iterator over all inputs.
    pub fn inputs(self) -> ChannelBufferIter<'a, T> {
        self.inputs.into_iter()
    }

    /// Create an iterator over all outputs.
    pub fn outputs(self) -> ChannelBufferIter<'a, T> {
        self.outputs.into_iter()
    }

    /// Zip together buffers.
    pub fn zip(self) -> Zip<ChannelBufferIter<'a, T>, ChannelBufferIter<'a, T>> {
        self.inputs.into_iter().zip(self.outputs.into_iter())
    }
}

/// Buffer samples for one channel.
pub struct ChannelBuffer<'a, T: 'a + Float> {
    data: &'a mut [T]
}

impl<'a, T: 'a + Float> ChannelBuffer<'a, T> {
    /// Construct a new `ChannelBuffer` from a slice.
    pub fn new(data: &'a mut [T]) -> ChannelBuffer<'a, T> {
        ChannelBuffer {
            data: data
        }
    }
}

impl<'a, T: 'a + Float> IntoIterator for ChannelBuffer<'a, T> {
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;

    fn into_iter(self) -> slice::IterMut<'a, T> {
        self.data.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use AudioBuffer;

    #[test]
    fn buffer_input() {
        // This test creates a channel for 2 inputs and 2 outputs. The input channels are simply
        // values from 0 to `SIZE` (e.g. [0, 1, 2, 3, 4, .. , SIZE - 1]) and the output channels
        // are just 0. This test assures that when the buffers are zipped together, the input
        // values do not change.
        const SIZE: usize = 1024;
        let vec: Vec<f32> = (0..SIZE).collect::<Vec<usize>>().iter().map(|&x| x as f32).collect();
        let mut in1 = vec.clone();
        let mut in2 = vec.clone();
        let mut out1 = vec![0.0; SIZE];
        let mut out2 = vec![0.0; SIZE];

        let buffer = AudioBuffer::new(vec![&mut in1, &mut in2],
                                      vec![&mut out1, &mut out2]);

        for (input, output) in buffer.zip() {
            input.into_iter().zip(output.into_iter())

            .fold(0.0, |acc, (input, output)| {
                assert!((*input - acc).abs() <= 0.001);
                assert_eq!(*output, 0.0);
                acc + 1.0
            });
        }
    }
}
