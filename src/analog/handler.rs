use crate::analog::{Analog, AnalogMessageType};
use crate::handler::{ChannelMessages, HiRes};

const MAX_ADC_VALUE: u16 = 4095; // (2 ^ 12) - 1

use midi2::{
    channel_voice1::{ControlChange, NoteOn, PitchBend},
    error::BufferOverflow,
    prelude::*,
    BytesMessage,
};

pub struct AnalogMessages<'a> {
    analog: &'a mut Analog,
    value: u16,
    channel_messages: ChannelMessages,
}
impl<'a> AnalogMessages<'a> {
    pub fn new(analog: &'a mut Analog, value: u16) -> Self {
        let mt = &analog.message_type;
        let nr_of_messages = match mt {
            AnalogMessageType::Button => 0,
            AnalogMessageType::PotentiometerWithCCMessage7Bit => 1,
            AnalogMessageType::PotentiometerWithCCMessage14Bit => 2,
            AnalogMessageType::PitchBend => 1,
            AnalogMessageType::PotentiometerWithNoteMessage => 1,
            AnalogMessageType::FSR => 1,
            AnalogMessageType::NRPN7 => 3,
            AnalogMessageType::NRPN14 => 4,
        };
        let ch = analog.channel;
        let channel_messages = ChannelMessages::new_with_multiple_messages(ch, nr_of_messages);
        Self {
            analog,
            value,
            channel_messages,
        }
    }
    pub fn next<'buf>(
        &mut self,
        buffer: &'buf mut [u8],
    ) -> Result<Option<BytesMessage<&'buf mut [u8]>>, BufferOverflow> {
        if !self.analog.enabled {
            return Ok(None);
        }
        let (channel, index) = match self.channel_messages.next() {
            Some((channel, index, _)) => (channel, index),
            None => return Ok(None),
        };
        match self.analog.message_type {
            AnalogMessageType::Button => Ok(None),
            AnalogMessageType::PotentiometerWithCCMessage7Bit => {
                let mut m = ControlChange::try_new_with_buffer(buffer)?;
                m.set_channel(channel);
                m.set_control(u7::new(self.analog.midi_id as u8));
                m.set_control_data(u7::new(self.value as u8));
                Ok(Some(m.into()))
            }
            AnalogMessageType::PotentiometerWithCCMessage14Bit => {
                let (value, id) = HiRes::new(self.value).control_change(index, self.analog.midi_id);
                let mut m = ControlChange::try_new_with_buffer(buffer)?;
                m.set_channel(channel);
                m.set_control(id);
                m.set_control_data(value);
                Ok(Some(m.into()))
            }
            AnalogMessageType::PitchBend => {
                let mut m = PitchBend::try_new_with_buffer(buffer)?;
                m.set_channel(channel);
                m.set_bend(u14::new(self.value));
                Ok(Some(m.into()))
            }
            AnalogMessageType::PotentiometerWithNoteMessage | AnalogMessageType::FSR => {
                let mut m = NoteOn::try_new_with_buffer(buffer)?;
                m.set_channel(channel);
                m.set_note_number(u7::new(self.analog.midi_id as u8));
                m.set_velocity(u7::new(self.value as u8));
                Ok(Some(m.into()))
            }
            AnalogMessageType::NRPN7 | AnalogMessageType::NRPN14 => {
                let (control, data) =
                    crate::handler::nprn::encode(index, self.analog.midi_id, self.value);
                let mut m = ControlChange::try_new_with_buffer(buffer)?;
                m.set_channel(channel);
                m.set_control(control);
                m.set_control_data(data);
                Ok(Some(m.into()))
            }
        }
    }
}

impl Analog {
    pub fn handle(&mut self, value: u16) -> AnalogMessages<'_> {
        AnalogMessages::new(self, self.scale_value(value))
    }
    fn scale_value(&self, value: u16) -> u16 {
        let input = if self.inverted {
            MAX_ADC_VALUE - value
        } else {
            value
        };

        let min_value = (MAX_ADC_VALUE as f32 * (self.lower_adc_offset as f32 / 100.0f32)) as u16;
        let max_value = MAX_ADC_VALUE
            - (MAX_ADC_VALUE as f32 * (self.upper_adc_offset as f32 / 100.0f32)) as u16;
        if input < min_value {
            return self.lower_limit;
        }
        if input > max_value {
            return self.upper_limit;
        }
        let factor = ((input - min_value) as f32) / ((max_value - min_value) as f32);
        self.lower_limit + (factor * (self.upper_limit - self.lower_limit) as f32) as u16
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::ChannelOrAll;

    #[test]
    fn test_disable() {
        let mut message_buffer = [0x00u8; 8];
        let mut analog = Analog {
            enabled: false,
            inverted: false,
            upper_limit: 99,
            lower_limit: 0,
            lower_adc_offset: 0,
            upper_adc_offset: 0,
            message_type: AnalogMessageType::PotentiometerWithCCMessage7Bit,
            midi_id: 0x03,
            channel: ChannelOrAll::default(),
        };
        let mut it = analog.handle(100);

        assert_eq!(Ok(None), it.next(&mut message_buffer));
    }
    #[test]
    fn test_cc_7bit() {
        let mut message_buffer = [0x00u8; 8];
        let mut analog = Analog {
            enabled: true,
            inverted: false,
            upper_limit: 99,
            lower_limit: 0,
            lower_adc_offset: 0,
            upper_adc_offset: 0,
            message_type: AnalogMessageType::PotentiometerWithCCMessage7Bit,
            midi_id: 0x03,
            channel: ChannelOrAll::Channel(1),
        };
        let mut it = analog.handle(100);

        let m = it.next(&mut message_buffer).unwrap().unwrap();
        assert_eq!(m.data(), [0xB1, 0x03, 0x02]);
        assert_eq!(Ok(None), it.next(&mut message_buffer));
    }
    #[test]
    fn test_cc_7bit_on_all_channels() {
        let mut buf = [0x00u8; 8];
        let mut analog = Analog {
            enabled: true,
            inverted: false,
            upper_limit: 99,
            lower_limit: 0,
            lower_adc_offset: 0,
            upper_adc_offset: 0,
            message_type: AnalogMessageType::PotentiometerWithCCMessage7Bit,
            midi_id: 0x03,
            channel: ChannelOrAll::All,
        };
        let mut it = analog.handle(100);

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
    fn test_cc_14bit() {
        let mut message_buffer = [0x00u8; 8];
        let mut analog = Analog {
            enabled: true,
            inverted: false,
            upper_limit: 1000,
            lower_limit: 0,
            lower_adc_offset: 0,
            upper_adc_offset: 0,
            message_type: AnalogMessageType::PotentiometerWithCCMessage14Bit,
            midi_id: 0x03,
            channel: ChannelOrAll::default(),
        };
        let mut it = analog.handle(MAX_ADC_VALUE);

        let m = it.next(&mut message_buffer).unwrap().unwrap();
        assert_eq!(m.data(), [176, 0x03, 0x7]);
        let m = it.next(&mut message_buffer).unwrap().unwrap();
        assert_eq!(m.data(), [176, 0x23, 0x68]);
        assert_eq!(Ok(None), it.next(&mut message_buffer));
    }
    #[test]
    fn test_pitch_bend() {
        let mut message_buffer = [0x00u8; 8];
        let mut analog = Analog {
            enabled: true,
            inverted: false,
            upper_limit: 1000,
            lower_limit: 0,
            lower_adc_offset: 0,
            upper_adc_offset: 0,
            message_type: AnalogMessageType::PitchBend,
            midi_id: 0x03,
            channel: ChannelOrAll::default(),
        };
        let mut it = analog.handle(MAX_ADC_VALUE);

        assert_eq!(
            it.next(&mut message_buffer).unwrap().unwrap().data(),
            [0xE0, 0x68, 0x7]
        );
        assert_eq!(Ok(None), it.next(&mut message_buffer));
    }
    #[test]
    fn test_note_on() {
        let mut message_buffer = [0x00u8; 8];
        let mut analog = Analog {
            enabled: true,
            inverted: false,
            upper_limit: 100,
            lower_limit: 0,
            lower_adc_offset: 0,
            upper_adc_offset: 0,
            message_type: AnalogMessageType::PotentiometerWithNoteMessage,
            midi_id: 0x03,
            channel: ChannelOrAll::default(),
        };
        let mut it = analog.handle(MAX_ADC_VALUE);

        assert_eq!(
            it.next(&mut message_buffer).unwrap().unwrap().data(),
            [0x90, 0x3, 0x64]
        );
        assert_eq!(Ok(None), it.next(&mut message_buffer));
    }
    #[test]
    fn test_nrpn7() {
        let mut message_buffer = [0x00u8; 8];
        let mut analog = Analog {
            enabled: true,
            inverted: false,
            upper_limit: 100,
            lower_limit: 0,
            lower_adc_offset: 0,
            upper_adc_offset: 0,
            message_type: AnalogMessageType::NRPN7,
            midi_id: 1624,
            channel: ChannelOrAll::default(),
        };
        let mut it = analog.handle(MAX_ADC_VALUE);

        // example see https://www.morningstar.io/post/sending-midi-nrpn-messages
        assert_eq!(
            it.next(&mut message_buffer).unwrap().unwrap().data(),
            [0xB0, 98, 88]
        );
        assert_eq!(
            it.next(&mut message_buffer).unwrap().unwrap().data(),
            [0xB0, 99, 12]
        );
        assert_eq!(
            it.next(&mut message_buffer).unwrap().unwrap().data(),
            [0xB0, 38, 100]
        );

        assert_eq!(Ok(None), it.next(&mut message_buffer));
    }
    #[test]
    fn test_nrpn14() {
        let mut message_buffer = [0x00u8; 8];
        let mut analog = Analog {
            enabled: true,
            inverted: false,
            upper_limit: 8234,
            lower_limit: 0,
            lower_adc_offset: 0,
            upper_adc_offset: 0,
            message_type: AnalogMessageType::NRPN14,
            midi_id: 1624,
            channel: ChannelOrAll::default(),
        };
        let mut it = analog.handle(MAX_ADC_VALUE);

        // example see https://www.morningstar.io/post/sending-midi-nrpn-messages
        assert_eq!(
            it.next(&mut message_buffer).unwrap().unwrap().data(),
            [0xB0, 98, 88]
        );
        assert_eq!(
            it.next(&mut message_buffer).unwrap().unwrap().data(),
            [0xB0, 99, 12]
        );
        assert_eq!(
            it.next(&mut message_buffer).unwrap().unwrap().data(),
            [0xB0, 38, 42]
        );
        assert_eq!(
            it.next(&mut message_buffer).unwrap().unwrap().data(),
            [0xB0, 6, 64]
        );

        assert_eq!(Ok(None), it.next(&mut message_buffer));
    }

    #[test]
    fn test_overflow() {
        let mut message_buffer = [0x00u8; 1];
        let mut analog = Analog {
            enabled: true,
            inverted: false,
            upper_limit: 127,
            lower_limit: 0,
            lower_adc_offset: 0,
            upper_adc_offset: 0,
            message_type: AnalogMessageType::PotentiometerWithCCMessage7Bit,
            midi_id: 0x03,
            channel: ChannelOrAll::default(),
        };
        let mut it = analog.handle(10);

        let m = it.next(&mut message_buffer);
        assert_eq!(m, Err(BufferOverflow));
    }

    #[test]
    fn test_scale() {
        let analog = Analog {
            enabled: true,
            inverted: false,
            upper_limit: 127,
            lower_limit: 0,
            lower_adc_offset: 0,
            upper_adc_offset: 0,
            message_type: AnalogMessageType::PotentiometerWithCCMessage7Bit,
            midi_id: 0x03,
            channel: ChannelOrAll::default(),
        };
        assert_eq!(0, analog.scale_value(0));
        assert_eq!(127, analog.scale_value(MAX_ADC_VALUE));
        assert_eq!(63, analog.scale_value(2047));
    }
    #[test]
    fn test_scale_with_offset() {
        let analog = Analog {
            enabled: true,
            inverted: false,
            upper_limit: 99,
            lower_limit: 0,
            lower_adc_offset: 10,
            upper_adc_offset: 10,
            message_type: AnalogMessageType::PotentiometerWithCCMessage7Bit,
            midi_id: 0x03,
            channel: ChannelOrAll::default(),
        };
        assert_eq!(0, analog.scale_value(0));
        assert_eq!(0, analog.scale_value(409));
        assert_eq!(99, analog.scale_value(MAX_ADC_VALUE));
        assert_eq!(99, analog.scale_value(3686));
        assert_eq!(49, analog.scale_value(2047));
    }
    #[test]
    fn test_scale_invert() {
        let analog = Analog {
            enabled: true,
            inverted: true,
            upper_limit: 127,
            lower_limit: 0,
            lower_adc_offset: 0,
            upper_adc_offset: 0,
            message_type: AnalogMessageType::PotentiometerWithCCMessage7Bit,
            midi_id: 0x03,
            channel: ChannelOrAll::default(),
        };
        assert_eq!(127, analog.scale_value(0));
        assert_eq!(0, analog.scale_value(MAX_ADC_VALUE));
        assert_eq!(63, analog.scale_value(2047));
    }
}
