use super::Amplitude;

use std::{f32::consts::E, ops::Index};

type Interpolation = fn(f32, f32, f32, f32) -> f32;

enum Interval {
    Attack,
    Decay1,
    Decay2,
    Sustain,
    Release,
}

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

pub struct EnvelopeInverval {
    duration: f32,
    start: Amplitude,
    end: Amplitude,
    interpolation: Interpolation,
}

impl EnvelopeInverval {
    pub fn new(
        duration: f32,
        start: Amplitude,
        end: Amplitude,
        interpolation: Interpolation,
    ) -> Self {
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

struct Envelope {
    attack: EnvelopeInverval,
    decay1: EnvelopeInverval,
    decay2: Option<EnvelopeInverval>,
    sustain: Option<EnvelopeInverval>,
    release_time: Option<f32>,
    release: EnvelopeInverval,
    timing_mappings: Vec<Option<f32>>,
}

impl Envelope {
    pub fn new(
        attack: EnvelopeInverval,
        decay1: EnvelopeInverval,
        decay2: Option<EnvelopeInverval>,
        sustain: Option<EnvelopeInverval>,
        release: EnvelopeInverval,
    ) -> Self {
        let mut timings = Vec::new();
        let mut tot_duration: f32 = 0_f32;
        tot_duration += attack.duration;
        timings.push(Some(tot_duration));
        tot_duration += decay1.duration;
        timings.push(Some(tot_duration));
        if let Some(dec2) = &decay2 {
            tot_duration += dec2.duration;
            timings.push(Some(tot_duration));
        } else {
            timings.push(None);
        }
        if let Some(sus) = &sustain {
            tot_duration += sus.duration;
            timings.push(Some(tot_duration));
        } else {
            timings.push(None);
        }
        Envelope {
            attack,
            decay1,
            decay2,
            sustain,
            release_time: None,
            release,
            timing_mappings: timings,
        }
    }

    pub fn clear(&mut self) {
        self.release_time = None;
    }

    pub fn set_release(&mut self, release_time: f32) {
        self.release_time = Some(release_time);
    }

    fn get_interval_envelope(&self, index: Interval) -> Option<&EnvelopeInverval> {
        match index {
            Interval::Attack => Some(&self.attack),
            Interval::Decay1 => Some(&self.decay1),
            Interval::Decay2 => self.decay2.as_ref(),
            Interval::Sustain => self.sustain.as_ref(),
            Interval::Release => Some(&self.release),
        }
    }

    pub fn get_amplitude_scaling(&self, step: f32) -> f32 {
        if let Some(release_time) = self.release_time {
            if step >= release_time {
                return self.release.interpolate(step - release_time);
            }
        };
    }
}
