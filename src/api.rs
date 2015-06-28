//! Structures and types for interfacing with the VST 2.4 API.
use std::mem;

use libc::c_void;

use Vst;
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

/// Host callback function passed to VST. Can be used to query host information from plugin.
pub type HostCallback = fn(effect: *mut AEffect, opcode: i32, index: i32, value: isize, ptr: *mut c_void, opt: f32) -> isize;

/// Dispatcher function used to process opcodes. Called by host as a callback function.
pub type DispatcherProc = fn(effect: *mut AEffect, opcode: i32, index: i32, value: isize, ptr: *mut c_void, opt: f32) -> isize;

/// Process function used to process 32 bit floating point samples.
pub type ProcessProc = fn(effect: *mut AEffect, inputs: *mut *mut f32, outputs: *mut *mut f32, sample_frames: i32);

/// Process function used to process 64 bit floating point samples.
pub type ProcessProcF64 = fn(effect: *mut AEffect, inputs: *mut *mut f64, outputs: *mut *mut f64, sample_frames: i32);

/// Callback function used to set parameter values. Called by host.
pub type SetParameterProc = fn(effect: *mut AEffect, index: i32, parameter: f32);

/// Callback function used to get parameter values. Called by host.
pub type GetParameterProc = fn(effect: *mut AEffect, index: i32) -> f32;

/// Used with the VST API to pass around plugin information.
#[allow(non_snake_case, dead_code)]
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


    /// Bitmask made of values from enums::flags::plugin.
    ///
    /// ```no_run
    /// use vst2::enums::flags::plugin;
    /// let flags = plugin::CAN_REPLACING | plugin::CAN_DOUBLE_REPLACING;
    /// // ...
    /// ```
    pub flags: i32,

    /// Reserved for host, must be 0.
    pub reserved1: isize,

    /// Reserved for host, must be 0.
    pub reserved2: isize,

    /// For algorithms which need input in the first place (Group delay or latency in samples).
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
    /// Return handle to VST object.
    pub unsafe fn get_vst(&mut self) -> &mut Box<Vst> {
        mem::transmute::<*mut c_void, &mut Box<Vst>>(self.object)
    }

    /// Drop the VST object.
    pub unsafe fn drop_vst(&mut self) {
        // Possibly a simpler way of doing this..?
        drop(*mem::transmute::<*mut c_void, Box<Box<Drop>>>(self.object))
    }
}

/// Information about a channel. Only some hosts use this information.
#[allow(dead_code)]
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
