bitflags::bitflags! {
    pub struct ActivityDataPoints: u16 {
        const CALORIES_BURNED = 0b00_0001;
        const DISTANCE = 0b00_0010;
        const DURATION = 0b00_0100;
        const HEART_RATE = 0b00_1000;
        const STEP_COUNT = 0b01_0000;
    }
}
