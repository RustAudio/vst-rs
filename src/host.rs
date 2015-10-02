//! Host specific structures.

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
    /// Deprecated.
    _SetTime,
    /// Deprecated.
    _TempoAt,
    /// Deprecated.
    _GetNumAutomatableParameters,
    /// Deprecated.
    _GetParameterQuantization,

    /// Notifies the host that the input/output setup has changed. This can allow the host to check
    /// numInputs/numOutputs or call `getSpeakerArrangement()`
    /// [return]: 1 if supported.
    IOChanged,

    /// Deprecated.
    _NeedIdle,
}
impl_clike!(OpCode);

/// Methods that can be called on a host
pub trait Host {
    /// Notify the host that a parameter value was changed.
    fn automate(&mut self, index: i32, value: f32);

    /// Get the plugin ID the host is requesting to load.
    ///
    /// This is only useful for shell plugins where this value will change the plugin returned.
    /// `TODO: implement shell plugins`
    fn get_plugin_id(&self) -> i32;

    /// Tell the host that it can idle.
    ///
    /// This is useful when the plugin is doing something such as mouse tracking in the UI.
    fn idle(&self);
}
