//! Interfaces to VST events.
// TODO: Update and explain both host and plugin events

use std::{mem, slice};

use api::flags::*;
use api::{self, flags};

/// A VST event.
#[derive(Copy, Clone)]
pub enum Event<'a> {
    /// A midi event.
    ///
    /// These are sent to the plugin before `Plugin::processing()` or `Plugin::processing_f64()` is
    /// called.
    Midi {
        /// The raw midi data associated with this event.
        data: [u8; 3],

        /// Number of samples into the current processing block that this event occurs on.
        ///
        /// E.g. if the block size is 512 and this value is 123, the event will occur on sample
        /// `samples[123]`.
        // TODO: Don't repeat this value in all event types
        delta_frames: i32,

        /// This midi event was created live as opposed to being played back in the sequencer.
        ///
        /// This can give the plugin priority over this event if it introduces a lot of latency.
        live: bool,

        /// The length of the midi note associated with this event, if available.
        note_length: Option<i32>,

        /// Offset in samples into note from note start, if available.
        note_offset: Option<i32>,

        /// Detuning between -63 and +64 cents.
        detune: i8,

        /// Note off velocity between 0 and 127.
        note_off_velocity: u8,
    },

    /// A system exclusive event.
    ///
    /// This is just a block of data and it is up to the plugin to interpret this. Generally used
    /// by midi controllers.
    SysEx {
        /// The SysEx payload.
        payload: &'a [u8],

        /// Number of samples into the current processing block that this event occurs on.
        ///
        /// E.g. if the block size is 512 and this value is 123, the event will occur on sample
        /// `samples[123]`.
        delta_frames: i32,
    },

    /// A deprecated event.
    ///
    /// Passes the raw midi event structure along with this so that implementors can handle
    /// optionally handle this event.
    Deprecated(api::Event),
}

impl<'a> From<api::Event> for Event<'a> {
    fn from(event: api::Event) -> Event<'a> {
        use api::EventType::*;

        match event.event_type {
            Midi => {
                let event: api::MidiEvent = unsafe { mem::transmute(event) };

                let length = if event.note_length > 0 { Some(event.note_length) } else { None };
                let offset = if event.note_offset > 0 { Some(event.note_offset) } else { None };
                let flags = flags::MidiEvent::from_bits(event.flags).unwrap();

                Event::Midi {
                    data: event.midi_data,
                    delta_frames: event.delta_frames,
                    live: flags.intersects(REALTIME_EVENT),
                    note_length: length,
                    note_offset: offset,
                    detune: event.detune,
                    note_off_velocity: event.note_off_velocity
                }
            }

            SysEx => Event::SysEx {
                payload: unsafe {
                    // We can safely transmute the event pointer to a `SysExEvent` pointer as
                    // event_type refers to a `SysEx` type.
                    let event: &api::SysExEvent = mem::transmute(&event);
                    slice::from_raw_parts(event.system_data, event.data_size as usize)
                },

                delta_frames: event.delta_frames
            },

            _ => Event::Deprecated(event),
        }
    }
}
