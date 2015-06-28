#![warn(missing_docs)]

//! rust-vst2 is a rust implementation of the VST2.4 API
//!
//! # Plugins
//! All Plugins must implement the `Plugin` trait and `std::default::Default`. The `plugin_main!`
//! macro must also be called in order to export the necessary functions for the plugin to function.
//!
//! ## `Plugin` Trait
//! All methods in this trait have a default implementation except for the `get_info` method which
//! must be implemented by the plugin. Any of the default implementations may be overriden for
//! custom functionality; the defaults do nothing on their own.
//!
//! ## `plugin_main!` macro
//! `plugin_main!` will export the necessary functions to create a proper VST plugin. This must be
//! called with your VST plugin struct name in order for the vst to work.
//!
//! ## Example plugin
//! A barebones VST plugin:
//!
//! ```no_run
//! #[macro_use]
//! extern crate vst2;
//!
//! use vst2::plugin::{Info, Plugin};
//!
//! #[derive(Default)]
//! struct BasicPlugin;
//!
//! impl Plugin for BasicPlugin {
//!     fn get_info(&self) -> Info {
//!         Info {
//!             name: "Basic Plugin".to_string(),
//!             unique_id: 1357, // Used by hosts to differentiate between plugins.
//!
//!             ..Default::default()
//!         }
//!     }
//! }
//!
//! plugin_main!(BasicPlugin); // Important!
//! # fn main() {} // For `extern crate vst2`
//! ```
//!
//! # Hosts
//! Hosts are currently not supported. TODO

extern crate libc;
extern crate num;
#[macro_use] extern crate log;
#[macro_use] extern crate bitflags;

use std::{ptr, mem};

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

pub mod buffer;
pub mod api;
pub mod editor;
pub mod channels;
pub mod host;
pub mod plugin;
mod interfaces;

use api::{HostCallback, AEffect};
use api::consts::VST_MAGIC;
use host::Host;
use plugin::Plugin;

/// Exports the necessary symbols for the plugin to be used by a VST host.
///
/// This macro takes a type which must implement the traits `plugin::Plugin` and
/// `std::default::Default`.
#[macro_export]
macro_rules! plugin_main {
    ($t:ty) => {
        #[cfg(target_os = "macos")]
        #[no_mangle]
        pub extern "system" fn main_macho(callback: $crate::api::HostCallback) -> *mut $crate::api::AEffect {
            VSTPluginMain(callback)
        }

        #[cfg(target_os = "windows")]
        #[allow(non_snake_case)]
        #[no_mangle]
        pub extern "system" fn MAIN(callback: $crate::api::HostCallback) -> *mut $crate::api::AEffect {
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
pub fn main<T: Plugin + Default>(callback: HostCallback) -> *mut AEffect {
    // Create a Box containing a zeroed AEffect. This is transmuted into a *mut pointer so that it
    // can be passed into the Host `wrap` method. The AEffect is then updated after the vst object
    // is created so that the host still contains a raw pointer to the AEffect struct.
    let effect = unsafe { mem::transmute(Box::new(mem::zeroed::<AEffect>())) };

    let host = Host::wrap(callback, effect);
    if host.vst_version() == 0 { // TODO: Better criteria would probably be useful here...
        return ptr::null_mut();
    }

    trace!("Creating VST plugin instance...");
    let mut plugin = <T>::new(host);
    let info = plugin.get_info().clone();

    // Update AEffect in place
    unsafe { *effect = AEffect {
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
            use api::flags::*;

            let mut flag = CAN_REPLACING;

            if info.f64_precision {
                flag = flag | CAN_DOUBLE_REPLACING;
            }

            if plugin.get_editor().is_some() {
                flag = flag | HAS_EDITOR;
            }

            if info.preset_chunks {
                flag = flag | PROGRAM_CHUNKS;
            }

            if let plugin::Category::Synth = info.category {
                flag = flag | IS_SYNTH;
            }

            if info.silent_when_stopped {
                flag = flag | NO_SOUND_IN_STOP;
            }

            flag.bits()
        },

        reserved1: 0,
        reserved2: 0,

        initialDelay: info.initial_delay,

        _realQualities: 0,
        _offQualities: 0,
        _ioRatio: 0.0,

        object: mem::transmute(Box::new(Box::new(plugin) as Box<Plugin>)),
        user: ptr::null_mut(),

        uniqueId: info.unique_id,
        version: info.version,

        processReplacing: interfaces::process_replacing, // fn pointer
        processReplacingF64: interfaces::process_replacing_f64, //fn pointer

        future: [0u8; 56]
    }};
    effect
}

#[cfg(test)]
#[allow(private_no_mangle_fns)] // For `plugin_main!`
mod tests {
    use std::default::Default;
    use std::{mem, ptr};

    use libc::c_void;

    use interfaces;
    use api::AEffect;
    use api::consts::VST_MAGIC;
    use plugin::{Info, Plugin};

    #[derive(Default)]
    struct TestPlugin;

    impl Plugin for TestPlugin {
        fn get_info(&self) -> Info {
            Info {
                name: "Test Plugin".to_string(),
                vendor: "overdrivenpotato".to_string(),

                presets: 1,
                parameters: 1,

                unique_id: 5678,
                version: 1234,

                initial_delay: 123,

                ..Default::default()
            }
        }
    }

    plugin_main!(TestPlugin);

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
    fn plugin_drop() {
        static mut drop_test: bool = false;

        impl Drop for TestPlugin {
            fn drop(&mut self) {
                unsafe { drop_test = true; }
            }
        }

        let aeffect = VSTPluginMain(pass_callback);
        assert!(!aeffect.is_null());

        unsafe { (*aeffect).drop_plugin() };

        // Assert that the VST is shut down and dropped.
        assert!(unsafe { drop_test });
    }

    #[test]
    fn plugin_no_drop() {
        let aeffect = VSTPluginMain(pass_callback);
        assert!(!aeffect.is_null());

        // Make sure this doesn't crash.
        unsafe { (*aeffect).drop_plugin() };
    }

    #[test]
    fn plugin_deref() {
        let aeffect = VSTPluginMain(pass_callback);
        assert!(!aeffect.is_null());

        let plugin = unsafe { (*aeffect).get_plugin() };
        // Assert that deref works correctly.
        assert!(plugin.get_info().name == "Test Plugin");
    }

    #[test]
    fn aeffect_params() {
        // Assert that 2 function pointers are equal.
        macro_rules! assert_fn_eq {
            ($a:expr, $b:expr) => {
                unsafe {
                    assert_eq!(
                        mem::transmute::<_, usize>($a),
                        mem::transmute::<_, usize>($b)
                    );
                }
            }
        }

        let aeffect = unsafe { &mut *VSTPluginMain(pass_callback) };

        assert_eq!(aeffect.magic, VST_MAGIC);
        assert_fn_eq!(aeffect.dispatcher, interfaces::dispatch);
        assert_fn_eq!(aeffect._process, interfaces::process_deprecated);
        assert_fn_eq!(aeffect.setParameter, interfaces::set_parameter);
        assert_fn_eq!(aeffect.getParameter, interfaces::get_parameter);
        assert_eq!(aeffect.numPrograms, 1);
        assert_eq!(aeffect.numParams, 1);
        assert_eq!(aeffect.numInputs, 2);
        assert_eq!(aeffect.numOutputs, 2);
        assert_eq!(aeffect.reserved1, 0);
        assert_eq!(aeffect.reserved2, 0);
        assert_eq!(aeffect.initialDelay, 123);
        assert_eq!(aeffect.uniqueId, 5678);
        assert_eq!(aeffect.version, 1234);
        assert_fn_eq!(aeffect.processReplacing, interfaces::process_replacing);
        assert_fn_eq!(aeffect.processReplacingF64, interfaces::process_replacing_f64);
    }
}
