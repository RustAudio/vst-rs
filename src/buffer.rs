//! Buffers to safely work with audio samples.

use num_traits::Float;

use std::slice;
use std::iter::Zip;

/// AudioBuffer contains references to the audio buffers for all input and output channels
pub struct AudioBuffer<'a, T: 'a + Float> {
    inputs: &'a [*const T],
    outputs: &'a mut [*mut T],
    samples: usize,
}

impl<'a, T: 'a + Float> AudioBuffer<'a, T> {

    /// Create an `AudioBuffer` from slices of raw pointers. Useful in a Rust VST host.
    #[inline(always)]
    pub fn new(inputs: &'a [*const T], outputs: &'a mut [*mut T], samples: usize) -> Self {
        Self {
            inputs: inputs,
            outputs: outputs,
            samples: samples,
        }
    }

    /// Create an `AudioBuffer` from raw pointers. Only really useful for interacting with the VST API.
    #[inline(always)]
    pub fn from_raw(input_count: usize, output_count: usize,
                    inputs_raw: *const *const T, outputs_raw: *mut *mut T, samples: usize) -> Self {
        Self {
            inputs: unsafe { slice::from_raw_parts(inputs_raw, input_count) },
            outputs: unsafe { slice::from_raw_parts_mut(outputs_raw, output_count) },
            samples: samples,
        }
    }

    /// The number of input channels that this buffer was created for
    #[inline(always)]
    pub fn input_count(&self) -> usize { self.inputs.len() }

    /// The number of output channels that this buffer was created for
    #[inline(always)]
    pub fn output_count(&self) -> usize { self.outputs.len() }

    /// The number of samples in this buffer (same for all channels)
    #[inline(always)]
    pub fn samples(&self) -> usize { self.samples }

    /// The raw inputs to pass to processReplacing
    #[inline(always)]
    pub(crate) fn raw_inputs(&self) -> &[*const T] { &self.inputs }

    /// The raw outputs to pass to processReplacing
    #[inline(always)]
    pub(crate) fn raw_outputs(&mut self) -> &mut [*mut T] { &mut self.outputs }

    /// Split this buffer into separate inputs and outputs.
    #[inline(always)]
    pub fn split<'b>(&'b mut self) -> (Inputs<'b, T>, Outputs<'b, T>) where 'a: 'b {
        (
            Inputs { bufs: &self.inputs, samples: self.samples },
            Outputs { bufs: &self.outputs, samples: self.samples }
        )
    }

    /// Zip together buffers.
    #[inline(always)]
    pub fn zip<'b>(&'b mut self) -> Zip<InputIterator<'b, T>, OutputIterator<'b, T>> where 'a: 'b {
        let (inputs, outputs) = self.split();
        inputs.into_iter().zip(outputs)
    }
}

use std::ops::{Index, IndexMut};

/// Wrapper type to access the buffers for the input channels of an AudioBuffer in a safe way.
/// Behaves like a slice.
#[derive(Copy, Clone)]
pub struct Inputs<'a, T: 'a> {
    bufs: &'a [*const T],
    samples: usize,
}

impl<'a, T> Inputs<'a, T> {

    /// Number of channels
    pub fn len(&self) -> usize { self.bufs.len() }

    /// Access channel at the given index, unchecked
    pub fn get(&self, i: usize) -> &'a [T] {
        unsafe { slice::from_raw_parts(self.bufs[i], self.samples) }
    }

    /// Split borrowing at the given index, like for slices
    pub fn split_at(&self, i: usize) -> (Inputs<'a, T>, Inputs<'a, T>) {
        let (l, r) = self.bufs.split_at(i);
        (
            Inputs { bufs: &l, samples: self.samples },
            Inputs { bufs: &r, samples: self.samples }
        )
    }
}

impl<'a, T> Index<usize> for Inputs<'a, T> {
    type Output = [T];

    fn index(&self, i: usize) -> &Self::Output {
        self.get(i)
    }
}

/// Iterator over buffers for input channels of an AudioBuffer.
pub struct InputIterator<'a, T: 'a> {
    data: Inputs<'a, T>,
    i: usize,
}

impl<'a, T> Iterator for InputIterator<'a, T> {
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        if self.i < self.data.len() {
            let val = self.data.get(self.i);
            self.i += 1;
            Some(val)
        } else {
            None
        }
    }
}

impl<'a, T: Sized> IntoIterator for Inputs<'a, T> {
    type Item = &'a [T];
    type IntoIter = InputIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        InputIterator { data: self, i: 0 }
    }
}

/// Wrapper type to access the buffers for the output channels of an AudioBuffer in a safe way.
/// Behaves like a slice.
#[derive(Copy, Clone)]
pub struct Outputs<'a, T: 'a> {
    bufs: &'a [*mut T],
    samples: usize,
}

impl<'a, T> Outputs<'a, T> {

    /// Number of channels
    pub fn len(&self) -> usize { self.bufs.len() }

    /// Access channel at the given index, unchecked
    pub fn get(&self, i: usize) -> &'a [T] {
        unsafe { slice::from_raw_parts(self.bufs[i], self.samples) }
    }

    /// Mutably access channel at the given index, unchecked
    pub fn get_mut(&self, i: usize) -> &'a mut [T] {
        unsafe { slice::from_raw_parts_mut(self.bufs[i], self.samples) }
    }

    /// Split borrowing at the given index, like for slices
    pub fn split_at_mut(&mut self, i: usize) -> (Outputs<'a, T>, Outputs<'a, T>) {
        let (l, r) = self.bufs.split_at(i);
        (
            Outputs { bufs: &l, samples: self.samples },
            Outputs { bufs: &r, samples: self.samples }
        )
    }
}

impl<'a, T> Index<usize> for Outputs<'a, T> {
    type Output = [T];

    fn index(&self, i: usize) -> &Self::Output {
        self.get(i)
    }
}

impl<'a, T> IndexMut<usize> for Outputs<'a, T> {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        self.get_mut(i)
    }
}

/// Iterator over buffers for output channels of an AudioBuffer.
pub struct OutputIterator<'a, T: 'a> {
    data: Outputs<'a, T>,
    i: usize,
}

impl<'a, T> Iterator for OutputIterator<'a, T> {
    type Item = &'a mut [T];

    fn next(&mut self) -> Option<Self::Item> {
        if self.i < self.data.len() {
            let val = self.data.get_mut(self.i);
            self.i += 1;
            Some(val)
        } else {
            None
        }
    }
}

impl<'a, T: Sized> IntoIterator for Outputs<'a, T> {
    type Item = &'a mut [T];
    type IntoIter = OutputIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        OutputIterator { data: self, i: 0 }
    }
}

use event::{Event, MidiEvent};
use api;
use std::mem;
use std::borrow::Borrow;

/// This buffer is used for sending midi events through the VST interface.
/// The purpose of this is to convert outgoing midi events from event::Event to api::Events.
/// It only allocates memory in new() and reuses the memory between calls.
pub struct SendEventBuffer {
    buf: Vec<u8>,
    api_events: Vec<api::SysExEvent>,
}

impl Default for SendEventBuffer {
    fn default() -> Self {
        SendEventBuffer::new(1024)
    }
}

impl SendEventBuffer {

    /// Creates a buffer for sending up to the given number of midi events per frame
    #[inline(always)]
    pub fn new(capacity: usize) -> Self {
        let header_size = mem::size_of::<api::Events>() - (mem::size_of::<*mut api::Event>() * 2);
        let body_size = mem::size_of::<*mut api::Event>() * capacity;
        let mut buf = vec![0u8; header_size + body_size];
        let api_events = vec![unsafe { mem::zeroed::<api::SysExEvent>() }; capacity];
        {
            let ptrs = {
                let e = Self::buf_as_api_events(&mut buf);
                e.num_events = capacity as i32;
                e.events_raw_mut()
            };
            for (ptr, event) in ptrs.iter_mut().zip(&api_events) {
                let (ptr, event): (&mut *const api::SysExEvent, &api::SysExEvent) = (ptr, event);
                *ptr = event;
            }
        }
        Self {
            buf: buf,
            api_events: api_events,
        }
    }

    #[inline(always)]
    fn buf_as_api_events(buf: &mut [u8]) -> &mut api::Events {
        unsafe { &mut *(buf.as_mut_ptr() as *mut api::Events) }
    }

    /// Use this for sending events to a host or plugin.
    ///
    /// # Example
    /// ```no_run
    /// # use vst2::plugin::{Info, Plugin, HostCallback};
    /// # use vst2::buffer::{AudioBuffer, SendEventBuffer};
    /// # use vst2::host::Host;
    /// # struct ExamplePlugin { host: HostCallback, send_buffer: SendEventBuffer }
    /// # impl Plugin for ExamplePlugin {
    /// #     fn get_info(&self) -> Info { Default::default() }
    /// #
    /// // Processor that clips samples above 0.4 or below -0.4:
    /// fn process(&mut self, buffer: &mut AudioBuffer<f32>){
    ///     let events = vec![
    ///         // ...
    ///     ];
    ///     self.send_buffer.store(&events);
    ///     self.host.process_events(self.send_buffer.events());
    /// }
    /// # }
    /// ```
    pub fn store<'a, T: IntoIterator<Item = U>, U: Borrow<Event<'a>>>(&mut self, events: T) {
        let count = events.into_iter().zip(self.api_events.iter_mut()).map(|(event, out)| {
            let (event, out): (&Event, &mut api::SysExEvent) = (event.borrow(), out);
            match *event {
                Event::Midi(ev) => {
                    Self::store_midi_impl(out, &ev);
                }
                Event::SysEx(ev) => {
                    *out = api::SysExEvent {
                        event_type: api::EventType::SysEx,
                        byte_size: mem::size_of::<api::SysExEvent>() as i32,
                        delta_frames: ev.delta_frames,
                        _flags: 0,
                        data_size: ev.payload.len() as i32,
                        _reserved1: 0,
                        system_data: ev.payload.as_ptr() as *const u8 as *mut u8,
                        _reserved2: 0,
                    };
                }
                Event::Deprecated(e) => {
                    let out = unsafe { &mut *(out as *mut _ as *mut _) };
                    *out = e;
                }
            };
        }).count();
        self.set_num_events(count);
    }

    /// Use this for sending midi events to a host or plugin.
    /// Like store() but for when you're not sending any SysExEvents, only MidiEvents.
    pub fn store_midi<T: IntoIterator<Item = U>, U: Borrow<MidiEvent>>(&mut self, events: T) {
        let count = events.into_iter().zip(self.api_events.iter_mut()).map(|(event, out)| {
            let (ev, out): (&MidiEvent, &mut api::SysExEvent) = (event.borrow(), out);
            Self::store_midi_impl(out, ev);
        }).count();
        self.set_num_events(count);
    }

    fn store_midi_impl(out: &mut api::SysExEvent, ev: &MidiEvent) {
        use api::flags::REALTIME_EVENT;
        let out = unsafe { &mut *(out as *mut _ as *mut _) };
        *out = api::MidiEvent {
            event_type: api::EventType::Midi,
            byte_size: mem::size_of::<api::MidiEvent>() as i32,
            delta_frames: ev.delta_frames,
            flags: if ev.live { REALTIME_EVENT.bits() } else { 0 },
            note_length: ev.note_length.unwrap_or(0),
            note_offset: ev.note_offset.unwrap_or(0),
            midi_data: ev.data,
            _midi_reserved: 0,
            detune: ev.detune,
            note_off_velocity: ev.note_off_velocity,
            _reserved1: 0,
            _reserved2: 0
        };
    }

    fn set_num_events(&mut self, events_len: usize) {
        use std::cmp::min;
        let e = Self::buf_as_api_events(&mut self.buf);
        e.num_events = min(self.api_events.len(), events_len) as i32;
    }

    /// Use this for sending midi events to a host or plugin.
    /// See `store()`
    #[inline(always)]
    pub fn events(&self) -> &api::Events {
        unsafe { &*(self.buf.as_ptr() as *const api::Events) }
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
        let in1: Vec<f32> = (0..SIZE).map(|x| x as f32).collect();
        let in2 = in1.clone();

        let mut out1 = vec![0.0; SIZE];
        let mut out2 = out1.clone();

        let inputs = vec![in1.as_ptr(), in2.as_ptr()];
        let mut outputs = vec![out1.as_mut_ptr(), out2.as_mut_ptr()];
        let mut buffer = AudioBuffer::new(&inputs, &mut outputs, SIZE);

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
        let in1: Vec<f32> = (0..SIZE).map(|x| x as f32).collect();
        let in2 = in1.clone();

        let mut out1 = vec![0.0; SIZE];
        let mut out2 = out1.clone();

        let inputs = vec![in1.as_ptr(), in2.as_ptr()];
        let mut outputs = vec![out1.as_mut_ptr(), out2.as_mut_ptr()];
        let mut buffer = AudioBuffer::from_raw(2, 2, inputs.as_ptr(), outputs.as_mut_ptr(), SIZE);

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