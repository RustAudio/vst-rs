//! Structures and types for interfacing with the VST 2.4 API.
use std::{mem, ptr};

use libc::c_void;

use Vst;
use self::consts::*;

/// Constant values
#[allow(dead_code)]
pub mod consts {
    pub const MAX_PRESET_NAME_LEN: u64 = 24;
    pub const MAX_PARAM_STR_LEN: u64 = 8;
    pub const MAX_LABEL: u64 = 64;
    pub const MAX_SHORT_LABEL: u64 = 8;
    pub const MAX_PRODUCT_STR_LEN: u64 = 64;
    pub const MAX_VENDOR_STR_LEN: u64 = 64;
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
}

impl Drop for AEffect {
    fn drop(&mut self) {
        unsafe {
            drop(Box::from_raw(
                mem::transmute::<&mut Box<Vst>, &mut Box<Drop>>(self.get_vst())
            ));
            ptr::read_and_zero(self.object);
        }
    }
}

// TODO: Implement this struct.
#[allow(dead_code)]
#[repr(C)]
struct VstPinProperties {
    label: [char; MAX_LABEL as usize], //pin name
    flags: i32, //See `VstPinPropertiesFlags`
    speaker_arrangement: i32, //See `VstSpeakerArragement`
    short_name: [char; MAX_SHORT_LABEL as usize], //Short name (recommended: 6 + delimiter)

    future: [char; 48] //Reserved for future use
}
