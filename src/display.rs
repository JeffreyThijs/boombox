use symphonia::core::audio::{AudioBufferRef, SignalSpec};
use symphonia::core::units::Duration;


pub trait Display {
    fn write(&mut self, decoded: AudioBufferRef<'_>) -> Result<()>;
    fn flush(&mut self);
}

#[allow(dead_code)]
#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
pub enum DisplayError {
    GenericError,
}

pub type Result<T> = result::Result<T, DisplayError>;


use std::result;


mod stft {
    use symphonia::core::audio::*;
    use symphonia::core::units::Duration;

    use super::{Display, Result};

    pub struct STFTDisplay {
        sample_buf: RawSampleBuffer<f32>,
    }

    impl STFTDisplay {
        pub fn try_open(spec: SignalSpec, duration: Duration) -> Result<Box<dyn Display>> {
            let sample_buf = RawSampleBuffer::<f32>::new(duration, spec);
            Ok(Box::new(STFTDisplay { sample_buf }))
        }
    }

    impl Display for STFTDisplay {

        fn write(&mut self, decoded: AudioBufferRef<'_>) -> Result<()> {

            println!("ok: {}", decoded.frames());

            Ok(())
        }

        fn flush(&mut self) {

        }
    }
}

#[cfg(target_os = "linux")]
pub fn try_open(spec: SignalSpec, duration: Duration) -> Result<Box<dyn Display>> {
    stft::STFTDisplay::try_open(spec, duration)
}