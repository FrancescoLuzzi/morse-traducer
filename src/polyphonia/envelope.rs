use super::Amplitude;
use std::error::Error;

use std::f32::consts::E;

type Interpolation = fn(f32, f32, f32, f32) -> f32;

enum Interval {
    Attack,
    Decay1,
    Decay2,
    Sustain,
    Release,
}

impl TryFrom<usize> for Interval {
    type Error = String;
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Attack),
            1 => Ok(Self::Decay1),
            2 => Ok(Self::Decay2),
            3 => Ok(Self::Sustain),
            4 => Ok(Self::Release),
            def => Err(format!("Interval out of bound {def}")),
        }
    }
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
    timing_mappings: [Option<f32>; 4],
}

impl Envelope {
    pub fn new(
        attack: EnvelopeInverval,
        decay1: EnvelopeInverval,
        decay2: Option<EnvelopeInverval>,
        sustain: Option<EnvelopeInverval>,
        release: EnvelopeInverval,
    ) -> Self {
        // - Attack(0)  -> 0
        // - Decay1(1)  -> sum of prev duration 
        // - Decay2(2)  -> sum of prev duration
        // - Sustain(2) -> sum of prev duration
        let mut timings: [Option<f32>; 4] = [None;4];
        let mut tot_duration: f32 = 0_f32;
        timings[Interval::Attack as usize] = Some(tot_duration);
        tot_duration += attack.duration;
        timings[Interval::Decay1 as usize] = Some(tot_duration);
        tot_duration += decay1.duration;
        if let Some(dec2) = &decay2 {
            timings[Interval::Decay2 as usize] = Some(tot_duration);
            tot_duration += dec2.duration;
        }
        if sustain.is_some() {
            timings[Interval::Sustain as usize] = Some(tot_duration);
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

    fn get_interval_envelope(&self, interval: Interval) -> Option<&EnvelopeInverval> {
        match interval {
            Interval::Attack => Some(&self.attack),
            Interval::Decay1 => Some(&self.decay1),
            Interval::Decay2 => self.decay2.as_ref(),
            Interval::Sustain => self.sustain.as_ref(),
            Interval::Release => Some(&self.release),
        }
    }

    pub fn get_amplitude_scaling(&self, step: f32) -> Result<f32, Box<dyn Error>> {
        if let Some(release_time) = self.release_time {
            if step >= release_time {
                return Ok(self.release.interpolate(step - release_time));
            }
        };
        // get last interval where step is less that
        let (interval, offset) = self
            .timing_mappings
            .iter()
            .enumerate()
            .filter_map(|(indx, timing)| {
                if let Some(timing) = timing {
                    if step < *timing {
                        Some((indx, *timing))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .last()
            .unwrap();
        let interval: Interval = interval.try_into()?;
        let interval_envelope = self.get_interval_envelope(interval);
        if let Some(interval_envelope) = interval_envelope {
            Ok(interval_envelope.interpolate(step - offset))
        } else {
            Err(format!("Internal envelope not found for given step={step}").into())
        }
    }
}
