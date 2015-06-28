//! Plugin specific structures.

use libc::c_void;

use channels::ChannelInfo;
use host::Host;
use api::Supported;
use buffer::AudioBuffer;
use editor::Editor;

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
    /// [ptr]: char buffer for current preset name, limited to `consts::MAX_PRSET_NAME_LEN`.
    GetCurrentPresetName,

    /// [ptr]: char buffer for parameter label (e.g. "db", "ms", etc).
    GetParameterLabel,
    /// [ptr]: char buffer (e.g. "0.5", "ROOM", etc).
    GetParameterDisplay,
    /// [ptr]: char buffer. (e.g. "Release", "Gain").
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

    //Bitwig specific?
    ReceiveSysexEvent,
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

            "receiveVstSysexEvent" => ReceiveSysexEvent,
            "midiSingleNoteTuningChange" => MidiSingleNoteTuningChange,
            "midiKeyBasedInstrumentControl" => MidiKeyBasedInstrumentControl,
            otherwise => Other(otherwise.to_string())
        })
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

    /// Called during initialization to pass a Host wrapper to the plugin.
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
    /// use vst2::host::Host;
    ///
    /// # #[derive(Default)]
    /// struct ExamplePlugin {
    ///     host: Host
    /// }
    ///
    /// impl Plugin for ExamplePlugin {
    ///     fn new(host: Host) -> ExamplePlugin {
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
    fn new(host: Host) -> Self where Self: Sized + Default {
        Default::default()
    }

    /// Called when plugin is fully initialized.
    fn init(&mut self) { trace!("Initialized vst plugin."); }


    /// Set the current preset to the index specified by `preset`.
    fn change_preset(&mut self, preset: i32) { }

    /// Get the current preset index.
    fn get_preset_num(&self) -> i32 { 0 }

    /// Set the current preset name.
    fn set_preset_name(&self, name: String) { }

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
    fn string_to_parameter(&self, index: i32, text: String) -> bool { false }


    /// Called when sample rate is changed by host.
    fn sample_rate_changed(&mut self, rate: f32) { }

    /// Called when block size is changed by host.
    fn block_size_changed(&mut self, size: i64) { }


    /// Called when plugin is turned on.
    fn on_resume(&mut self) { }

    /// Called when plugin is turned off.
    fn on_suspend(&mut self) { }


    /// Vendor specific handling.
    fn vendor_specific(&mut self, index: i32, value: isize, ptr: *mut c_void, opt: f32) { }


    /// Return whether plugin supports specified action.
    fn can_do(&self, can_do: CanDo) -> Supported {
        info!("Host is asking if plugin can: {:?}.", can_do);
        Supported::Maybe
    }

    /// Get the tail size of plugin when it is stopped. Used in offline processing as well.
    fn get_tail_size(&self) -> isize { 0 }


    /// Process an audio buffer containing `f32` values. TODO: Examples
    fn process(&mut self, buffer: AudioBuffer<f32>) {
        // For each input and output
        for (input, output) in buffer.zip() {
            // For each input sample and output sample in buffer
            for (in_frame, out_frame) in input.into_iter().zip(output.into_iter()) {
                *out_frame = *in_frame;
            }
        }
    }

    /// Process an audio buffer containing `f64` values. TODO: Examples
    fn process_f64(&mut self, buffer: AudioBuffer<f64>) {
        // For each input and output
        for (input, output) in buffer.zip() {
            // For each input sample and output sample in buffer
            for (in_frame, out_frame) in input.into_iter().zip(output.into_iter()) {
                *out_frame = *in_frame;
            }
        }
    }

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
    fn load_preset_data(&mut self, data: Vec<u8>) {}

    /// If `preset_chunks` is set to true in plugin info, this should load a preset bank from the
    /// given chunk data.
    fn load_bank_data(&mut self, data: Vec<u8>) {}

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
