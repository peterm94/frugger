use crate::FrugTimer;

pub struct RollingAverage {
    window: [u64; 10],
    index: usize,
    sum: u64,
}

pub struct Bencher<'a> {
    timer: &'a dyn FrugTimer,
    last: u64,
}

impl<'a> Bencher<'a> {
    pub fn new(timer: &'a dyn FrugTimer) -> Self {
        Self {
            timer,
            last: timer.ticks(),
        }
    }

    pub fn start(&mut self) {
        self.last = self.timer.ticks();
    }

    pub fn cp(&mut self, msg: &str) {
        let end = self.timer.ticks();
        let time = end - self.last;
        // log!("{msg}: {time}ms");
        self.last = self.timer.ticks();
    }
}

impl RollingAverage {
    pub fn new() -> Self {
        RollingAverage {
            window: [0; 10],
            index: 0,
            sum: 0,
        }
    }

    pub fn add(&mut self, val: u64) {
        self.sum = self.sum - self.window[self.index] + val;
        self.window[self.index] = val;
        self.index = (self.index + 1) % 10;
    }

    pub fn average(&self) -> u64 {
        self.sum / 10
    }
}
