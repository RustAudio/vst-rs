//! Function interfaces for VST 2.4 API.

#![doc(hidden)]

use std::ffi::{CStr, CString};
use std::mem;

use libc::{self, size_t, c_char, c_void};

use buffer::AudioBuffer;
use api::consts::*;
use api::{AEffect, ChannelProperties};
use editor::{Rect, KeyCode, Key, KnobMode};
use plugin::{CanDo, OpCode, Plugin};

/// Deprecated process function.
pub fn process_deprecated(_effect: *mut AEffect, _inputs_raw: *mut *mut f32, _outputs_raw: *mut *mut f32, _samples: i32) { }

/// VST2.4 replacing function.
pub fn process_replacing(effect: *mut AEffect, inputs_raw: *mut *mut f32, outputs_raw: *mut *mut f32, samples: i32) {
    // Handle to the vst
    let mut plugin = unsafe { (*effect).get_plugin() };

    let buffer = unsafe {
        AudioBuffer::from_raw(inputs_raw,
                              outputs_raw,
                              plugin.get_info().inputs as usize,
                              plugin.get_info().outputs as usize,
                              samples as usize)
    };

    plugin.process(buffer);
}

/// VST2.4 replacing function with `f64` values.
pub fn process_replacing_f64(effect: *mut AEffect, inputs_raw: *mut *mut f64, outputs_raw: *mut *mut f64, samples: i32) {
    let mut plugin = unsafe { (*effect).get_plugin() };

    if plugin.get_info().f64_precision {
        let buffer = unsafe {
            AudioBuffer::from_raw(inputs_raw,
                                  outputs_raw,
                                  plugin.get_info().inputs as usize,
                                  plugin.get_info().outputs as usize,
                                  samples as usize)
        };

        plugin.process_f64(buffer);
    }
}

/// VST2.4 set parameter function.
pub fn set_parameter(effect: *mut AEffect, index: i32, value: f32) {
    unsafe { (*effect).get_plugin() }.set_parameter(index, value);
}

/// VST2.4 get parameter function.
pub fn get_parameter(effect: *mut AEffect, index: i32) -> f32 {
    unsafe { (*effect).get_plugin() }.get_parameter(index)
}

/// VST2.4 dispatch function. This function handles dispatching all opcodes to the vst plugin.
pub fn dispatch(effect: *mut AEffect, opcode: i32, index: i32, value: isize, ptr: *mut c_void, opt: f32) -> isize {
    // Convert passed in opcode to enum
    let opcode = OpCode::from(opcode);
    // Vst handle
    let mut plugin = unsafe { (*effect).get_plugin() };

    // Copy a string into the `ptr` buffer
    let copy_string = |string: &String, max: size_t| {
        unsafe {
            libc::strncpy(ptr as *mut c_char,
                          CString::new(string.clone()).unwrap().as_ptr(),
                          max);
        }
    };

    // Read a string from the `ptr` buffer
    let read_string = || -> String {
        String::from_utf8_lossy(
            unsafe { CStr::from_ptr(ptr as *mut c_char).to_bytes() }
        ).into_owned()
    };

    match opcode {
        OpCode::Initialize => plugin.init(),
        OpCode::Shutdown => unsafe {
            (*effect).drop_plugin();
            drop(mem::transmute::<*mut AEffect, Box<AEffect>>(effect));
        },

        OpCode::ChangePreset => plugin.change_preset(value as i32),
        OpCode::GetCurrentPresetNum => return plugin.get_preset_num() as isize,
        OpCode::SetCurrentPresetName => plugin.set_preset_name(read_string()),
        OpCode::GetCurrentPresetName => {
            let num = plugin.get_preset_num();
            copy_string(&plugin.get_preset_name(num), MAX_PRESET_NAME_LEN);
        }

        OpCode::GetParameterLabel => copy_string(&plugin.get_parameter_label(index), MAX_PARAM_STR_LEN),
        OpCode::GetParameterDisplay => copy_string(&plugin.get_parameter_text(index), MAX_PARAM_STR_LEN),
        OpCode::GetParameterName => copy_string(&plugin.get_parameter_name(index), MAX_PARAM_STR_LEN),

        OpCode::SetSampleRate => plugin.sample_rate_changed(opt),
        OpCode::SetBlockSize => plugin.block_size_changed(value as i64),
        OpCode::StateChanged => {
            if value == 1 {
                plugin.on_resume();
            } else {
                plugin.on_suspend();
            }
        }

        OpCode::EditorGetRect => {
            if let Some(editor) = plugin.get_editor() {
                let size = editor.size();
                let pos = editor.position();

                unsafe {
                    //given a Rect** structure
                    *(ptr as *mut *mut c_void) =
                        mem::transmute(Box::new(Rect {
                            left: pos.0 as i16, //x coord of position
                            top: pos.1 as i16, //y coord of position
                            right: (pos.0 + size.0) as i16, //x coord of pos + x coord of size
                            bottom: (pos.1 + size.1) as i16 //y coord of pos + y coord of size
                        }));
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

            // u8 array to **void ptr.
            unsafe {
                *mem::transmute::<_, *mut *mut c_void>(ptr) =
                    chunks.into_boxed_slice().as_ptr() as *mut c_void;
            }

            return len;
        }
        OpCode::SetData => {
            let chunks = unsafe { Vec::from_raw_parts(ptr as *mut u8, value as usize, value as usize) };
            if index == 0 {
                plugin.load_bank_data(chunks);
            } else {
                plugin.load_preset_data(chunks);
            }
        }

        OpCode::CanBeAutomated => return plugin.can_be_automated(index) as isize,
        OpCode::StringToParameter => return plugin.string_to_parameter(index, read_string()) as isize,

        OpCode::GetPresetName => copy_string(&plugin.get_preset_name(index), MAX_PRESET_NAME_LEN),

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

        OpCode::GetVendorName => copy_string(&plugin.get_info().vendor, MAX_VENDOR_STR_LEN),
        OpCode::GetProductName => copy_string(&plugin.get_info().name, MAX_PRODUCT_STR_LEN),
        OpCode::GetVendorVersion => return plugin.get_info().version as isize,
        OpCode::VendorSpecific => plugin.vendor_specific(index, value, ptr, opt),
        OpCode::CanDo => {
            let can_do: CanDo = match read_string().parse() {
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
