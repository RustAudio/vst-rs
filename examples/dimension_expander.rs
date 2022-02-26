// author: Marko Mijalkovic <marko.mijalkovic97@gmail.com>

#[macro_use]
extern crate vst;

use std::collections::VecDeque;
use std::f64::consts::PI;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use vst::prelude::*;

/// Calculate the length in samples for a delay. Size ranges from 0.0 to 1.0.
fn delay(index: usize, mut size: f32) -> isize {
    const SIZE_OFFSET: f32 = 0.06;
    const SIZE_MULT: f32 = 1_000.0;

    size += SIZE_OFFSET;

    // Spread ratio between delays
    const SPREAD: f32 = 0.3;

    let base = size * SIZE_MULT;
    let mult = (index as f32 * SPREAD) + 1.0;
    let offset = if index > 2 { base * SPREAD / 2.0 } else { 0.0 };

    (base * mult + offset) as isize
}

/// A left channel and right channel sample.
type SamplePair = (f32, f32);

/// The Dimension Expander.
struct DimensionExpander {
    buffers: Vec<VecDeque<SamplePair>>,
    params: Arc<DimensionExpanderParameters>,
    old_size: f32,
}

struct DimensionExpanderParameters {
    dry_wet: AtomicFloat,
    size: AtomicFloat,
}

impl DimensionExpander {
    fn new(size: f32, dry_wet: f32) -> DimensionExpander {
        const NUM_DELAYS: usize = 4;

        let mut buffers = Vec::new();

        // Generate delay buffers
        for i in 0..NUM_DELAYS {
            let samples = delay(i, size);
            let mut buffer = VecDeque::with_capacity(samples as usize);

            // Fill in the delay buffers with empty samples
            for _ in 0..samples {
                buffer.push_back((0.0, 0.0));
            }

            buffers.push(buffer);
        }

        DimensionExpander {
            buffers,
            params: Arc::new(DimensionExpanderParameters {
                dry_wet: AtomicFloat::new(dry_wet),
                size: AtomicFloat::new(size),
            }),
            old_size: size,
        }
    }

    /// Update the delay buffers with a new size value.
    fn resize(&mut self, n: f32) {
        let old_size = self.old_size;

        for (i, buffer) in self.buffers.iter_mut().enumerate() {
            // Calculate the size difference between delays
            let old_delay = delay(i, old_size);
            let new_delay = delay(i, n);

            let diff = new_delay - old_delay;

            // Add empty samples if the delay was increased, remove if decreased
            if diff > 0 {
                for _ in 0..diff {
                    buffer.push_back((0.0, 0.0));
                }
            } else if diff < 0 {
                for _ in 0..-diff {
                    let _ = buffer.pop_front();
                }
            }
        }

        self.old_size = n;
    }
}

impl Plugin for DimensionExpander {
    fn new(_host: HostCallback) -> Self {
        DimensionExpander::new(0.12, 0.66)
    }

    fn get_info(&self) -> Info {
        Info {
            name: "Dimension Expander".to_string(),
            vendor: "overdrivenpotato".to_string(),
            unique_id: 243723071,
            version: 1,
            inputs: 2,
            outputs: 2,
            parameters: 2,
            category: Category::Effect,

            ..Default::default()
        }
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        let (inputs, outputs) = buffer.split();

        // Assume 2 channels
        if inputs.len() < 2 || outputs.len() < 2 {
            return;
        }

        // Resize if size changed
        let size = self.params.size.get();
        if size != self.old_size {
            self.resize(size);
        }

        // Iterate over inputs as (&f32, &f32)
        let (l, r) = inputs.split_at(1);
        let stereo_in = l[0].iter().zip(r[0].iter());

        // Iterate over outputs as (&mut f32, &mut f32)
        let (mut l, mut r) = outputs.split_at_mut(1);
        let stereo_out = l[0].iter_mut().zip(r[0].iter_mut());

        // Zip and process
        for ((left_in, right_in), (left_out, right_out)) in stereo_in.zip(stereo_out) {
            // Push the new samples into the delay buffers.
            for buffer in &mut self.buffers {
                buffer.push_back((*left_in, *right_in));
            }

            let mut left_processed = 0.0;
            let mut right_processed = 0.0;

            // Recalculate time per sample
            let time_s = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64();

            // Use buffer index to offset volume LFO
            for (n, buffer) in self.buffers.iter_mut().enumerate() {
                if let Some((left_old, right_old)) = buffer.pop_front() {
                    const LFO_FREQ: f64 = 0.5;
                    const WET_MULT: f32 = 0.66;

                    let offset = 0.25 * (n % 4) as f64;

                    // Sine wave volume LFO
                    let lfo = ((time_s * LFO_FREQ + offset) * PI * 2.0).sin() as f32;

                    let wet = self.params.dry_wet.get() * WET_MULT;
                    let mono = (left_old + right_old) / 2.0;

                    // Flip right channel and keep left mono so that the result is
                    // entirely stereo
                    left_processed += mono * wet * lfo;
                    right_processed += -mono * wet * lfo;
                }
            }

            // By only adding to the input, the output value always remains the same in mono
            *left_out = *left_in + left_processed;
            *right_out = *right_in + right_processed;
        }
    }

    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        Arc::clone(&self.params) as Arc<dyn PluginParameters>
    }
}

impl PluginParameters for DimensionExpanderParameters {
    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            0 => self.size.get(),
            1 => self.dry_wet.get(),
            _ => 0.0,
        }
    }

    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            0 => format!("{}", (self.size.get() * 1000.0) as isize),
            1 => format!("{:.1}%", self.dry_wet.get() * 100.0),
            _ => "".to_string(),
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "Size",
            1 => "Dry/Wet",
            _ => "",
        }
        .to_string()
    }

    fn set_parameter(&self, index: i32, val: f32) {
        match index {
            0 => self.size.set(val),
            1 => self.dry_wet.set(val),
            _ => (),
        }
    }
}

plugin_main!(DimensionExpander);
