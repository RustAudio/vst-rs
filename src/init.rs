//! Entry point for initializing a VST plugin

use api::consts::VST_MAGIC;
use api::{AEffect, HostCallbackProc};
use cache::PluginCache;
use interfaces;
use plugin::{self, HostCallback, Plugin};
use std::ptr;

/// Exports the necessary symbols for the plugin to be used by a VST host.
///
/// This macro takes a type which must implement the `Plugin` trait.
#[macro_export]
macro_rules! plugin_main {
    ($t:ty) => {
        #[cfg(target_os = "macos")]
        #[no_mangle]
        pub extern "system" fn main_macho(callback: $crate::api::HostCallbackProc) -> *mut $crate::api::AEffect {
            VSTPluginMain(callback)
        }

        #[cfg(target_os = "windows")]
        #[allow(non_snake_case)]
        #[no_mangle]
        pub extern "system" fn MAIN(callback: $crate::api::HostCallbackProc) -> *mut $crate::api::AEffect {
            VSTPluginMain(callback)
        }

        #[allow(non_snake_case)]
        #[no_mangle]
        pub extern "C" fn VSTPluginMain(callback: $crate::api::HostCallbackProc) -> *mut $crate::api::AEffect {
            $crate::init::main::<$t>(callback)
        }
    };
}

/// Initializes a VST plugin and returns a raw pointer to an AEffect struct.
#[doc(hidden)]
pub fn main<T: Plugin>(callback: HostCallbackProc) -> *mut AEffect {
    // Initialize as much of the AEffect as we can before creating the plugin.
    // In particular, initialize all the function pointers, since initializing
    // these to zero is undefined behavior.
    let boxed_effect = Box::new(AEffect {
        magic: VST_MAGIC,
        dispatcher: interfaces::dispatch, // fn pointer

        _process: interfaces::process_deprecated, // fn pointer

        setParameter: interfaces::set_parameter, // fn pointer
        getParameter: interfaces::get_parameter, // fn pointer

        numPrograms: 0, // To be updated with plugin specific value.
        numParams: 0,   // To be updated with plugin specific value.
        numInputs: 0,   // To be updated with plugin specific value.
        numOutputs: 0,  // To be updated with plugin specific value.

        flags: 0, // To be updated with plugin specific value.

        reserved1: 0,
        reserved2: 0,

        initialDelay: 0, // To be updated with plugin specific value.

        _realQualities: 0,
        _offQualities: 0,
        _ioRatio: 0.0,

        object: ptr::null_mut(),
        user: ptr::null_mut(),

        uniqueId: 0, // To be updated with plugin specific value.
        version: 0,  // To be updated with plugin specific value.

        processReplacing: interfaces::process_replacing, // fn pointer
        processReplacingF64: interfaces::process_replacing_f64, //fn pointer

        future: [0u8; 56],
    });
    let raw_effect = Box::into_raw(boxed_effect);

    let host = HostCallback::wrap(callback, raw_effect);
    if host.vst_version() == 0 {
        // TODO: Better criteria would probably be useful here...
        return ptr::null_mut();
    }

    trace!("Creating VST plugin instance...");
    let mut plugin = T::new(host);
    let info = plugin.get_info();
    let params = plugin.get_parameter_object();
    let editor = plugin.get_editor();

    // Update AEffect in place
    let effect = unsafe { &mut *raw_effect };
    effect.numPrograms = info.presets;
    effect.numParams = info.parameters;
    effect.numInputs = info.inputs;
    effect.numOutputs = info.outputs;
    effect.flags = {
        use api::PluginFlags;

        let mut flag = PluginFlags::CAN_REPLACING;

        if info.f64_precision {
            flag |= PluginFlags::CAN_DOUBLE_REPLACING;
        }

        if editor.is_some() {
            flag |= PluginFlags::HAS_EDITOR;
        }

        if info.preset_chunks {
            flag |= PluginFlags::PROGRAM_CHUNKS;
        }

        if let plugin::Category::Synth = info.category {
            flag |= PluginFlags::IS_SYNTH;
        }

        if info.silent_when_stopped {
            flag |= PluginFlags::NO_SOUND_IN_STOP;
        }

        flag.bits()
    };
    effect.initialDelay = info.initial_delay;
    effect.object = Box::into_raw(Box::new(Box::new(plugin) as Box<dyn Plugin>)) as *mut _;
    effect.user = Box::into_raw(Box::new(PluginCache::new(&info, params, editor))) as *mut _;
    effect.uniqueId = info.unique_id;
    effect.version = info.version;

    effect
}

#[cfg(test)]
mod tests {
    use std::ptr;

    use std::os::raw::c_void;

    use api::consts::VST_MAGIC;
    use api::AEffect;
    use interfaces;
    use plugin::{HostCallback, Info, Plugin};

    struct TestPlugin;

    impl Plugin for TestPlugin {
        fn new(_host: HostCallback) -> Self {
            TestPlugin
        }

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

    extern "C" fn pass_callback(
        _effect: *mut AEffect,
        _opcode: i32,
        _index: i32,
        _value: isize,
        _ptr: *mut c_void,
        _opt: f32,
    ) -> isize {
        1
    }

    extern "C" fn fail_callback(
        _effect: *mut AEffect,
        _opcode: i32,
        _index: i32,
        _value: isize,
        _ptr: *mut c_void,
        _opt: f32,
    ) -> isize {
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
        static mut DROP_TEST: bool = false;

        impl Drop for TestPlugin {
            fn drop(&mut self) {
                unsafe {
                    DROP_TEST = true;
                }
            }
        }

        let aeffect = VSTPluginMain(pass_callback);
        assert!(!aeffect.is_null());

        unsafe { (*aeffect).drop_plugin() };

        // Assert that the VST is shut down and dropped.
        assert!(unsafe { DROP_TEST });
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
                assert_eq!($a as usize, $b as usize);
            };
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
