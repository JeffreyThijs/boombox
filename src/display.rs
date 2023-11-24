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
    use rustfft::{FftPlanner, num_complex::Complex};

    use super::{Display, Result};

    pub struct STFTDisplay {
        sample_buf: RawSampleBuffer<f32>,
    }

    impl STFTDisplay {
        pub fn try_open(spec: SignalSpec, duration: Duration) -> Result<Box<dyn Display>> {
            let sample_buf = RawSampleBuffer::<f32>::new(duration, spec);
            Ok(Box::new(STFTDisplay { sample_buf }))
        }

        fn process(&mut self) {
            let mono: Vec<f32> = self.sample_buf.as_bytes().chunks_exact(4)
                .map(|chunk| f32::from_ne_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                .collect::<Vec<f32>>() // Collect the f32 values
                .windows(2) // Create an iterator over pairs of adjacent elements
                .map(|pair| (pair[0] + pair[1]) / 2.0) // Calculate the average of each pair
                .collect();

            let stft_result = self.stft(&mono, 256, 128);
            // Do something with the STFT result, e.g., print it
            // for row in &stft_result {
            //     for &value in row {
            //         print!("{:.2} ", value);
            //     }
            //     println!();
            // }
        }

        // Function to perform the STFT on a Vec<f32> input
        fn stft(&self, signal: &[f32], window_size: usize, hop_size: usize) -> Vec<Vec<f32>> {
            let fft_size = window_size.next_power_of_two();
            let fft = FftPlanner::new().plan_fft_forward(fft_size);

            let mut stft_result: Vec<Vec<f32>> = Vec::new();

            for i in (0..signal.len()).step_by(hop_size) {
                // Apply window function to the current frame
                let window: Vec<_> = (0..window_size)
                    .map(|j| {
                        let value = signal.get(i + j).unwrap_or(&0.0);
                        value * 0.5 * (1.0 - 2.0 * std::f32::consts::PI * j as f32 / window_size as f32).cos()
                    })
                    .collect();

                // Zero-pad the windowed signal if necessary
                let mut input: Vec<_> = window
                    .iter()
                    .cloned()
                    .chain(vec![0.0; fft_size - window_size])
                    .map(|value| Complex::new(value, 0.0))
                    .collect();

                // Perform FFT in-place
                fft.process(&mut input);

                // Compute magnitude spectrum
                let magnitude_spectrum: Vec<_> = input.iter().map(|c| c.norm()).collect();

                stft_result.push(magnitude_spectrum);
            }

            stft_result
        }
    }

    impl Display for STFTDisplay {

        fn write(&mut self, decoded: AudioBufferRef<'_>) -> Result<()> {

            if decoded.frames() == 0 {
                return Ok(());
            }

            // Interleave samples from the audio buffer into the sample buffer.
            self.sample_buf.copy_interleaved_ref(decoded);

            self.process();

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