//! Plugin specific structures.

use std::{mem, ptr};

use std::os::raw::c_void;

use channels::ChannelInfo;
use host::{self, Host};
use api::{AEffect, HostCallbackProc, Supported};
use api::consts::VST_MAGIC;
use buffer::AudioBuffer;
use editor::Editor;
use api;

/// Plugin type. Generally either Effect or Synth.
///
/// Other types are not necessary to build a plugin and are only useful for the host to categorize
/// the plugin.
#[repr(usize)]
#[derive(Clone, Copy, Debug)]
pub enum Category {
    /// Unknown / not implemented
    Unknown,
    /// Any effect
    Effect,
    /// VST instrument
    Synth,
    /// Scope, tuner, spectrum analyser, etc.
    Analysis,
    /// Dynamics, etc.
    Mastering,
    /// Panners, etc.
    Spacializer,
    /// Delays and Reverbs
    RoomFx,
    /// Dedicated surround processor.
    SurroundFx,
    /// Denoiser, etc.
    Restoration,
    /// Offline processing.
    OfflineProcess,
    /// Contains other plugins.
    Shell,
    /// Tone generator, etc.
    Generator
}
impl_clike!(Category);

#[repr(usize)]
#[derive(Clone, Copy, Debug)]
#[doc(hidden)]
pub enum OpCode {
    /// Called when plugin is initialized.
    Initialize,
    /// Called when plugin is being shut down.
    Shutdown,

    /// [value]: preset number to change to.
    ChangePreset,
    /// [return]: current preset number.
    GetCurrentPresetNum,
    /// [ptr]: char array with new preset name, limited to `consts::MAX_PRESET_NAME_LEN`.
    SetCurrentPresetName,
    /// [ptr]: char buffer for current preset name, limited to `consts::MAX_PRESET_NAME_LEN`.
    GetCurrentPresetName,

    /// [index]: parameter
    /// [ptr]: char buffer, limited to `consts::MAX_PARAM_STR_LEN` (e.g. "db", "ms", etc)
    GetParameterLabel,
    /// [index]: paramter
    /// [ptr]: char buffer, limited to `consts::MAX_PARAM_STR_LEN` (e.g. "0.5", "ROOM", etc).
    GetParameterDisplay,
    /// [index]: parameter
    /// [ptr]: char buffer, limited to `consts::MAX_PARAM_STR_LEN` (e.g. "Release", "Gain")
    GetParameterName,

    /// Deprecated.
    _GetVu,

    /// [opt]: new sample rate.
    SetSampleRate,
    /// [value]: new maximum block size.
    SetBlockSize,
    /// [value]: 1 when plugin enabled, 0 when disabled.
    StateChanged,

    /// [ptr]: Rect** receiving pointer to editor size.
    EditorGetRect,
    /// [ptr]: system dependent window pointer, eg HWND on Windows.
    EditorOpen,
    /// Close editor. No arguments.
    EditorClose,

    /// Deprecated.
    _EditorDraw,
    /// Deprecated.
    _EditorMouse,
    /// Deprecated.
    _EditorKey,

    /// Idle call from host.
    EditorIdle,

    /// Deprecated.
    _EditorTop,
    /// Deprecated.
    _EditorSleep,
    /// Deprecated.
    _EditorIdentify,

    /// [ptr]: pointer for chunk data address (void**).
    /// [index]: 0 for bank, 1 for program
    GetData,
    /// [ptr]: data (void*)
    /// [value]: data size in bytes
    /// [index]: 0 for bank, 1 for program
    SetData,

    /// [ptr]: VstEvents* TODO: Events
    ProcessEvents,
    /// [index]: param index
    /// [return]: 1=true, 0=false
    CanBeAutomated,
    ///  [index]: param index
    ///  [ptr]: parameter string
    ///  [return]: true for success
    StringToParameter,

    /// Deprecated.
    _GetNumCategories,

    /// [index]: program name
    /// [ptr]: char buffer for name, limited to `consts::MAX_PRESET_NAME_LEN`
    /// [return]: true for success
    GetPresetName,

    /// Deprecated.
    _CopyPreset,
    /// Deprecated.
    _ConnectIn,
    /// Deprecated.
    _ConnectOut,

    /// [index]: input index
    /// [ptr]: `VstPinProperties`
    /// [return]: 1 if supported
    GetInputInfo,
    /// [index]: output index
    /// [ptr]: `VstPinProperties`
    /// [return]: 1 if supported
    GetOutputInfo,
    /// [return]: `PluginCategory` category.
    GetCategory,

    /// Deprecated.
    _GetCurrentPosition,
    /// Deprecated.
    _GetDestinationBuffer,

    /// [ptr]: `VstAudioFile` array
    /// [value]: count
    /// [index]: start flag
    OfflineNotify,
    /// [ptr]: `VstOfflineTask` array
    /// [value]: count
    OfflinePrepare,
    /// [ptr]: `VstOfflineTask` array
    /// [value]: count
    OfflineRun,

    /// [ptr]: `VstVariableIo`
    /// [use]: used for variable I/O processing (offline e.g. timestretching)
    ProcessVarIo,
    /// TODO: implement
    /// [value]: input `*mut VstSpeakerArrangement`.
    /// [ptr]: output `*mut VstSpeakerArrangement`.
    SetSpeakerArrangement,

    /// Deprecated.
    _SetBlocksizeAndSampleRate,

    /// Soft bypass (automatable).
    /// [value]: 1 = bypass, 0 = nobypass.
    SoftBypass,
    // [ptr]: buffer for effect name, limited to `kVstMaxEffectNameLen`
    GetEffectName,

    /// Deprecated.
    _GetErrorText,

    /// [ptr]: buffer for vendor name, limited to `consts::MAX_VENDOR_STR_LEN`.
    GetVendorName,
    /// [ptr]: buffer for product name, limited to `consts::MAX_PRODUCT_STR_LEN`.
    GetProductName,
    /// [return]: vendor specific version.
    GetVendorVersion,
    /// no definition, vendor specific.
    VendorSpecific,
    /// [ptr]: "Can do" string.
    /// [return]: 1 = yes, 0 = maybe, -1 = no.
    CanDo,
    /// [return]: tail size (e.g. reverb time). 0 is defualt, 1 means no tail.
    GetTailSize,

    /// Deprecated.
    _Idle,
    /// Deprecated.
    _GetIcon,
    /// Deprecated.
    _SetVewPosition,

    /// [index]: param index
    /// [ptr]: `*mut VstParamInfo` //TODO: Implement
    /// [return]: 1 if supported
    GetParamInfo,

    /// Deprecated.
    _KeysRequired,

    /// [return]: 2400 for vst 2.4.
    GetApiVersion,

    /// [index]: ASCII char.
    /// [value]: `Key` keycode.
    /// [opt]: `flags::modifier_key` bitmask.
    /// [return]: 1 if used.
    EditorKeyDown,
    /// [index]: ASCII char.
    /// [value]: `Key` keycode.
    /// [opt]: `flags::modifier_key` bitmask.
    /// [return]: 1 if used.
    EditorKeyUp,
    /// [value]: 0 = circular, 1 = circular relative, 2 = linear.
    EditorSetKnobMode,

    /// [index]: MIDI channel.
    /// [ptr]: `*mut MidiProgramName`. //TODO: Implement
    /// [return]: number of used programs, 0 = unsupported.
    GetMidiProgramName,
    /// [index]: MIDI channel.
    /// [ptr]: `*mut MidiProgramName`. //TODO: Implement
    /// [return]: index of current program.
    GetCurrentMidiProgram,
    /// [index]: MIDI channel.
    /// [ptr]: `*mut MidiProgramCategory`. //TODO: Implement
    /// [return]: number of used categories.
    GetMidiProgramCategory,
    /// [index]: MIDI channel.
    /// [return]: 1 if `MidiProgramName` or `MidiKeyName` has changed. //TODO: Implement
    HasMidiProgramsChanged,
    /// [index]: MIDI channel.
    /// [ptr]: `*mut MidiKeyName`. //TODO: Implement
    /// [return]: 1 = supported 0 = not.
    GetMidiKeyName,

    /// Called before a preset is loaded.
    BeginSetPreset,
    /// Called after a preset is loaded.
    EndSetPreset,

    /// [value]: inputs `*mut VstSpeakerArrangement` //TODO: Implement
    /// [ptr]: Outputs `*mut VstSpeakerArrangement`
    GetSpeakerArrangement,
    /// [ptr]: buffer for plugin name, limited to `consts::MAX_PRODUCT_STR_LEN`.
    /// [return]: next plugin's uniqueID.
    ShellGetNextPlugin,

    /// No args. Called once before start of process call. This indicates that the process call
    /// will be interrupted (e.g. Host reconfiguration or bypass when plugin doesn't support
    /// SoftBypass)
    StartProcess,
    /// No arguments. Called after stop of process call.
    StopProcess,
    /// [value]: number of samples to process. Called in offline mode before process.
    SetTotalSampleToProcess,
    /// [value]: pan law `PanLaw`. //TODO: Implement
    /// [opt]: gain.
    SetPanLaw,

    /// [ptr]: `*mut VstPatchChunkInfo`. //TODO: Implement
    /// [return]: -1 = bank cant be loaded, 1 = can be loaded, 0 = unsupported.
    BeginLoadBank,
    /// [ptr]: `*mut VstPatchChunkInfo`. //TODO: Implement
    /// [return]: -1 = bank cant be loaded, 1 = can be loaded, 0 = unsupported.
    BeginLoadPreset,

    /// [value]: 0 if 32 bit, anything else if 64 bit.
    SetPrecision,

    /// [return]: number of used MIDI Inputs (1-15).
    GetNumMidiInputs,
    /// [return]: number of used MIDI Outputs (1-15).
    GetNumMidiOutputs,
}
impl_clike!(OpCode);

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

    /// Plugin version (e.g. 0001 = `v0.0.0.1`, 1283 = `v1.2.8.3`).
    pub version: i32,

    /// Plugin category. Possible values are found in `enums::PluginCategory`.
    pub category: Category,

    /// Latency of the plugin in samples.
    ///
    /// This reports how many samples it takes for the plugin to create an output (group delay).
    pub initial_delay: i32,

    /// Indicates that preset data is handled in formatless chunks.
    ///
    /// If false, host saves and restores plugin by reading/writing parameter data. If true, it is
    /// up to the plugin to manage saving preset data by implementing the
    /// `{get, load}_{preset, bank}_chunks()` methods. Default is `false`.
    pub preset_chunks: bool,

    /// Indicates whether this plugin can process f64 based `AudioBuffer` buffers.
    ///
    /// Default is `true`.
    pub f64_precision: bool,

    /// If this is true, the plugin will not produce sound when the input is silence.
    ///
    /// Default is `false`.
    pub silent_when_stopped: bool,
}

impl Default for Info {
    fn default() -> Info {
        Info {
            name: "VST".to_string(),
            vendor: String::new(),

            presets: 1, // default preset
            parameters: 0,
            inputs: 2, // Stereo in,out
            outputs: 2,

            unique_id: 0, // This must be changed.
            version: 0001, // v0.0.0.1

            category: Category::Effect,

            initial_delay: 0,

            preset_chunks: false,
            f64_precision: true,
            silent_when_stopped: false,
        }
    }
}

/// Features which are optionally supported by a plugin. These are queried by the host at run time.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum CanDo {
    SendEvents,
    SendMidiEvent,
    ReceiveEvents,
    ReceiveMidiEvent,
    ReceiveTimeInfo,
    Offline,
    MidiProgramNames,
    Bypass,
    ReceiveSysExEvent,

    //Bitwig specific?
    MidiSingleNoteTuningChange,
    MidiKeyBasedInstrumentControl,

    Other(String)
}

use std::str::FromStr;
impl FromStr for CanDo {
    type Err = String;

    fn from_str(s: &str) -> Result<CanDo, String> {
        use self::CanDo::*;

        Ok(match s {
            "sendVstEvents" => SendEvents,
            "sendVstMidiEvent" => SendMidiEvent,
            "receiveVstEvents" => ReceiveEvents,
            "receiveVstMidiEvent" => ReceiveMidiEvent,
            "receiveVstTimeInfo" => ReceiveTimeInfo,
            "offline" => Offline,
            "midiProgramNames" => MidiProgramNames,
            "bypass" => Bypass,

            "receiveVstSysexEvent" => ReceiveSysExEvent,
            "midiSingleNoteTuningChange" => MidiSingleNoteTuningChange,
            "midiKeyBasedInstrumentControl" => MidiKeyBasedInstrumentControl,
            otherwise => Other(otherwise.to_string())
        })
    }
}

impl Into<String> for CanDo {
    fn into(self) -> String {
        use self::CanDo::*;

        match self {
            SendEvents => "sendVstEvents".to_string(),
            SendMidiEvent => "sendVstMidiEvent".to_string(),
            ReceiveEvents => "receiveVstEvents".to_string(),
            ReceiveMidiEvent => "receiveVstMidiEvent".to_string(),
            ReceiveTimeInfo => "receiveVstTimeInfo".to_string(),
            Offline => "offline".to_string(),
            MidiProgramNames => "midiProgramNames".to_string(),
            Bypass => "bypass".to_string(),

            ReceiveSysExEvent => "receiveVstSysexEvent".to_string(),
            MidiSingleNoteTuningChange => "midiSingleNoteTuningChange".to_string(),
            MidiKeyBasedInstrumentControl => "midiKeyBasedInstrumentControl".to_string(),
            Other(other) => other
        }
    }

}

/// Must be implemented by all VST plugins.
///
/// All methods except `get_info` provide a default implementation which does nothing and can be
/// safely overridden.
#[allow(unused_variables)]
pub trait Plugin {
    /// This method must return an `Info` struct.
    fn get_info(&self) -> Info;

    /// Called during initialization to pass a `HostCallback` to the plugin.
    ///
    /// This method can be overriden to set `host` as a field in the plugin struct.
    ///
    /// # Example
    ///
    /// ```
    /// // ...
    /// # extern crate vst2;
    /// # #[macro_use] extern crate log;
    /// # use vst2::plugin::{Plugin, Info};
    /// use vst2::plugin::HostCallback;
    ///
    /// # #[derive(Default)]
    /// struct ExamplePlugin {
    ///     host: HostCallback
    /// }
    ///
    /// impl Plugin for ExamplePlugin {
    ///     fn new(host: HostCallback) -> ExamplePlugin {
    ///         ExamplePlugin {
    ///             host: host
    ///         }
    ///     }
    ///
    ///     fn init(&mut self) {
    ///         info!("loaded with host vst version: {}", self.host.vst_version());
    ///     }
    ///
    ///     // ...
    /// #     fn get_info(&self) -> Info {
    /// #         Info {
    /// #             name: "Example Plugin".to_string(),
    /// #             ..Default::default()
    /// #         }
    /// #     }
    /// }
    ///
    /// # fn main() {}
    /// ```
    fn new(host: HostCallback) -> Self where Self: Sized + Default {
        Default::default()
    }

    /// Called when plugin is fully initialized.
    fn init(&mut self) { trace!("Initialized vst plugin."); }


    /// Set the current preset to the index specified by `preset`.
    fn change_preset(&mut self, preset: i32) { }

    /// Get the current preset index.
    fn get_preset_num(&self) -> i32 { 0 }

    /// Set the current preset name.
    fn set_preset_name(&mut self, name: String) { }

    /// Get the name of the preset at the index specified by `preset`.
    fn get_preset_name(&self, preset: i32) -> String { "".to_string() }


    /// Get parameter label for parameter at `index` (e.g. "db", "sec", "ms", "%").
    fn get_parameter_label(&self, index: i32) -> String { "".to_string() }

    /// Get the parameter value for parameter at `index` (e.g. "1.0", "150", "Plate", "Off").
    fn get_parameter_text(&self, index: i32) -> String {
        format!("{:.3}", self.get_parameter(index))
    }

    /// Get the name of parameter at `index`.
    fn get_parameter_name(&self, index: i32) -> String { format!("Param {}", index) }

    /// Get the value of paramater at `index`. Should be value between 0.0 and 1.0.
    fn get_parameter(&self, index: i32) -> f32 { 0.0 }

    /// Set the value of parameter at `index`. `value` is between 0.0 and 1.0.
    fn set_parameter(&mut self, index: i32, value: f32) { }

    /// Return whether parameter at `index` can be automated.
    fn can_be_automated(&self, index: i32) -> bool { false }

    /// Use String as input for parameter value. Used by host to provide an editable field to
    /// adjust a parameter value. E.g. "100" may be interpreted as 100hz for parameter. Returns if
    /// the input string was used.
    fn string_to_parameter(&mut self, index: i32, text: String) -> bool { false }


    /// Called when sample rate is changed by host.
    fn set_sample_rate(&mut self, rate: f32) { }

    /// Called when block size is changed by host.
    fn set_block_size(&mut self, size: i64) { }


    /// Called when plugin is turned on.
    fn resume(&mut self) { }

    /// Called when plugin is turned off.
    fn suspend(&mut self) { }


    /// Vendor specific handling.
    fn vendor_specific(&mut self, index: i32, value: isize, ptr: *mut c_void, opt: f32) -> isize { 0 }


    /// Return whether plugin supports specified action.
    fn can_do(&self, can_do: CanDo) -> Supported {
        info!("Host is asking if plugin can: {:?}.", can_do);
        Supported::Maybe
    }

    /// Get the tail size of plugin when it is stopped. Used in offline processing as well.
    fn get_tail_size(&self) -> isize { 0 }


    /// Process an audio buffer containing `f32` values.
    ///
    /// # Example
    /// ```no_run
    /// # use vst2::plugin::{Info, Plugin};
    /// # use vst2::buffer::AudioBuffer;
    /// #
    /// # struct ExamplePlugin;
    /// # impl Plugin for ExamplePlugin {
    /// #     fn get_info(&self) -> Info { Default::default() }
    /// #
    /// // Processor that clips samples above 0.4 or below -0.4:
    /// fn process(&mut self, buffer: &mut AudioBuffer<f32>){
    ///     // For each input and output
    ///     for (input, output) in buffer.zip() {
    ///         // For each input sample and output sample in buffer
    ///         for (in_sample, out_sample) in input.into_iter().zip(output.into_iter()) {
    ///             *out_sample = if *in_sample > 0.4 {
    ///                 0.4
    ///             } else if *in_sample < -0.4 {
    ///                 -0.4
    ///             } else {
    ///                 *in_sample
    ///             };
    ///         }
    ///     }
    /// }
    /// # }
    /// ```
    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        // For each input and output
        for (input, output) in buffer.zip() {
            // For each input sample and output sample in buffer
            for (in_frame, out_frame) in input.into_iter().zip(output.into_iter()) {
                *out_frame = *in_frame;
            }
        }
    }

    /// Process an audio buffer containing `f64` values.
    ///
    /// # Example
    /// ```no_run
    /// # use vst2::plugin::{Info, Plugin};
    /// # use vst2::buffer::AudioBuffer;
    /// #
    /// # struct ExamplePlugin;
    /// # impl Plugin for ExamplePlugin {
    /// #     fn get_info(&self) -> Info { Default::default() }
    /// #
    /// // Processor that clips samples above 0.4 or below -0.4:
    /// fn process_f64(&mut self, buffer: &mut AudioBuffer<f64>){
    ///     // For each input and output
    ///     for (input, output) in buffer.zip() {
    ///         // For each input sample and output sample in buffer
    ///         for (in_sample, out_sample) in input.into_iter().zip(output.into_iter()) {
    ///             *out_sample = if *in_sample > 0.4 {
    ///                 0.4
    ///             } else if *in_sample < -0.4 {
    ///                 -0.4
    ///             } else {
    ///                 *in_sample
    ///             };
    ///         }
    ///     }
    /// }
    /// # }
    /// ```
    fn process_f64(&mut self, buffer: &mut AudioBuffer<f64>) {
        // For each input and output
        for (input, output) in buffer.zip() {
            // For each input sample and output sample in buffer
            for (in_frame, out_frame) in input.into_iter().zip(output.into_iter()) {
                *out_frame = *in_frame;
            }
        }
    }

    /// Handle incoming events sent from the host.
    ///
    /// This is always called before the start of `process` or `process_f64`.
    fn process_events(&mut self, events: &api::Events) {}

    /// Return handle to plugin editor if supported.
    fn get_editor(&mut self) -> Option<&mut Editor> { None }


    /// If `preset_chunks` is set to true in plugin info, this should return the raw chunk data for
    /// the current preset.
    fn get_preset_data(&mut self) -> Vec<u8> { Vec::new() }

    /// If `preset_chunks` is set to true in plugin info, this should return the raw chunk data for
    /// the current plugin bank.
    fn get_bank_data(&mut self) -> Vec<u8> { Vec::new() }

    /// If `preset_chunks` is set to true in plugin info, this should load a preset from the given
    /// chunk data.
    fn load_preset_data(&mut self, data: &[u8]) {}

    /// If `preset_chunks` is set to true in plugin info, this should load a preset bank from the
    /// given chunk data.
    fn load_bank_data(&mut self, data: &[u8]) {}

    /// Get information about an input channel. Only used by some hosts.
    fn get_input_info(&self, input: i32) -> ChannelInfo {
        ChannelInfo::new(format!("Input channel {}", input),
                         Some(format!("In {}", input)),
                         true, None)
    }

    /// Get information about an output channel. Only used by some hosts.
    fn get_output_info(&self, output: i32) -> ChannelInfo {
        ChannelInfo::new(format!("Output channel {}", output),
                         Some(format!("Out {}", output)),
                         true, None)
    }
}

/// A reference to the host which allows the plugin to call back and access information.
///
/// # Panics
///
/// All methods in this struct will panic if the plugin has not yet been initialized. In practice,
/// this can only occur if the plugin queries the host for information when `Default::default()` is
/// called.
///
/// ```should_panic
/// # use vst2::plugin::{Info, Plugin, HostCallback};
/// struct ExamplePlugin;
///
/// impl Default for ExamplePlugin {
///     fn default() -> ExamplePlugin {
///         // Will panic, don't do this. If needed, you can query
///         // the host during initialization via Vst::new()
///         let host: HostCallback = Default::default();
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
pub struct HostCallback {
    callback: Option<HostCallbackProc>,
    effect: *mut AEffect,
}

/// `HostCallback` implements `Default` so that the plugin can implement `Default` and have a
/// `HostCallback` field.
impl Default for HostCallback {
    fn default() -> HostCallback {
        HostCallback {
            callback: None,
            effect: ptr::null_mut(),
        }
    }
}

impl HostCallback {
    /// Wrap callback in a function to avoid using fn pointer notation.
    #[doc(hidden)]
    fn callback(&self,
                effect: *mut AEffect,
                opcode: host::OpCode,
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
    pub fn wrap(callback: HostCallbackProc, effect: *mut AEffect) -> HostCallback {
        HostCallback {
            callback: Some(callback),
            effect: effect,
        }
    }

    /// Get the VST API version supported by the host e.g. `2400 = VST 2.4`.
    pub fn vst_version(&self) -> i32 {
        self.callback(self.effect, host::OpCode::Version,
                      0, 0, ptr::null_mut(), 0.0) as i32
    }

    fn read_string(&self, opcode: host::OpCode, max: usize) -> String {
        self.read_string_param(opcode, 0, 0, 0.0, max)
    }

    fn read_string_param(&self,
                         opcode: host::OpCode,
                         index: i32,
                         value: isize,
                         opt: f32,
                         max: usize)
                         -> String {
        let mut buf = vec![0; max];
        self.callback(self.effect, opcode, index, value, buf.as_mut_ptr() as *mut c_void, opt);
        String::from_utf8_lossy(&buf).chars().take_while(|c| *c != '\0').collect()
    }
}

impl Host for HostCallback {
    fn automate(&mut self, index: i32, value: f32) {
        if self.is_effect_valid() { // TODO: Investigate removing this check, should be up to host
            self.callback(self.effect, host::OpCode::Automate,
                          index, 0, ptr::null_mut(), value);
        }
    }

    fn get_plugin_id(&self) -> i32 {
        self.callback(self.effect, host::OpCode::CurrentId,
                      0, 0, ptr::null_mut(), 0.0) as i32
    }

    fn idle(&self) {
        self.callback(self.effect, host::OpCode::Idle,
                      0, 0, ptr::null_mut(), 0.0);
    }

    fn get_info(&self) -> (isize, String, String) {
        use api::consts::*;
        let version = self.callback(self.effect, host::OpCode::CurrentId, 0, 0, ptr::null_mut(), 0.0) as isize;
        let vendor_name = self.read_string(host::OpCode::GetVendorString, MAX_VENDOR_STR_LEN);
        let product_name = self.read_string(host::OpCode::GetProductString, MAX_PRODUCT_STR_LEN);
        (version, vendor_name, product_name)
    }

    /// Send events to the host.
    ///
    /// This should only be called within [`process`] or [`process_f64`]. Calling `process_events`
    /// anywhere else is undefined behaviour and may crash some hosts.
    ///
    /// [`process`]: trait.Plugin.html#method.process
    /// [`process_f64`]: trait.Plugin.html#method.process_f64
    fn process_events(&mut self, events: &api::Events) {
        self.callback(
            self.effect,
            host::OpCode::ProcessEvents,
            0,
            0,
            events as *const _ as *mut _,
            0.0
        );
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
            use std::os::raw::c_void;

            use main;
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
        (unsafe { (*aeffect).dispatcher })(aeffect, plugin::OpCode::Initialize.into(),
                                           0, 0, ptr::null_mut(), 0.0);
    }
}
