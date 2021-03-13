// This example illustrates how an existing plugin can be ported to the new,
// thread-safe API with the help of the ParameterTransfer struct.
// It shows how the parameter iteration feature of ParameterTransfer can be
// used to react explicitly to parameter changes in an efficient way (here,
// to implement smoothing of parameters).

#[macro_use]
extern crate vst;

use std::f32;
use std::sync::Arc;

use vst::prelude::*;

const PARAMETER_COUNT: usize = 100;
const BASE_FREQUENCY: f32 = 5.0;
const FILTER_FACTOR: f32 = 0.01; // Set this to 1.0 to disable smoothing.
const TWO_PI: f32 = 2.0 * f32::consts::PI;

// 1. Define a struct to hold parameters. Put a ParameterTransfer inside it,
// plus optionally a HostCallback.
struct MyPluginParameters {
    #[allow(dead_code)]
    host: HostCallback,
    transfer: ParameterTransfer,
}

// 2. Put an Arc reference to your parameter struct in your main Plugin struct.
struct MyPlugin {
    params: Arc<MyPluginParameters>,
    states: Vec<Smoothed>,
    sample_rate: f32,
    phase: f32,
}

// 3. Implement PluginParameters for your parameter struct.
// The set_parameter and get_parameter just access the ParameterTransfer.
// The other methods can be implemented on top of this as well.
impl PluginParameters for MyPluginParameters {
    fn set_parameter(&self, index: i32, value: f32) {
        self.transfer.set_parameter(index as usize, value);
    }

    fn get_parameter(&self, index: i32) -> f32 {
        self.transfer.get_parameter(index as usize)
    }
}

impl Plugin for MyPlugin {
    fn new(host: HostCallback) -> Self {
        MyPlugin {
            // 4. Initialize your main Plugin struct with a parameter struct
            // wrapped in an Arc, and put the HostCallback inside it.
            params: Arc::new(MyPluginParameters {
                host,
                transfer: ParameterTransfer::new(PARAMETER_COUNT),
            }),
            states: vec![Smoothed::default(); PARAMETER_COUNT],
            sample_rate: 44100.0,
            phase: 0.0,
        }
    }

    fn get_info(&self) -> Info {
        Info {
            parameters: PARAMETER_COUNT as i32,
            inputs: 0,
            outputs: 2,
            category: Category::Synth,
            f64_precision: false,

            name: "transfer_and_smooth".to_string(),
            vendor: "Loonies".to_string(),
            unique_id: 0x500007,
            version: 100,

            ..Info::default()
        }
    }

    // 5. Return a reference to the parameter struct from get_parameter_object.
    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        Arc::clone(&self.params) as Arc<dyn PluginParameters>
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        // 6. In the process method, iterate over changed parameters and do
        // for each what you would previously do in set_parameter. Since this
        // runs in the processing thread, it has mutable access to the Plugin.
        for (p, value) in self.params.transfer.iterate(true) {
            // Example: Update filter state of changed parameter.
            self.states[p].set(value);
        }

        // Example: Dummy synth adding together a bunch of sines.
        let samples = buffer.samples();
        let mut outputs = buffer.split().1;
        for i in 0..samples {
            let mut sum = 0.0;
            for p in 0..PARAMETER_COUNT {
                let amp = self.states[p].get();
                if amp != 0.0 {
                    sum += (self.phase * p as f32 * TWO_PI).sin() * amp;
                }
            }
            outputs[0][i] = sum;
            outputs[1][i] = sum;
            self.phase = (self.phase + BASE_FREQUENCY / self.sample_rate).fract();
        }
    }
}

// Example: Parameter smoothing as an example of non-trivial parameter handling
// that has to happen when a parameter changes.
#[derive(Clone, Default)]
struct Smoothed {
    state: f32,
    target: f32,
}

impl Smoothed {
    fn set(&mut self, value: f32) {
        self.target = value;
    }

    fn get(&mut self) -> f32 {
        self.state += (self.target - self.state) * FILTER_FACTOR;
        self.state
    }
}

plugin_main!(MyPlugin);
