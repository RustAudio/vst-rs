#![warn(missing_docs)]

//! rust-vst2 is a rust implementation of the VST2.4 API
//!
//! # Plugins
//! All Plugins must implement the `Vst` trait and `std::default::Default`. The `vst_main!` macro
//! must also be called in order to export the necessary functions for the VST to function.
//!
//! ## `Vst` Trait
//! All methods in this trait have a default implementation except for the `get_info` method which
//! must be implemented by the Vst object. Any of the default implementations may be overriden for
//! custom functionality; the defaults do nothing on their own.
//!
//! ## `vst_main!` macro
//! `vst_main!` will export the necessary functions to create a proper VST. This must be called
//! with your VST struct name in order for the vst to work.
//!
//! ## Example plugin
//! A barebones VST plugin:
//!
//! ```no_run
//! #[macro_use]
//! extern crate vst2;
//!
//! use vst2::{Vst, Info};
//!
//! #[derive(Default)]
//! struct BasicVst;
//!
//! impl Vst for BasicVst {
//!     fn get_info(&self) -> Info {
//!         Info {
//!             name: "BasicVst".to_string(),
//!             unique_id: 1357, // Used by hosts to differentiate between plugins.
//!
//!             ..Default::default()
//!         }
//!     }
//! }
//!
//! vst_main!(BasicVst); //Important!
//! # fn main() {} //no_run
//! ```
//!
//! # Hosts
//! Hosts are currently not supported. TODO

extern crate libc;
extern crate num;
#[macro_use] extern crate log;
#[macro_use] extern crate bitflags;

use std::default::Default;
use std::{ptr, mem};
use std::iter::IntoIterator;

use libc::c_void;

pub mod buffer;
pub mod enums;
pub mod api;
pub mod editor;
pub mod channels;
mod interfaces;

use enums::flags::plugin::*;
use enums::{PluginCategory, CanDo, Supported};
use api::{HostCallback, AEffect};
pub use buffer::AudioBuffer;
use editor::Editor;
use channels::ChannelInfo;

/// VST plugins are identified by a magic number. This corresponds to 0x56737450.
pub const VST_MAGIC: i32 = ('V' as i32) << 24 |
                           ('s' as i32) << 16 |
                           ('t' as i32) << 8  |
                           ('P' as i32) << 0  ;

/// Exports the necessary symbols for the plugin to be used by a vst host.
///
/// This macro takes a type which must implement the traits `Vst` and `std::default::Default`.
#[macro_export]
macro_rules! vst_main {
    ($t:ty) => {
        #[cfg(target_os = "macos")]
        #[no_mangle]
        pub extern "system" fn main_macho (callback: $crate::api::HostCallback) -> *mut $crate::api::AEffect {
            VSTPluginMain(callback)
        }

        #[cfg(target_os = "windows")]
        #[allow(non_snake_case)]
        #[no_mangle]
        pub extern "system" fn MAIN (callback: $crate::api::HostCallback) -> *mut $crate::api::AEffect {
            VSTPluginMain(callback)
        }

        #[allow(non_snake_case)]
        #[no_mangle]
        pub extern "system" fn VSTPluginMain(callback: $crate::api::HostCallback) -> *mut $crate::api::AEffect {
            $crate::main::<$t>(callback)
        }
    }
}

/// Initializes a VST plugin and returns a raw pointer to an AEffect struct.
#[doc(hidden)]
pub fn main<T: Vst + Default>(callback: HostCallback) -> *mut AEffect {
    // 1 is audioMasterVersion TODO: abstract this
    if callback(ptr::null_mut(), 1, 0, 0, ptr::null_mut(), 0.0) == 0 {
        return ptr::null_mut();
    }

    let mut vst: T = Default::default();
    let info = vst.get_info().clone();
    trace!("Creating VST plugin instance...");

    unsafe { mem::transmute(Box::new(AEffect {
        magic: VST_MAGIC,
        dispatcher: interfaces::dispatch, // fn pointer

        _process: interfaces::process_deprecated, // fn pointer

        setParameter: interfaces::set_parameter, // fn pointer
        getParameter: interfaces::get_parameter, // fn pointer

        numPrograms: info.presets,
        numParams: info.parameters,
        numInputs: info.inputs,
        numOutputs: info.outputs,

        flags: {
            let mut flag = CAN_REPLACING;

            if info.f64_precision {
                flag = flag | CAN_DOUBLE_REPLACING;
            }

            if vst.get_editor().is_some() {
                flag = flag | HAS_EDITOR;
            }

            if info.preset_chunks {
                flag = flag | PROGRAM_CHUNKS;
            }

            if let PluginCategory::Synth = info.category {
                flag = flag | IS_SYNTH;
            }

            flag.bits()
        },

        reserved1: 0,
        reserved2: 0,

        initialDelay: info.initial_delay,

        _realQualities: 0,
        _offQualities: 0,
        _ioRatio: 0.0,

        object: mem::transmute(Box::new(Box::new(vst) as Box<Vst>)),
        user: ptr::null_mut(),

        uniqueId: info.unique_id,
        version: info.version,

        processReplacing: interfaces::process_replacing, // fn pointer
        processReplacingF64: interfaces::process_replacing_f64, //fn pointer

        future: [0u8; 56]
    })) }
}

/// A structure representing static plugin information.
#[derive(Clone, Debug)]
pub struct Info {
    /// Plugin Name.
    pub name: String,

    /// Plugin Vendor.
    pub vendor: String,


    /// Number of different presets.
    pub presets: i32,

    /// Number of parameters.
    pub parameters: i32,


    /// Number of inputs.
    pub inputs: i32,

    /// Number of outputs.
    pub outputs: i32,


    /// Unique plugin ID. Can be registered with Steinberg to prevent conflicts with other plugins.
    ///
    /// This ID is used to identify a plugin during save and load of a preset and project.
    pub unique_id: i32,

    /// Plugin version (e.g. 0001 = `v0.0.0.1`, 1283 = `v1.2.8.3`).
    pub version: i32,

    /// Plugin category. Possible values are found in `enums::PluginCategory`.
    pub category: PluginCategory,

    //TODO: Doc
    pub initial_delay: i32,

    /// Indicates whether preset data is handled in formatless chunks. If false,
    /// host saves and restores plugins by reading/writing parameter data. If true, it is up to
    /// the plugin to manage saving preset data by implementing  the
    /// `{get, load}_{preset, bank}_chunks()` methods. Default is `false`.
    pub preset_chunks: bool,

    /// Indicates whether this plugin can process f64 based `AudioBuffer` buffers. Default is
    /// `false`.
    pub f64_precision: bool,

    //no_sound_in_stop: bool, //TODO: Implement this somehow
}

impl Default for Info {
    fn default() -> Info {
        Info {
            name: "VST".to_string(),
            vendor: String::new(),

            presets: 1, // default preset
            parameters: 0,
            inputs: 2, // Stereo in,out
            outputs: 2,

            unique_id: 0, // This must be changed.
            version: 0001, // v0.0.0.1

            category: PluginCategory::Effect,

            initial_delay: 0,

            preset_chunks: false,
            f64_precision: false,
        }
    }
}

/// Must be implemented by all VST plugins.
///
/// All methods except `get_info` provide a default implementation which does nothing and can be
/// safely overridden.
#[allow(unused_variables)]
pub trait Vst {
    /// This method must return an `Info` struct.
    fn get_info(&self) -> Info;


    /// Called when VST is initialized.
    fn init(&mut self) { trace!("Initialized vst plugin."); }


    /// Set the current preset to the index specified by `preset`.
    fn change_preset(&mut self, preset: i32) { }

    /// Get the current preset index.
    fn get_preset_num(&self) -> i32 { 0 }

    /// Set the current preset name.
    fn set_preset_name(&self, name: String) { }

    /// Get the name of the preset at the index specified by `preset`.
    fn get_preset_name(&self, preset: i32) -> String { "".to_string() }


    /// Get parameter label for parameter at `index` (e.g. "db", "sec", "ms", "%").
    fn get_parameter_label(&self, index: i32) -> String { "".to_string() }

    /// Get the parameter value for parameter at `index` (e.g. "1.0", "150", "Plate", "Off").
    fn get_parameter_text(&self, index: i32) -> String {
        format!("{:.3}", self.get_parameter(index))
    }

    /// Get the name of parameter at `index`.
    fn get_parameter_name(&self, index: i32) -> String { format!("Param {}", index) }

    /// Get the value of paramater at `index`. Should be value between 0.0 and 1.0.
    fn get_parameter(&self, index: i32) -> f32 { 0.0 }

    /// Set the value of parameter at `index`. `value` is between 0.0 and 1.0.
    fn set_parameter(&mut self, index: i32, value: f32) { }

    /// Return whether parameter at `index` can be automated.
    fn can_be_automated(&self, index: i32) -> bool { false }

    /// Use String as input for parameter value. Used by host to provide an editable field to
    /// adjust a parameter value. E.g. "100" may be interpreted as 100hz for parameter. Returns if
    /// the input string was used.
    fn string_to_parameter(&self, index: i32, text: String) -> bool { false }


    /// Called when sample rate is changed by host.
    fn sample_rate_changed(&mut self, rate: f32) { }

    /// Called when block size is changed by host.
    fn block_size_changed(&mut self, size: i64) { }


    /// Called when plugin is turned on.
    fn on_resume(&mut self) { }

    /// Called when plugin is turned off.
    fn on_suspend(&mut self) { }


    /// Vendor specific handling.
    fn vendor_specific(&mut self, index: i32, value: isize, ptr: *mut c_void, opt: f32) { }


    /// Return whether plugin supports specified action.
    fn can_do(&self, can_do: CanDo) -> Supported {
        info!("Host is asking if plugin can: {:?}.", can_do);
        Supported::Maybe
    }

    /// Get the tail size of plugin when it is stopped. Used in offline processing as well.
    fn get_tail_size(&self) -> isize { 0 }


    /// Process an audio buffer containing `f32` values. TODO: Examples
    fn process(&mut self, buffer: AudioBuffer<f32>) {
        // For each input and output
        for (input, output) in buffer.zip() {
            // For each input sample and output sample in buffer
            for (in_frame, out_frame) in input.into_iter().zip(output.into_iter()) {
                *out_frame = *in_frame;
            }
        }
    }

    /// Process an audio buffer containing `f64` values. TODO: Examples
    fn process_f64(&mut self, buffer: AudioBuffer<f64>) {
        // For each input and output
        for (input, output) in buffer.zip() {
            // For each input sample and output sample in buffer
            for (in_frame, out_frame) in input.into_iter().zip(output.into_iter()) {
                *out_frame = *in_frame;
            }
        }
    }

    /// Return handle to plugin editor if supported.
    fn get_editor(&mut self) -> Option<&mut Editor> { None }


    /// If `preset_chunks` is set to true in plugin info, this should return the raw chunk data for
    /// the current preset.
    fn get_preset_data(&mut self) -> Vec<u8> { Vec::new() }

    /// If `preset_chunks` is set to true in plugin info, this should return the raw chunk data for
    /// the current plugin bank.
    fn get_bank_data(&mut self) -> Vec<u8> { Vec::new() }

    /// If `preset_chunks` is set to true in plugin info, this should load a preset from the given
    /// chunk data.
    fn load_preset_data(&mut self, data: Vec<u8>) {}

    /// If `preset_chunks` is set to true in plugin info, this should load a preset bank from the
    /// given chunk data.
    fn load_bank_data(&mut self, data: Vec<u8>) {}

    /// Get information about an input channel. Only used by some hosts.
    fn get_input_info(&self, input: i32) -> ChannelInfo {
        ChannelInfo::new(format!("Input channel {}", input),
                         Some(format!("In {}", input)),
                         true, None)
    }

    /// Get information about an output channel. Only used by some hosts.
    fn get_output_info(&self, output: i32) -> ChannelInfo {
        ChannelInfo::new(format!("Output channel {}", output),
                         Some(format!("Out {}", output)),
                         true, None)
    }
}


#[cfg(test)]
#[allow(private_no_mangle_fns)] // For `vst_main!`
mod tests {
    use std::default::Default;
    use std::ptr;

    use libc::c_void;

    use Vst;
    use Info;
    use api::AEffect;

    #[derive(Default)]
    struct TestVst;

    impl Vst for TestVst {
        fn get_info(&self) -> Info {
            Info {
                name: "TestVST".to_string(),
                ..Default::default()
            }
        }
    }

    vst_main!(TestVst);

    fn pass_callback(_effect: *mut AEffect, _opcode: i32, _index: i32, _value: isize, _ptr: *mut c_void, _opt: f32) -> isize {
        1
    }

    fn fail_callback(_effect: *mut AEffect, _opcode: i32, _index: i32, _value: isize, _ptr: *mut c_void, _opt: f32) -> isize {
        0
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn old_hosts() {
        assert_eq!(MAIN(fail_callback), ptr::null_mut());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn old_hosts() {
        assert_eq!(main_macho(fail_callback), ptr::null_mut());
    }

    #[test]
    fn host_callback() {
        assert_eq!(VSTPluginMain(fail_callback), ptr::null_mut());
    }

    #[test]
    fn aeffect_created() {
        let aeffect = VSTPluginMain(pass_callback);
        assert!(!aeffect.is_null());
    }

    #[test]
    fn vst_drop() {
        static mut drop_test: bool = false;

        impl Drop for TestVst {
            fn drop(&mut self) {
                unsafe { drop_test = true; }
            }
        }

        let aeffect = VSTPluginMain(pass_callback);
        assert!(!aeffect.is_null());

        unsafe { (*aeffect).drop_vst() };

        // Assert that the VST is shut down and dropped.
        assert!(unsafe { drop_test });
    }

    #[test]
    fn vst_no_drop() {
        let aeffect = VSTPluginMain(pass_callback);
        assert!(!aeffect.is_null());

        // Make sure this doesn't crash.
        unsafe { (*aeffect).drop_vst() };
    }

    #[test]
    fn vst_deref() {
        let aeffect = VSTPluginMain(pass_callback);
        assert!(!aeffect.is_null());

        let vst = unsafe { (*aeffect).get_vst() };
        // Assert that deref works correctly.
        assert!(vst.get_info().name == "TestVST");
    }
}
