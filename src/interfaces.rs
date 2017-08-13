//! Function interfaces for VST 2.4 API.

#![doc(hidden)]

use std::{mem, slice};

use std::os::raw::{c_char, c_void};

use buffer::AudioBuffer;
use api::consts::*;
use api::{self, AEffect, ChannelProperties};
use editor::{Rect, KeyCode, Key, KnobMode};
use host::Host;

/// Deprecated process function.
pub fn process_deprecated(_effect: *mut AEffect, _raw_inputs: *const *const f32, _raw_outputs: *mut *mut f32, _samples: i32) { }

/// VST2.4 replacing function.
pub fn process_replacing(effect: *mut AEffect, raw_inputs: *const *const f32, raw_outputs: *mut *mut f32, samples: i32) {
    // Handle to the vst
    let mut plugin = unsafe { (*effect).get_plugin() };
    let cache = unsafe { (*effect).get_cache() };
    let info = &mut cache.info;
    let (input_count, output_count) = (info.inputs as usize, info.outputs as usize);
    let mut buffer = AudioBuffer::from_raw(input_count, output_count, raw_inputs, raw_outputs, samples as usize);
    plugin.process(&mut buffer);
}

/// VST2.4 replacing function with `f64` values.
pub fn process_replacing_f64(effect: *mut AEffect, raw_inputs: *const *const f64, raw_outputs: *mut *mut f64, samples: i32) {
    let mut plugin = unsafe { (*effect).get_plugin() };
    let cache = unsafe { (*effect).get_cache() };
    let info = &mut cache.info;
    let (input_count, output_count) = (info.inputs as usize, info.outputs as usize);
    let mut buffer = AudioBuffer::from_raw(input_count, output_count, raw_inputs, raw_outputs, samples as usize);
    plugin.process_f64(&mut buffer);
}

/// VST2.4 set parameter function.
pub fn set_parameter(effect: *mut AEffect, index: i32, value: f32) {
    unsafe { (*effect).get_plugin() }.set_parameter(index, value);
}

/// VST2.4 get parameter function.
pub fn get_parameter(effect: *mut AEffect, index: i32) -> f32 {
    unsafe { (*effect).get_plugin() }.get_parameter(index)
}

/// Copy a string into a destination buffer.
///
/// String will be cut at `max` characters.
fn copy_string(dst: *mut c_void, src: &str, max: usize) {
    unsafe {
        use std::cmp::min;
        use libc::{c_void, memset, memcpy};

        let dst = dst as *mut c_void;
        memset(dst, 0, max);
        memcpy(dst, src.as_ptr() as *const c_void, min(max, src.len()));
    }
}

/// VST2.4 dispatch function. This function handles dispatching all opcodes to the vst plugin.
pub fn dispatch(effect: *mut AEffect, opcode: i32, index: i32, value: isize, ptr: *mut c_void, opt: f32) -> isize {
    use plugin::{CanDo, OpCode};

    // Convert passed in opcode to enum
    let opcode = OpCode::from(opcode);
    // Plugin handle
    let mut plugin = unsafe { (*effect).get_plugin() };

    match opcode {
        OpCode::Initialize => plugin.init(),
        OpCode::Shutdown => unsafe {
            (*effect).drop_plugin();
            drop(mem::transmute::<*mut AEffect, Box<AEffect>>(effect));
        },

        OpCode::ChangePreset => plugin.change_preset(value as i32),
        OpCode::GetCurrentPresetNum => return plugin.get_preset_num() as isize,
        OpCode::SetCurrentPresetName => plugin.set_preset_name(read_string(ptr)),
        OpCode::GetCurrentPresetName => {
            let num = plugin.get_preset_num();
            copy_string(ptr, &plugin.get_preset_name(num), MAX_PRESET_NAME_LEN);
        }

        OpCode::GetParameterLabel => copy_string(ptr, &plugin.get_parameter_label(index), MAX_PARAM_STR_LEN),
        OpCode::GetParameterDisplay => copy_string(ptr, &plugin.get_parameter_text(index), MAX_PARAM_STR_LEN),
        OpCode::GetParameterName => copy_string(ptr, &plugin.get_parameter_name(index), MAX_PARAM_STR_LEN),

        OpCode::SetSampleRate => plugin.set_sample_rate(opt),
        OpCode::SetBlockSize => plugin.set_block_size(value as i64),
        OpCode::StateChanged => {
            if value == 1 {
                plugin.resume();
            } else {
                plugin.suspend();
            }
        }

        OpCode::EditorGetRect => {
            if let Some(editor) = plugin.get_editor() {
                let size = editor.size();
                let pos = editor.position();

                unsafe {
                    // Given a Rect** structure
                    // TODO: Investigate whether we are given a valid Rect** pointer already
                    *(ptr as *mut *mut c_void) =
                        Box::into_raw(Box::new(Rect {
                            left: pos.0 as i16, // x coord of position
                            top: pos.1 as i16, // y coord of position
                            right: (pos.0 + size.0) as i16, // x coord of pos + x coord of size
                            bottom: (pos.1 + size.1) as i16 // y coord of pos + y coord of size
                        })) as *mut _; // TODO: free memory
                }
            }
        }
        OpCode::EditorOpen => {
            if let Some(editor) = plugin.get_editor() {
                editor.open(ptr); //ptr is raw window handle, eg HWND* on windows
            }
        }
        OpCode::EditorClose => {
            if let Some(editor) = plugin.get_editor() {
                editor.close();
            }
        }

        OpCode::EditorIdle => {
            if let Some(editor) = plugin.get_editor() {
                editor.idle();
            }
        }

        OpCode::GetData => {
            let chunks = if index == 0 {
                plugin.get_bank_data()
            } else {
                plugin.get_preset_data()
            };

            let len = chunks.len() as isize;

            // u8 array to **void ptr
            // TODO: Release the allocated memory for the chunk in resume / suspend event
            unsafe {
                *(ptr as *mut *mut c_void) =
                    chunks.into_boxed_slice().as_ptr() as *mut c_void;
            }

            return len;
        }
        OpCode::SetData => {
            let chunks = unsafe { slice::from_raw_parts(ptr as *mut u8, value as usize) };
            if index == 0 {
                plugin.load_bank_data(chunks);
            } else {
                plugin.load_preset_data(chunks);
            }
        }

        OpCode::ProcessEvents => {
            plugin.process_events(unsafe { &*(ptr as *const api::Events) });
        }
        OpCode::CanBeAutomated => return plugin.can_be_automated(index) as isize,
        OpCode::StringToParameter => return plugin.string_to_parameter(index, read_string(ptr)) as isize,

        OpCode::GetPresetName => copy_string(ptr, &plugin.get_preset_name(index), MAX_PRESET_NAME_LEN),

        OpCode::GetInputInfo => {
            if index >= 0 && index < plugin.get_info().inputs {
                unsafe {
                    let ptr = mem::transmute::<_, *mut ChannelProperties>(ptr);
                    *ptr = plugin.get_input_info(index).into();
                }
            }
        }
        OpCode::GetOutputInfo => {
            if index >= 0 && index < plugin.get_info().outputs {
                unsafe {
                    let ptr = mem::transmute::<_, *mut ChannelProperties>(ptr);
                    *ptr = plugin.get_output_info(index).into();
                }
            }
        }
        OpCode::GetCategory => {
            return plugin.get_info().category.into();
        }

        OpCode::GetVendorName => copy_string(ptr, &plugin.get_info().vendor, MAX_VENDOR_STR_LEN),
        OpCode::GetProductName => copy_string(ptr, &plugin.get_info().name, MAX_PRODUCT_STR_LEN),
        OpCode::GetVendorVersion => return plugin.get_info().version as isize,
        OpCode::VendorSpecific => return plugin.vendor_specific(index, value, ptr, opt),
        OpCode::CanDo => {
            let can_do: CanDo = match read_string(ptr).parse() {
                Ok(c) => c,
                Err(e) => { warn!("{}", e); return 0; }
            };
            return plugin.can_do(can_do).into();
        }
        OpCode::GetTailSize => if plugin.get_tail_size() == 0 { return 1; } else { return plugin.get_tail_size() },

        //OpCode::GetParamInfo => { /*TODO*/ }

        OpCode::GetApiVersion => return 2400,

        OpCode::EditorKeyDown => {
            if let Some(editor) = plugin.get_editor() {
                editor.key_down(KeyCode {
                    character: index as u8 as char,
                    key: Key::from(value),
                    modifier: unsafe { mem::transmute::<f32, i32>(opt) } as u8
                });
            }
        }
        OpCode::EditorKeyUp => {
            if let Some(editor) = plugin.get_editor() {
                editor.key_up(KeyCode {
                    character: index as u8 as char,
                    key: Key::from(value),
                    modifier: unsafe { mem::transmute::<f32, i32>(opt) } as u8
                });
            }
        }
        OpCode::EditorSetKnobMode => {
            if let Some(editor) = plugin.get_editor() {
                editor.set_knob_mode(KnobMode::from(value));
            }
        }

        _ => {
            warn!("Unimplemented opcode ({:?})", opcode);
            trace!("Arguments; index: {}, value: {}, ptr: {:?}, opt: {}", index, value, ptr, opt);
        }
    }

    0
}

pub fn host_dispatch(host: &mut Host,
                     effect: *mut AEffect,
                     opcode: i32,
                     index: i32,
                     value: isize,
                     ptr: *mut c_void,
                     opt: f32) -> isize {
    use host::OpCode;

    match OpCode::from(opcode) {
        OpCode::Version => return 2400,
        OpCode::Automate => host.automate(index, opt),

        OpCode::Idle => host.idle(),

        // ...

        OpCode::CanDo => {
            info!("Plugin is asking if host can: {}.", read_string(ptr));
        }

        OpCode::GetVendorVersion => return host.get_info().0,
        OpCode::GetVendorString => copy_string(ptr, &host.get_info().1, MAX_VENDOR_STR_LEN),
        OpCode::GetProductString => copy_string(ptr, &host.get_info().2, MAX_PRODUCT_STR_LEN),
        OpCode::ProcessEvents => {
            host.process_events(unsafe { &*(ptr as *const api::Events) });
        }

        unimplemented => {
            trace!("VST: Got unimplemented host opcode ({:?})", unimplemented);
            trace!("Arguments; effect: {:?}, index: {}, value: {}, ptr: {:?}, opt: {}",
                    effect, index, value, ptr, opt);
        }
    }
    0
}

// Read a string from the `ptr` buffer
fn read_string(ptr: *mut c_void) -> String {
    use std::ffi::CStr;

    String::from_utf8_lossy(
        unsafe { CStr::from_ptr(ptr as *mut c_char).to_bytes() }
    ).into_owned()
}