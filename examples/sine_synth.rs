// author: Rob Saunders <hello@robsaunders.io>

#[macro_use]
extern crate vst;

use vst::prelude::*;

use std::f64::consts::PI;

/// Convert the midi note's pitch into the equivalent frequency.
///
/// This function assumes A4 is 440hz.
fn midi_pitch_to_freq(pitch: u8) -> f64 {
    const A4_PITCH: i8 = 69;
    const A4_FREQ: f64 = 440.0;

    // Midi notes can be 0-127
    ((f64::from(pitch as i8 - A4_PITCH)) / 12.).exp2() * A4_FREQ
}

struct SineSynth {
    sample_rate: f64,
    time: f64,
    note_duration: f64,
    note: Option<u8>,
}

impl SineSynth {
    fn time_per_sample(&self) -> f64 {
        1.0 / self.sample_rate
    }

    /// Process an incoming midi event.
    ///
    /// The midi data is split up like so:
    ///
    /// `data[0]`: Contains the status and the channel. Source: [source]
    /// `data[1]`: Contains the supplemental data for the message - so, if this was a NoteOn then
    ///            this would contain the note.
    /// `data[2]`: Further supplemental data. Would be velocity in the case of a NoteOn message.
    ///
    /// [source]: http://www.midimountain.com/midi/midi_status.htm
    fn process_midi_event(&mut self, data: [u8; 3]) {
        match data[0] {
            128 => self.note_off(data[1]),
            144 => self.note_on(data[1]),
            _ => (),
        }
    }

    fn note_on(&mut self, note: u8) {
        self.note_duration = 0.0;
        self.note = Some(note)
    }

    fn note_off(&mut self, note: u8) {
        if self.note == Some(note) {
            self.note = None
        }
    }
}

pub const TAU: f64 = PI * 2.0;

impl Plugin for SineSynth {
    fn new(_host: HostCallback) -> Self {
        SineSynth {
            sample_rate: 44100.0,
            note_duration: 0.0,
            time: 0.0,
            note: None,
        }
    }

    fn get_info(&self) -> Info {
        Info {
            name: "SineSynth".to_string(),
            vendor: "DeathDisco".to_string(),
            unique_id: 6667,
            category: Category::Synth,
            inputs: 2,
            outputs: 2,
            parameters: 0,
            initial_delay: 0,
            ..Info::default()
        }
    }

    #[allow(unused_variables)]
    #[allow(clippy::single_match)]
    fn process_events(&mut self, events: &Events) {
        for event in events.events() {
            match event {
                Event::Midi(ev) => self.process_midi_event(ev.data),
                // More events can be handled here.
                _ => (),
            }
        }
    }

    fn set_sample_rate(&mut self, rate: f32) {
        self.sample_rate = f64::from(rate);
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        let samples = buffer.samples();
        let (_, mut outputs) = buffer.split();
        let output_count = outputs.len();
        let per_sample = self.time_per_sample();
        let mut output_sample;
        for sample_idx in 0..samples {
            let time = self.time;
            let note_duration = self.note_duration;
            if let Some(current_note) = self.note {
                let signal = (time * midi_pitch_to_freq(current_note) * TAU).sin();

                // Apply a quick envelope to the attack of the signal to avoid popping.
                let attack = 0.5;
                let alpha = if note_duration < attack {
                    note_duration / attack
                } else {
                    1.0
                };

                output_sample = (signal * alpha) as f32;

                self.time += per_sample;
                self.note_duration += per_sample;
            } else {
                output_sample = 0.0;
            }
            for buf_idx in 0..output_count {
                let buff = outputs.get_mut(buf_idx);
                buff[sample_idx] = output_sample;
            }
        }
    }

    fn can_do(&self, can_do: CanDo) -> Supported {
        match can_do {
            CanDo::ReceiveMidiEvent => Supported::Yes,
            _ => Supported::Maybe,
        }
    }
}

plugin_main!(SineSynth);

#[cfg(test)]
mod tests {
    use midi_pitch_to_freq;

    #[test]
    fn test_midi_pitch_to_freq() {
        for i in 0..127 {
            // expect no panics
            midi_pitch_to_freq(i);
        }
    }
}
