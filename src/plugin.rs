//! Plugin specific structures.

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

    GetData, //[ptr]: void** for chunk data address. [index]: 0 for bank, 1 for program
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
