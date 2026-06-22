use crate::led::{Led, LedSection, LedSectionId};
use crate::{Amount, Block, NewValues, OpenDeckResponse, Wish};

pub struct LedBackupIterator {
    index: u16,
    section_id: LedSectionId,
    done: bool,
}

impl LedBackupIterator {
    pub fn new(index: usize) -> Self {
        LedBackupIterator {
            index: index as u16,
            section_id: LedSectionId::State,
            done: false,
        }
    }
    pub fn next(&mut self, led: &Led) -> Option<OpenDeckResponse> {
        if self.done {
            return None;
        }
        let new_values = NewValues::new();
        let led_section = match self.section_id {
            LedSectionId::State => {
                self.section_id = LedSectionId::BlinkTesting;
                LedSection::State(led.state)
            }
            LedSectionId::BlinkTesting => {
                self.section_id = LedSectionId::ActivationId;
                LedSection::BlinkTesting(led.blink_testing)
            }
            LedSectionId::ActivationId => {
                self.section_id = LedSectionId::Reserved;
                LedSection::ActivationId(led.activation_id)
            }
            LedSectionId::Reserved => {
                self.section_id = LedSectionId::ControlType;
                LedSection::Reserved(0)
            }
            LedSectionId::ControlType => {
                self.section_id = LedSectionId::ActivationValue;
                LedSection::ControlType(led.control_type)
            }
            LedSectionId::ActivationValue => {
                self.section_id = LedSectionId::Channel;
                LedSection::ActivationValue(led.activation_value)
            }
            LedSectionId::Channel => {
                self.done = true;
                LedSection::Channel(led.channel)
            }
            LedSectionId::Global => {
                self.done = true;
                LedSection::Channel(led.channel)
            }
        };

        Some(OpenDeckResponse::Configuration(
            Wish::Set,
            Amount::Single,
            Block::Led(self.index, led_section),
            new_values,
        ))
    }
}
