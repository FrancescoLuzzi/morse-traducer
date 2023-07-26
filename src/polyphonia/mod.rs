//public modules
pub mod amplitude;
pub mod envelope;
pub mod notable_notes;
pub mod note;

// public uses
pub use amplitude::Amplitude;
pub use note::Note;

use std::ops::Fn;

pub const SAMPLE_RATE: u32 = 44100;
pub const MAX_AMPLITUDE: f32 = i16::MAX as f32;

fn oscillator(w: f32, amplitute: f32) -> f32 {
    amplitute * f32::sin(w)
}

fn get_w(frequency: f32, time: f32, sample_rate: f32) -> f32 {
    2.0 * std::f32::consts::PI * frequency * time / sample_rate
}

fn natural_oscillator<F>(
    frequency: f32,
    time_start: f32,
    time_curr: f32,
    amplitude_calculator: F,
    amplitude_modulator: F,
) -> f32
where
    F: Fn(f32) -> f32,
{
    let time_delta = time_curr - time_start;
    oscillator(
        frequency * amplitude_modulator(time_delta),
        amplitude_calculator(time_delta),
    )
}
