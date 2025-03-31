use crate::analog::{Analog, AnalogSection, AnalogSectionId};
use crate::{Amount, Block, NewValues, OpenDeckResponse, Wish};

pub struct AnalogBackupIterator {
    index: u16,
    section_id: AnalogSectionId,
    done: bool,
}

impl AnalogBackupIterator {
    pub fn new(index: usize) -> Self {
        AnalogBackupIterator {
            index: index as u16,
            section_id: AnalogSectionId::Enabled,
            done: false,
        }
    }
    pub fn next(&mut self, analog: &Analog) -> Option<OpenDeckResponse> {
        if self.done {
            return None;
        }
        let new_values = NewValues::new();
        let analog_section = match self.section_id {
            AnalogSectionId::Enabled => {
                self.section_id = AnalogSectionId::InvertState;
                AnalogSection::Enabled(analog.enabled)
            }
            AnalogSectionId::InvertState => {
                self.section_id = AnalogSectionId::MessageType;
                AnalogSection::Inverted(analog.inverted)
            }
            AnalogSectionId::MessageType => {
                self.section_id = AnalogSectionId::MidiIdLSB;
                AnalogSection::MessageType(analog.message_type)
            }
            AnalogSectionId::MidiIdLSB => {
                self.section_id = AnalogSectionId::LowerCCLimitLSB;
                AnalogSection::MidiId(analog.midi_id)
            }
            AnalogSectionId::LowerCCLimitLSB => {
                self.section_id = AnalogSectionId::UpperCCLimitLSB;
                AnalogSection::LowerCCLimit(analog.lower_limit)
            }
            AnalogSectionId::UpperCCLimitLSB => {
                self.section_id = AnalogSectionId::Channel;
                AnalogSection::UpperCCLimit(analog.upper_limit)
            }
            AnalogSectionId::Channel => {
                self.section_id = AnalogSectionId::LowerADCOffset;
                AnalogSection::Channel(analog.channel)
            }
            AnalogSectionId::LowerADCOffset => {
                self.section_id = AnalogSectionId::UpperADCOffset;
                AnalogSection::LowerADCOffset(analog.lower_adc_offset)
            }
            AnalogSectionId::UpperADCOffset => {
                self.done = true;
                AnalogSection::UpperADCOffset(analog.upper_adc_offset)
            }
        };

        Some(OpenDeckResponse::Configuration(
            Wish::Set,
            Amount::Single,
            Block::Analog(self.index, analog_section),
            new_values,
        ))
    }
}
