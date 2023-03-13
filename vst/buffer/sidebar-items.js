window.SIDEBAR_ITEMS = {"struct":[["AudioBuffer","`AudioBuffer` contains references to the audio buffers for all input and output channels."],["AudioBufferIterator","Iterator over pairs of buffers of input channels and output channels."],["InputIterator","Iterator over buffers for input channels of an `AudioBuffer`."],["Inputs","Wrapper type to access the buffers for the input channels of an `AudioBuffer` in a safe way. Behaves like a slice."],["OutputIterator","Iterator over buffers for output channels of an `AudioBuffer`."],["Outputs","Wrapper type to access the buffers for the output channels of an `AudioBuffer` in a safe way. Behaves like a slice."],["SendEventBuffer","This buffer is used for sending midi events through the VST interface. The purpose of this is to convert outgoing midi events from `event::Event` to `api::Events`. It only allocates memory in new() and reuses the memory between calls."]],"trait":[["WriteIntoPlaceholder","This trait is used by `SendEventBuffer::send_events` to accept iterators over midi events"]],"type":[["PlaceholderEvent","This is used as a placeholder to pre-allocate space for a fixed number of midi events in the re-useable `SendEventBuffer`, because `SysExEvent` is larger than `MidiEvent`, so either one can be stored in a `SysExEvent`."]]};