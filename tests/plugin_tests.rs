extern crate vst2;

#[macro_use]
extern crate log;

use std::ptr;

use vst2::{plugin, host, api};

/// Create a plugin instance.
///
/// This is a macro to allow you to specify attributes on the created struct.
macro_rules! make_plugin {
        ($($attr:meta) *) => {
            use std::os::raw::c_void;

            use vst2::main;
            use api::AEffect;
            use host::{Host, OpCode};
            use plugin::{HostCallback, Info, Plugin};

            $(#[$attr]) *
            struct TestPlugin {
                host: HostCallback
            }

            impl Plugin for TestPlugin {
                fn get_info(&self) -> Info {
                    Info {
                        name: "Test Plugin".to_string(),
                        ..Default::default()
                    }
                }

                fn new(host: HostCallback) -> TestPlugin {
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

            #[allow(dead_code)]
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
#[should_panic]
fn null_panic() {
    make_plugin!(/* no `derive(Default)` */);

    impl Default for TestPlugin {
        fn default() -> TestPlugin {
            let plugin = TestPlugin { host: Default::default() };

            // Should panic
            let version = plugin.host.vst_version();
            info!("Loaded with host vst version: {}", version);

            plugin
        }
    }

    TestPlugin::default();
}

#[test]
fn host_callbacks() {
    let aeffect = instance();
    (unsafe { (*aeffect).dispatcher })(
        aeffect,
        plugin::OpCode::Initialize.into(),
        0,
        0,
        ptr::null_mut(),
        0.0,
    );
}
