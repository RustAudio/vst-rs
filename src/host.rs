//! Wrapper around host callback for plugins.
//!
//! Helpful in facilitating communcation between plugin and host from the plugin side.
use std::{mem, ptr};

use libc::c_void;

use api::{AEffect, HostCallback};
use api::consts::VST_MAGIC;

#[repr(usize)]
#[derive(Clone, Copy, Debug)]
#[doc(hidden)]
pub enum OpCode {
    /// [index]: parameter index
    /// [opt]: parameter value
    Automate = 0,
    /// [return]: host vst version (e.g. 2400 for VST 2.4)
    Version,
    /// [return]: current plugin ID (useful for shell plugins to figure out which plugin to load in
    ///           `VSTPluginMain()`).
    CurrentId,
    /// No arguments. Give idle time to Host application, e.g. if plug-in editor is doing mouse
    /// tracking in a modal loop.
    Idle,
    /// Deprecated.
    _PinConnected = 4,

    /// Deprecated.
    _WantMidi = 6, // Not a typo
    /// [value]: request mask. see `VstTimeInfoFlags`
    /// [return]: `VstTimeInfo` pointer or null if not supported.
    GetTime,
    /// Deprecated.
    _SetTime,
    /// Deprecated.
    _TempoAt,
    /// Deprecated.
    _GetNumAutomatableParameters,
    /// Deprecated.
    _GetParameterQuantization,

    /// Notifies the host that the input/output setup has changed. This can allow the host to check
    /// numInputs/numOutputs or call `getSpeakerArrangement()`
    /// [return]: 1 if supported.
    IOChanged,

    /// Deprecated.
    _NeedIdle,
}
impl_clike!(OpCode);

/// A reference to the host which allows the plugin to call back and access information.
///
/// # Panics
///
/// All methods in this struct will panic if the plugin has not yet been initialized. In practice,
/// this can only occur if the plugin queries the host for information when `Default::default()` is
/// called.
///
/// ```should_panic
/// # use vst2::plugin::{Info, Plugin};
/// # use vst2::host::Host;
/// struct ExamplePlugin;
///
/// impl Default for ExamplePlugin {
///     fn default() -> ExamplePlugin {
///         // Will panic, don't do this. If needed, you can query
///         // the host during initialization via Vst::new()
///         let host: Host = Default::default();
///         let version = host.vst_version();
///
///         // ...
/// #         ExamplePlugin
///     }
/// }
/// #
/// # impl Plugin for ExamplePlugin {
/// #     fn get_info(&self) -> Info { Default::default() }
/// # }
/// # fn main() { let plugin: ExamplePlugin = Default::default(); }
/// ```
pub struct Host {
    callback: Option<HostCallback>,
    effect: *mut AEffect,
}

/// `Host` implements `Default` so that the plugin can implement `Default` and have a `Host` field.
impl Default for Host {
    fn default() -> Host {
        Host {
            callback: None,
            effect: ptr::null_mut(),
        }
    }
}

impl Host {
    /// Wrap callback in a function to avoid using fn pointer notation.
    #[doc(hidden)]
    fn callback(&self,
                effect: *mut AEffect,
                opcode: OpCode,
                index: i32,
                value: isize,
                ptr: *mut c_void,
                opt: f32)
                -> isize {
        let callback = self.callback.unwrap_or_else(|| panic!("Host not yet initialized."));
        callback(effect, opcode.into(), index, value, ptr, opt)
    }

    /// Check whether the plugin has been initialized.
    #[doc(hidden)]
    fn is_effect_valid(&self) -> bool {
        // Check whether `effect` points to a valid AEffect struct
        unsafe { *mem::transmute::<*mut AEffect, *mut i32>(self.effect) == VST_MAGIC }
    }

    /// Create a new Host structure wrapping a host callback.
    #[doc(hidden)]
    pub fn wrap(callback: HostCallback, effect: *mut AEffect) -> Host {
        Host {
            callback: Some(callback),
            effect: effect,
        }
    }

    /// Notify the host that a parameter value was changed.
    pub fn automate(&mut self, index: i32, value: f32) {
        if self.is_effect_valid() { // TODO: Investigate removing this check, should be up to host
            self.callback(self.effect, OpCode::Automate,
                          index, 0, ptr::null_mut(), value);
        }
    }

    /// Get the VST API version supported by the host e.g. `2400 = VST 2.4`.
    pub fn vst_version(&self) -> i32 {
        self.callback(self.effect, OpCode::Version,
                      0, 0, ptr::null_mut(), 0.0) as i32
    }

    /// Get the plugin ID the host is requesting to load.
    ///
    /// This is only useful for shell plugins where this value will change the plugin returned.
    /// `TODO: implement shell plugins`
    pub fn get_plugin_id(&self) -> i32 {
        self.callback(self.effect, OpCode::CurrentId,
                      0, 0, ptr::null_mut(), 0.0) as i32
    }

    /// Tell the host that it can idle.
    ///
    /// This is useful when the plugin is doing something such as mouse tracking in the UI.
    pub fn idle(&self) {
        self.callback(self.effect, OpCode::Idle,
                      0, 0, ptr::null_mut(), 0.0);
    }
}

#[cfg(test)]
mod tests {
    use std::ptr;

    use plugin;

    /// Create a plugin instance.
    ///
    /// This is a macro to allow you to specify attributes on the created struct.
    macro_rules! make_plugin {
        ($($attr:meta) *) => {
            use libc::c_void;

            use main;
            use api::AEffect;
            use host::{Host, OpCode};
            use plugin::{Info, Plugin};

            $(#[$attr]) *
            struct TestPlugin {
                host: Host
            }

            impl Plugin for TestPlugin {
                fn get_info(&self) -> Info {
                    Info {
                        name: "Test Plugin".to_string(),
                        ..Default::default()
                    }
                }

                fn new(host: Host) -> TestPlugin {
                    TestPlugin {
                        host: host
                    }
                }

                fn init(&mut self) {
                    info!("Loaded with host vst version: {}", self.host.vst_version());
                    assert_eq!(2400, self.host.vst_version());
                    assert_eq!(9876, self.host.get_plugin_id());
                    // Callback will assert these.
                    self.host.automate(123, 12.3);
                    self.host.idle();
                }
            }

            fn instance() -> *mut AEffect {
                fn host_callback(_effect: *mut AEffect,
                                 opcode: i32,
                                 index: i32,
                                 _value: isize,
                                 _ptr: *mut c_void,
                                 opt: f32)
                                 -> isize {
                    let opcode = OpCode::from(opcode);
                    match opcode {
                        OpCode::Automate => {
                            assert_eq!(index, 123);
                            assert_eq!(opt, 12.3);
                            0
                        }
                        OpCode::Version => 2400,
                        OpCode::CurrentId => 9876,
                        OpCode::Idle => 0,
                        _ => 0
                    }
                }

                main::<TestPlugin>(host_callback)
            }
        }
    }

    make_plugin!(derive(Default));

    #[test]
    fn null_panic() {
        make_plugin!(/* no `derive(Default)` */);

        impl Default for TestPlugin {
            fn default() -> TestPlugin {
                let plugin = TestPlugin { host: Default::default() };

                // Should panic
                info!("Loaded with host vst version: {}", plugin.host.vst_version());

                plugin
            }
        }

        let _aeffect = instance();
    }

    #[test]
    fn host_callbacks() {
        let aeffect = instance();
        (unsafe { (*aeffect).dispatcher })(aeffect, plugin::OpCode::Initialize.into(),
                                           0, 0, ptr::null_mut(), 0.0);
    }
}
