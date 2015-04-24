#![feature(libc, alloc, core, collections)]
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
//! use std::default::Default;
//!
//! use vst2::{Vst, Info};
//!
//! struct BasicVst {
//!     info: Info
//! }
//!
//! impl Vst for BasicVst {
//!     fn get_info(&mut self) -> &mut Info {
//!         &mut self.info
//!     }
//! }
//!
//! impl Default for BasicVst {
//!     fn default() -> BasicVst {
//!         BasicVst {
//!             info: Info {
//!                 name: "BasicVst".to_string(),
//!
//!                 ..Default::default()
//!             }
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

extern crate collections;
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
mod interfaces;

use enums::flags::plugin::*;
use enums::{KnobMode, PluginCategory, CanDo, Supported};
use api::{KeyCode, HostCallback, AEffect};
pub use buffer::AudioBuffer;

/// VST plugins are identified by a magic number. This corresponds to 0x56737450.
const VST_MAGIC: i32 = ('V' as i32) << 24 |
                       ('s' as i32) << 16 |
                       ('t' as i32) << 8  |
                       ('P' as i32) << 0  ;

/// Exports the necessary symbols for the plugin to be used by a vst host.
///
/// This macro takes a type which must implement the traits `Vst` and `std::default::Default`.
#[macro_export]
macro_rules! vst_main {
    ($t:ty) => {
        #[cfg(target_os = "mac")]
        pub extern "system" fn main_macho (callback: $crate::callbacks::HostCallback) -> *mut $crate::api::AEffect {
            VSTPluginMain(callback)
        }

        #[cfg(target_os = "windows")]
        pub extern "system" fn MAIN (callback: $crate::callbacks::HostCallback) -> *mut $crate::api::AEffect {
            VSTPluginMain(callback)
        }

        #[allow(non_snake_case, unused_variables)]
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
    trace!("Creating VST Instance...");

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

        initialDelay: 0,

        _realQualities: 0,
        _offQualities: 0,
        _ioRatio: 0.0,

        object: mem::transmute(Box::new(Box::new(vst) as Box<Vst>)),
        user: ptr::null_mut(),

        uniqueId: 0,
        version: 0001,

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

    /// Plugin version
    ///
    /// # Examples
    /// * 0001 = `v0.0.0.1`
    /// * 1283 = `v1.2.8.3`
    pub version: i32,

    /// Plugin category. Possible values are found in `enums::PluginCategory`
    pub category: PluginCategory,

    //TODO: Doc
    pub initial_delay: i32,

    /// Indicates whether preset data is handled in formatless chunks. Default is `true`.
    pub preset_chunks: bool,

    /// Indicates whether this plugin can process f64 based `AudioBuffer` buffers. Default is
    /// `false`.
    pub f64_precision: bool,

    //no_sound_in_stop: bool, //TODO: Implement this somehow
}

impl Default for Info {
    fn default() -> Info {
        Info {
            name: String::from_str("VST"),
            vendor: String::new(),

            presets: 1, // default preset
            parameters: 0,
            inputs: 2, // Stereo in,out
            outputs: 2,

            unique_id: 0,
            version: 0001, // v0.0.0.1

            category: PluginCategory::Effect,

            initial_delay: 0,

            preset_chunks: true,
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
    /// This method must return a mutable reference to an `Info` struct.
    fn get_info(&mut self) -> &mut Info;


    /// Called when VST is initialized.
    fn init(&mut self) { trace!("Initialized vst."); }


    /// Set the current preset to the index specified by `preset`.
    fn set_preset(&mut self, preset: i32) { }

    /// Get the current preset index.
    fn get_preset_num(&mut self) -> i32 { 0 }

    /// Set the current preset name.
    fn set_preset_name(&mut self, name: String) { }

    /// Get the name of the preset at the index specified by `preset`.
    fn get_preset_name(&mut self, preset: i32) -> String { "".to_string() }


    /// Get parameter label for parameter at `index` (e.g. "db", "sec", "ms", "%").
    fn get_parameter_label(&mut self, index: i32) -> String { "".to_string() }

    /// Get the parameter value for parameter at `index` (e.g. "1.0", "150", "Plate", "Off").
    fn get_parameter_text(&mut self, index: i32) -> String {
        format!("{:.3}", self.get_parameter(index))
    }

    /// Get the name of parameter at `index`.
    fn get_parameter_name(&mut self, index: i32) -> String { format!("Param {}", index) }

    /// Get the value of paramater at `index`. Should be value between 0.0 and 1.0.
    fn get_parameter(&mut self, index: i32) -> f32 { 0.0 }

    /// Set the value of parameter at `index`. `value` is between 0.0 and 1.0.
    fn set_parameter(&mut self, index: i32, value: f32) { }

    /// Return whether parameter at `index` can be automated.
    fn can_be_automated(&mut self, index: i32) -> bool { false }


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
    fn can_do(&mut self, can_do: CanDo) -> Supported {
        info!("Host is asking if plugin can: {:?}.", can_do);
        Supported::Maybe
    }

    /// Get the tail size of plugin when it is stopped. Used in offline processing as well.
    fn get_tail_size(&mut self) -> isize { 0 }


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

    /// Return handle to plugin editor. //TODO: Unimplemented
    #[doc(hidden)]
    fn get_editor(&mut self) -> Option<&mut Editor> { None }
}

/// Implemented by plugin editors. //TODO: Implement editors in editor specific module
#[doc(hidden)]
pub trait Editor {
    fn size(&self) -> (i32, i32);
    fn position(&self) -> (i32, i32);

    fn idle(&mut self);
    fn close(&mut self);
    fn open(&mut self, window: *mut c_void);

    fn set_knob_mode(&mut self, mode: KnobMode);
    fn key_up(&mut self, keycode: KeyCode);
    fn key_down(&mut self, keycode: KeyCode);
}

#[cfg(test)]
mod test {
    use std::default::Default;
    use std::ptr;

    use libc::c_void;

    use Vst;
    use Info;
    use main;
    use api::AEffect;

    struct TestVst { info: Info }

    impl Vst for TestVst {
        fn get_info(&mut self) -> &mut Info {
            &mut self.info
        }
    }

    impl Default for TestVst {
        fn default() -> TestVst {
            TestVst {
                info: Info {
                    name: "TestVST".to_string(),
                    ..Default::default()
                }
            }
        }
    }

    fn pass_callback(_effect: *mut AEffect, _opcode: i32, _index: i32, _value: isize, _ptr: *mut c_void, _opt: f32) -> isize {
        1
    }

    fn fail_callback(_effect: *mut AEffect, _opcode: i32, _index: i32, _value: isize, _ptr: *mut c_void, _opt: f32) -> isize {
        0
    }

    #[test]
    fn host_callback() {
        assert_eq!(main::<TestVst>(fail_callback), ptr::null_mut());
    }

    #[test]
    fn aeffect_created() {
        let aeffect = main::<TestVst>(pass_callback);
        assert!(!aeffect.is_null());
    }

    #[test]
    fn vst_drop() {
        use std::mem;

        static mut drop_test: bool = false;

        impl Drop for TestVst {
            fn drop(&mut self) {
                unsafe { drop_test = true; }
            }
        }

        let aeffect = main::<TestVst>(pass_callback);
        assert!(!aeffect.is_null());

        unsafe { drop(mem::transmute::<*mut AEffect, Box<AEffect>>(aeffect)) };

        // Assert that the VST is shut down and dropped.
        assert!(unsafe { drop_test });
    }

    #[test]
    fn vst_deref() {
        let aeffect = main::<TestVst>(pass_callback);
        assert!(!aeffect.is_null());

        let vst = unsafe { (*aeffect).get_vst() };
        // Assert that deref works correctly.
        assert!(vst.get_info().name == "TestVST");
    }
}
