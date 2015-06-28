//! Enums for use in this library.

/// Implements `From` and `Into` for enums with `#[repr(usize)]`. Useful for interfacing with C
/// enums.
macro_rules! impl_clike {
    ($t:ty, $($c:ty) +) => {
        $(
            impl From<$c> for $t {
                fn from(v: $c) -> $t {
                    use std::mem;
                    unsafe { mem::transmute(v as usize) }
                }
            }

            impl Into<$c> for $t {
                fn into(self) -> $c {
                    self as $c
                }
            }
        )*
    };

    ($t:ty) => {
        impl_clike!($t, i8 i16 i32 i64 isize u8 u16 u32 u64 usize);
    }
}

/// Features which are optionally supported by a plugin. These are queried by the host at run time.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum CanDo {
    SendEvents,
    SendMidiEvent,
    ReceiveEvents,
    ReceiveMidiEvent,
    ReceiveTimeInfo,
    Offline,
    MidiProgramNames,
    Bypass,

    //Bitwig specific?
    ReceiveSysexEvent,
    MidiSingleNoteTuningChange,
    MidiKeyBasedInstrumentControl,

    Other(String)
}

use std::str::FromStr;
impl FromStr for CanDo {
    type Err = String;

    fn from_str(s: &str) -> Result<CanDo, String> {
        use self::CanDo::*;

        Ok(match s {
            "sendVstEvents" => SendEvents,
            "sendVstMidiEvent" => SendMidiEvent,
            "receiveVstEvents" => ReceiveEvents,
            "receiveVstMidiEvent" => ReceiveMidiEvent,
            "receiveVstTimeInfo" => ReceiveTimeInfo,
            "offline" => Offline,
            "midiProgramNames" => MidiProgramNames,
            "bypass" => Bypass,

            "receiveVstSysexEvent" => ReceiveSysexEvent,
            "midiSingleNoteTuningChange" => MidiSingleNoteTuningChange,
            "midiKeyBasedInstrumentControl" => MidiKeyBasedInstrumentControl,
            otherwise => Other(otherwise.to_string())
        })
    }
}

/// Used to specify whether functionality is supported.
#[allow(missing_docs)]
pub enum Supported {
    Yes,
    Maybe,
    No
}

impl Into<isize> for Supported {
    /// Convert to integer ordinal for interop with VST api.
    fn into(self) -> isize {
        use self::Supported::*;

        match self {
            Yes => 1,
            Maybe => 0,
            No => -1
        }
    }
}

/// Bitflags.
#[allow(dead_code, missing_docs)]
pub mod flags {
    /// Flags for VST channel.
    pub mod channel_flags {
        bitflags! {
            flags Channel: i32 {
                /// Indicates channel is active. Ignored by host.
                const ACTIVE = 1,
                /// Indicates channel is first of stereo pair.
                const STEREO = 1 << 1,
                /// Use channel's specified speaker_arrangement instead of stereo flag.
                const SPEAKER = 1 << 2
            }
        }
    }

    /// Flags for VST plugins.
    pub mod plugin {
        bitflags! {
            flags Effect: i32 {
                /// Plugin has an editor.
                const HAS_EDITOR = 1 << 0,
                /// Plugin can process 32 bit audio. (Mandatory in VST 2.4).
                const CAN_REPLACING = 1 << 4,
                /// Plugin preset data is handled in formatless chunks.
                const PROGRAM_CHUNKS = 1 << 5,
                /// Plugin is a synth.
                const IS_SYNTH = 1 << 8,
                //TODO: Implement and doc.
                const NO_SOUND_IN_STOP = 1 << 9,
                /// Supports 64 bit audio processing.
                const CAN_DOUBLE_REPLACING = 1 << 12
            }
        }
    }

    /// Cross platform modifier key bitflags.
    pub mod modifier_key {
        bitflags!{
            flags ModifierKey: u8 {
                /// Shift key.
                const SHIFT = 1 << 0, // Shift
                /// Alt key.
                const ALT = 1 << 1, // Alt
                /// Control on mac.
                const COMMAND = 1 << 2, // Control on Mac
                /// Command on mac, ctrl on other.
                const CONTROL = 1 << 3  // Ctrl on PC, Apple on Mac
            }
        }
    }
}
