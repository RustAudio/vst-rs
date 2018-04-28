#[macro_use]
extern crate vst;

use vst::plugin::{Info, Plugin, CanDo, HostCallback};
use vst::event::{Event, MidiEvent};
use vst::buffer::{AudioBuffer, SendEventBuffer};
use vst::api;

plugin_main!(MyPlugin); // Important!

#[derive(Default)]
struct MyPlugin {
	host: HostCallback,
	events: Vec<MidiEvent>,
	send_buffer: SendEventBuffer,
}

impl MyPlugin {
	fn send_midi(&mut self) {
		self.send_buffer.send_events(&self.events, &mut self.host);
		self.events.clear();
	}
}

impl Plugin for MyPlugin {
	fn new(host: HostCallback) -> Self {
		let mut p = MyPlugin::default();
		p.host = host;
		p
	}

	fn get_info(&self) -> Info {
		Info {
			name: "fwd_midi".to_string(),
			unique_id: 7357001, // Used by hosts to differentiate between plugins.
			..Default::default()
		}
	}

	fn process_events(&mut self, events: &api::Events) {
		for e in events.events() {
			match e {
				Event::Midi(e) => self.events.push(e),
				_ => (),
			}
		}
	}

	fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
		for (input, output) in buffer.zip() {
			for (in_sample, out_sample) in input.iter().zip(output) {
				*out_sample = *in_sample;
			}
		}
		self.send_midi();
	}

	fn process_f64(&mut self, buffer: &mut AudioBuffer<f64>) {
		for (input, output) in buffer.zip() {
			for (in_sample, out_sample) in input.iter().zip(output) {
				*out_sample = *in_sample;
			}
		}
		self.send_midi();
	}

	fn can_do(&self, can_do: CanDo) -> vst::api::Supported {
		use vst::api::Supported::*;
		use vst::plugin::CanDo::*;

		match can_do {
			SendEvents | SendMidiEvent | ReceiveEvents | ReceiveMidiEvent => Yes,
			_ => No,
		}
	}
}

