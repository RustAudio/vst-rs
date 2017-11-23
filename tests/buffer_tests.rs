#[macro_use]
extern crate vst2;

use vst2::buffer::AudioBuffer;

/// Size of buffers used in tests.
const SIZE: usize = 1024;

/// Test that creating and zipping buffers works.
///
/// This test creates a channel for 2 inputs and 2 outputs. The input channels are simply values
/// from 0 to `SIZE-1` (e.g. [0, 1, 2, 3, 4, .. , SIZE - 1]) and the output channels are just 0.
/// This test assures that when the buffers are zipped together, the input values do not change.
#[test]
fn buffer_zip() {
    let in1: Vec<f32> = (0..SIZE).map(|x| x as f32).collect();
    let in2 = in1.clone();

    let mut out1 = vec![0.0; SIZE];
    let mut out2 = out1.clone();

    let inputs = vec![in1.as_ptr(), in2.as_ptr()];
    let mut outputs = vec![out1.as_mut_ptr(), out2.as_mut_ptr()];
    let mut buffer = AudioBuffer::new(&inputs, &mut outputs, SIZE);

    for (input, output) in buffer.zip() {
        input.into_iter().zip(output.into_iter()).fold(0, |acc,
         (input,
          output)| {
            assert_eq!(*input - acc as f32, 0.0);
            assert_eq!(*output, 0.0);
            acc + 1
        });
    }
}

/// Test that creating buffers from raw pointers works.
#[test]
fn from_raw() {
    let in1: Vec<f32> = (0..SIZE).map(|x| x as f32).collect();
    let in2 = in1.clone();

    let mut out1 = vec![0.0; SIZE];
    let mut out2 = out1.clone();

    let inputs = vec![in1.as_ptr(), in2.as_ptr()];
    let mut outputs = vec![out1.as_mut_ptr(), out2.as_mut_ptr()];
    let mut buffer = AudioBuffer::from_raw(2, 2, inputs.as_ptr(), outputs.as_mut_ptr(), SIZE);

    for (input, output) in buffer.zip() {
        input.into_iter().zip(output.into_iter()).fold(0, |acc,
         (input,
          output)| {
            assert_eq!(*input - acc as f32, 0.0);
            assert_eq!(*output, 0.0);
            acc + 1
        });
    }
}
