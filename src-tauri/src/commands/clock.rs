use std::time;
use rust_utils::timer::{self, PreciseTimer};

#[derive(Debug, Clone)]
pub struct TauriClock {
    pub timer: PreciseTimer,
    instant: time::Instant,
}

impl TauriClock {
    pub fn new() -> TauriClock {
        let instant = time::Instant::now();
        let timer: PreciseTimer;
        timer = PreciseTimer::now(timer::TimerType::Clock);
        TauriClock {
            timer,
            instant,
        }
    }

    pub fn corner_clock_call(&mut self) {
        self.timer.tick(&self.instant);
    }

}
