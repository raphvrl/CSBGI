use std::time::Instant;

pub struct Clock {
    last_tick: Instant,
    delta_time: f32,
    frame_time: f32,
}

impl Clock {
    pub fn new() -> Self {
        Self {
            last_tick: Instant::now(),
            delta_time: 0.0,
            frame_time: 0.0,
        }
    }

    pub fn tick(&mut self) {
        let now = Instant::now();

        self.frame_time = (now - self.last_tick).as_secs_f32();
        self.delta_time = self.frame_time.min(0.1);
        self.last_tick = now;
    }

    pub fn delta_time(&self) -> f32 {
        self.delta_time
    }

    pub fn frame_time(&self) -> f32 {
        self.frame_time
    }
}

impl Default for Clock {
    fn default() -> Self {
        Self::new()
    }
}
