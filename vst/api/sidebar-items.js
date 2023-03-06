window.SIDEBAR_ITEMS = {"enum":[["EventType","The type of event that has occurred. See `api::Event.event_type`."],["FileSelectCommand","The file operation to perform."],["FileSelectType","Format to select files."],["HostLanguage","Language that the host is using."],["ProcessLevel","Denotes in which thread the host is in."],["SmpteFrameRate","SMPTE Frame Rates."],["SpeakerArrangementType","Tells the host how the channels are intended to be used in the plugin. Only useful for some hosts."],["Supported","Used to specify whether functionality is supported."]],"mod":[["consts","Constant values"]],"struct":[["AEffect","Used with the VST API to pass around plugin information."],["ChannelFlags","Flags for VST channels."],["ChannelProperties","Information about a channel. Only some hosts use this information."],["Event","A VST event intended to be casted to a corresponding type."],["Events","A struct which contains events."],["FileSelect","File selector descriptor used in `host::OpCode::OpenFileSelector`."],["FileType","File type descriptor."],["MidiEvent","A midi event."],["MidiEventFlags","MIDI event flags."],["ModifierKey","Cross platform modifier key flags."],["PluginFlags","Flags for VST plugins."],["SysExEvent","A midi system exclusive event."],["TimeInfo","Describes the time at the start of the block currently being processed"],["TimeInfoFlags","Used in the `flags` field of `TimeInfo`, and for querying the host for specific values"]],"type":[["DispatcherProc","Dispatcher function used to process opcodes. Called by host."],["GetParameterProc","Callback function used to get parameter values. Called by host."],["HostCallbackProc","Host callback function passed to plugin. Can be used to query host information from plugin side."],["PluginMain","`VSTPluginMain` function signature."],["ProcessProc","Process function used to process 32 bit floating point samples. Called by host."],["ProcessProcF64","Process function used to process 64 bit floating point samples. Called by host."],["SetParameterProc","Callback function used to set parameter values. Called by host."]]};