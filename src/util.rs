/// Convert the SRGB color channel to linear color space.
pub fn srgb(c: u8) -> f32 {
    let c = c as f32 / 255.0;
    if c < 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn linear(c: u8) -> f32 {
    srgb(c).powf(2.2f32.recip())
}

pub trait IntoSrgb {
    fn into_srgb(self, alpha: f32) -> [f32; 4];
}

impl IntoSrgb for Rgb {
    fn into_srgb(self, alpha: f32) -> [f32; 4] {
        [linear(self.r), linear(self.g), linear(self.b), alpha]
    }
}

pub fn nice_s_curve(t: f32, length: f32) -> f32 {
    let t = normalize(t, length);
    let v = 1. - t;
    1. - v * v
}

fn normalize(t: f32, length: f32) -> f32 {
    let length = length.sqrt() + length.ln_1p();
    (t / length).clamp(0., 1.)
}

#[allow(unused)]
macro_rules! time_execution {
    ($e:expr) => {{
        let now = ::std::time::Instant::now();
        let out = $e;
        ::log::trace!(
            "EXECUTION_TIME({}): {}",
            ::std::stringify!($e),
            now.elapsed().as_micros()
        );
        out
    }};
}

use neophyte_ui_event::rgb::Rgb;
#[allow(unused)]
pub(crate) use time_execution;
