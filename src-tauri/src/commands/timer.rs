use std::time;
use rust_utils::timer::{self, PreciseTimer};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct TauriTimer {
    #[serde(skip_serializing)]
    pub timer: PreciseTimer,
    #[serde(skip_serializing)]
    instant: time::Instant,
    hours: u8,
    minutes: u8,
    seconds: u8,
    milliseconds: usize,
    id: usize,
}

impl TauriTimer {
    pub fn new(seconds: usize) -> TauriTimer {
        let instant = time::Instant::now();
        let timer: PreciseTimer;
        timer = PreciseTimer::from_secs(seconds, timer::TimerType::Timer);
        let (hours, minutes, seconds) = timer.get_digits();
        let milliseconds = timer.get_millis();
        TauriTimer {
            timer,
            instant,
            hours, minutes, seconds, milliseconds,
            id: 0,
        }
    }

    pub fn fill_timer(&mut self) {
        let (hours, minutes, seconds) = self.timer.get_digits();
        let milliseconds = self.timer.get_millis();
        self.hours = hours;
        self.minutes = minutes;
        self.seconds = seconds;
        self.milliseconds = milliseconds;
    }

    pub fn timer_call(&mut self) {
        self.timer.tick(&self.instant);
    }

    pub fn set_id(&mut self, id: usize) {
        self.id = id;
    }
}
