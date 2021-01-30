bitflags::bitflags! {
    pub struct ActivityDataPoints: u16 {
        const CALORIES_BURNED = 0b000001;
        const DISTANCE = 0b000010;
        const DURATION = 0b000100;
        const HEART_RATE = 0b001000;
        const STEP_COUNT = 0b010000;
    }
}
