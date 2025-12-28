#[derive(Debug, Clone)]
pub struct Counter {
    min: u32,
    max: u32,
    current: u32,
}

impl Counter {
    pub fn new(min: u32, max: u32) -> Self {
        Self {
            min,
            max,
            current: min,
        }
    }

    pub fn next(&mut self) -> u32 {
        let range = self.max - self.min + 1;
        self.current = self.min + ((self.current - self.min + 1) % range);
        self.current
    }
}
