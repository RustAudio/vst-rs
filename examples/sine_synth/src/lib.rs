#[macro_use] extern crate vst2;

use vst2::buffer::AudioBuffer;
use vst2::plugin::{Category, Plugin, Info};
use vst2::event::{Event};

use std::f64::consts::PI;

fn midi_note_to_hz(note: u8) -> f64 {
    let a = 440.0; // a0 is 440 hz...
    (a / 32.0) * ((note as f64 - 9.0) / 12.0).exp2()
}

fn sample_count<'a>(buff: &Vec<&'a mut [f32]>) -> Option<usize> {
    buff.first().map(|b| b.len() )
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

    // midiData[0] : Contains the status and the channel http://www.midimountain.com/midi/midi_status.htm
    // midiData[1] : Contains the supplemental data for the message - so, if this was a NoteOn then this would contain the note.
    // midiData[2] : Further supplemental data. Would be velocity in the case of a NoteOn message.
    // midiData[3] : Reserved.
    fn process_midi_event(&mut self, data: [u8; 3]) {
        match data[0] {
            128 => self.note_off(data[1]),
            144 => self.note_on(data[1]),
            _ => ()
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

pub const TAU : f64 = PI * 2.0;

impl Default for SineSynth {
    fn default() -> SineSynth {
        SineSynth {
            sample_rate: 44100.0,
            note_duration: 0.0,
            time: 0.0,
            note: None,
        }
    }
}

impl Plugin for SineSynth {

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
    fn process_events(&mut self, events: Vec<Event>) {
        for event in events {
            match event {
                Event::Midi { data, delta_frames, live,
                          note_length, note_offset,
                          detune, note_off_velocity } => self.process_midi_event(data),
                _ => ()
            }
        }
    }

    fn set_sample_rate(&mut self, rate: f32) { 
        self.sample_rate = rate as f64;
    }

    fn process(&mut self, buffer: AudioBuffer<f32>) {
        let (inputs, outputs) = buffer.split();

        let samples = sample_count(&inputs).expect("some damn samples once in a while");
        let per_sample = self.time_per_sample();

        for (input_buffer, output_buffer) in inputs.iter().zip(outputs) {
            let mut t = self.time;
            for (_, output_sample) in input_buffer.iter().zip(output_buffer) {

                if let Some(current_note) = self.note {
                    let signal = (t * midi_note_to_hz(current_note) * TAU).sin();

                    // apply a quick envelope to the attack of the signal to avoid popping.
                    let attack = 0.5;
                    let alpha = if self.note_duration < attack {
                        self.note_duration / attack
                    } else {
                        1.0
                    };

                    *output_sample = (signal * alpha) as f32;

                    t += per_sample;
                } else {
                    *output_sample = 0.0;
                }
            }
        }

        self.time += samples as f64 * per_sample;
        self.note_duration += samples as f64 * per_sample;
    }
}

plugin_main!(SineSynth);
