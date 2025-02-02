use crate::analog::{Analog, AnalogMessageType};

use midi2::{channel_voice1::ControlChange, error::BufferOverflow, prelude::*, BytesMessage};

//struct AnalogIterator

impl Analog {
    pub fn handle<'a>(
        &mut self,
        value: u16,
        buffer: &'a mut [u8],
    ) -> Result<Option<BytesMessage<&'a mut [u8]>>, BufferOverflow> {
        match self.message_type {
            AnalogMessageType::Button => {
                let mut m = ControlChange::try_new_with_buffer(buffer)?;
                m.set_channel(self.channel.into_midi());
                m.set_control(u7::new(self.midi_id as u8));
                m.set_control_data(u7::new(value as u8));
                Ok(Some(m.into()))
            }
            AnalogMessageType::PotentiometerWithCCMessage7Bit => Ok(None),
            AnalogMessageType::PotentiometerWithNoteMessage => Ok(None),
            AnalogMessageType::FSR => Ok(None),
            AnalogMessageType::NRPN7 => Ok(None),
            AnalogMessageType::NRPN8 => Ok(None),
            AnalogMessageType::PitchBend => Ok(None),
            AnalogMessageType::PotentiometerWithCCMessage14Bit => Ok(None),
        }
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
        let result = button.handle(1000, &mut message_buffer).unwrap().unwrap();

        let buf = result.data();
        assert_eq!(buf, [0x90, 0x03, 0x7F]);
    }
}
