use chrono::{DateTime, FixedOffset};

#[derive(Debug)]
pub struct Steps {
    pub date: DateTime<FixedOffset>,
    pub steps: u32,
}

impl Steps {
    pub fn new(date: DateTime<FixedOffset>, steps: u32) -> Self {
        Self { date, steps }
    }
}
