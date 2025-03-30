use crate::button::{Button, ButtonSection, ButtonSectionId};
use crate::{Amount, Block, NewValues, OpenDeckResponse, Wish};

pub struct ButtonBackupIterator {
    button_index: u16,
    button_section_id: ButtonSectionId,
    done: bool,
}

impl ButtonBackupIterator {
    pub fn new(button_index: usize) -> Self {
        ButtonBackupIterator {
            button_index: button_index as u16,
            button_section_id: ButtonSectionId::Type,
            done: false,
        }
    }
    pub fn next(&mut self, button: &Button) -> Option<OpenDeckResponse> {
        if self.done {
            return None;
        }
        let new_values = NewValues::new();
        let button_section = match self.button_section_id {
            ButtonSectionId::Type => {
                self.button_section_id = ButtonSectionId::MessageType;
                ButtonSection::Type(button.button_type)
            }
            ButtonSectionId::MessageType => {
                self.button_section_id = ButtonSectionId::MidiId;
                ButtonSection::MessageType(button.message_type)
            }
            ButtonSectionId::MidiId => {
                self.button_section_id = ButtonSectionId::Value;
                ButtonSection::MidiId(button.midi_id)
            }
            ButtonSectionId::Value => {
                self.button_section_id = ButtonSectionId::Channel;
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
            Block::Button(self.button_index, button_section),
            new_values,
        ))
    }
}
