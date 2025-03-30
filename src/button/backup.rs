use crate::button::{Button, ButtonSection, ButtonSectionId};
use crate::{Amount, Block, NewValues, OpenDeckResponse, Wish};

pub struct ButtonBackupIterator {
    index: u16,
    section_id: ButtonSectionId,
    done: bool,
}

impl ButtonBackupIterator {
    pub fn new(index: usize) -> Self {
        ButtonBackupIterator {
            index: index as u16,
            section_id: ButtonSectionId::Type,
            done: false,
        }
    }
    pub fn next(&mut self, button: &Button) -> Option<OpenDeckResponse> {
        if self.done {
            return None;
        }
        let new_values = NewValues::new();
        let button_section = match self.section_id {
            ButtonSectionId::Type => {
                self.section_id = ButtonSectionId::MessageType;
                ButtonSection::Type(button.button_type)
            }
            ButtonSectionId::MessageType => {
                self.section_id = ButtonSectionId::MidiId;
                ButtonSection::MessageType(button.message_type)
            }
            ButtonSectionId::MidiId => {
                self.section_id = ButtonSectionId::Value;
                ButtonSection::MidiId(button.midi_id)
            }
            ButtonSectionId::Value => {
                self.section_id = ButtonSectionId::Channel;
                ButtonSection::Value(button.value)
            }
            ButtonSectionId::Channel => {
                self.done = true;
                ButtonSection::Channel(button.channel)
            }
        };

        Some(OpenDeckResponse::Configuration(
            Wish::Set,
            Amount::Single,
            Block::Button(self.index, button_section),
            new_values,
        ))
    }
}
