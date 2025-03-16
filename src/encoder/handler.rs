use crate::encoder::{Encoder, EncoderMessageType};
use crate::handler::{ChannelMessages, HiRes};

use channel_voice1::ProgramChange;
use midi2::{
    channel_voice1::{ControlChange, NoteOn, PitchBend},
    error::BufferOverflow,
    prelude::*,
    BytesMessage,
};

pub enum EncoderPulse {
    Clockwise,
    CounterClockwise,
}

pub struct EncoderMessages<'a> {
    encoder: &'a mut Encoder,
    channel_messages: ChannelMessages,
    pulse: EncoderPulse,
}
impl<'a> EncoderMessages<'a> {
    fn none(encoder: &'a mut Encoder) -> Self {
        Self {
            encoder,
            channel_messages: ChannelMessages::none(),
            pulse: EncoderPulse::Clockwise,
        }
    }
    fn new(encoder: &'a mut Encoder, pulse: EncoderPulse) -> Self {
        let mt = &encoder.message_type;
        let nr_of_messages = match mt {
            EncoderMessageType::ControlChange => 1,
            EncoderMessageType::ControlChange14bit => 2,
            EncoderMessageType::ControlChange7Fh01h => 1,
            EncoderMessageType::ControlChange3Fh41h => 1,
            EncoderMessageType::ControlChange41h01h => 1,
            EncoderMessageType::SingleNoteWithVariableValue => 1,
            EncoderMessageType::SingleNoteWithFixedValueBothDirections => 1,
            EncoderMessageType::SingleNoteWithFixedValueOneDirection0OtherDirection => 1,
            EncoderMessageType::TwoNoteWithFixedValueBothDirections => 1,
            EncoderMessageType::PitchBend => 1,
            EncoderMessageType::ProgramChange => 1,
            EncoderMessageType::NRPN7 => 3,
            EncoderMessageType::NRPN14 => 4,
            EncoderMessageType::PresetChange => 0,
            EncoderMessageType::BPM => 0,
        };
        let ch = encoder.channel;
        let channel_messages = ChannelMessages::new_with_multiple_messages(ch, nr_of_messages);
        Self {
            encoder,
            channel_messages,
            pulse,
        }
    }
    pub fn next<'buf>(
        &mut self,
        buffer: &'buf mut [u8],
    ) -> Result<Option<BytesMessage<&'buf mut [u8]>>, BufferOverflow> {
        if !self.encoder.enabled {
            return Ok(None);
        }
        let (channel, index, incr) = match self.channel_messages.next() {
            Some((channel, index, incr)) => (channel, index, incr),
            None => return Ok(None),
        };
        match self.encoder.message_type {
            EncoderMessageType::ControlChange => {
                self.encoder.increment(&self.pulse, incr);
                let mut m = ControlChange::try_new_with_buffer(buffer)?;
                m.set_channel(channel);
                m.set_control(u7::new(self.encoder.midi_id as u8));
                m.set_control_data(u7::new(self.encoder.value as u8));
                Ok(Some(m.into()))
            }
            EncoderMessageType::ControlChange14bit => {
                self.encoder.increment(&self.pulse, incr);
                let (value, id) =
                    HiRes::new(self.encoder.value).control_change(index, self.encoder.midi_id);
                let mut m = ControlChange::try_new_with_buffer(buffer)?;
                m.set_channel(channel);
                m.set_control(id);
                m.set_control_data(value);
                Ok(Some(m.into()))
            }
            EncoderMessageType::ControlChange7Fh01h => {
                let value = match self.pulse {
                    EncoderPulse::Clockwise => 0x01,
                    EncoderPulse::CounterClockwise => 0x7F,
                };
                let mut m = ControlChange::try_new_with_buffer(buffer)?;
                m.set_channel(channel);
                m.set_control(u7::new(self.encoder.midi_id as u8));
                m.set_control_data(u7::new(value));
                Ok(Some(m.into()))
            }
            EncoderMessageType::ControlChange3Fh41h => {
                let value = match self.pulse {
                    EncoderPulse::Clockwise => 0x3F,
                    EncoderPulse::CounterClockwise => 0x41,
                };
                let mut m = ControlChange::try_new_with_buffer(buffer)?;
                m.set_channel(channel);
                m.set_control(u7::new(self.encoder.midi_id as u8));
                m.set_control_data(u7::new(value));
                Ok(Some(m.into()))
            }
            EncoderMessageType::ControlChange41h01h => {
                let value = match self.pulse {
                    EncoderPulse::Clockwise => 0x41,
                    EncoderPulse::CounterClockwise => 0x01,
                };
                let mut m = ControlChange::try_new_with_buffer(buffer)?;
                m.set_channel(channel);
                m.set_control(u7::new(self.encoder.midi_id as u8));
                m.set_control_data(u7::new(value));
                Ok(Some(m.into()))
            }
            EncoderMessageType::PitchBend => {
                self.encoder.increment(&self.pulse, incr);
                let mut m = PitchBend::try_new_with_buffer(buffer)?;
                m.set_channel(channel);
                m.set_bend(u14::new(self.encoder.value));
                Ok(Some(m.into()))
            }
            EncoderMessageType::ProgramChange => {
                self.encoder.increment(&self.pulse, incr);
                let mut m = ProgramChange::try_new_with_buffer(buffer)?;
                m.set_channel(channel);
                m.set_program(u7::new(self.encoder.value as u8));
                Ok(Some(m.into()))
            }
            EncoderMessageType::NRPN7 | EncoderMessageType::NRPN14 => {
                self.encoder.increment(&self.pulse, incr);
                let (control, data) =
                    crate::handler::nprn::encode(index, self.encoder.midi_id, self.encoder.value);

                let mut m = ControlChange::try_new_with_buffer(buffer)?;
                m.set_channel(channel);
                m.set_control(control);
                m.set_control_data(data);
                Ok(Some(m.into()))
            }
            EncoderMessageType::SingleNoteWithVariableValue => {
                self.encoder.increment(&self.pulse, incr);
                let mut m = NoteOn::try_new_with_buffer(buffer)?;
                m.set_channel(channel);
                m.set_note_number(u7::new(self.encoder.midi_id as u8));
                m.set_velocity(u7::new(self.encoder.value as u8));
                Ok(Some(m.into()))
            }
            EncoderMessageType::SingleNoteWithFixedValueBothDirections => {
                let mut m = NoteOn::try_new_with_buffer(buffer)?;
                m.set_channel(channel);
                m.set_note_number(u7::new(self.encoder.midi_id as u8));
                m.set_velocity(u7::new(self.encoder.value as u8));
                Ok(Some(m.into()))
            }
            EncoderMessageType::SingleNoteWithFixedValueOneDirection0OtherDirection => {
                let note = match self.pulse {
                    EncoderPulse::Clockwise => self.encoder.midi_id,
                    EncoderPulse::CounterClockwise => 0,
                };
                let mut m = NoteOn::try_new_with_buffer(buffer)?;
                m.set_channel(channel);
                m.set_note_number(u7::new(note as u8));
                m.set_velocity(u7::new(self.encoder.value as u8));
                Ok(Some(m.into()))
            }
            EncoderMessageType::TwoNoteWithFixedValueBothDirections => {
                let note = match self.pulse {
                    EncoderPulse::Clockwise => self.encoder.midi_id,
                    EncoderPulse::CounterClockwise => self.encoder.second_midi_id,
                };
                let mut m = NoteOn::try_new_with_buffer(buffer)?;
                m.set_channel(channel);
                m.set_note_number(u7::new(note as u8));
                m.set_velocity(u7::new(self.encoder.value as u8));
                Ok(Some(m.into()))
            }
            EncoderMessageType::PresetChange => Ok(None),
            EncoderMessageType::BPM => Ok(None),
        }
    }
}

impl Encoder {
    pub fn handle(&mut self, p: EncoderPulse) -> EncoderMessages<'_> {
        if !self.pulse_count_reached() {
            return EncoderMessages::none(self);
        }
        if self.inverted {
            EncoderMessages::new(self, p.invert())
        } else {
            EncoderMessages::new(self, p)
        }
    }
    fn increment(&mut self, p: &EncoderPulse, incr: bool) {
        if !incr {
            return;
        }
        match p {
            EncoderPulse::Clockwise => {
                self.value += 1;
            }
            EncoderPulse::CounterClockwise => {
                self.value -= 1;
            }
        }
        if self.value > self.upper_limit {
            self.value = self.upper_limit;
        }
        if self.value < self.lower_limit {
            self.value = self.lower_limit;
        }
    }
    fn pulse_count_reached(&mut self) -> bool {
        self.state.pulse_count += 1;
        if self.state.pulse_count < self.pulses_per_step {
            return false;
        }
        self.state.pulse_count = 0;
        true
    }
}

impl EncoderPulse {
    fn invert(self) -> EncoderPulse {
        match self {
            EncoderPulse::Clockwise => EncoderPulse::CounterClockwise,
            EncoderPulse::CounterClockwise => EncoderPulse::Clockwise,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::ChannelOrAll;

    #[test]
    fn test_increment() {
        let mut encoder = Encoder {
            lower_limit: 2,
            upper_limit: 4,
            pulses_per_step: 1,
            ..Encoder::default()
        };
        encoder.increment(&EncoderPulse::Clockwise, true);
        assert_eq!(2, encoder.value);
        encoder.increment(&EncoderPulse::Clockwise, true);
        assert_eq!(3, encoder.value);
        encoder.increment(&EncoderPulse::Clockwise, true);
        assert_eq!(4, encoder.value);
        encoder.increment(&EncoderPulse::Clockwise, true);
        assert_eq!(4, encoder.value);
        encoder.increment(&EncoderPulse::CounterClockwise, true);
        assert_eq!(3, encoder.value);
        encoder.increment(&EncoderPulse::CounterClockwise, true);
        assert_eq!(2, encoder.value);
        encoder.increment(&EncoderPulse::CounterClockwise, true);
        assert_eq!(2, encoder.value);
    }

    #[test]
    fn test_pulses_per_count() {
        let mut encoder = Encoder {
            value: 0,
            pulses_per_step: 4,
            ..Encoder::default()
        };
        assert!(!encoder.pulse_count_reached());
        assert!(!encoder.pulse_count_reached());
        assert!(!encoder.pulse_count_reached());
        assert!(encoder.pulse_count_reached());
    }

    #[test]
    fn test_disable() {
        let mut buf = [0x00u8; 8];
        let mut encoder = Encoder {
            enabled: false,
            ..Encoder::default()
        };
        let mut it = encoder.handle(EncoderPulse::Clockwise);

        assert_eq!(Ok(None), it.next(&mut buf));
    }
    #[test]
    fn test_cc_7bit() {
        let mut buf = [0x00u8; 8];
        let mut encoder = Encoder {
            enabled: true,
            message_type: EncoderMessageType::ControlChange,
            value: 1,
            pulses_per_step: 1,
            midi_id: 0x03,
            channel: ChannelOrAll::Channel(1),
            ..Encoder::default()
        };
        let mut it = encoder.handle(EncoderPulse::Clockwise);

        let m = it.next(&mut buf).unwrap().unwrap();
        assert_eq!(m.data(), [0xB1, 0x03, 0x02]);
        assert_eq!(Ok(None), it.next(&mut buf));
    }
    #[test]
    fn test_cc_7bit_all_channels() {
        let mut buf = [0x00u8; 8];
        let mut encoder = Encoder {
            enabled: true,
            message_type: EncoderMessageType::ControlChange,
            value: 1,
            pulses_per_step: 1,
            midi_id: 0x03,
            channel: ChannelOrAll::All,
            ..Encoder::default()
        };
        let mut it = encoder.handle(EncoderPulse::Clockwise);

        let m = it.next(&mut buf).unwrap().unwrap();
        assert_eq!(m.data(), [0xB0, 0x03, 0x02]);
        let m = it.next(&mut buf).unwrap().unwrap();
        assert_eq!(m.data(), [0xB1, 0x03, 0x02]);
        let m = it.next(&mut buf).unwrap().unwrap();
        assert_eq!(m.data(), [0xB2, 0x03, 0x02]);
        let m = it.next(&mut buf).unwrap().unwrap();
        assert_eq!(m.data(), [0xB3, 0x03, 0x02]);
        let m = it.next(&mut buf).unwrap().unwrap();
        assert_eq!(m.data(), [0xB4, 0x03, 0x02]);
        let m = it.next(&mut buf).unwrap().unwrap();
        assert_eq!(m.data(), [0xB5, 0x03, 0x02]);
        let m = it.next(&mut buf).unwrap().unwrap();
        assert_eq!(m.data(), [0xB6, 0x03, 0x02]);
        let m = it.next(&mut buf).unwrap().unwrap();
        assert_eq!(m.data(), [0xB7, 0x03, 0x02]);
        let m = it.next(&mut buf).unwrap().unwrap();
        assert_eq!(m.data(), [0xB8, 0x03, 0x02]);
        let m = it.next(&mut buf).unwrap().unwrap();
        assert_eq!(m.data(), [0xB9, 0x03, 0x02]);
        let m = it.next(&mut buf).unwrap().unwrap();
        assert_eq!(m.data(), [0xBA, 0x03, 0x02]);
        let m = it.next(&mut buf).unwrap().unwrap();
        assert_eq!(m.data(), [0xBB, 0x03, 0x02]);
        let m = it.next(&mut buf).unwrap().unwrap();
        assert_eq!(m.data(), [0xBC, 0x03, 0x02]);
        let m = it.next(&mut buf).unwrap().unwrap();
        assert_eq!(m.data(), [0xBD, 0x03, 0x02]);
        let m = it.next(&mut buf).unwrap().unwrap();
        assert_eq!(m.data(), [0xBE, 0x03, 0x02]);
        let m = it.next(&mut buf).unwrap().unwrap();
        assert_eq!(m.data(), [0xBF, 0x03, 0x02]);

        assert_eq!(Ok(None), it.next(&mut buf));
    }

    #[test]
    fn test_cc_7bit_inverted() {
        let mut buf = [0x00u8; 8];
        let mut encoder = Encoder {
            enabled: true,
            inverted: true,
            message_type: EncoderMessageType::ControlChange,
            value: 1,
            midi_id: 0x03,
            pulses_per_step: 1,
            channel: ChannelOrAll::Channel(1),
            ..Encoder::default()
        };
        let mut it = encoder.handle(EncoderPulse::Clockwise);

        let m = it.next(&mut buf).unwrap().unwrap();
        assert_eq!(m.data(), [0xB1, 0x03, 0x00]);
        assert_eq!(Ok(None), it.next(&mut buf));
    }
    #[test]
    fn test_cc_7bit_more_pulses_needed() {
        let mut buf = [0x00u8; 8];
        let mut encoder = Encoder {
            enabled: true,
            inverted: true,
            message_type: EncoderMessageType::ControlChange,
            value: 1,
            midi_id: 0x03,
            pulses_per_step: 2,
            channel: ChannelOrAll::Channel(1),
            ..Encoder::default()
        };
        let mut it = encoder.handle(EncoderPulse::Clockwise);
        assert_eq!(Ok(None), it.next(&mut buf));

        let mut it = encoder.handle(EncoderPulse::Clockwise);
        let m = it.next(&mut buf).unwrap().unwrap();
        assert_eq!(m.data(), [0xB1, 0x03, 0x00]);
        assert_eq!(Ok(None), it.next(&mut buf));
    }
    #[test]
    fn test_cc_14bit() {
        let mut buf = [0x00u8; 8];
        let mut encoder = Encoder {
            enabled: true,
            message_type: EncoderMessageType::ControlChange14bit,
            value: 999,
            upper_limit: 0x3FFF,
            pulses_per_step: 1,
            midi_id: 0x03,
            channel: ChannelOrAll::Channel(1),
            ..Encoder::default()
        };
        let mut it = encoder.handle(EncoderPulse::Clockwise);

        let m = it.next(&mut buf).unwrap().unwrap();
        assert_eq!(m.data(), [0xB1, 0x03, 7]);
        let m = it.next(&mut buf).unwrap().unwrap();
        assert_eq!(m.data(), [0xB1, 0x23, 104]);
        assert_eq!(Ok(None), it.next(&mut buf));
    }
    #[test]
    fn test_control_change_7fh01h() {
        let mut buf = [0x00u8; 8];
        let mut encoder = Encoder {
            enabled: true,
            message_type: EncoderMessageType::ControlChange7Fh01h,
            midi_id: 0x03,
            pulses_per_step: 1,
            channel: ChannelOrAll::Channel(1),
            ..Encoder::default()
        };

        // Test Clockwise pulse
        let mut it = encoder.handle(EncoderPulse::Clockwise);
        let m = it.next(&mut buf).unwrap().unwrap();
        assert_eq!(m.data(), [0xB1, 0x03, 0x01]);
        assert_eq!(Ok(None), it.next(&mut buf));

        // Test CounterClockwise pulse
        let mut it = encoder.handle(EncoderPulse::CounterClockwise);
        let m = it.next(&mut buf).unwrap().unwrap();
        assert_eq!(m.data(), [0xB1, 0x03, 0x7F]);
        assert_eq!(Ok(None), it.next(&mut buf));
    }
    #[test]
    fn test_control_change_3fh41h() {
        let mut buf = [0x00u8; 8];
        let mut encoder = Encoder {
            enabled: true,
            message_type: EncoderMessageType::ControlChange3Fh41h,
            midi_id: 0x03,
            pulses_per_step: 1,
            channel: ChannelOrAll::Channel(1),
            ..Encoder::default()
        };

        // Test Clockwise pulse
        let mut it = encoder.handle(EncoderPulse::Clockwise);
        let m = it.next(&mut buf).unwrap().unwrap();
        assert_eq!(m.data(), [0xB1, 0x03, 0x3F]);
        assert_eq!(Ok(None), it.next(&mut buf));

        // Test CounterClockwise pulse
        let mut it = encoder.handle(EncoderPulse::CounterClockwise);
        let m = it.next(&mut buf).unwrap().unwrap();
        assert_eq!(m.data(), [0xB1, 0x03, 0x41]);
        assert_eq!(Ok(None), it.next(&mut buf));
    }
    #[test]
    fn test_control_change_41h01h() {
        let mut buf = [0x00u8; 8];
        let mut encoder = Encoder {
            enabled: true,
            message_type: EncoderMessageType::ControlChange41h01h,
            midi_id: 0x03,
            pulses_per_step: 1,
            channel: ChannelOrAll::Channel(1),
            ..Encoder::default()
        };

        // Test Clockwise pulse
        let mut it = encoder.handle(EncoderPulse::Clockwise);
        let m = it.next(&mut buf).unwrap().unwrap();
        assert_eq!(m.data(), [0xB1, 0x03, 0x41]);
        assert_eq!(Ok(None), it.next(&mut buf));

        // Test CounterClockwise pulse
        let mut it = encoder.handle(EncoderPulse::CounterClockwise);
        let m = it.next(&mut buf).unwrap().unwrap();
        assert_eq!(m.data(), [0xB1, 0x03, 0x01]);
        assert_eq!(Ok(None), it.next(&mut buf));
    }
    #[test]
    fn test_pitch_bend() {
        let mut buf = [0x00u8; 8];
        let mut encoder = Encoder {
            enabled: true,
            message_type: EncoderMessageType::PitchBend,
            value: 999,
            upper_limit: 0x3FFF,
            pulses_per_step: 1,
            midi_id: 0x03,
            ..Encoder::default()
        };
        let mut it = encoder.handle(EncoderPulse::Clockwise);

        let m = it.next(&mut buf).unwrap().unwrap();
        assert_eq!(m.data(), [0xE0, 104, 7]);
        assert_eq!(Ok(None), it.next(&mut buf));
    }
    #[test]
    fn test_program_change() {
        let mut buf = [0x00u8; 8];
        let mut encoder = Encoder {
            enabled: true,
            message_type: EncoderMessageType::ProgramChange,
            value: 9,
            pulses_per_step: 1,
            midi_id: 0x03,
            ..Encoder::default()
        };
        let mut it = encoder.handle(EncoderPulse::Clockwise);

        let m = it.next(&mut buf).unwrap().unwrap();
        assert_eq!(m.data(), [0xC0, 10]);
        assert_eq!(Ok(None), it.next(&mut buf));
    }
    #[test]
    fn test_nrpn7() {
        let mut buf = [0x00u8; 8];
        let mut encoder = Encoder {
            enabled: true,
            message_type: EncoderMessageType::NRPN7,
            value: 99,
            midi_id: 1624,
            pulses_per_step: 1,
            channel: ChannelOrAll::Channel(1),
            ..Encoder::default()
        };
        let mut it = encoder.handle(EncoderPulse::Clockwise);

        let m = it.next(&mut buf).unwrap().unwrap();
        assert_eq!(m.data(), [0xB1, 98, 88]);
        let m = it.next(&mut buf).unwrap().unwrap();
        assert_eq!(m.data(), [0xB1, 99, 12]);
        let m = it.next(&mut buf).unwrap().unwrap();
        assert_eq!(m.data(), [0xB1, 38, 100]);
        assert_eq!(Ok(None), it.next(&mut buf));
    }

    #[test]
    fn test_nrpn14() {
        let mut buf = [0x00u8; 8];
        let mut encoder = Encoder {
            enabled: true,
            message_type: EncoderMessageType::NRPN14,
            value: 999,
            upper_limit: 0x3FFF,
            midi_id: 1624,
            pulses_per_step: 1,
            channel: ChannelOrAll::Channel(1),
            ..Encoder::default()
        };
        let mut it = encoder.handle(EncoderPulse::Clockwise);

        let m = it.next(&mut buf).unwrap().unwrap();
        assert_eq!(m.data(), [0xB1, 98, 88]);
        let m = it.next(&mut buf).unwrap().unwrap();
        assert_eq!(m.data(), [0xB1, 99, 12]);
        let m = it.next(&mut buf).unwrap().unwrap();
        assert_eq!(m.data(), [0xB1, 38, 104]);
        let m = it.next(&mut buf).unwrap().unwrap();
        assert_eq!(m.data(), [0xB1, 6, 7]);
        assert_eq!(Ok(None), it.next(&mut buf));
    }
    #[test]
    fn test_single_note_with_variable_value() {
        let mut buf = [0x00u8; 8];
        let mut encoder = Encoder {
            enabled: true,
            message_type: EncoderMessageType::SingleNoteWithVariableValue,
            value: 99,
            midi_id: 60,
            pulses_per_step: 1,
            channel: ChannelOrAll::Channel(1),
            ..Encoder::default()
        };

        let mut it = encoder.handle(EncoderPulse::Clockwise);
        let m = it.next(&mut buf).unwrap().unwrap();
        assert_eq!(m.data(), [0x91, 60, 100]);
        assert_eq!(Ok(None), it.next(&mut buf));
    }

    #[test]
    fn test_single_note_with_fixed_value_both_directions() {
        let mut buf = [0x00u8; 8];
        let mut encoder = Encoder {
            enabled: true,
            message_type: EncoderMessageType::SingleNoteWithFixedValueBothDirections,
            value: 100,
            midi_id: 60,
            pulses_per_step: 1,
            channel: ChannelOrAll::Channel(1),
            ..Encoder::default()
        };

        let mut it = encoder.handle(EncoderPulse::Clockwise);
        let m = it.next(&mut buf).unwrap().unwrap();
        assert_eq!(m.data(), [0x91, 60, 100]);
        assert_eq!(Ok(None), it.next(&mut buf));
    }

    #[test]
    fn test_single_note_with_fixed_value_one_direction_0_other_direction() {
        let mut buf = [0x00u8; 8];
        let mut encoder = Encoder {
            enabled: true,
            message_type: EncoderMessageType::SingleNoteWithFixedValueOneDirection0OtherDirection,
            value: 100,
            midi_id: 60,
            pulses_per_step: 1,
            channel: ChannelOrAll::Channel(1),
            ..Encoder::default()
        };

        // Test clockwise pulse
        let mut it = encoder.handle(EncoderPulse::Clockwise);
        let m = it.next(&mut buf).unwrap().unwrap();
        assert_eq!(m.data(), [0x91, 60, 100]);
        assert_eq!(Ok(None), it.next(&mut buf));

        // Test counter-clockwise pulse
        let mut it = encoder.handle(EncoderPulse::CounterClockwise);
        let m = it.next(&mut buf).unwrap().unwrap();
        assert_eq!(m.data(), [0x91, 0, 100]);
        assert_eq!(Ok(None), it.next(&mut buf));
    }

    #[test]
    fn test_two_note_with_fixed_value_both_directions() {
        let mut buf = [0x00u8; 8];
        let mut encoder = Encoder {
            enabled: true,
            message_type: EncoderMessageType::TwoNoteWithFixedValueBothDirections,
            value: 100,
            midi_id: 60,
            second_midi_id: 61,
            pulses_per_step: 1,
            channel: ChannelOrAll::Channel(1),
            ..Encoder::default()
        };

        // Test clockwise pulse
        let mut it = encoder.handle(EncoderPulse::Clockwise);
        let m = it.next(&mut buf).unwrap().unwrap();
        assert_eq!(m.data(), [0x91, 60, 100]);
        assert_eq!(Ok(None), it.next(&mut buf));

        // Test counter-clockwise pulse
        let mut it = encoder.handle(EncoderPulse::CounterClockwise);
        let m = it.next(&mut buf).unwrap().unwrap();
        assert_eq!(m.data(), [0x91, 61, 100]);
        assert_eq!(Ok(None), it.next(&mut buf));
    }
}
