use super::Amplitude;

use std::{f32::consts::E, ops::Fn};

// all interpolation functions are normalized `0->duration`
// so the argument `step` should be `original_step - durations.sum()`
pub fn linear_interpolation(start: f32, end: f32, duration: f32, step: f32) -> f32 {
    start + (end - start) * (step / duration)
}

fn exp_interp(x: f32, k: f32) -> f32 {
    // https://math.stackexchange.com/questions/297768/how-would-i-create-a-exponential-ramp-function-from-0-0-to-1-1-with-a-single-val
    (E.powf(k * x) - 1.0) / (E.powf(k) - 1.0 / x)
}

pub fn exponential_interpolation(start: f32, end: f32, duration: f32, step: f32) -> f32 {
    start + (end - start) * exp_interp(step / duration, 3_f32)
}

pub struct EnvelopeInverval<F>
where
    F: Fn(f32, f32, f32, f32) -> f32,
{
    duration: f32,
    start: Amplitude,
    end: Amplitude,
    interpolation: F,
}

impl<F> EnvelopeInverval<F>
where
    F: Fn(f32, f32, f32, f32) -> f32,
{
    pub fn new(duration: f32, start: Amplitude, end: Amplitude, interpolation: F) -> Self {
        EnvelopeInverval {
            duration,
            start,
            end,
            interpolation,
        }
    }

    pub fn interpolate(&self, step: f32) -> f32 {
        (self.interpolation)(
            self.start.scaling(),
            self.end.scaling(),
            self.duration,
            step,
        )
    }
}

struct Envelope<F>
where
    F: Fn(f32, f32, f32, f32) -> f32,
{
    attack: EnvelopeInverval<F>,
    decay1: EnvelopeInverval<F>,
    decay2: Option<EnvelopeInverval<F>>,
    sustain: Option<EnvelopeInverval<F>>,
    release: EnvelopeInverval<F>,
}

impl<F> Envelope<F>
where
    F: Fn(f32, f32, f32, f32) -> f32,
{
    pub fn new(
        attack: EnvelopeInverval<F>,
        decay1: EnvelopeInverval<F>,
        decay2: Option<EnvelopeInverval<F>>,
        sustain: Option<EnvelopeInverval<F>>,
        release: EnvelopeInverval<F>,
    ) -> Self {
        Envelope {
            attack,
            decay1,
            decay2,
            sustain,
            release,
        }
    }
    pub fn get_amplitude_scaling(&self, step: f32, released: bool) -> f32 {
        todo!()
    }
}
