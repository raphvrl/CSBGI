use std::time::Instant;

pub struct Clock {
    last_tick: Instant,
    delta_time: f32,
}

impl Clock {
    pub fn new() -> Self {
        Self {
            last_tick: Instant::now(),
            delta_time: 0.0,
        }
    }

    pub fn tick(&mut self) {
        let now = Instant::now();

        self.delta_time = (now - self.last_tick).as_secs_f32().min(0.1);
        self.last_tick = now;
    }

    pub fn delta_time(&self) -> f32 {
        self.delta_time
    }
}

impl Default for Clock {
    fn default() -> Self {
        Self::new()
    }
}
