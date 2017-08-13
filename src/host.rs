//! Host specific structures.

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::error::Error;
use std::{fmt, mem, ptr, slice};
use std::ffi::CString;

use libloading::Library;
use std::os::raw::c_void;

use interfaces;
use plugin::{self, Plugin, Info, Category};
use api::{self, AEffect, PluginMain, Supported};
use api::consts::*;
use buffer::AudioBuffer;
use channels::ChannelInfo;

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
    /// Inform host that the plugin has MIDI events ready to be processed. Should be called at the
    /// end of `Plugin::process`.
    /// [ptr]: `VstEvents*` the events to be processed.
    /// [return]: 1 if supported and processed OK.
    ProcessEvents,
    /// Deprecated.
    _SetTime,
    /// Deprecated.
    _TempoAt,
    /// Deprecated.
    _GetNumAutomatableParameters,
    /// Deprecated.
    _GetParameterQuantization,

    /// Notifies the host that the input/output setup has changed. This can allow the host to check
    /// numInputs/numOutputs or call `getSpeakerArrangement()`.
    /// [return]: 1 if supported.
    IOChanged,

    /// Deprecated.
    _NeedIdle,

    /// Request the host to resize the plugin window.
    /// [index]: new width.
    /// [value]: new height.
    SizeWindow,
    /// [return]: the current sample rate.
    GetSampleRate,
    /// [return]: the current block size.
    GetBlockSize,
    /// [return]: the input latency in samples.
    GetInputLatency,
    /// [return]: the output latency in samples.
    GetOutputLatency,

    /// Deprecated.
    _GetPreviousPlug,
    /// Deprecated.
    _GetNextPlug,
    /// Deprecated.
    _WillReplaceOrAccumulate,

    /// [return]: the current process level, see `VstProcessLevels`
    GetCurrentProcessLevel,
    /// [return]: the current automation state, see `VstAutomationStates`
    GetAutomationState,

    /// The plugin is ready to begin offline processing.
    /// [index]: number of new audio files.
    /// [value]: number of audio files.
    /// [ptr]: `AudioFile*` the host audio files. Flags can be updated from plugin.
    OfflineStart,
    /// Called by the plugin to read data.
    /// [index]: (bool)
    ///    VST offline processing allows a plugin to overwrite existing files. If this value is
    ///    true then the host will read the original file's samples, but if it is false it will
    ///    read the samples which the plugin has written via `OfflineWrite`
    /// [value]: see `OfflineOption`
    /// [ptr]: `OfflineTask*` describing the task.
    /// [return]: 1 on success
    OfflineRead,
    /// Called by the plugin to write data.
    /// [value]: see `OfflineOption`
    /// [ptr]: `OfflineTask*` describing the task.
    OfflineWrite,
    /// Unknown. Used in offline processing.
    OfflineGetCurrentPass,
    /// Unknown. Used in offline processing.
    OfflineGetCurrentMetaPass,

    /// Deprecated.
    _SetOutputSampleRate,
    /// Deprecated.
    _GetOutputSpeakerArrangement,

    /// Get the vendor string.
    /// [ptr]: `char*` for vendor string, limited to `MAX_VENDOR_STR_LEN`.
    GetVendorString,
    /// Get the product string.
    /// [ptr]: `char*` for vendor string, limited to `MAX_PRODUCT_STR_LEN`.
    GetProductString,
    /// [return]: vendor-specific version
    GetVendorVersion,
    /// Vendor specific handling.
    VendorSpecific,

    /// Deprecated.
    _SetIcon,

    /// Check if the host supports a feature.
    /// [ptr]: `char*` can do string
    /// [return]: 1 if supported
    CanDo,
    /// Get the language of the host.
    /// [return]: `VstHostLanguage`
    GetLanguage,

    /// Deprecated.
    _OpenWindow,
    /// Deprecated.
    _CloseWindow,

    /// Get the current directory.
    /// [return]: `FSSpec` on OS X, `char*` otherwise
    GetDirectory,
    /// No arguments. TODO: Figure out what this does.
    UpdateDisplay,
    /// Tell the host that if needed, it should record automation data for a control.
    ///
    /// Typically called when the plugin editor begins changing a control.
    ///
    /// [index]: index of the control.
    /// [return]: true on success.
    BeginEdit,
    /// A control is no longer being changed.
    ///
    /// Typically called after the plugin editor is done.
    ///
    /// [index]: index of the control.
    /// [return]: true on success.
    EndEdit,
    /// Open the host file selector.
    /// [ptr]: `VstFileSelect*`
    /// [return]: true on success.
    OpenFileSelector,
    /// Close the host file selector.
    /// [ptr]: `VstFileSelect*`
    /// [return]: true on success.
    CloseFileSelector,

    /// Deprecated.
    _EditFile,
    /// Deprecated.
    /// [ptr]: char[2048] or sizeof (FSSpec).
    /// [return]: 1 if supported.
    _GetChunkFile,
    /// Deprecated.
    _GetInputSpeakerArrangement
}
impl_clike!(OpCode);

/// Implemented by all VST hosts.
#[allow(unused_variables)]
pub trait Host {
    /// Automate a parameter; the value has been changed.
    fn automate(&mut self, index: i32, value: f32) {}

    /// Get the plugin ID of the currently loading plugin.
    ///
    /// This is only useful for shell plugins where this value will change the plugin returned.
    /// `TODO: implement shell plugins`
    fn get_plugin_id(&self) -> i32 {
        // TODO: Handle this properly
        0
    }

    /// An idle call.
    ///
    /// This is useful when the plugin is doing something such as mouse tracking in the UI.
    fn idle(&self) {}

    /// Get vendor and product information.
    ///
    /// Returns a tuple in the form of `(version, vendor_name, product_name)`.
    fn get_info(&self) -> (isize, String, String) {
        (1, "vendor string".to_owned(), "product string".to_owned())
    }

    /// Handle incoming events from the plugin.
    fn process_events(&mut self, events: &api::Events) {}
}

/// All possible errors that can occur when loading a VST plugin.
#[derive(Debug)]
pub enum PluginLoadError {
    /// Could not load given path.
    InvalidPath,

    /// Given path is not a VST plugin.
    NotAPlugin,

    /// Failed to create an instance of this plugin.
    ///
    /// This can happen for many reasons, such as if the plugin requires a different version of
    /// the VST API to be used, or due to improper licensing.
    InstanceFailed,

    /// The API version which the plugin used is not supported by this library.
    InvalidApiVersion,
}

impl fmt::Display for PluginLoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl Error for PluginLoadError {
    fn description(&self) -> &str {
        use self::PluginLoadError::*;

        match *self {
            InvalidPath => "Could not open the requested path",
            NotAPlugin => "The given path does not contain a VST2.4 compatible library",
            InstanceFailed => "Failed to create a plugin instance",
            InvalidApiVersion => "The plugin API version is not compatible with this library"
        }
    }
}

/// Wrapper for an externally loaded VST plugin.
///
/// The only functionality this struct provides is loading plugins, which can be done via the
/// [`load`](#method.load) method.
pub struct PluginLoader<T: Host> {
    main: PluginMain,
    lib: Arc<Library>,
    host: Arc<Mutex<T>>,
}

/// An instance of an externally loaded VST plugin.
#[allow(dead_code)] // To keep `lib` around.
pub struct PluginInstance {
    effect: *mut AEffect,
    lib: Arc<Library>,
    info: Info,
}

impl Drop for PluginInstance {
    fn drop(&mut self) {
        self.dispatch(plugin::OpCode::Shutdown, 0, 0, ptr::null_mut(), 0.0);
    }
}


impl<T: Host> PluginLoader<T> {
    /// Load a plugin at the given path with the given host.
    ///
    /// Because of the possibility of multi-threading problems that can occur when using plugins,
    /// the host must be passed in via an `Arc<Mutex<T>>` object. This makes sure that even if the
    /// plugins are multi-threaded no data race issues can occur.
    ///
    /// Upon success, this method returns a [`PluginLoader`](.) object which you can use to call
    /// [`instance`](#method.instance) to create a new instance of the plugin.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use std::path::Path;
    /// # use std::sync::{Arc, Mutex};
    /// # use vst2::host::{Host, PluginLoader};
    /// # let path = Path::new(".");
    /// # struct MyHost;
    /// # impl MyHost { fn new() -> MyHost { MyHost } }
    /// # impl Host for MyHost {
    /// #     fn automate(&mut self, _: i32, _: f32) {}
    /// #     fn get_plugin_id(&self) -> i32 { 0 }
    /// # }
    /// // ...
    /// let host = Arc::new(Mutex::new(MyHost::new()));
    ///
    /// let mut plugin = PluginLoader::load(path, host.clone()).unwrap();
    ///
    /// let instance = plugin.instance().unwrap();
    /// // ...
    /// ```
    ///
    /// # Linux/Windows
    ///   * This should be a path to the library, typically ending in `.so`/`.dll`.
    ///   * Possible full path: `/home/overdrivenpotato/.vst/u-he/Zebra2.64.so`
    ///   * Possible full path: `C:\Program Files (x86)\VSTPlugins\iZotope Ozone 5.dll`
    ///
    /// # OS X
    ///   * This should point to the mach-o file within the `.vst` bundle.
    ///   * Plugin: `/Library/Audio/Plug-Ins/VST/iZotope Ozone 5.vst`
    ///   * Possible full path:
    ///     `/Library/Audio/Plug-Ins/VST/iZotope Ozone 5.vst/Contents/MacOS/PluginHooksVST`
    pub fn load(path: &Path, host: Arc<Mutex<T>>) -> Result<PluginLoader<T>, PluginLoadError> {
        // Try loading the library at the given path
        let lib = match Library::new(path) {
            Ok(l) => l,
            Err(_) => return Err(PluginLoadError::InvalidPath)
        };

        Ok(PluginLoader {
            main: unsafe {
                      // Search the library for the VSTAPI entry point
                      match lib.get(b"VSTPluginMain") {
                          Ok(s) => *s,
                          _ => return Err(PluginLoadError::NotAPlugin)
                      }
                  },
            lib: Arc::new(lib),
            host: host,
        })
    }

    /// Call the VST entry point and retrieve a (possibly null) pointer.
    unsafe fn call_main(&mut self) -> *mut AEffect {
        LOAD_POINTER = Box::into_raw(Box::new(self.host.clone())) as *mut c_void;
        (self.main)(callback_wrapper::<T>)
    }

    /// Try to create an instance of this VST plugin.
    ///
    /// If the instance is successfully created, a [`PluginInstance`](struct.PluginInstance.html)
    /// is returned. This struct implements the [`Plugin` trait](../plugin/trait.Plugin.html).
    pub fn instance(&mut self) -> Result<PluginInstance, PluginLoadError> {
        // Call the plugin main function. This also passes the plugin main function as the closure
        // could not return an error if the symbol wasn't found
        let effect = unsafe { self.call_main() };

        if effect.is_null() {
            return Err(PluginLoadError::InstanceFailed);
        }

        unsafe {
            // Move the host to the heap and add it to the `AEffect` struct for future reference
            (*effect).reserved1 = Box::into_raw(Box::new(self.host.clone())) as isize;
        }

        let instance = PluginInstance::new(
            effect,
            self.lib.clone()
        );

        let api_ver = instance.dispatch(plugin::OpCode::GetApiVersion, 0, 0, ptr::null_mut(), 0.0);
        if api_ver >= 2400 {
            Ok(instance)
        } else {
            trace!("Could not load plugin with api version {}", api_ver);
            Err(PluginLoadError::InvalidApiVersion)
        }
    }
}

impl PluginInstance {
    fn new(effect: *mut AEffect, lib: Arc<Library>) -> PluginInstance {
        use plugin::OpCode as op;

        let mut plug = PluginInstance {
            effect: effect,
            lib: lib,
            info: Default::default()
        };

        unsafe {
            use api::flags::*;

            let effect: &AEffect = &*effect;
            let flags = Plugin::from_bits_truncate(effect.flags);

            plug.info = Info {
                name: plug.read_string(op::GetProductName, MAX_PRODUCT_STR_LEN),
                vendor: plug.read_string(op::GetVendorName, MAX_VENDOR_STR_LEN),

                presets: effect.numPrograms,
                parameters: effect.numParams,
                inputs: effect.numInputs,
                outputs: effect.numOutputs,

                unique_id: effect.uniqueId,
                version: effect.version,

                category: Category::from(plug.opcode(op::GetCategory)),

                initial_delay: effect.initialDelay,

                preset_chunks: flags.intersects(PROGRAM_CHUNKS),
                f64_precision: flags.intersects(CAN_DOUBLE_REPLACING),
                silent_when_stopped: flags.intersects(NO_SOUND_IN_STOP),
            };
        }

        plug
    }

    /// Send a dispatch message to the plugin.
    fn dispatch(&self,
                opcode: plugin::OpCode,
                index: i32,
                value: isize,
                ptr: *mut c_void,
                opt: f32)
                -> isize {
        let dispatcher = unsafe {
            (*self.effect).dispatcher
        };
        if (dispatcher as *mut u8).is_null() {
            panic!("Plugin was not loaded correctly.");
        }
        dispatcher(self.effect, opcode.into(), index, value, ptr, opt)
    }

    /// Send a lone opcode with no parameters.
    fn opcode(&self, opcode: plugin::OpCode) -> isize {
        self.dispatch(opcode, 0, 0, ptr::null_mut(), 0.0)
    }

    /// Like `dispatch`, except takes a `&str` to send via `ptr`.
    fn write_string(&self,
                    opcode: plugin::OpCode,
                    index: i32,
                    value: isize,
                    string: &str,
                    opt: f32)
                    -> isize {
        let string = CString::new(string).expect("Invalid string data");
        self.dispatch(opcode, index, value, string.as_bytes().as_ptr() as *mut c_void, opt)
    }

    fn read_string(&self, opcode: plugin::OpCode, max: usize) -> String {
        self.read_string_param(opcode, 0, 0, 0.0, max)
    }

    fn read_string_param(&self,
                         opcode: plugin::OpCode,
                         index: i32,
                         value: isize,
                         opt: f32,
                         max: usize)
                         -> String {
        let mut buf = vec![0; max];
        self.dispatch(opcode, index, value, buf.as_mut_ptr() as *mut c_void, opt);
        String::from_utf8_lossy(&buf).chars().take_while(|c| *c != '\0').collect()
    }
}

impl Plugin for PluginInstance {
    fn get_info(&self) -> plugin::Info {
        self.info.clone()
    }

    fn init(&mut self) {
        self.opcode(plugin::OpCode::Initialize);
    }


    fn change_preset(&mut self, preset: i32) {
        self.dispatch(plugin::OpCode::ChangePreset, 0, preset as isize, ptr::null_mut(), 0.0);
    }

    fn get_preset_num(&self) -> i32 {
        self.opcode(plugin::OpCode::GetCurrentPresetNum) as i32
    }

    fn set_preset_name(&mut self, name: String) {
        self.write_string(plugin::OpCode::SetCurrentPresetName, 0, 0, &name, 0.0);
    }

    fn get_preset_name(&self, preset: i32) -> String {
        self.read_string_param(plugin::OpCode::GetPresetName, preset, 0, 0.0, MAX_PRESET_NAME_LEN)
    }


    fn get_parameter_label(&self, index: i32) -> String {
        self.read_string_param(plugin::OpCode::GetParameterLabel, index, 0, 0.0, MAX_PARAM_STR_LEN)
    }

    fn get_parameter_text(&self, index: i32) -> String {
        self.read_string_param(plugin::OpCode::GetParameterDisplay, index, 0, 0.0, MAX_PARAM_STR_LEN)
    }

    fn get_parameter_name(&self, index: i32) -> String {
        self.read_string_param(plugin::OpCode::GetParameterName, index, 0, 0.0, MAX_PARAM_STR_LEN)
    }

    fn get_parameter(&self, index: i32) -> f32 {
        unsafe {
            ((*self.effect).getParameter)(self.effect, index)
        }
    }

    fn set_parameter(&mut self, index: i32, value: f32) {
        unsafe {
            ((*self.effect).setParameter)(self.effect, index, value)
        }
    }

    fn can_be_automated(&self, index: i32) -> bool {
        self.dispatch(plugin::OpCode::CanBeAutomated, index, 0, ptr::null_mut(), 0.0) > 0
    }

    fn string_to_parameter(&mut self, index: i32, text: String) -> bool {
        self.write_string(plugin::OpCode::StringToParameter, index, 0, &text, 0.0) > 0
    }


    fn set_sample_rate(&mut self, rate: f32) {
        self.dispatch(plugin::OpCode::SetSampleRate, 0, 0, ptr::null_mut(), rate);
    }

    fn set_block_size(&mut self, size: i64) {
        self.dispatch(plugin::OpCode::SetBlockSize, 0, size as isize, ptr::null_mut(), 0.0);
    }


    fn resume(&mut self) {
        self.dispatch(plugin::OpCode::StateChanged, 0, 1, ptr::null_mut(), 0.0);
    }

    fn suspend(&mut self) {
        self.dispatch(plugin::OpCode::StateChanged, 0, 0, ptr::null_mut(), 0.0);
    }


    fn vendor_specific(&mut self, index: i32, value: isize, ptr: *mut c_void, opt: f32) -> isize {
        self.dispatch(plugin::OpCode::VendorSpecific, index, value, ptr, opt)
    }


    fn can_do(&self, can_do: plugin::CanDo) -> Supported {
        let s: String = can_do.into();
        Supported::from(
            self.write_string(plugin::OpCode::CanDo, 0, 0, &s, 0.0)
        ).expect("Invalid response received when querying plugin CanDo")
    }

    fn get_tail_size(&self) -> isize {
        self.opcode(plugin::OpCode::GetTailSize)
    }


    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        unsafe {
            ((*self.effect).processReplacing)
                (self.effect, buffer.raw_inputs().as_ptr() as *const *const _, buffer.raw_outputs().as_mut_ptr() as *mut *mut _, buffer.samples() as i32)
        }
    }

    fn process_f64(&mut self, buffer: &mut AudioBuffer<f64>) {
        unsafe {
            ((*self.effect).processReplacingF64)
                (self.effect, buffer.raw_inputs().as_ptr() as *const *const _, buffer.raw_outputs().as_mut_ptr() as *mut *mut _, buffer.samples() as i32)
        }
    }

    fn process_events(&mut self, events: &api::Events) {
        self.dispatch(
            plugin::OpCode::ProcessEvents,
            0,
            0,
            events as *const _ as *mut _,
            0.0
        );
    }

    // TODO: Editor

    fn get_preset_data(&mut self) -> Vec<u8> {
        // Create a pointer that can be updated from the plugin.
        let mut ptr: *mut u8 = ptr::null_mut();
        let len = self.dispatch(plugin::OpCode::GetData,
                                1 /*preset*/, 0,
                                &mut ptr as *mut *mut u8 as *mut c_void, 0.0);
        let slice = unsafe { slice::from_raw_parts(ptr, len as usize) };
        slice.to_vec()
    }

    fn get_bank_data(&mut self) -> Vec<u8> {
        // Create a pointer that can be updated from the plugin.
        let mut ptr: *mut u8 = ptr::null_mut();
        let len = self.dispatch(plugin::OpCode::GetData,
                                0 /*bank*/, 0,
                                &mut ptr as *mut *mut u8 as *mut c_void, 0.0);
        let slice = unsafe { slice::from_raw_parts(ptr, len as usize) };
        slice.to_vec()
    }

    fn load_preset_data(&mut self, data: &[u8]) {
        self.dispatch(plugin::OpCode::SetData, 1, data.len() as isize,
                      data.as_ptr() as *mut c_void, 0.0);
    }

    fn load_bank_data(&mut self, data: &[u8]) {
        self.dispatch(plugin::OpCode::SetData, 0, data.len() as isize,
                      data.as_ptr() as *mut c_void, 0.0);
    }

    fn get_input_info(&self, input: i32) -> ChannelInfo {
        let mut props = unsafe { mem::uninitialized() };
        let ptr = &mut props as *mut api::ChannelProperties as *mut c_void;

        self.dispatch(plugin::OpCode::GetInputInfo, input, 0, ptr, 0.0);

        ChannelInfo::from(props)
    }

    fn get_output_info(&self, output: i32) -> ChannelInfo {
        let mut props = unsafe { mem::uninitialized() };
        let ptr = &mut props as *mut api::ChannelProperties as *mut c_void;

        self.dispatch(plugin::OpCode::GetOutputInfo, output, 0, ptr, 0.0);

        ChannelInfo::from(props)
    }
}

/// HACK: a pointer to store the host so that it can be accessed from the `callback_wrapper`
/// function passed to the plugin.
///
/// When the plugin is being loaded, a `Box<Arc<Mutex<T>>>` is transmuted to a *mut c_void pointer
/// and placed here. When the plugin calls the callback during initialization, the host refers to
/// this pointer to get a handle to the Host. After initialization, this pointer is invalidated and
/// the host pointer is placed into a [reserved field] in the instance `AEffect` struct.
///
/// The issue with this approach is that if 2 plugins are simultaneously loaded with 2 different
/// host instances, this might fail as one host may receive a pointer to the other one. In practice
/// this is a rare situation as you normally won't have 2 seperate host instances loading at once.
///
/// [reserved field]: ../api/struct.AEffect.html#structfield.reserved1
static mut LOAD_POINTER: *mut c_void = 0 as *mut c_void;

/// Function passed to plugin to handle dispatching host opcodes.
fn callback_wrapper<T: Host>(effect: *mut AEffect, opcode: i32, index: i32,
                             value: isize, ptr: *mut c_void, opt: f32) -> isize {
    unsafe {
        // If the effect pointer is not null and the host pointer is not null, the plugin has
        // already been initialized
        if !effect.is_null() && (*effect).reserved1 != 0 {
            let reserved = (*effect).reserved1 as *const Arc<Mutex<T>>;
            let host = &*reserved;

            let host = &mut *host.lock().unwrap();

            interfaces::host_dispatch(host, effect, opcode, index, value, ptr, opt)
        // In this case, the plugin is still undergoing initialization and so `LOAD_POINTER` is
        // dereferenced
        } else {
            // Used only during the plugin initialization
            let host = LOAD_POINTER as *const Arc<Mutex<T>>;
            let host = &*host;
            let host = &mut *host.lock().unwrap();

            interfaces::host_dispatch(host, effect, opcode, index, value, ptr, opt)
        }
    }
}
