use crate::analog::{Analog, AnalogMessageType};

use midi2::{channel_voice1::ControlChange, error::BufferOverflow, prelude::*, BytesMessage};

pub struct AnalogMessages<'a> {
    analog: &'a mut Analog,
    value: u16,
    index: usize,
}
impl<'a> AnalogMessages<'a> {
    pub fn new(analog: &'a mut Analog, value: u16) -> Self {
        Self {
            analog,
            value,
            index: 0,
        }
    }
    pub fn next<'buf>(
        &mut self,
        buffer: &'buf mut [u8],
    ) -> Option<Result<BytesMessage<&'buf mut [u8]>, BufferOverflow>> {
        match self.analog.message_type {
            AnalogMessageType::Button => None,
            AnalogMessageType::PotentiometerWithCCMessage7Bit => {
                if self.index > 0 {
                    return None;
                }
                let mut m = ControlChange::try_new_with_buffer(buffer).unwrap();
                m.set_channel(self.analog.channel.into_midi());
                m.set_control(u7::new(self.analog.midi_id as u8));
                m.set_control_data(u7::new(self.value as u8));
                self.index += 1;
                Some(Ok(m.into()))
            }
            AnalogMessageType::PotentiometerWithNoteMessage => None,
            AnalogMessageType::FSR => None,
            AnalogMessageType::NRPN7 => None,
            AnalogMessageType::NRPN8 => None,
            AnalogMessageType::PitchBend => None,
            AnalogMessageType::PotentiometerWithCCMessage14Bit => None,
        }
    }
}

impl Analog {
    pub fn handle(&mut self, value: u16) -> AnalogMessages<'_> {
        AnalogMessages::new(self, value)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::ChannelOrAll;

    #[test]
    fn test_note_on() {
        let mut message_buffer = [0x00u8; 8];
        let mut button = Analog {
            enabled: true,
            invert_state: false,
            upper_limit: 127,
            lower_limit: 127,
            lower_adc_offset: 0,
            upper_adc_offset: 0,
            message_type: AnalogMessageType::PotentiometerWithCCMessage7Bit,
            midi_id: 0x03,
            channel: ChannelOrAll::default(),
        };
        let mut it = button.handle(10);

        let m = it.next(&mut message_buffer).unwrap().unwrap();
        assert_eq!(m.data(), [176, 0x03, 10]);
        assert_eq!(None, it.next(&mut message_buffer));
    }
}
