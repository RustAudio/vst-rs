/*
    This zero-delay feedback filter is based on a 4-stage transistor ladder filter.
    It follows the following equations: 
    x = input - tanh(self.res * self.vout[3])
    vout[0] = self.g * (tanh(x) - tanh(self.vout[0])) + self.s[0]
    vout[1] = self.g * (tanh(self.vout[0]) - tanh(self.vout[1])) + self.s[1]
    vout[0] = self.g * (tanh(self.vout[1]) - tanh(self.vout[2])) + self.s[2]
    vout[0] = self.g * (tanh(self.vout[2]) - tanh(self.vout[3])) + self.s[3]
    since we can't easily solve a nonlinear equation, 
    Mystran's fixed-pivot method is used to approximate the tanh() parts. 
    Quality can be improved a lot by oversampling a bit. 
    Feedback is clipped independently of the input, so it doesn't disappear at high gains.
        
*/
 #[macro_use] extern crate vst;
use vst::buffer::AudioBuffer;
use vst::plugin::{Info, Plugin, Category};

#[derive(PartialEq)]
enum Method {
    Linear,  // linear solution
    Pivotal, // Mystran's "cheap" method, using x=0 as pivot
}

//this is a 4-pole filter with resonance, which is why there's 4 states and vouts
#[derive(Clone)]
struct MoogFilter {
    //the output of the different filter stages
    vout: [f32; 4],
    //s is the "state" parameter. In an IIR it would be the last value from the filter
    //In this we find it by trapezoidal integration to avoid the unit delay
    s: [f32; 4],
    //the "cutoff" parameter. Determines how heavy filtering is
    cutoff: f32,
    g: f32,
    //needed to calculate cutoff. 
    sample_rate: f32,
    //makes a peak at cutoff
    res: f32,
    //used to choose if we want it to output 1 or 2 order filtering
    poles: usize,
    //a drive parameter. Just used to increase the volume, which results in heavier distortion
    drive: f32,
}
//member methods for the struct
impl MoogFilter {
    pub fn set_cutoff(&mut self, value: f32) {
        //cutoff formula gives us a natural feeling cutoff knob that spends more time in the low frequencies
        self.cutoff = 20000. * (1.8f32.powf(10. * value - 10.));
        //bilinear transformation for g gives us a very accurate cutoff
        self.g = (3.1415 * self.cutoff / (self.sample_rate)).tan();
    }
    //the state needs to be updated after each process. Found by trapezoidal integration
    fn update_state(&mut self) {
        self.s[0] = 2. * self.vout[0] - self.s[0];
        self.s[1] = 2. * self.vout[1] - self.s[1];
        self.s[2] = 2. * self.vout[2] - self.s[2];
        self.s[3] = 2. * self.vout[3] - self.s[3];
    }
    //performs a complete filter process (mystran's method)
    fn tick_pivotal(&mut self, input: f32) {
        if self.drive > 0. {
            self.run_moog_nonlinear(input * (self.drive + 0.7), Method::Pivotal);
        } else {
            //
            self.run_moog_nonlinear(input, Method::Linear);
        }
        self.update_state();
    }
    //instead of proper nonlinearities, this just has soft-clipping on the input
    fn _tick_simple(&mut self, input: f32) {
        if self.drive > 0. {
            self.run_moog_simple(input * (self.drive + 0.7));
        }
        else {
            self.run_moog_nonlinear(input, Method::Linear);
        }
        self.update_state();
    }
    fn run_moog_simple(&mut self, input: f32) {
        let x = input.tanh();
        //denominators of solutions of individual stages. Simplifies the math a bit
            let g0 = 1. / (1. + self.g);
            let g1 = self.g * g0 * g0;
            let g2 = self.g * g1 * g0;
            let g3 = self.g * g2 * g0;
            //outputs a 24db filter
            self.vout[3] = (g3 * self.g * x
                + g0 * self.s[3]
                + g1 * self.s[2]
                + g2 * self.s[1]
                + g3 * self.s[0])
                / (g3 * self.g * self.res + 1.);
            //since we know the feedback, we can solve the remaining outputs:
            self.vout[0] = g0 * (self.g * (x - self.res * self.vout[3]) + self.s[0]);
            self.vout[1] = g0 * (self.g * self.vout[0] + self.s[1]);
            self.vout[2] = g0 * (self.g * self.vout[1] + self.s[2]);
    }
    //nonlinear ladder filter function.  
    fn run_moog_nonlinear(&mut self, input: f32, method: Method) {
        let mut a = [1f32; 5];
        //version with drive
        if method == Method::Pivotal {
            let base = [
                0.,//self.res * self.s[3],
                self.s[0],
                self.s[1],
                self.s[2],
                self.s[3],
            ];
            //a[n] is the fixed-pivot approximation for tanh()
            for n in 0..base.len() {
                if base[n] != 0. {
                    a[n] = base[n].tanh() / base[n];
                } else {
                    a[n] = 1.;
                }
            }
            //denominators of solutions of individual stages. Simplifies the math a bit
            let g0 = 1. / (1. + self.g * a[1]); let g1 = 1. / (1. + self.g * a[2]);
            let g2 = 1. / (1. + self.g * a[3]); let g3 = 1. / (1. + self.g * a[4]);
            // these are just factored out of the feedback solution. Makes the math way easier to read
            let f3 = self.g * a[3] * g3; let f2 = self.g * a[2] * g2 * f3;
            let f1 = self.g * a[1] * g1 * f2; let f0 = self.g * g0 * f1;
            //outputs a 24db filter
            self.vout[3] = (f0 * input + f1 * g0 * self.s[0]
                + f2 * g1 * self.s[1]
                + f3 * g2 * self.s[2]
                + g3 * self.s[3])
                / (f0 * self.res * a[3] + 1.);
            //since we know the feedback, we can solve the remaining outputs:
            self.vout[0] =
                g0 * (self.g * a[1] * (input - self.res * a[3] * self.vout[3]) + self.s[0]);
            self.vout[1] = g1 * (self.g * a[2] * self.vout[0] + self.s[1]);
            self.vout[2] = g2 * (self.g * a[3] * self.vout[1] + self.s[2]);
        }
        //linear version without drive
        else {
            //denominators of solutions of individual stages. Simplifies the math a bit
            let g0 = 1. / (1. + self.g);
            let g1 = self.g * g0 * g0;
            let g2 = self.g * g1 * g0;
            let g3 = self.g * g2 * g0;
            //outputs a 24db filter
            self.vout[3] = (g3 * self.g * input
                + g0 * self.s[3]
                + g1 * self.s[2]
                + g2 * self.s[1]
                + g3 * self.s[0])
                / (g3 * self.g * self.res + 1.);
            //since we know the feedback, we can solve the remaining outputs:
            self.vout[0] = g0 * (self.g * (input - self.res * self.vout[3]) + self.s[0]);
            self.vout[1] = g0 * (self.g * self.vout[0] + self.s[1]);
            self.vout[2] = g0 * (self.g * self.vout[1] + self.s[2]);
        }
    }
}
//default values for parameters
impl Default for MoogFilter {
    fn default() -> DecentFilter {
        DecentFilter {
            vout: [0f32; 4],
            s: [0f32; 4],
            sample_rate: 88200.,
            cutoff: 1000.,
            res: 2.0,
            g: 0.07135868087,
            poles: 3,
            drive: 0.,
        }
    }
}

impl Plugin for MoogFilter
{
    fn set_sample_rate(&mut self, rate: f32) {
        self.sample_rate = rate;
    }
    fn get_info(&self) -> Info
    {
        Info  {
            name: "ZeroDelayFilter".to_string(),
            unique_id: 9263,
            inputs: 1,
            outputs: 1,
            category: Category::Effect,
            parameters: 4,
            ..Default::default()
        }
    }
    fn get_parameter(&self, index: i32) -> f32 {
    match index {
        0 => self.cutoff,
        1 => self.res,
        2 => (self.poles) as f32 + 1.,
        3 => self.drive,
        _ => 0.0,
        }
    }
    fn set_parameter(&mut self, index: i32, value: f32) {
        match index {
            0 => self.set_cutoff(value),
            1 => self.res = value * 4.,
            2 => self.poles = ((value * 3.).round()) as usize,
            3 => self.drive = value * 5.,
            _ => (),
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "cutoff".to_string(),
            1 => "resonance".to_string(),
            2 => "filter order".to_string(),
            3 => "drive".to_string(),
            _ => "".to_string(),
        }
    }
    fn get_parameter_label(&self, index: i32) -> String {
        match index {
            0 => "Hz".to_string(),
            1 => "%".to_string(),
            2 => "poles".to_string(),
            3 => "%".to_string(),
            _ => "".to_string(),
        }
    }
    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        for (input_buffer, output_buffer) in buffer.zip() {
            for (input_sample, output_sample) in input_buffer.iter().zip(output_buffer) {
                self.tick_pivotal(*input_sample);
                //the poles parameter chooses which filter stage we take our output from.
                *output_sample = self.vout[self.poles];
            }
        }
    }
}
plugin_main!(MoogFilter);