//! Function interfaces for VST 2.4 API.

#![doc(hidden)]

use std::cell::Cell;
use std::os::raw::{c_char, c_void};
use std::{mem, slice};

use crate::{
    api::{self, consts::*, AEffect, TimeInfo},
    buffer::AudioBuffer,
    editor::{Key, KeyCode, KnobMode, Rect},
    host::Host,
};

/// Deprecated process function.
pub extern "C" fn process_deprecated(
    _effect: *mut AEffect,
    _raw_inputs: *const *const f32,
    _raw_outputs: *mut *mut f32,
    _samples: i32,
) {
}

/// VST2.4 replacing function.
pub extern "C" fn process_replacing(
    effect: *mut AEffect,
    raw_inputs: *const *const f32,
    raw_outputs: *mut *mut f32,
    samples: i32,
) {
    // Handle to the VST
    let plugin = unsafe { (*effect).get_plugin() };
    let info = unsafe { (*effect).get_info() };
    let (input_count, output_count) = (info.inputs as usize, info.outputs as usize);
    let mut buffer =
        unsafe { AudioBuffer::from_raw(input_count, output_count, raw_inputs, raw_outputs, samples as usize) };
    plugin.process(&mut buffer);
}

/// VST2.4 replacing function with `f64` values.
pub extern "C" fn process_replacing_f64(
    effect: *mut AEffect,
    raw_inputs: *const *const f64,
    raw_outputs: *mut *mut f64,
    samples: i32,
) {
    let plugin = unsafe { (*effect).get_plugin() };
    let info = unsafe { (*effect).get_info() };
    let (input_count, output_count) = (info.inputs as usize, info.outputs as usize);
    let mut buffer =
        unsafe { AudioBuffer::from_raw(input_count, output_count, raw_inputs, raw_outputs, samples as usize) };
    plugin.process_f64(&mut buffer);
}

/// VST2.4 set parameter function.
pub extern "C" fn set_parameter(effect: *mut AEffect, index: i32, value: f32) {
    unsafe { (*effect).get_params() }.set_parameter(index, value);
}

/// VST2.4 get parameter function.
pub extern "C" fn get_parameter(effect: *mut AEffect, index: i32) -> f32 {
    unsafe { (*effect).get_params() }.get_parameter(index)
}

/// Copy a string into a destination buffer.
///
/// String will be cut at `max` characters.
fn copy_string(dst: *mut c_void, src: &str, max: usize) -> isize {
    unsafe {
        use libc::{memcpy, memset};
        use std::cmp::min;

        let dst = dst as *mut c_void;
        memset(dst, 0, max);
        memcpy(dst, src.as_ptr() as *const c_void, min(max, src.as_bytes().len()));
    }

    1 // Success
}

/// VST2.4 dispatch function. This function handles dispatching all opcodes to the VST plugin.
pub extern "C" fn dispatch(
    effect: *mut AEffect,
    opcode: i32,
    index: i32,
    value: isize,
    ptr: *mut c_void,
    opt: f32,
) -> isize {
    use crate::plugin::{CanDo, OpCode};

    // Convert passed in opcode to enum
    let opcode = OpCode::try_from(opcode);
    // Only query plugin or editor when needed to avoid creating multiple
    // concurrent mutable references to the same object.
    let get_plugin = || unsafe { (*effect).get_plugin() };
    let get_editor = || unsafe { (*effect).get_editor() };
    let params = unsafe { (*effect).get_params() };

    match opcode {
        Ok(OpCode::Initialize) => get_plugin().init(),
        Ok(OpCode::Shutdown) => unsafe {
            (*effect).drop_plugin();
            drop(Box::from_raw(effect))
        },

        Ok(OpCode::ChangePreset) => params.change_preset(value as i32),
        Ok(OpCode::GetCurrentPresetNum) => return params.get_preset_num() as isize,
        Ok(OpCode::SetCurrentPresetName) => params.set_preset_name(read_string(ptr)),
        Ok(OpCode::GetCurrentPresetName) => {
            let num = params.get_preset_num();
            return copy_string(ptr, &params.get_preset_name(num), MAX_PRESET_NAME_LEN);
        }

        Ok(OpCode::GetParameterLabel) => {
            return copy_string(ptr, &params.get_parameter_label(index), MAX_PARAM_STR_LEN)
        }
        Ok(OpCode::GetParameterDisplay) => {
            return copy_string(ptr, &params.get_parameter_text(index), MAX_PARAM_STR_LEN)
        }
        Ok(OpCode::GetParameterName) => return copy_string(ptr, &params.get_parameter_name(index), MAX_PARAM_STR_LEN),

        Ok(OpCode::SetSampleRate) => get_plugin().set_sample_rate(opt),
        Ok(OpCode::SetBlockSize) => get_plugin().set_block_size(value as i64),
        Ok(OpCode::StateChanged) => {
            if value == 1 {
                get_plugin().resume();
            } else {
                get_plugin().suspend();
            }
        }

        Ok(OpCode::EditorGetRect) => {
            if let Some(ref mut editor) = get_editor() {
                let size = editor.size();
                let pos = editor.position();

                unsafe {
                    // Given a Rect** structure
                    // TODO: Investigate whether we are given a valid Rect** pointer already
                    *(ptr as *mut *mut c_void) = Box::into_raw(Box::new(Rect {
                        left: pos.0 as i16,              // x coord of position
                        top: pos.1 as i16,               // y coord of position
                        right: (pos.0 + size.0) as i16,  // x coord of pos + x coord of size
                        bottom: (pos.1 + size.1) as i16, // y coord of pos + y coord of size
                    })) as *mut _; // TODO: free memory
                }

                return 1;
            }
        }
        Ok(OpCode::EditorOpen) => {
            if let Some(ref mut editor) = get_editor() {
                // `ptr` is a window handle to the parent window.
                // See the documentation for `Editor::open` for details.
                if editor.open(ptr) {
                    return 1;
                }
            }
        }
        Ok(OpCode::EditorClose) => {
            if let Some(ref mut editor) = get_editor() {
                editor.close();
            }
        }

        Ok(OpCode::EditorIdle) => {
            if let Some(ref mut editor) = get_editor() {
                editor.idle();
            }
        }

        Ok(OpCode::GetData) => {
            let mut chunks = if index == 0 {
                params.get_bank_data()
            } else {
                params.get_preset_data()
            };

            chunks.shrink_to_fit();
            let len = chunks.len() as isize; // eventually we should be using ffi::size_t

            unsafe {
                *(ptr as *mut *mut c_void) = chunks.as_ptr() as *mut c_void;
            }

            mem::forget(chunks);
            return len;
        }
        Ok(OpCode::SetData) => {
            let chunks = unsafe { slice::from_raw_parts(ptr as *mut u8, value as usize) };

            if index == 0 {
                params.load_bank_data(chunks);
            } else {
                params.load_preset_data(chunks);
            }
        }

        Ok(OpCode::ProcessEvents) => {
            get_plugin().process_events(unsafe { &*(ptr as *const api::Events) });
        }
        Ok(OpCode::CanBeAutomated) => return params.can_be_automated(index) as isize,
        Ok(OpCode::StringToParameter) => return params.string_to_parameter(index, read_string(ptr)) as isize,

        Ok(OpCode::GetPresetName) => return copy_string(ptr, &params.get_preset_name(index), MAX_PRESET_NAME_LEN),

        Ok(OpCode::GetInputInfo) => {
            if index >= 0 && index < get_plugin().get_info().inputs {
                unsafe {
                    let ptr = ptr as *mut api::ChannelProperties;
                    *ptr = get_plugin().get_input_info(index).into();
                }
            }
        }
        Ok(OpCode::GetOutputInfo) => {
            if index >= 0 && index < get_plugin().get_info().outputs {
                unsafe {
                    let ptr = ptr as *mut api::ChannelProperties;
                    *ptr = get_plugin().get_output_info(index).into();
                }
            }
        }
        Ok(OpCode::GetCategory) => {
            return get_plugin().get_info().category.into();
        }

        Ok(OpCode::GetEffectName) => return copy_string(ptr, &get_plugin().get_info().name, MAX_VENDOR_STR_LEN),

        Ok(OpCode::GetVendorName) => return copy_string(ptr, &get_plugin().get_info().vendor, MAX_VENDOR_STR_LEN),
        Ok(OpCode::GetProductName) => return copy_string(ptr, &get_plugin().get_info().name, MAX_PRODUCT_STR_LEN),
        Ok(OpCode::GetVendorVersion) => return get_plugin().get_info().version as isize,
        Ok(OpCode::VendorSpecific) => return get_plugin().vendor_specific(index, value, ptr, opt),
        Ok(OpCode::CanDo) => {
            let can_do = CanDo::from_str(&read_string(ptr));
            return get_plugin().can_do(can_do).into();
        }
        Ok(OpCode::GetTailSize) => {
            if get_plugin().get_tail_size() == 0 {
                return 1;
            } else {
                return get_plugin().get_tail_size();
            }
        }

        //OpCode::GetParamInfo => { /*TODO*/ }
        Ok(OpCode::GetApiVersion) => return 2400,

        Ok(OpCode::EditorKeyDown) => {
            if let Some(ref mut editor) = get_editor() {
                if let Ok(key) = Key::try_from(value) {
                    editor.key_down(KeyCode {
                        character: index as u8 as char,
                        key,
                        modifier: opt.to_bits() as u8,
                    });
                }
            }
        }
        Ok(OpCode::EditorKeyUp) => {
            if let Some(ref mut editor) = get_editor() {
                if let Ok(key) = Key::try_from(value) {
                    editor.key_up(KeyCode {
                        character: index as u8 as char,
                        key,
                        modifier: opt.to_bits() as u8,
                    });
                }
            }
        }
        Ok(OpCode::EditorSetKnobMode) => {
            if let Some(ref mut editor) = get_editor() {
                if let Ok(knob_mode) = KnobMode::try_from(value) {
                    editor.set_knob_mode(knob_mode);
                }
            }
        }

        Ok(OpCode::StartProcess) => get_plugin().start_process(),
        Ok(OpCode::StopProcess) => get_plugin().stop_process(),

        Ok(OpCode::GetNumMidiInputs) => return unsafe { (*effect).get_info() }.midi_inputs as isize,
        Ok(OpCode::GetNumMidiOutputs) => return unsafe { (*effect).get_info() }.midi_outputs as isize,

        _ => {
            debug!("Unimplemented opcode ({:?})", opcode);
            trace!(
                "Arguments; index: {}, value: {}, ptr: {:?}, opt: {}",
                index,
                value,
                ptr,
                opt
            );
        }
    }

    0
}

pub fn host_dispatch(
    host: &mut dyn Host,
    effect: *mut AEffect,
    opcode: i32,
    index: i32,
    value: isize,
    ptr: *mut c_void,
    opt: f32,
) -> isize {
    use crate::host::OpCode;

    let opcode = OpCode::try_from(opcode);
    match opcode {
        Ok(OpCode::Version) => return 2400,
        Ok(OpCode::Automate) => host.automate(index, opt),
        Ok(OpCode::BeginEdit) => host.begin_edit(index),
        Ok(OpCode::EndEdit) => host.end_edit(index),

        Ok(OpCode::Idle) => host.idle(),

        // ...
        Ok(OpCode::CanDo) => {
            info!("Plugin is asking if host can: {}.", read_string(ptr));
        }

        Ok(OpCode::GetVendorVersion) => return host.get_info().0,
        Ok(OpCode::GetVendorString) => return copy_string(ptr, &host.get_info().1, MAX_VENDOR_STR_LEN),
        Ok(OpCode::GetProductString) => return copy_string(ptr, &host.get_info().2, MAX_PRODUCT_STR_LEN),
        Ok(OpCode::ProcessEvents) => {
            host.process_events(unsafe { &*(ptr as *const api::Events) });
        }

        Ok(OpCode::GetTime) => {
            return match host.get_time_info(value as i32) {
                None => 0,
                Some(result) => {
                    thread_local! {
                        static TIME_INFO: Cell<TimeInfo> =
                            Cell::new(TimeInfo::default());
                    }
                    TIME_INFO.with(|time_info| {
                        (*time_info).set(result);
                        time_info.as_ptr() as isize
                    })
                }
            };
        }
        Ok(OpCode::GetBlockSize) => return host.get_block_size(),

        _ => {
            trace!("VST: Got unimplemented host opcode ({:?})", opcode);
            trace!(
                "Arguments; effect: {:?}, index: {}, value: {}, ptr: {:?}, opt: {}",
                effect,
                index,
                value,
                ptr,
                opt
            );
        }
    }
    0
}

// Read a string from the `ptr` buffer
fn read_string(ptr: *mut c_void) -> String {
    use std::ffi::CStr;

    String::from_utf8_lossy(unsafe { CStr::from_ptr(ptr as *mut c_char).to_bytes() }).into_owned()
}
