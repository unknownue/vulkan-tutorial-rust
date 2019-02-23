use std::thread;
use std::time::Duration;
use std::time::Instant;

const SAMPLE_COUNT: usize = 5;
const SAMPLE_COUNT_FLOAT: f32 = SAMPLE_COUNT as f32;

pub struct FPSLimiter {
    counter: Instant,
    frame_time_prefer: u32, // unit microseconds
    samples: [u32; SAMPLE_COUNT],
    current_frame: usize,
    delta_frame: u32,
}

impl FPSLimiter {
    pub fn new() -> FPSLimiter {
        const DEFAULT_PREFER_FPS: f32 = 60.0;

        FPSLimiter {
            counter: Instant::now(),
            frame_time_prefer: (1000_000.0_f32 / DEFAULT_PREFER_FPS) as u32,
            samples: [0; SAMPLE_COUNT],
            current_frame: 0,
            delta_frame: 0,
        }
    }

    pub fn set_prefer_fps(&mut self, prefer_fps: f32) {
        self.frame_time_prefer = (1000_000.0_f32 / prefer_fps) as u32;
    }

    /// Call this function in game loop to update its inner status.
    pub fn tick_frame(&mut self) {
        let time_elapsed = self.counter.elapsed();
        self.counter = Instant::now();

        self.delta_frame = time_elapsed.subsec_micros();
        self.samples[self.current_frame] = self.delta_frame;
        self.current_frame = (self.current_frame + 1) % SAMPLE_COUNT;
    }

    // TODO: this function seems not work.
    pub fn keep_fps(&self) {
        if self.frame_time_prefer > self.delta_frame {
            let delay = Duration::from_micros((self.frame_time_prefer - self.delta_frame) as u64);

            thread::sleep(delay);
        }
    }

    /// Calculate the current FPS.
    pub fn fps(&self) -> f32 {
        let mut sum = 0_u32;
        self.samples.iter().for_each(|val| {
            sum += val;
        });

        1000_000.0_f32 / (sum as f32 / SAMPLE_COUNT_FLOAT)
    }

    /// Return current delta time in seconds
    /// this function ignore its second part, since the second is mostly zero.
    pub fn delta_time(&self) -> f32 {
        self.delta_frame as f32 / 1000_000.0_f32 // time in second
    }
}
