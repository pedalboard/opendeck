//! BPM state for MIDI Clock generation.
//!
//! The library stores and adjusts BPM via button/encoder handlers.
//! Firmware uses `tick_interval_us()` to schedule 0xF8 timing messages at 24 PPQN.

const MIN_BPM: u16 = 30;
const MAX_BPM: u16 = 300;
const DEFAULT_BPM: u16 = 120;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Bpm {
    value: u16,
}

impl Default for Bpm {
    fn default() -> Self {
        Self { value: DEFAULT_BPM }
    }
}

impl Bpm {
    pub fn new(bpm: u16) -> Self {
        Self {
            value: bpm.clamp(MIN_BPM, MAX_BPM),
        }
    }

    pub fn get(&self) -> u16 {
        self.value
    }

    pub fn set(&mut self, bpm: u16) {
        self.value = bpm.clamp(MIN_BPM, MAX_BPM);
    }

    pub fn increment(&mut self) {
        self.set(self.value.saturating_add(1));
    }

    pub fn decrement(&mut self) {
        self.set(self.value.saturating_sub(1));
    }

    /// Returns the interval in microseconds between MIDI Clock (0xF8) messages.
    /// MIDI Clock runs at 24 PPQN (pulses per quarter note).
    pub fn tick_interval_us(&self) -> u32 {
        60_000_000 / (self.value as u32 * 24)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_bpm_is_120() {
        let bpm = Bpm::default();
        assert_eq!(bpm.get(), 120);
    }

    #[test]
    fn test_increment() {
        let mut bpm = Bpm::new(120);
        bpm.increment();
        assert_eq!(bpm.get(), 121);
    }

    #[test]
    fn test_decrement() {
        let mut bpm = Bpm::new(120);
        bpm.decrement();
        assert_eq!(bpm.get(), 119);
    }

    #[test]
    fn test_clamps_at_max() {
        let mut bpm = Bpm::new(300);
        bpm.increment();
        assert_eq!(bpm.get(), 300);
    }

    #[test]
    fn test_clamps_at_min() {
        let mut bpm = Bpm::new(30);
        bpm.decrement();
        assert_eq!(bpm.get(), 30);
    }

    #[test]
    fn test_new_clamps_out_of_range() {
        assert_eq!(Bpm::new(0).get(), 30);
        assert_eq!(Bpm::new(500).get(), 300);
    }

    #[test]
    fn test_tick_interval_at_120_bpm() {
        // 60_000_000 / (120 * 24) = 20833
        let bpm = Bpm::new(120);
        assert_eq!(bpm.tick_interval_us(), 20833);
    }

    #[test]
    fn test_tick_interval_at_60_bpm() {
        // 60_000_000 / (60 * 24) = 41666
        let bpm = Bpm::new(60);
        assert_eq!(bpm.tick_interval_us(), 41666);
    }
}
