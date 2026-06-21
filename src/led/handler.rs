use crate::led::{ControlType, Led, LedSection};
use crate::ChannelOrAll;

/// Result of processing a MIDI message against an output
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputState {
    NoChange,
    On,
    Off,
}

impl Led {
    /// Process an incoming MIDI message and return the new output state.
    /// `channel` is 1-based MIDI channel, `id` is note/CC number, `value` is velocity/CC value.
    pub fn process_midi(&self, channel: u8, id: u8, value: u8, is_note_on: bool) -> OutputState {
        if self.get_control_type() == ControlType::Static {
            return OutputState::On;
        }

        if !self.channel_matches(channel) {
            return OutputState::NoChange;
        }

        if !self.id_matches(id) {
            return OutputState::NoChange;
        }

        match self.get_control_type() {
            ControlType::MidiInNoteSingleValue | ControlType::LocalNoteSingleValue => {
                if is_note_on && value == self.get_activation_value() {
                    OutputState::On
                } else if !is_note_on {
                    OutputState::Off
                } else {
                    OutputState::NoChange
                }
            }
            ControlType::MidiInCcSingleValue | ControlType::LocalCcSingleValue => {
                if value == self.get_activation_value() {
                    OutputState::On
                } else {
                    OutputState::Off
                }
            }
            _ => OutputState::NoChange,
        }
    }

    fn channel_matches(&self, channel: u8) -> bool {
        match self.get_channel() {
            ChannelOrAll::All => true,
            ChannelOrAll::Channel(ch) => ch == channel,
            ChannelOrAll::None => false,
        }
    }

    fn id_matches(&self, id: u8) -> bool {
        self.get_activation_id() == id
    }

    pub fn get_channel(&self) -> ChannelOrAll {
        // Access via set/get roundtrips through protocol encoding.
        // We need direct access to avoid encoding issues.
        // The Led struct stores channel as ChannelOrAll directly.
        // Use a dedicated accessor that doesn't roundtrip.
        self.channel_direct()
    }

    pub fn get_activation_id(&self) -> u8 {
        self.get(LedSection::ActivationId(0)) as u8
    }

    pub fn get_activation_value(&self) -> u8 {
        self.get(LedSection::ActivationValue(0)) as u8
    }

    pub fn get_control_type(&self) -> ControlType {
        let v = self.get(LedSection::ControlType(ControlType::default()));
        ControlType::try_from(v).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_led(id: u8, value: u8, channel: u8, control_type: ControlType) -> Led {
        let mut led = Led::new(id);
        led.set(LedSection::ActivationValue(value));
        // Channel in OpenDeck protocol: 1 = first channel
        // ChannelOrAll::Channel stores 0-based internally
        led.set(LedSection::Channel(ChannelOrAll::Channel(channel)));
        led.set(LedSection::ControlType(control_type));
        led
    }

    #[test]
    fn test_note_on_matching_turns_output_on() {
        let led = make_led(60, 127, 1, ControlType::MidiInNoteSingleValue);
        assert_eq!(led.process_midi(1, 60, 127, true), OutputState::On);
    }

    #[test]
    fn test_note_off_turns_output_off() {
        let led = make_led(60, 127, 1, ControlType::MidiInNoteSingleValue);
        assert_eq!(led.process_midi(1, 60, 0, false), OutputState::Off);
    }

    #[test]
    fn test_wrong_channel_no_change() {
        let led = make_led(60, 127, 1, ControlType::MidiInNoteSingleValue);
        assert_eq!(led.process_midi(2, 60, 127, true), OutputState::NoChange);
    }

    #[test]
    fn test_wrong_id_no_change() {
        let led = make_led(60, 127, 1, ControlType::MidiInNoteSingleValue);
        assert_eq!(led.process_midi(1, 61, 127, true), OutputState::NoChange);
    }

    #[test]
    fn test_wrong_value_no_change() {
        let led = make_led(60, 127, 1, ControlType::MidiInNoteSingleValue);
        assert_eq!(led.process_midi(1, 60, 100, true), OutputState::NoChange);
    }

    #[test]
    fn test_channel_all_matches_any() {
        let mut led = Led::new(60);
        led.set(LedSection::ActivationValue(127));
        led.set(LedSection::Channel(ChannelOrAll::All));
        led.set(LedSection::ControlType(ControlType::MidiInNoteSingleValue));
        assert_eq!(led.process_midi(5, 60, 127, true), OutputState::On);
    }

    #[test]
    fn test_cc_single_value_on() {
        let led = make_led(7, 127, 1, ControlType::MidiInCcSingleValue);
        assert_eq!(led.process_midi(1, 7, 127, true), OutputState::On);
    }

    #[test]
    fn test_cc_single_value_off_when_value_differs() {
        let led = make_led(7, 127, 1, ControlType::MidiInCcSingleValue);
        assert_eq!(led.process_midi(1, 7, 0, true), OutputState::Off);
    }

    #[test]
    fn test_static_always_on() {
        let mut led = Led::new(0);
        led.set(LedSection::ControlType(ControlType::Static));
        assert_eq!(led.process_midi(1, 99, 99, true), OutputState::On);
    }

    #[test]
    fn test_local_note_single_value_on() {
        let led = make_led(60, 127, 1, ControlType::LocalNoteSingleValue);
        assert_eq!(led.process_midi(1, 60, 127, true), OutputState::On);
    }

    #[test]
    fn test_local_note_single_value_off() {
        let led = make_led(60, 127, 1, ControlType::LocalNoteSingleValue);
        assert_eq!(led.process_midi(1, 60, 0, false), OutputState::Off);
    }

    #[test]
    fn test_local_cc_single_value_on() {
        let led = make_led(7, 127, 1, ControlType::LocalCcSingleValue);
        assert_eq!(led.process_midi(1, 7, 127, true), OutputState::On);
    }

    #[test]
    fn test_local_cc_single_value_off() {
        let led = make_led(7, 127, 1, ControlType::LocalCcSingleValue);
        assert_eq!(led.process_midi(1, 7, 0, true), OutputState::Off);
    }
}

#[cfg(test)]
mod control_type_test {
    use super::*;

    #[test]
    fn test_control_type_roundtrip() {
        let ct = ControlType::LocalNoteSingleValue;
        let v: u16 = ct.into();
        assert_eq!(v, 1);
        let back = ControlType::try_from(v).unwrap();
        assert_eq!(back, ControlType::LocalNoteSingleValue);
    }
}
