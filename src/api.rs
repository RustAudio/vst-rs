//! Structures and types for interfacing with the VST 2.4 API.

use std::os::raw::c_void;

use plugin::Plugin;
use self::consts::*;

/// Constant values
#[allow(missing_docs)] // For obvious constants
pub mod consts {

    pub const MAX_PRESET_NAME_LEN: usize = 24;
    pub const MAX_PARAM_STR_LEN: usize = 32;
    pub const MAX_LABEL: usize = 64;
    pub const MAX_SHORT_LABEL: usize = 8;
    pub const MAX_PRODUCT_STR_LEN: usize = 64;
    pub const MAX_VENDOR_STR_LEN: usize = 64;

    /// VST plugins are identified by a magic number. This corresponds to 0x56737450.
    pub const VST_MAGIC: i32 = ('V' as i32) << 24 |
                               ('s' as i32) << 16 |
                               ('t' as i32) << 8  |
                               ('P' as i32) << 0  ;
}

/// `VSTPluginMain` function signature.
pub type PluginMain = fn(callback: HostCallbackProc) -> *mut AEffect;

/// Host callback function passed to plugin. Can be used to query host information from plugin side.
pub type HostCallbackProc = fn(effect: *mut AEffect, opcode: i32, index: i32, value: isize, ptr: *mut c_void, opt: f32) -> isize;

/// Dispatcher function used to process opcodes. Called by host.
pub type DispatcherProc = fn(effect: *mut AEffect, opcode: i32, index: i32, value: isize, ptr: *mut c_void, opt: f32) -> isize;

/// Process function used to process 32 bit floating point samples. Called by host.
pub type ProcessProc = fn(effect: *mut AEffect, inputs: *const *const f32, outputs: *mut *mut f32, sample_frames: i32);

/// Process function used to process 64 bit floating point samples. Called by host.
pub type ProcessProcF64 = fn(effect: *mut AEffect, inputs: *const *const f64, outputs: *mut *mut f64, sample_frames: i32);

/// Callback function used to set parameter values. Called by host.
pub type SetParameterProc = fn(effect: *mut AEffect, index: i32, parameter: f32);

/// Callback function used to get parameter values. Called by host.
pub type GetParameterProc = fn(effect: *mut AEffect, index: i32) -> f32;

/// Used with the VST API to pass around plugin information.
#[allow(non_snake_case)]
#[repr(C)]
pub struct AEffect {
    /// Magic number. Must be `['V', 'S', 'T', 'P']`.
    pub magic: i32,

    /// Host to plug-in dispatcher.
    pub dispatcher: DispatcherProc,

    /// Accumulating process mode is deprecated in VST 2.4! Use `processReplacing` instead!
    pub _process: ProcessProc,

    /// Set value of automatable parameter.
    pub setParameter: SetParameterProc,

    /// Get value of automatable parameter.
    pub getParameter: GetParameterProc,


    /// Number of programs (Presets).
    pub numPrograms: i32,

    /// Number of parameters. All programs are assumed to have this many parameters.
    pub numParams: i32,

    /// Number of audio inputs.
    pub numInputs: i32,

    /// Number of audio outputs.
    pub numOutputs: i32,


    /// Bitmask made of values from api::flags.
    ///
    /// ```no_run
    /// use vst2::api::flags;
    /// let flags = flags::CAN_REPLACING | flags::CAN_DOUBLE_REPLACING;
    /// // ...
    /// ```
    pub flags: i32,

    /// Reserved for host, must be 0.
    pub reserved1: isize,

    /// Reserved for host, must be 0.
    pub reserved2: isize,

    /// For algorithms which need input in the first place (Group delay or latency in samples).
    ///
    /// This value should be initially in a resume state.
    pub initialDelay: i32,


    /// Deprecated unused member.
    pub _realQualities: i32,

    /// Deprecated unused member.
    pub _offQualities: i32,

    /// Deprecated unused member.
    pub _ioRatio: f32,


    /// Void pointer usable by api to store object data.
    pub object: *mut c_void,

    /// User defined pointer.
    pub user: *mut c_void,

    /// Registered unique identifier (register it at Steinberg 3rd party support Web). This is used
    /// to identify a plug-in during save+load of preset and project.
    pub uniqueId: i32,

    /// Plug-in version (e.g. 1100 for v1.1.0.0).
    pub version: i32,

    /// Process audio samples in replacing mode.
    pub processReplacing: ProcessProc,

    /// Process double-precision audio samples in replacing mode.
    pub processReplacingF64: ProcessProcF64,

    /// Reserved for future use (please zero).
    pub future: [u8; 56]
}

impl AEffect {
    /// Return handle to Plugin object. Only works for plugins created using this library.
    pub unsafe fn get_plugin(&mut self) -> &mut Box<Plugin> {
        &mut *(self.object as *mut Box<Plugin>)
    }

    /// Drop the Plugin object. Only works for plugins created using this library.
    pub unsafe fn drop_plugin(&mut self) {
        drop(Box::from_raw(self.object as *mut Box<Plugin>));
        drop(Box::from_raw(self.user as *mut super::PluginCache));
    }

    pub(crate) unsafe fn get_cache(&mut self) -> &mut super::PluginCache {
        &mut *(self.user as *mut _)
    }
}

/// Information about a channel. Only some hosts use this information.
#[repr(C)]
pub struct ChannelProperties {
    /// Channel name.
    pub name: [u8; MAX_LABEL as usize],

    /// Flags found in `channel_flags` module.
    pub flags: i32,

    /// Type of speaker arrangement this channel is a part of.
    pub arrangement_type: SpeakerArrangementType,

    /// Name of channel (recommended: 6 characters + delimiter).
    pub short_name: [u8; MAX_SHORT_LABEL as usize],

    /// Reserved for future use.
    pub future: [u8; 48]
}

/// Tells the host how the channels are intended to be used in the plugin. Only useful for some
/// hosts.
#[repr(i32)]
#[derive(Clone, Copy)]
pub enum SpeakerArrangementType {
    /// User defined arrangement.
    Custom = -2,
    /// Empty arrangement.
    Empty = -1,

    /// Mono.
    Mono = 0,

    /// L R
    Stereo,
    /// Ls Rs
    StereoSurround,
    /// Lc Rc
    StereoCenter,
    /// Sl Sr
    StereoSide,
    /// C Lfe
    StereoCLfe,

    /// L R C
    Cinema30,
    /// L R S
    Music30,

    /// L R C Lfe
    Cinema31,
    /// L R Lfe S
    Music31,

    /// L R C S (LCRS)
    Cinema40,
    /// L R Ls Rs (Quadro)
    Music40,

    /// L R C Lfe S (LCRS + Lfe)
    Cinema41,
    /// L R Lfe Ls Rs (Quadro + Lfe)
    Music41,

    /// L R C Ls Rs
    Surround50,
    /// L R C Lfe Ls Rs
    Surround51,

    /// L R C Ls  Rs Cs
    Cinema60,
    /// L R Ls Rs  Sl Sr
    Music60,

    /// L R C Lfe Ls Rs Cs
    Cinema61,
    /// L R Lfe Ls Rs Sl Sr
    Music61,

    /// L R C Ls Rs Lc Rc
    Cinema70,
    /// L R C Ls Rs Sl Sr
    Music70,

    /// L R C Lfe Ls Rs Lc Rc
    Cinema71,
    /// L R C Lfe Ls Rs Sl Sr
    Music71,

    /// L R C Ls Rs Lc Rc Cs
    Cinema80,
    /// L R C Ls Rs Cs Sl Sr
    Music80,

    /// L R C Lfe Ls Rs Lc Rc Cs
    Cinema81,
    /// L R C Lfe Ls Rs Cs Sl Sr
    Music81,

    /// L R C Lfe Ls Rs Tfl Tfc Tfr Trl Trr Lfe2
    Surround102,
}

/// Used to specify whether functionality is supported.
#[allow(missing_docs)]
pub enum Supported {
    Yes,
    Maybe,
    No
}

impl Supported {
    /// Create a `Supported` value from an integer if possible.
    pub fn from(val: isize) -> Option<Supported> {
        use self::Supported::*;

        match val {
             1 => Some(Yes),
             0 => Some(Maybe),
            -1 => Some(No),
            _ => None
        }
    }
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

/// Denotes in which thread the host is in.
#[repr(i32)]
pub enum ProcessLevel {
    /// Unsupported by host.
    Unknown = 0,

    /// GUI thread.
    User,
    /// Audio process thread.
    Realtime,
    /// Sequence thread (MIDI, etc).
    Prefetch,
    /// Offline processing thread (therefore GUI/user thread).
    Offline,
}

/// Language that the host is using.
#[repr(i32)]
#[allow(missing_docs)]
pub enum HostLanguage {
    English = 1,
    German,
    French,
    Italian,
    Spanish,
    Japanese,
}

/// The file operation to perform.
#[repr(i32)]
pub enum FileSelectCommand {
    /// Load a file.
    Load = 0,
    /// Save a file.
    Save,
    /// Load multiple files simultaneously.
    LoadMultipleFiles,
    /// Choose a directory.
    SelectDirectory,
}

// TODO: investigate removing this.
/// Format to select files.
pub enum FileSelectType {
    /// Regular file selector.
    Regular
}

/// File type descriptor.
#[repr(C)]
pub struct FileType {
    /// Display name of file type.
    pub name: [u8; 128],

    /// OS X file type.
    pub osx_type: [u8; 8],
    /// Windows file type.
    pub win_type: [u8; 8],
    /// Unix file type.
    pub nix_type: [u8; 8],

    /// MIME type.
    pub mime_type_1: [u8; 128],
    /// Additional MIME type.
    pub mime_type_2: [u8; 128],
}

/// File selector descriptor used in `host::OpCode::OpenFileSelector`.
#[repr(C)]
pub struct FileSelect {
    /// The type of file selection to perform.
    pub command: FileSelectCommand,
    /// The file selector to open.
    pub select_type: FileSelectType,
    /// Unknown. 0 = no creator.
    pub mac_creator: i32,
    /// Number of file types.
    pub num_types: i32,
    /// List of file types to show.
    pub file_types: *mut FileType,

    /// File selector's title.
    pub title: [u8; 1024],
    /// Initial path.
    pub initial_path: *mut u8,
    /// Used when operation returns a single path.
    pub return_path: *mut u8,
    /// Size of the path buffer in bytes.
    pub size_return_path: i32,

    /// Used when operation returns multiple paths.
    pub return_multiple_paths: *mut *mut u8,
    /// Number of paths returned.
    pub num_paths: i32,

    /// Reserved by host.
    pub reserved: isize,
    /// Reserved for future use.
    pub future: [u8; 116]
}

/// A struct which contains events.
#[repr(C)]
pub struct Events {
    /// Number of events.
    pub num_events: i32,

    /// Reserved for future use. Should be 0.
    pub _reserved: isize,

    /// Variable-length array of pointers to `api::Event` objects.
    ///
    /// The VST standard specifies a variable length array of initial size 2. If there are more
    /// than 2 elements a larger array must be stored in this structure.
    pub events: [*mut Event; 2],
}

impl Events {
    /// Use this in your impl of process_events() to process the incoming midi events (on stable).
    ///
    /// # Example
    /// ```no_run
    /// # use vst2::plugin::{Info, Plugin, HostCallback};
    /// # use vst2::buffer::{AudioBuffer, SendEventBuffer};
    /// # use vst2::host::Host;
    /// # use vst2::api;
    /// # use vst2::event::Event;
    /// # struct ExamplePlugin { host: HostCallback, send_buf: SendEventBuffer }
    /// # impl Plugin for ExamplePlugin {
    /// #     fn get_info(&self) -> Info { Default::default() }
    /// #
    /// fn process_events(&mut self, events: &api::Events) {
    ///     for &e in events.events_raw() {
    ///         let event: Event = Event::from(unsafe { *e });
    ///         match event {
    ///             Event::Midi(ev) => {
    ///                 // ...
    ///             }
    ///             _ => ()
    ///         }
    ///     }
    /// }
    /// # }
    /// ```
    #[inline(always)]
    pub fn events_raw(&self) -> &[*const Event] {
        use std::slice;
        unsafe { slice::from_raw_parts(&self.events[0] as *const *mut _ as *const *const _, self.num_events as usize) }
    }

    #[inline(always)]
    pub(crate) fn events_raw_mut(&mut self) -> &mut [*const SysExEvent] {
        use std::slice;
        unsafe { slice::from_raw_parts_mut(&mut self.events[0] as *mut *mut _ as *mut *const _, self.num_events as usize) }
    }

    /// Use this in your impl of process_events() to process the incoming midi events (on nightly).
    ///
    /// # Example
    /// ```no_run
    /// # use vst2::plugin::{Info, Plugin, HostCallback};
    /// # use vst2::buffer::{AudioBuffer, SendEventBuffer};
    /// # use vst2::host::Host;
    /// # use vst2::api;
    /// # use vst2::event::Event;
    /// # struct ExamplePlugin { host: HostCallback, send_buf: SendEventBuffer }
    /// # impl Plugin for ExamplePlugin {
    /// #     fn get_info(&self) -> Info { Default::default() }
    /// #
    /// fn process_events(&mut self, events: &api::Events) {
    ///     for e in events.events() {
    ///         match e {
    ///             Event::Midi { data, .. } => {
    ///                 // ...
    ///             }
    ///             _ => ()
    ///         }
    ///     }
    /// }
    /// # }
    /// ```
    #[cfg(feature = "nightly")]
    #[inline(always)]
    pub fn events<'a>(&'a self) -> impl Iterator<Item = ::event::Event> + 'a {
        self.events_raw().into_iter().map(|&e| ::event::Event::from(unsafe { *e }))
    }
}

/// The type of event that has occured. See `api::Event.event_type`.
#[repr(i32)]
#[derive(Copy, Clone, Debug)]
pub enum EventType {
    /// Midi event. See `api::MidiEvent`.
    Midi = 1,

    /// Deprecated.
    _Audio,
    /// Deprecated.
    _Video,
    /// Deprecated.
    _Parameter,
    /// Deprecated.
    _Trigger,

    /// System exclusive event. See `api::SysExEvent`.
    SysEx,
}

/// A VST event intended to be casted to a corresponding type.
///
/// The event types are not all guaranteed to be the same size, so casting between them can be done
/// via `mem::transmute()` while leveraging pointers, e.g.
///
/// ```
/// # use vst2::api::{Event, EventType, MidiEvent, SysExEvent};
/// # let mut event: *mut Event = &mut unsafe { std::mem::zeroed() };
/// // let event: *const Event = ...;
/// let midi_event: &MidiEvent = unsafe { std::mem::transmute(event) };
/// ```
#[repr(C)]
#[derive(Copy, Clone)]
pub struct Event {
    /// The type of event. This lets you know which event this object should be casted to.
    ///
    /// # Example
    ///
    /// ```
    /// # use vst2::api::{Event, EventType, MidiEvent, SysExEvent};
    /// #
    /// # // Valid for test
    /// # let mut event: *mut Event = &mut unsafe { std::mem::zeroed() };
    /// #
    /// // let mut event: *mut Event = ...
    /// match unsafe { (*event).event_type } {
    ///     EventType::Midi => {
    ///         let midi_event: &MidiEvent = unsafe {
    ///             std::mem::transmute(event)
    ///         };
    ///
    ///         // ...
    ///     }
    ///     EventType::SysEx => {
    ///         let sys_event: &SysExEvent = unsafe {
    ///             std::mem::transmute(event)
    ///         };
    ///
    ///         // ...
    ///     }
    ///     // ...
    /// #     _ => {}
    /// }
    /// ```
    pub event_type: EventType,

    /// Size of this structure; `mem::sizeof::<Event>()`.
    pub byte_size: i32,

    /// Number of samples into the current processing block that this event occurs on.
    ///
    /// E.g. if the block size is 512 and this value is 123, the event will occur on sample
    /// `samples[123]`.
    pub delta_frames: i32,

    /// Generic flags, none defined in VST api yet.
    pub _flags: i32,

    /// The `Event` type is cast appropriately, so this acts as reserved space.
    ///
    /// The actual size of the data may vary as this type is not guaranteed to be the same size as
    /// the other event types.
    pub _reserved: [u8; 16]
}

/// A midi event.
#[repr(C)]
pub struct MidiEvent {
    /// Should be `EventType::Midi`.
    pub event_type: EventType,

    /// Size of this structure; `mem::sizeof::<MidiEvent>()`.
    pub byte_size: i32,

    /// Number of samples into the current processing block that this event occurs on.
    ///
    /// E.g. if the block size is 512 and this value is 123, the event will occur on sample
    /// `samples[123]`.
    pub delta_frames: i32,

    /// See `flags::MidiFlags`.
    pub flags: i32,


    /// Length in sample frames of entire note if available, otherwise 0.
    pub note_length: i32,

    /// Offset in samples into note from start if available, otherwise 0.
    pub note_offset: i32,

    /// 1 to 3 midi bytes. TODO: Doc
    pub midi_data: [u8; 3],

    /// Reserved midi byte (0).
    pub _midi_reserved: u8,

    /// Detuning between -63 and +64 cents, for scales other than 'well-tempered'. e.g. 'microtuning'
    pub detune: i8,

    /// Note off velocity between 0 and 127.
    pub note_off_velocity: u8,

    /// Reserved for future use. Should be 0.
    pub _reserved1: u8,
    /// Reserved for future use. Should be 0.
    pub _reserved2: u8,
}

/// A midi system exclusive event.
///
/// This event only contains raw byte data, and is up to the plugin to interpret it correctly.
/// `plugin::CanDo` has a `ReceiveSysExEvent` variant which lets the host query the plugin as to
/// whether this event is supported.
#[repr(C)]
#[derive(Clone)]
pub struct SysExEvent {
    /// Should be `EventType::SysEx`.
    pub event_type: EventType,

    /// Size of this structure; `mem::sizeof::<SysExEvent>()`.
    pub byte_size: i32,

    /// Number of samples into the current processing block that this event occurs on.
    ///
    /// E.g. if the block size is 512 and this value is 123, the event will occur on sample
    /// `samples[123]`.
    pub delta_frames: i32,

    /// Generic flags, none defined in VST api yet.
    pub _flags: i32,


    /// Size of payload in bytes.
    pub data_size: i32,

    /// Reserved for future use. Should be 0.
    pub _reserved1: isize,

    /// Pointer to payload.
    pub system_data: *mut u8,

    /// Reserved for future use. Should be 0.
    pub _reserved2: isize,
}

/// Bitflags.
pub mod flags {
    bitflags! {
        /// Flags for VST channels.
        pub flags Channel: i32 {
            /// Indicates channel is active. Ignored by host.
            const ACTIVE = 1,
            /// Indicates channel is first of stereo pair.
            const STEREO = 1 << 1,
            /// Use channel's specified speaker_arrangement instead of stereo flag.
            const SPEAKER = 1 << 2
        }
    }

    bitflags! {
        /// Flags for VST plugins.
        pub flags Plugin: i32 {
            /// Plugin has an editor.
            const HAS_EDITOR = 1 << 0,
            /// Plugin can process 32 bit audio. (Mandatory in VST 2.4).
            const CAN_REPLACING = 1 << 4,
            /// Plugin preset data is handled in formatless chunks.
            const PROGRAM_CHUNKS = 1 << 5,
            /// Plugin is a synth.
            const IS_SYNTH = 1 << 8,
            /// Plugin does not produce sound when all input is silence.
            const NO_SOUND_IN_STOP = 1 << 9,
            /// Supports 64 bit audio processing.
            const CAN_DOUBLE_REPLACING = 1 << 12
        }
    }

    bitflags!{
        /// Cross platform modifier key flags.
        pub flags ModifierKey: u8 {
            /// Shift key.
            const SHIFT = 1 << 0,
            /// Alt key.
            const ALT = 1 << 1,
            /// Control on mac.
            const COMMAND = 1 << 2,
            /// Command on mac, ctrl on other.
            const CONTROL = 1 << 3, // Ctrl on PC, Apple on Mac
        }
    }

    bitflags! {
        /// MIDI event flags.
        pub flags MidiEvent: i32 {
            /// This event is played live (not in playback from a sequencer track). This allows the
            /// plugin to handle these flagged events with higher priority, especially when the
            /// plugin has a big latency as per `plugin::Info::initial_delay`.
            const REALTIME_EVENT = 1 << 0,
        }
    }
}
