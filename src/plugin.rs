//! Plugin specific structures.

use num_enum::{IntoPrimitive, TryFromPrimitive};

use std::os::raw::c_void;
use std::ptr;
use std::sync::Arc;

use crate::{
    api::{self, consts::VST_MAGIC, AEffect, HostCallbackProc, Supported, TimeInfo},
    buffer::AudioBuffer,
    channels::ChannelInfo,
    editor::Editor,
    host::{self, Host},
};

/// Plugin type. Generally either Effect or Synth.
///
/// Other types are not necessary to build a plugin and are only useful for the host to categorize
/// the plugin.
#[repr(isize)]
#[derive(Clone, Copy, Debug, TryFromPrimitive, IntoPrimitive)]
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
    Generator,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, TryFromPrimitive, IntoPrimitive)]
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
    /// [index]: parameter
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
    /// [return]: tail size (e.g. reverb time). 0 is default, 1 means no tail.
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

    /// Number of MIDI input channels (1-16), or 0 for the default of 16 channels.
    pub midi_inputs: i32,

    /// Number of MIDI output channels (1-16), or 0 for the default of 16 channels.
    pub midi_outputs: i32,

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
    /// Default is `false`.
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

            midi_inputs: 0,
            midi_outputs: 0,

            unique_id: 0, // This must be changed.
            version: 1,   // v0.0.0.1

            category: Category::Effect,

            initial_delay: 0,

            preset_chunks: false,
            f64_precision: false,
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

    Other(String),
}

impl CanDo {
    // TODO: implement FromStr
    #![allow(clippy::should_implement_trait)]
    /// Converts a string to a `CanDo` instance. Any given string that does not match the predefined
    /// values will return a `CanDo::Other` value.
    pub fn from_str(s: &str) -> CanDo {
        use self::CanDo::*;

        match s {
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
            otherwise => Other(otherwise.to_string()),
        }
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
            Other(other) => other,
        }
    }
}

/// Must be implemented by all VST plugins.
///
/// All methods except `new` and `get_info` provide a default implementation
/// which does nothing and can be safely overridden.
///
/// At any time, a plugin is in one of two states: *suspended* or *resumed*.
/// While a plugin is in the *suspended* state, various processing parameters,
/// such as the sample rate and block size, can be changed by the host, but no
/// audio processing takes place. While a plugin is in the *resumed* state,
/// audio processing methods and parameter access methods can be called by
/// the host. A plugin starts in the *suspended* state and is switched between
/// the states by the host using the `resume` and `suspend` methods.
///
/// Hosts call methods of the plugin on two threads: the UI thread and the
/// processing thread. For this reason, the plugin API is separated into two
/// traits: The `Plugin` trait containing setup and processing methods, and
/// the `PluginParameters` trait containing methods for parameter access.
#[cfg_attr(
    not(feature = "disable_deprecation_warning"),
    deprecated = "This crate has been deprecated. See https://github.com/RustAudio/vst-rs for more information."
)]
#[allow(unused_variables)]
pub trait Plugin: Send {
    /// This method must return an `Info` struct.
    fn get_info(&self) -> Info;

    /// Called during initialization to pass a `HostCallback` to the plugin.
    ///
    /// This method can be overridden to set `host` as a field in the plugin struct.
    ///
    /// # Example
    ///
    /// ```
    /// // ...
    /// # extern crate vst;
    /// # #[macro_use] extern crate log;
    /// # use vst::plugin::{Plugin, Info};
    /// use vst::plugin::HostCallback;
    ///
    /// struct ExamplePlugin {
    ///     host: HostCallback
    /// }
    ///
    /// impl Plugin for ExamplePlugin {
    ///     fn new(host: HostCallback) -> ExamplePlugin {
    ///         ExamplePlugin {
    ///             host
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
    fn new(host: HostCallback) -> Self
    where
        Self: Sized;

    /// Called when plugin is fully initialized.
    ///
    /// This method is only called while the plugin is in the *suspended* state.
    fn init(&mut self) {
        trace!("Initialized vst plugin.");
    }

    /// Called when sample rate is changed by host.
    ///
    /// This method is only called while the plugin is in the *suspended* state.
    fn set_sample_rate(&mut self, rate: f32) {}

    /// Called when block size is changed by host.
    ///
    /// This method is only called while the plugin is in the *suspended* state.
    fn set_block_size(&mut self, size: i64) {}

    /// Called to transition the plugin into the *resumed* state.
    fn resume(&mut self) {}

    /// Called to transition the plugin into the *suspended* state.
    fn suspend(&mut self) {}

    /// Vendor specific handling.
    fn vendor_specific(&mut self, index: i32, value: isize, ptr: *mut c_void, opt: f32) -> isize {
        0
    }

    /// Return whether plugin supports specified action.
    ///
    /// This method is only called while the plugin is in the *suspended* state.
    fn can_do(&self, can_do: CanDo) -> Supported {
        info!("Host is asking if plugin can: {:?}.", can_do);
        Supported::Maybe
    }

    /// Get the tail size of plugin when it is stopped. Used in offline processing as well.
    fn get_tail_size(&self) -> isize {
        0
    }

    /// Process an audio buffer containing `f32` values.
    ///
    /// # Example
    /// ```no_run
    /// # use vst::plugin::{HostCallback, Info, Plugin};
    /// # use vst::buffer::AudioBuffer;
    /// #
    /// # struct ExamplePlugin;
    /// # impl Plugin for ExamplePlugin {
    /// #     fn new(_host: HostCallback) -> Self { Self }
    /// #
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
    ///
    /// This method is only called while the plugin is in the *resumed* state.
    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        // For each input and output
        for (input, output) in buffer.zip() {
            // For each input sample and output sample in buffer
            for (in_frame, out_frame) in input.iter().zip(output.iter_mut()) {
                *out_frame = *in_frame;
            }
        }
    }

    /// Process an audio buffer containing `f64` values.
    ///
    /// # Example
    /// ```no_run
    /// # use vst::plugin::{HostCallback, Info, Plugin};
    /// # use vst::buffer::AudioBuffer;
    /// #
    /// # struct ExamplePlugin;
    /// # impl Plugin for ExamplePlugin {
    /// #     fn new(_host: HostCallback) -> Self { Self }
    /// #
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
    ///
    /// This method is only called while the plugin is in the *resumed* state.
    fn process_f64(&mut self, buffer: &mut AudioBuffer<f64>) {
        // For each input and output
        for (input, output) in buffer.zip() {
            // For each input sample and output sample in buffer
            for (in_frame, out_frame) in input.iter().zip(output.iter_mut()) {
                *out_frame = *in_frame;
            }
        }
    }

    /// Handle incoming events sent from the host.
    ///
    /// This is always called before the start of `process` or `process_f64`.
    ///
    /// This method is only called while the plugin is in the *resumed* state.
    fn process_events(&mut self, events: &api::Events) {}

    /// Get a reference to the shared parameter object.
    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        Arc::new(DummyPluginParameters)
    }

    /// Get information about an input channel. Only used by some hosts.
    fn get_input_info(&self, input: i32) -> ChannelInfo {
        ChannelInfo::new(
            format!("Input channel {}", input),
            Some(format!("In {}", input)),
            true,
            None,
        )
    }

    /// Get information about an output channel. Only used by some hosts.
    fn get_output_info(&self, output: i32) -> ChannelInfo {
        ChannelInfo::new(
            format!("Output channel {}", output),
            Some(format!("Out {}", output)),
            true,
            None,
        )
    }

    /// Called one time before the start of process call.
    ///
    /// This indicates that the process call will be interrupted (due to Host reconfiguration
    /// or bypass state when the plug-in doesn't support softBypass).
    ///
    /// This method is only called while the plugin is in the *resumed* state.
    fn start_process(&mut self) {}

    /// Called after the stop of process call.
    ///
    /// This method is only called while the plugin is in the *resumed* state.
    fn stop_process(&mut self) {}

    /// Return handle to plugin editor if supported.
    /// The method need only return the object on the first call.
    /// Subsequent calls can just return `None`.
    ///
    /// The editor object will typically contain an `Arc` reference to the parameter
    /// object through which it can communicate with the audio processing.
    fn get_editor(&mut self) -> Option<Box<dyn Editor>> {
        None
    }
}

/// Parameter object shared between the UI and processing threads.
/// Since access is shared, all methods take `self` by immutable reference.
/// All mutation must thus be performed using thread-safe interior mutability.
#[allow(unused_variables)]
pub trait PluginParameters: Sync {
    /// Set the current preset to the index specified by `preset`.
    ///
    /// This method can be called on the processing thread for automation.
    fn change_preset(&self, preset: i32) {}

    /// Get the current preset index.
    fn get_preset_num(&self) -> i32 {
        0
    }

    /// Set the current preset name.
    fn set_preset_name(&self, name: String) {}

    /// Get the name of the preset at the index specified by `preset`.
    fn get_preset_name(&self, preset: i32) -> String {
        "".to_string()
    }

    /// Get parameter label for parameter at `index` (e.g. "db", "sec", "ms", "%").
    fn get_parameter_label(&self, index: i32) -> String {
        "".to_string()
    }

    /// Get the parameter value for parameter at `index` (e.g. "1.0", "150", "Plate", "Off").
    fn get_parameter_text(&self, index: i32) -> String {
        format!("{:.3}", self.get_parameter(index))
    }

    /// Get the name of parameter at `index`.
    fn get_parameter_name(&self, index: i32) -> String {
        format!("Param {}", index)
    }

    /// Get the value of parameter at `index`. Should be value between 0.0 and 1.0.
    fn get_parameter(&self, index: i32) -> f32 {
        0.0
    }

    /// Set the value of parameter at `index`. `value` is between 0.0 and 1.0.
    ///
    /// This method can be called on the processing thread for automation.
    fn set_parameter(&self, index: i32, value: f32) {}

    /// Return whether parameter at `index` can be automated.
    fn can_be_automated(&self, index: i32) -> bool {
        true
    }

    /// Use String as input for parameter value. Used by host to provide an editable field to
    /// adjust a parameter value. E.g. "100" may be interpreted as 100hz for parameter. Returns if
    /// the input string was used.
    fn string_to_parameter(&self, index: i32, text: String) -> bool {
        false
    }

    /// If `preset_chunks` is set to true in plugin info, this should return the raw chunk data for
    /// the current preset.
    fn get_preset_data(&self) -> Vec<u8> {
        Vec::new()
    }

    /// If `preset_chunks` is set to true in plugin info, this should return the raw chunk data for
    /// the current plugin bank.
    fn get_bank_data(&self) -> Vec<u8> {
        Vec::new()
    }

    /// If `preset_chunks` is set to true in plugin info, this should load a preset from the given
    /// chunk data.
    fn load_preset_data(&self, data: &[u8]) {}

    /// If `preset_chunks` is set to true in plugin info, this should load a preset bank from the
    /// given chunk data.
    fn load_bank_data(&self, data: &[u8]) {}
}

struct DummyPluginParameters;

impl PluginParameters for DummyPluginParameters {}

/// A reference to the host which allows the plugin to call back and access information.
///
/// # Panics
///
/// All methods in this struct will panic if the `HostCallback` was constructed using
/// `Default::default()` rather than being set to the value passed to `Plugin::new`.
#[derive(Copy, Clone)]
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

unsafe impl Send for HostCallback {}
unsafe impl Sync for HostCallback {}

impl HostCallback {
    /// Wrap callback in a function to avoid using fn pointer notation.
    #[doc(hidden)]
    fn callback(
        &self,
        effect: *mut AEffect,
        opcode: host::OpCode,
        index: i32,
        value: isize,
        ptr: *mut c_void,
        opt: f32,
    ) -> isize {
        let callback = self.callback.unwrap_or_else(|| panic!("Host not yet initialized."));
        callback(effect, opcode.into(), index, value, ptr, opt)
    }

    /// Check whether the plugin has been initialized.
    #[doc(hidden)]
    fn is_effect_valid(&self) -> bool {
        // Check whether `effect` points to a valid AEffect struct
        unsafe { (*self.effect).magic as i32 == VST_MAGIC }
    }

    /// Create a new Host structure wrapping a host callback.
    #[doc(hidden)]
    pub fn wrap(callback: HostCallbackProc, effect: *mut AEffect) -> HostCallback {
        HostCallback {
            callback: Some(callback),
            effect,
        }
    }

    /// Get the VST API version supported by the host e.g. `2400 = VST 2.4`.
    pub fn vst_version(&self) -> i32 {
        self.callback(self.effect, host::OpCode::Version, 0, 0, ptr::null_mut(), 0.0) as i32
    }

    /// Get the callback for calling host-specific extensions
    #[inline(always)]
    pub fn raw_callback(&self) -> Option<HostCallbackProc> {
        self.callback
    }

    /// Get the effect pointer for calling host-specific extensions
    #[inline(always)]
    pub fn raw_effect(&self) -> *mut AEffect {
        self.effect
    }

    fn read_string(&self, opcode: host::OpCode, max: usize) -> String {
        self.read_string_param(opcode, 0, 0, 0.0, max)
    }

    fn read_string_param(&self, opcode: host::OpCode, index: i32, value: isize, opt: f32, max: usize) -> String {
        let mut buf = vec![0; max];
        self.callback(self.effect, opcode, index, value, buf.as_mut_ptr() as *mut c_void, opt);
        String::from_utf8_lossy(&buf)
            .chars()
            .take_while(|c| *c != '\0')
            .collect()
    }
}

impl Host for HostCallback {
    /// Signal the host that the value for the parameter has changed.
    ///
    /// Make sure to also call `begin_edit` and `end_edit` when a parameter
    /// has been touched. This is important for the host to determine
    /// if a user interaction is happening and the automation should be recorded.
    fn automate(&self, index: i32, value: f32) {
        if self.is_effect_valid() {
            // TODO: Investigate removing this check, should be up to host
            self.callback(self.effect, host::OpCode::Automate, index, 0, ptr::null_mut(), value);
        }
    }

    /// Signal the host the start of a parameter change a gesture (mouse down on knob dragging).
    fn begin_edit(&self, index: i32) {
        self.callback(self.effect, host::OpCode::BeginEdit, index, 0, ptr::null_mut(), 0.0);
    }

    /// Signal the host the end of a parameter change gesture (mouse up after knob dragging).
    fn end_edit(&self, index: i32) {
        self.callback(self.effect, host::OpCode::EndEdit, index, 0, ptr::null_mut(), 0.0);
    }

    fn get_plugin_id(&self) -> i32 {
        self.callback(self.effect, host::OpCode::CurrentId, 0, 0, ptr::null_mut(), 0.0) as i32
    }

    fn idle(&self) {
        self.callback(self.effect, host::OpCode::Idle, 0, 0, ptr::null_mut(), 0.0);
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
    fn process_events(&self, events: &api::Events) {
        self.callback(
            self.effect,
            host::OpCode::ProcessEvents,
            0,
            0,
            events as *const _ as *mut _,
            0.0,
        );
    }

    /// Request time information from Host.
    ///
    /// The mask parameter is composed of the same flags which will be found in the `flags` field of `TimeInfo` when returned.
    /// That is, if you want the host's tempo, the parameter passed to `get_time_info()` should have the `TEMPO_VALID` flag set.
    /// This request and delivery system is important, as a request like this may cause
    /// significant calculations at the application's end, which may take a lot of our precious time.
    /// This obviously means you should only set those flags that are required to get the information you need.
    ///
    /// Also please be aware that requesting information does not necessarily mean that that information is provided in return.
    /// Check the flags field in the `TimeInfo` structure to see if your request was actually met.
    fn get_time_info(&self, mask: i32) -> Option<TimeInfo> {
        let opcode = host::OpCode::GetTime;
        let mask = mask as isize;
        let null = ptr::null_mut();
        let ptr = self.callback(self.effect, opcode, 0, mask, null, 0.0);

        match ptr {
            0 => None,
            ptr => Some(unsafe { *(ptr as *const TimeInfo) }),
        }
    }

    /// Get block size.
    fn get_block_size(&self) -> isize {
        self.callback(self.effect, host::OpCode::GetBlockSize, 0, 0, ptr::null_mut(), 0.0)
    }

    /// Refresh UI after the plugin's parameters changed.
    fn update_display(&self) {
        self.callback(self.effect, host::OpCode::UpdateDisplay, 0, 0, ptr::null_mut(), 0.0);
    }
}

#[cfg(test)]
mod tests {
    use std::ptr;

    use crate::plugin;

    /// Create a plugin instance.
    ///
    /// This is a macro to allow you to specify attributes on the created struct.
    macro_rules! make_plugin {
        ($($attr:meta) *) => {
            use std::convert::TryFrom;
            use std::os::raw::c_void;

            use crate::main;
            use crate::api::AEffect;
            use crate::host::{Host, OpCode};
            use crate::plugin::{HostCallback, Info, Plugin};

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
                        host
                    }
                }

                fn init(&mut self) {
                    info!("Loaded with host vst version: {}", self.host.vst_version());
                    assert_eq!(2400, self.host.vst_version());
                    assert_eq!(9876, self.host.get_plugin_id());
                    // Callback will assert these.
                    self.host.begin_edit(123);
                    self.host.automate(123, 12.3);
                    self.host.end_edit(123);
                    self.host.idle();
                }
            }

            #[allow(dead_code)]
            fn instance() -> *mut AEffect {
                extern "C" fn host_callback(
                    _effect: *mut AEffect,
                    opcode: i32,
                    index: i32,
                    _value: isize,
                    _ptr: *mut c_void,
                    opt: f32,
                ) -> isize {
                    match OpCode::try_from(opcode) {
                        Ok(OpCode::BeginEdit) => {
                            assert_eq!(index, 123);
                            0
                        },
                        Ok(OpCode::Automate) => {
                            assert_eq!(index, 123);
                            assert_eq!(opt, 12.3);
                            0
                        },
                        Ok(OpCode::EndEdit) => {
                            assert_eq!(index, 123);
                            0
                        },
                        Ok(OpCode::Version) => 2400,
                        Ok(OpCode::CurrentId) => 9876,
                        Ok(OpCode::Idle) => 0,
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
                let plugin = TestPlugin {
                    host: Default::default(),
                };

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
        (unsafe { (*aeffect).dispatcher })(aeffect, plugin::OpCode::Initialize.into(), 0, 0, ptr::null_mut(), 0.0);
    }
}
