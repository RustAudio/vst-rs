//! Interfaces to VST events.
// TODO: Update and explain both host and plugin events

use std::{mem, slice};

use crate::api;

/// A VST event.
#[derive(Copy, Clone)]
pub enum Event<'a> {
    /// A midi event.
    ///
    /// These are sent to the plugin before `Plugin::processing()` or `Plugin::processing_f64()` is
    /// called.
    Midi(MidiEvent),

    /// A system exclusive event.
    ///
    /// This is just a block of data and it is up to the plugin to interpret this. Generally used
    /// by midi controllers.
    SysEx(SysExEvent<'a>),

    /// A deprecated event.
    ///
    /// Passes the raw midi event structure along with this so that implementors can handle
    /// optionally handle this event.
    Deprecated(api::Event),
}

/// A midi event.
///
/// These are sent to the plugin before `Plugin::processing()` or `Plugin::processing_f64()` is
/// called.
#[derive(Copy, Clone)]
pub struct MidiEvent {
    /// The raw midi data associated with this event.
    pub data: [u8; 3],

    /// Number of samples into the current processing block that this event occurs on.
    ///
    /// E.g. if the block size is 512 and this value is 123, the event will occur on sample
    /// `samples[123]`.
    // TODO: Don't repeat this value in all event types
    pub delta_frames: i32,

    /// This midi event was created live as opposed to being played back in the sequencer.
    ///
    /// This can give the plugin priority over this event if it introduces a lot of latency.
    pub live: bool,

    /// The length of the midi note associated with this event, if available.
    pub note_length: Option<i32>,

    /// Offset in samples into note from note start, if available.
    pub note_offset: Option<i32>,

    /// Detuning between -63 and +64 cents.
    pub detune: i8,

    /// Note off velocity between 0 and 127.
    pub note_off_velocity: u8,
}

/// A system exclusive event.
///
/// This is just a block of data and it is up to the plugin to interpret this. Generally used
/// by midi controllers.
#[derive(Copy, Clone)]
pub struct SysExEvent<'a> {
    /// The SysEx payload.
    pub payload: &'a [u8],

    /// Number of samples into the current processing block that this event occurs on.
    ///
    /// E.g. if the block size is 512 and this value is 123, the event will occur on sample
    /// `samples[123]`.
    pub delta_frames: i32,
}

impl<'a> Event<'a> {
    /// Creates a high-level event from the given low-level API event.
    ///
    /// # Safety
    ///
    /// You must ensure that the given pointer refers to a valid event of the correct event type.
    /// For example, if the event type is [`api::EventType::SysEx`], it should point to a
    /// [`SysExEvent`]. In case of a [`SysExEvent`], `system_data` and `data_size` must be correct.
    pub unsafe fn from_raw_event(event: *const api::Event) -> Event<'a> {
        use api::EventType::*;
        let event = &*event;
        match event.event_type {
            Midi => {
                let event: api::MidiEvent = mem::transmute(*event);

                let length = if event.note_length > 0 {
                    Some(event.note_length)
                } else {
                    None
                };
                let offset = if event.note_offset > 0 {
                    Some(event.note_offset)
                } else {
                    None
                };
                let flags = api::MidiEventFlags::from_bits(event.flags).unwrap();

                Event::Midi(MidiEvent {
                    data: event.midi_data,
                    delta_frames: event.delta_frames,
                    live: flags.intersects(api::MidiEventFlags::REALTIME_EVENT),
                    note_length: length,
                    note_offset: offset,
                    detune: event.detune,
                    note_off_velocity: event.note_off_velocity,
                })
            }

            SysEx => Event::SysEx(SysExEvent {
                payload: {
                    // We can safely cast the event pointer to a `SysExEvent` pointer as
                    // event_type refers to a `SysEx` type.
                    #[allow(clippy::cast_ptr_alignment)]
                    let event: &api::SysExEvent = &*(event as *const api::Event as *const api::SysExEvent);
                    slice::from_raw_parts(event.system_data, event.data_size as usize)
                },

                delta_frames: event.delta_frames,
            }),

            _ => Event::Deprecated(*event),
        }
    }
}
