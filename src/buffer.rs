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

    /// Create an `AudioBuffer` from slices of raw pointers. Uuseful in a Rust VST host.
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

use event::Event;
use api;
use std::mem;

/// This buffer is used for sending midi events through the VST interface.
/// The purpose of this is to convert outgoing midi events from event::Event to api::Events.
/// It only allocates memory in new() and reuses the memory between calls.
pub struct SendEventBuffer {
    buf: Vec<u8>,
}

impl SendEventBuffer {

    /// Creates a buffer for sending up to the given number of midi events per frame
    #[inline(always)]
    pub fn new(len: usize) -> Self {
        let header_size = mem::size_of::<api::Events>() - (mem::size_of::<*mut api::Event>() * 2);
        let body_size = mem::size_of::<*mut api::Event>() * len;
        Self {
            buf: vec![0u8; header_size + body_size]
        }
    }

    /// Use this for sending midi events to a host or plugin.
    ///
    /// # Example
    /// ```no_run
    /// # use vst2::plugin::{Info, Plugin, HostCallback};
    /// # use vst2::buffer::{AudioBuffer, SendEventBuffer};
    /// # use vst2::host::Host;
    /// # struct ExamplePlugin { host: HostCallback, send_buf: SendEventBuffer }
    /// # impl Plugin for ExamplePlugin {
    /// #     fn get_info(&self) -> Info { Default::default() }
    /// #
    /// // Processor that clips samples above 0.4 or below -0.4:
    /// fn process(&mut self, buffer: &mut AudioBuffer<f32>){
    ///     let events = vec![
    ///         // ...
    ///     ];
    ///     let host = &mut self.host;
    ///     self.send_buf.send(events, |events| host.process_events(events));
    /// }
    /// # }
    /// ```
    pub fn send<F: FnOnce(&api::Events)>(&mut self, events: Vec<Event>, callback: F) {
        use std::cmp::min;
        use api::flags::REALTIME_EVENT;

        let len = min(self.buf.len(), events.len());

        // The `api::Events` structure uses a variable length array which is difficult to represent in
        // rust. We begin by creating a vector with the appropriate byte size by calculating the header
        // and the variable length body seperately.

        let send_events: &mut [*mut api::Event] = unsafe {
            // The header is updated by casting the array to the `api::Events` type and specifying the
            // required fields. We create a slice from the position of the first event and the length
            // of the array.
            let ptr = self.buf.as_mut_ptr() as *mut api::Events;
            (*ptr).num_events = len as i32;

            // A slice view of the body
            slice::from_raw_parts_mut(&mut (*ptr).events[0], len)
        };

        // Each event is zipped with the target body array slot. Most of what's happening here is just
        // copying data but the key thing to notice is that each event is boxed and cast to
        // (*mut api::Event). This way we can let the callback handle the event, and then later create
        // the box again from the raw pointer so that it can be properly dropped.
        for (event, out) in events.into_iter().zip(send_events.iter_mut()) {
            *out = match event {
                Event::Midi { data, delta_frames, live,
                              note_length, note_offset,
                              detune, note_off_velocity } => {
                    Box::into_raw(Box::new(api::MidiEvent {
                        event_type: api::EventType::Midi,
                        byte_size: mem::size_of::<api::MidiEvent>() as i32,
                        delta_frames: delta_frames,
                        flags: if live { REALTIME_EVENT.bits() } else { 0 },
                        note_length: note_length.unwrap_or(0),
                        note_offset: note_offset.unwrap_or(0),
                        midi_data: data,
                        _midi_reserved: 0,
                        detune: detune,
                        note_off_velocity: note_off_velocity,
                        _reserved1: 0,
                        _reserved2: 0
                    })) as *mut api::Event
                }
                Event::SysEx { payload, delta_frames } => {
                    Box::into_raw(Box::new(api::SysExEvent {
                        event_type: api::EventType::SysEx,
                        byte_size: mem::size_of::<api::SysExEvent>() as i32,
                        delta_frames: delta_frames,
                        _flags: 0,
                        data_size: payload.len() as i32,
                        _reserved1: 0,
                        system_data: payload.as_ptr() as *const u8 as *mut u8,
                        _reserved2: 0,
                    })) as *mut api::Event
                }
                Event::Deprecated(e) => Box::into_raw(Box::new(e))
            };
        }

        // Allow the callback to use the pointer
        callback(unsafe { &*(self.buf.as_ptr() as *const api::Events) });

        // Clean up the created events
        unsafe {
            for &mut event in send_events {
                match (*event).event_type {
                    api::EventType::Midi => {
                        drop(Box::from_raw(event as *mut api::MidiEvent));
                    }
                    api::EventType::SysEx => {
                        drop(Box::from_raw(event as *mut api::SysExEvent));
                    }
                    _ => {
                        drop(Box::from_raw(event));
                    }
                }
            }
        }
    }
}