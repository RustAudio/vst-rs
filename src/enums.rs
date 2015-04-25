//! Enums for use in this library.

use collections::enum_set::CLike;

/// Quick implementation of CLike
macro_rules! impl_clike {
    ($t:ty) => {
        impl CLike for $t {
            fn to_usize(&self) -> usize {
                *self as usize
            }

            fn from_usize(v: usize) -> $t {
                use std::mem;
                unsafe { mem::transmute(v) }
            }
        }
    }
}

// generally, only effect and synth are necessary.
/// Plugin type. Generally either Effect or Synth.
///
/// Other types are not necessary to build a plugin and are only useful for the host to categorize
/// the plugin.
#[repr(usize)]
#[derive(Clone, Copy, Debug)]
pub enum PluginCategory {
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
impl_clike!(PluginCategory);

#[repr(usize)]
#[derive(Clone, Copy, Debug)]
#[doc(hidden)]
pub enum OpCode {
    Initialize,
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

    GetData, //void** for chunk data address. [index]: 0 for bank, 1 for program
    SetData, //[ptr]: data [value]: byte size [index]: 0 for bank, 1 for program

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

/// Used to specify whether functionality is supported.
#[allow(dead_code, missing_docs)]
pub enum Supported {
    Yes,
    Maybe,
    No
}

impl Supported {
    /// Convert to integer ordinal for interop with VST api.
    pub fn ordinal(self) -> i32 {
        use self::Supported::*;

        match self {
            Yes => 1,
            Maybe => 0,
            No => -1
        }
    }
}

/// For use in editor. Allows host to set how a parameter knob works.
#[repr(usize)]
#[derive(Copy, Clone, Debug)]
#[allow(missing_docs)]
pub enum KnobMode {
    Circular,
    CircularRelative,
    Linear
}
impl_clike!(KnobMode);

/// Platform independent key codes.
#[allow(dead_code, missing_docs)]
#[repr(usize)]
#[derive(Debug, Copy, Clone)]
pub enum Key {
	Back = 1,
	Tab,
	Clear,
	Return,
	Pause,
	Escape,
	Space,
	Next,
	End,
	Home,
	Left,
	Up,
	Right,
	Down,
	PageUp,
	PageDown,
	Select,
	Print,
	Enter,
	Snapshot,
	Insert,
	Delete,
	Help,
	Numpad0,
	Numpad1,
	Numpad2,
	Numpad3,
	Numpad4,
	Numpad5,
	Numpad6,
	Numpad7,
	Numpad8,
	Numpad9,
	Multiply,
	Add,
	Separator,
	Subtract,
	Decimal,
	Divide,
	F1,
	F2,
	F3,
	F4,
	F5,
	F6,
	F7,
	F8,
	F9,
	F10,
	F11,
	F12,
	Numlock,
	Scroll,
	Shift,
	Control,
	Alt,
	Equals
}
impl_clike!(Key);

/// Bitflags.
#[allow(dead_code, missing_docs)]
pub mod flags {
    /// Flags for VST plugins.
    pub mod plugin {
        bitflags! {
            flags Effect: i32 {
                /// Plugin has an editor.
                const HAS_EDITOR = 1 << 0,
                /// Plugin can process 32 bit audio. (Mandatory in VST 2.4).
                const CAN_REPLACING = 1 << 4,
                /// Plugin preset data is handled in formatless chunks.
                const PROGRAM_CHUNKS = 1 << 5,
                /// Plugin is a synth.
                const IS_SYNTH = 1 << 8,
                //TODO: Implement and doc.
                const NO_SOUND_IN_STOP = 1 << 9,
                /// Supports 64 bit audio processing.
                const CAN_DOUBLE_REPLACING = 1 << 12
            }
        }
    }

    /// Cross platform modifier key bitflags.
    pub mod modifier_key {
        bitflags!{
            flags ModifierKey: u8 {
                /// Shift key.
                const SHIFT = 1 << 0, // Shift
                /// Alt key.
                const ALT = 1 << 1, // Alt
                /// Control on mac.
                const COMMAND = 1 << 2, // Control on Mac
                /// Command on mac, ctrl on other.
                const CONTROL = 1 << 3  // Ctrl on PC, Apple on Mac
            }
        }
    }
}
