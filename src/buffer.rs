//! Buffers to safely work with audio samples.

use std::iter::{Zip, IntoIterator};
use std::vec::IntoIter;
use std::slice;

use num::Float;

/// A buffer containing `ChannelBuffer` buffers for each input/output.
pub struct AudioBuffer<'a, T: 'a + Float> {
    inputs: Vec<&'a mut [T]>,
    outputs: Vec<&'a mut [T]>,
}

/// Iterator over channel buffers for either inputs or outputs.
pub type ChannelBufferIter<'a, T> = IntoIter<&'a mut [T]>;

impl<'a, T: 'a + Float> AudioBuffer<'a, T> {
    /// Create an `AudioBuffer` from vectors of slices.
    ///
    /// Each vector item represents either an input or output, and contains an array of samples. Eg
    /// if inputs was a vector of size 2 containing slices of size 512, it would hold 2 inputs where
    /// each input holds 512 samples.
    pub fn new(inputs: Vec<&'a mut [T]>, outputs: Vec<&'a mut [T]>) -> AudioBuffer<'a, T> {
        AudioBuffer {
            inputs: inputs,
            outputs: outputs,
        }
    }

    /// Create an `AudioBuffer` from raw pointers. Only really useful for interacting with the VST
    /// API.
    pub unsafe fn from_raw(inputs_raw: *mut *mut T, outputs_raw: *mut *mut T, num_inputs: usize, num_outputs: usize, samples: usize) -> AudioBuffer<'a, T> {
        let inputs =
            // Create a slice of type &mut [*mut f32]
            slice::from_raw_parts_mut(inputs_raw, num_inputs).iter()
            // Convert to &mut [&mut [f32]]
            .map(|input| slice::from_raw_parts_mut(*input, samples))
            // Collect into Vec<&mut [f32]>
            .collect();

        let outputs =
            // Create a slice of type &mut [*mut f32]
            slice::from_raw_parts_mut(outputs_raw, num_outputs).iter()
            // Convert to &mut [&mut [f32]]
            .map(|output| slice::from_raw_parts_mut(*output, samples))
            // Collect into Vec<&mut [f32]>
            .collect();

        // Call constructor with vectors
        AudioBuffer::new(inputs, outputs)
    }

    /// Return a reference to all inputs.
    pub fn inputs(&'a mut self) -> &'a mut Vec<&'a mut [T]> {
        &mut self.inputs
    }

    /// Return a reference to all outputs.
    pub fn outputs(&'a mut self) -> &'a mut Vec<&'a mut [T]> {
        &mut self.outputs
    }

    /// Consume this buffer and split it into separate inputs and outputs.
    ///
    /// # Example
    ///
    /// ```
    /// # use vst2::buffer::AudioBuffer;
    /// # let mut in1 = vec![0.0; 512];
    /// # let (mut in2, mut out1, mut out2) = (in1.clone(), in1.clone(), in1.clone());
    /// #
    /// # let buffer = AudioBuffer::new(vec![&mut in1, &mut in2],
    /// #                               vec![&mut out1, &mut out2]);
    /// let (mut inputs, mut outputs) = buffer.split();
    /// let input: &mut [f32] = &mut inputs[0]; // First input
    /// ```
    pub fn split(self) -> (Vec<&'a mut [T]>, Vec<&'a mut [T]>) {
        (self.inputs, self.outputs)
    }

    /// Zip together buffers.
    pub fn zip(self) -> Zip<ChannelBufferIter<'a, T>, ChannelBufferIter<'a, T>> {
        self.inputs.into_iter().zip(self.outputs.into_iter())
    }
}

#[cfg(test)]
mod tests {
    use buffer::AudioBuffer;

    /// Size of buffers used in tests.
    const SIZE: usize = 1024;

    /// Test that creating and zipping buffers works.
    ///
    /// This test creates a channel for 2 inputs and 2 outputs. The input channels are simply values
    /// from 0 to `SIZE-1` (e.g. [0, 1, 2, 3, 4, .. , SIZE - 1]) and the output channels are just 0.
    /// This test assures that when the buffers are zipped together, the input values do not change.
    #[test]
    fn buffer_zip() {
        let mut in1: Vec<f32> = (0..SIZE).map(|x| x as f32).collect();
        let mut in2 = in1.clone();

        let mut out1 = vec![0.0; SIZE];
        let mut out2 = out1.clone();

        let buffer = AudioBuffer::new(vec![&mut in1, &mut in2],
                                      vec![&mut out1, &mut out2]);

        for (input, output) in buffer.zip() {
            input.into_iter().zip(output.into_iter())
            .fold(0, |acc, (input, output)| {
                assert_eq!(*input - acc as f32, 0.0);
                assert_eq!(*output, 0.0);
                acc + 1
            });
        }
    }

    /// Test that creating buffers from raw pointers works.
    #[test]
    fn from_raw() {
        let mut in1: Vec<f32> = (0..SIZE).map(|x| x as f32).collect();
        let mut in2 = in1.clone();

        let mut out1 = vec![0.0; SIZE];
        let mut out2 = out1.clone();

        let buffer = unsafe {
            AudioBuffer::from_raw(vec![in1.as_mut_ptr(), in2.as_mut_ptr()].as_mut_ptr(),
                                  vec![out1.as_mut_ptr(), out2.as_mut_ptr()].as_mut_ptr(),
                                  2, 2, SIZE)
        };

        for (input, output) in buffer.zip() {
            input.into_iter().zip(output.into_iter())
            .fold(0, |acc, (input, output)| {
                assert_eq!(*input - acc as f32, 0.0);
                assert_eq!(*output, 0.0);
                acc + 1
            });
        }
    }
}
