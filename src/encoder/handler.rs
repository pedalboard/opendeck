use crate::encoder::{Encoder, EncoderMessageType};
use crate::handler::ChannelMessages;

use midi2::{channel_voice1::ControlChange, error::BufferOverflow, prelude::*, BytesMessage};

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
    pub fn new(encoder: &'a mut Encoder, pulse: EncoderPulse) -> Self {
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
            EncoderMessageType::TwoNoteWithFixedValueBothDirections => 2,
            EncoderMessageType::PitchBend => 1,
            EncoderMessageType::ProgramChange => 1,
            EncoderMessageType::NRPN7 => 1,
            EncoderMessageType::NRPN14 => 2,
            EncoderMessageType::PresetChange => 0,
            EncoderMessageType::BPM => 1,
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
        let (channel, _index) = match self.channel_messages.next() {
            Some((channel, index)) => (channel, index),
            None => return Ok(None),
        };
        match self.encoder.message_type {
            EncoderMessageType::ControlChange => {
                self.encoder.increment(&self.pulse);
                let mut m = ControlChange::try_new_with_buffer(buffer)?;
                m.set_channel(channel);
                m.set_control(u7::new(self.encoder.midi_id as u8));
                m.set_control_data(u7::new(self.encoder.value as u8));
                Ok(Some(m.into()))
            }
            EncoderMessageType::ControlChange14bit => Ok(None),
            EncoderMessageType::ControlChange7Fh01h => Ok(None),
            EncoderMessageType::ControlChange3Fh41h => Ok(None),
            EncoderMessageType::ControlChange41h01h => Ok(None),
            EncoderMessageType::SingleNoteWithVariableValue => Ok(None),
            EncoderMessageType::SingleNoteWithFixedValueBothDirections => Ok(None),
            EncoderMessageType::SingleNoteWithFixedValueOneDirection0OtherDirection => Ok(None),
            EncoderMessageType::TwoNoteWithFixedValueBothDirections => Ok(None),
            EncoderMessageType::PitchBend => Ok(None),
            EncoderMessageType::ProgramChange => Ok(None),
            EncoderMessageType::NRPN7 => Ok(None),
            EncoderMessageType::NRPN14 => Ok(None),
            EncoderMessageType::PresetChange => Ok(None),
            EncoderMessageType::BPM => Ok(None),
        }
    }
}

impl Encoder {
    pub fn handle(&mut self, p: EncoderPulse) -> EncoderMessages<'_> {
        if self.inverted {
            EncoderMessages::new(self, p.invert())
        } else {
            EncoderMessages::new(self, p)
        }
    }
    fn increment(&mut self, p: &EncoderPulse) {
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
    use crate::encoder::Accelleration;
    use crate::ChannelOrAll;

    #[test]
    fn test_increment() {
        let mut encoder = Encoder {
            value: 0,
            lower_limit: 2,
            upper_limit: 4,
            ..Encoder::default()
        };
        encoder.increment(&EncoderPulse::Clockwise);
        assert_eq!(2, encoder.value);
        encoder.increment(&EncoderPulse::Clockwise);
        assert_eq!(3, encoder.value);
        encoder.increment(&EncoderPulse::Clockwise);
        assert_eq!(4, encoder.value);
        encoder.increment(&EncoderPulse::Clockwise);
        assert_eq!(4, encoder.value);
        encoder.increment(&EncoderPulse::CounterClockwise);
        assert_eq!(3, encoder.value);
        encoder.increment(&EncoderPulse::CounterClockwise);
        assert_eq!(2, encoder.value);
        encoder.increment(&EncoderPulse::CounterClockwise);
        assert_eq!(2, encoder.value);
    }

    #[test]
    fn test_disable() {
        let mut buf = [0x00u8; 8];
        let mut encoder = Encoder {
            enabled: false,
            inverted: false,
            upper_limit: 99,
            lower_limit: 0,
            message_type: EncoderMessageType::ControlChange,
            midi_id: 0x03,
            second_midi_id: 0x00,
            pulses_per_step: 4,
            remote_sync: false,
            value: 0,
            accelleration: Accelleration::None,

            channel: ChannelOrAll::default(),
        };
        let mut it = encoder.handle(EncoderPulse::Clockwise);

        assert_eq!(Ok(None), it.next(&mut buf));
    }
    #[test]
    fn test_cc_7bit() {
        let mut buf = [0x00u8; 8];
        let mut encoder = Encoder {
            enabled: true,
            inverted: false,
            upper_limit: 99,
            lower_limit: 0,
            message_type: EncoderMessageType::ControlChange,
            midi_id: 0x03,
            second_midi_id: 0x00,
            pulses_per_step: 1,
            remote_sync: false,
            value: 1,
            accelleration: Accelleration::None,

            channel: ChannelOrAll::Channel(1),
        };
        let mut it = encoder.handle(EncoderPulse::Clockwise);

        let m = it.next(&mut buf).unwrap().unwrap();
        assert_eq!(m.data(), [0xB1, 0x03, 0x02]);
        assert_eq!(Ok(None), it.next(&mut buf));
    }
    #[test]
    fn test_cc_7bit_inverted() {
        let mut buf = [0x00u8; 8];
        let mut encoder = Encoder {
            enabled: true,
            inverted: true,
            upper_limit: 99,
            lower_limit: 0,
            message_type: EncoderMessageType::ControlChange,
            midi_id: 0x03,
            second_midi_id: 0x00,
            pulses_per_step: 1,
            remote_sync: false,
            value: 1,
            accelleration: Accelleration::None,

            channel: ChannelOrAll::Channel(1),
        };
        let mut it = encoder.handle(EncoderPulse::Clockwise);

        let m = it.next(&mut buf).unwrap().unwrap();
        assert_eq!(m.data(), [0xB1, 0x03, 0x00]);
        assert_eq!(Ok(None), it.next(&mut buf));
    }
}
