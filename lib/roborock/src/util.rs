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
        let range = (self.max as u64 - self.min as u64) + 1;
        let offset = (self.current as u64 - self.min as u64 + 1) % range;
        self.current = self.min + offset as u32;
        self.current
    }
}

#[cfg(test)]
mod tests {
    use super::Counter;

    #[test]
    fn counter_starts_after_min() {
        let mut counter = Counter::new(10, 12);
        assert_eq!(counter.next(), 11);
    }

    #[test]
    fn counter_wraps_inclusive_range() {
        let mut counter = Counter::new(1, 3);
        assert_eq!(counter.next(), 2);
        assert_eq!(counter.next(), 3);
        assert_eq!(counter.next(), 1);
    }

    #[test]
    fn counter_handles_u32_max_range() {
        let mut counter = Counter::new(0, u32::MAX);
        assert_eq!(counter.next(), 1);
    }
}
