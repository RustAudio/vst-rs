//! Function interfaces for VST 2.4 API.

#![doc(hidden)]

use std::ffi::{CStr, CString};
use std::{mem, slice};

use std::os::raw::{c_char, c_void};
use libc::strncpy;

use buffer::AudioBuffer;
use api::consts::*;
use api::{self, AEffect, ChannelProperties};
use editor::{Rect, KeyCode, Key, KnobMode};
use host::Host;
use event::Event;

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
    use plugin::{CanDo, OpCode};

    // Convert passed in opcode to enum
    let opcode = OpCode::from(opcode);
    // Plugin handle
    let mut plugin = unsafe { (*effect).get_plugin() };

    // Copy a string into the `ptr` buffer
    let copy_string = |string: &String, max: usize| {
        unsafe {
            strncpy(ptr as *mut c_char,
                          CString::new(string.clone()).unwrap().as_ptr(),
                          max);
        }
    };

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
            copy_string(&plugin.get_preset_name(num), MAX_PRESET_NAME_LEN);
        }

        OpCode::GetParameterLabel => copy_string(&plugin.get_parameter_label(index), MAX_PARAM_STR_LEN),
        OpCode::GetParameterDisplay => copy_string(&plugin.get_parameter_text(index), MAX_PARAM_STR_LEN),
        OpCode::GetParameterName => copy_string(&plugin.get_parameter_name(index), MAX_PARAM_STR_LEN),

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
                        mem::transmute(Box::new(Rect {
                            left: pos.0 as i16, // x coord of position
                            top: pos.1 as i16, // y coord of position
                            right: (pos.0 + size.0) as i16, // x coord of pos + x coord of size
                            bottom: (pos.1 + size.1) as i16 // y coord of pos + y coord of size
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
            let events: *const api::Events = ptr as *const api::Events;

            let events: Vec<Event> = unsafe {
                // Create a slice of type &mut [*mut Event]
                slice::from_raw_parts(&(*events).events[0], (*events).num_events as usize)
                // Deref and clone each event to get a slice
                .iter().map(|item| Event::from(**item)).collect()
            };

            plugin.process_events(events);
        }
        OpCode::CanBeAutomated => return plugin.can_be_automated(index) as isize,
        OpCode::StringToParameter => return plugin.string_to_parameter(index, read_string(ptr)) as isize,

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

    // Copy a string into the `ptr` buffer
    let copy_string = |string: &String, max: usize| {
        unsafe {
            strncpy(ptr as *mut c_char,
                          CString::new(string.clone()).unwrap().as_ptr(),
                          max);
        }
    };

    match OpCode::from(opcode) {
        OpCode::Version => return 2400,
        OpCode::Automate => host.automate(index, opt),

        OpCode::Idle => host.idle(),

        // ...

        OpCode::CanDo => {
            info!("Plugin is asking if host can: {}.", read_string(ptr));
        }

        OpCode::GetVendorVersion => return host.get_info().0,
        OpCode::GetVendorString => copy_string(&host.get_info().1, MAX_VENDOR_STR_LEN),
        OpCode::GetProductString => copy_string(&host.get_info().2, MAX_PRODUCT_STR_LEN),
        OpCode::ProcessEvents => {
            let events: *const api::Events = ptr as *const api::Events;

            let events: Vec<Event> = unsafe {
                // Create a slice of type &mut [*mut Event]
                slice::from_raw_parts(&(*events).events[0], (*events).num_events as usize)
                // Deref and clone each event to get a slice
                .iter().map(|item| Event::from(**item)).collect()
            };

            host.process_events(events);
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
    String::from_utf8_lossy(
        unsafe { CStr::from_ptr(ptr as *mut c_char).to_bytes() }
    ).into_owned()
}

/// Translate `Vec<Event>` into `&api::Events` and use via a callback.
///
/// Both plugins and hosts can receive VST events, this simply translates the rust structure into
/// the equivalent API structure and takes care of cleanup.
pub fn process_events<F>(events: Vec<Event>, callback: F)
    where F: FnOnce(*mut c_void)
{
    use api::flags::REALTIME_EVENT;

    let len = events.len();

    // The `api::Events` structure uses a variable length array which is difficult to represent in
    // rust. We begin by creating a vector with the appropriate byte size by calculating the header
    // and the variable length body seperately.
    let header_size = mem::size_of::<api::Events>() - (mem::size_of::<*mut api::Event>() * 2);
    let body_size = mem::size_of::<*mut api::Event>() * len;

    let mut send = vec![0u8; header_size + body_size];

    let send_events: &mut [*mut api::Event] = unsafe {
        // The header is updated by casting the array to the `api::Events` type and specifying the
        // required fields. We create a slice from the position of the first event and the length
        // of the array.
        let ptr = send.as_mut_ptr() as *mut api::Events;
        (*ptr).num_events = len as i32;

        // A slice view of the body
        slice::from_raw_parts_mut(&mut (*ptr).events[0], len)
    };

    // Each event is zipped with the target body array slot. Most of what's happening here is just
    // copying data but the key thing to notice is that each event is boxed and cast to
    // (*mut api::Event). This way we can let the callback handle the event, and then later create
    // the box again from the raw pointer so that it can be properly dropped.
    for (event, out) in events.iter().zip(send_events.iter_mut()) {
        *out = match *event {
            Event::Midi { data, delta_frames, live,
                          note_length, note_offset,
                          detune, note_off_velocity } => {
                Box::into_raw(Box::new(api::MidiEvent {
                    event_type: api::EventType::Midi,
                    byte_size: mem::size_of::<api::MidiEvent>() as i32,
                    delta_frames: delta_frames,
                    flags: if live { REALTIME_EVENT.bits() } else { 0 },
                    note_length: note_length.unwrap_or(0),
                    note_offset: note_offset.unwrap_or(0),
                    midi_data: data,
                    _midi_reserved: 0,
                    detune: detune,
                    note_off_velocity: note_off_velocity,
                    _reserved1: 0,
                    _reserved2: 0
                })) as *mut api::Event
            }
            Event::SysEx { payload, delta_frames } => {
                Box::into_raw(Box::new(api::SysExEvent {
                    event_type: api::EventType::SysEx,
                    byte_size: mem::size_of::<api::SysExEvent>() as i32,
                    delta_frames: delta_frames,
                    _flags: 0,
                    data_size: payload.len() as i32,
                    _reserved1: 0,
                    system_data: payload.as_ptr() as *const u8 as *mut u8,
                    _reserved2: 0,
                })) as *mut api::Event
            }
            Event::Deprecated(e) => Box::into_raw(Box::new(e))
        };
    }

    // Allow the callback to use the pointer
    callback(send.as_mut_ptr() as *mut c_void);

    // Clean up the created events
    unsafe {
        for &mut event in send_events {
            match (*event).event_type {
                api::EventType::Midi => {
                    drop(Box::from_raw(event as *mut api::MidiEvent));
                }
                api::EventType::SysEx => {
                    drop(Box::from_raw(event as *mut api::SysExEvent));
                }
                _ => {
                    drop(Box::from_raw(event));
                }
            }
        }
    }
}
