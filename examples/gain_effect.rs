// author: doomy <alexander@resamplr.com>

#[macro_use]
extern crate vst;
extern crate time;

use vst::buffer::AudioBuffer;
use vst::plugin::{Category, Info, Plugin};

/// Simple Gain Effect.
/// Note that this does not use a proper scale for sound and shouldn't be used in
/// a production amplification effect!  This is purely for demonstration purposes,
/// as well as to keep things simple as this is meant to be a starting point for
/// any effect.
struct GainEffect {
    // Here, we can store a variable that keeps track of the plugin's state.
    amplitude: f32,
}

// All plugins using the `vst` crate will either need to implement the `Default`
// trait, or derive from it.  By implementing the trait, we can set a default value.
// Note that controls will always return a value from 0 - 1.  Setting a default to
// 0.5 means it's halfway up.
impl Default for GainEffect {
    fn default() -> GainEffect {
        GainEffect { amplitude: 0.5f32 }
    }
}

// All plugins using `vst` also need to implement the `Plugin` trait.  Here, we
// define functions that give necessary info to our host.
impl Plugin for GainEffect {
    fn get_info(&self) -> Info {
        Info {
            name: "Gain Effect in Rust".to_string(),
            vendor: "Rust DSP".to_string(),
            unique_id: 243723072,
            version: 0001,
            inputs: 2,
            outputs: 2,
            // This `parameters` bit is important; without it, none of our
            // parameters will be shown!
            parameters: 1,
            category: Category::Effect,
            ..Default::default()
        }
    }

    // the `get_parameter` and `set_parameter` functions are required if we
    // want to interact with the plugin.  If we were creating an effect that
    // didn't allow the user to modify it at runtime or have any controls,
    // we could omit these next parts.
    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            0 => self.amplitude,
            _ => 0.0,
        }
    }

    fn set_parameter(&mut self, index: i32, val: f32) {
        match index {
            0 => self.amplitude = val,
            _ => (),
        }
    }

    // This is what will display underneath our control.  We can
    // format it into a string that makes the most since.
    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            0 => format!("{:.2}", (self.amplitude - 0.5) * 2f32),
            _ => "".to_string(),
        }
    }

    // This shows the control's name.
    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "Amplitude",
            _ => "",
        }.to_string()
    }

    // Here is where the bulk of our audio processing code goes.
    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        // First, we destructure our audio buffer into an arbitrary number of
        // input and output buffers.  Usually, we'll be dealing with stereo (2 of each)
        // but that might change.
        for (input_buffer, output_buffer) in buffer.zip() {
            // Next, we'll loop through each individual sample so we can apply the amplitude
            // value to it.
            for (input_sample, output_sample) in input_buffer.iter().zip(output_buffer) {
                *output_sample = *input_sample * self.amplitude;
            }
        }
    }
}

// This part is important!  Without it, our plugin won't work.
plugin_main!(GainEffect);
