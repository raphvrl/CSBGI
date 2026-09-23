use std::collections::VecDeque;

use super::data::{ProfileSample, ProfileStage};

const HISTORY_SECONDS: f64 = 5.0;
const FPS_WINDOW_SECONDS: f64 = 1.0;

#[derive(Default)]
pub struct ProfilerHistory {
    samples: VecDeque<ProfileSample>,
    current: ProfileSample,
    elapsed: f64,
    has_current: bool,
    gpu_supported: bool,
    latest_gpu_ms: [f32; ProfileStage::COUNT],
}

impl ProfilerHistory {
    pub fn begin_frame(&mut self, frame_time_seconds: f32) {
        self.elapsed += frame_time_seconds.max(0.0) as f64;
        if self.has_current {
            self.samples.push_back(self.current);
        }

        self.current = ProfileSample {
            time: self.elapsed,
            frame_time_ms: frame_time_seconds.max(0.0) * 1000.0,
            gpu_ms: self.latest_gpu_ms,
            ..ProfileSample::default()
        };
        self.has_current = true;

        while self
            .samples
            .front()
            .is_some_and(|sample| self.elapsed - sample.time > HISTORY_SECONDS)
        {
            self.samples.pop_front();
        }
    }

    pub fn record_cpu(&mut self, stage: ProfileStage, milliseconds: f32) {
        self.current.cpu_ms[stage as usize] = milliseconds.max(0.0);
    }

    pub fn set_gpu_sample(&mut self, values_ms: [f32; ProfileStage::COUNT]) {
        self.gpu_supported = true;
        self.latest_gpu_ms = values_ms;
        self.current.gpu_ms = values_ms;
    }

    pub fn set_gpu_unavailable(&mut self) {
        self.gpu_supported = false;
    }

    pub fn samples(&self) -> impl DoubleEndedIterator<Item = &ProfileSample> {
        self.samples
            .iter()
            .chain(self.has_current.then_some(&self.current))
    }

    pub fn latest(&self) -> Option<&ProfileSample> {
        self.samples
            .back()
            .or_else(|| self.has_current.then_some(&self.current))
    }

    pub fn fps_average(&self) -> f32 {
        let mut count = 0;
        let mut elapsed = 0.0;
        for sample in self.samples().rev() {
            if self.elapsed - sample.time > FPS_WINDOW_SECONDS {
                break;
            }
            count += 1;
            elapsed += sample.frame_time_ms as f64 / 1000.0;
        }

        if elapsed > 0.0 {
            count as f32 / elapsed as f32
        } else {
            0.0
        }
    }

    pub fn gpu_supported(&self) -> bool {
        self.gpu_supported
    }

    pub fn cpu_stats(&self, stage: ProfileStage) -> (f32, f32) {
        self.stage_stats(stage, false)
    }

    pub fn gpu_stats(&self, stage: ProfileStage) -> (f32, f32) {
        self.stage_stats(stage, true)
    }

    fn stage_stats(&self, stage: ProfileStage, gpu: bool) -> (f32, f32) {
        let mut count = 0;
        let mut total = 0.0;
        let mut peak: f32 = 0.0;

        for sample in self
            .samples
            .iter()
            .filter(|sample| self.elapsed - sample.time <= HISTORY_SECONDS)
        {
            let milliseconds = if gpu {
                sample.gpu_ms[stage as usize]
            } else {
                sample.cpu_ms[stage as usize]
            };
            total += milliseconds;
            peak = peak.max(milliseconds);
            count += 1;
        }

        if count == 0 {
            (0.0, 0.0)
        } else {
            (total / count as f32, peak)
        }
    }
}
