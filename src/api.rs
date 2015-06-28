//! Structures and types for interfacing with the VST 2.4 API.
use std::mem;

use libc::c_void;

use plugin::Plugin;
use self::consts::*;

/// Constant values
pub mod consts {
    use libc::size_t;

    pub const MAX_PRESET_NAME_LEN: size_t = 24;
    pub const MAX_PARAM_STR_LEN: size_t = 8;
    pub const MAX_LABEL: usize = 64;
    pub const MAX_SHORT_LABEL: usize = 8;
    pub const MAX_PRODUCT_STR_LEN: size_t = 64;
    pub const MAX_VENDOR_STR_LEN: size_t = 64;

    /// VST plugins are identified by a magic number. This corresponds to 0x56737450.
    pub const VST_MAGIC: i32 = ('V' as i32) << 24 |
                               ('s' as i32) << 16 |
                               ('t' as i32) << 8  |
                               ('P' as i32) << 0  ;
}

/// Host callback function passed to plugin. Can be used to query host information from plugin side.
pub type HostCallback = fn(effect: *mut AEffect, opcode: i32, index: i32, value: isize, ptr: *mut c_void, opt: f32) -> isize;

/// Dispatcher function used to process opcodes. Called by host.
pub type DispatcherProc = fn(effect: *mut AEffect, opcode: i32, index: i32, value: isize, ptr: *mut c_void, opt: f32) -> isize;

/// Process function used to process 32 bit floating point samples. Called by host.
pub type ProcessProc = fn(effect: *mut AEffect, inputs: *mut *mut f32, outputs: *mut *mut f32, sample_frames: i32);

/// Process function used to process 64 bit floating point samples. Called by host.
pub type ProcessProcF64 = fn(effect: *mut AEffect, inputs: *mut *mut f64, outputs: *mut *mut f64, sample_frames: i32);

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
        mem::transmute::<_, &mut Box<Plugin>>(self.object)
    }

    /// Drop the Plugin object. Only works for plugins created using this library.
    pub unsafe fn drop_plugin(&mut self) {
        // Possibly a simpler way of doing this..?
        drop(mem::transmute::<_, Box<Box<Plugin>>>(self.object))
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
pub mod flags {
    bitflags! {
        /// Flags for VST channels.
        flags Channel: i32 {
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
        flags Plugin: i32 {
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
