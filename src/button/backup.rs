use crate::button::{Button, ButtonSection, ButtonSectionId};
use crate::{Amount, Block, NewValues, OpenDeckResponse, Wish};

pub struct ButtonBackupIterator<'a> {
    button: &'a Button,
    button_index: u16,
    button_section_id: ButtonSectionId,
    done: bool,
}

impl<'a> ButtonBackupIterator<'a> {
    pub fn new(button: &'a Button, button_index: usize) -> Self {
        ButtonBackupIterator {
            button,
            button_index: button_index as u16,
            button_section_id: ButtonSectionId::Type,
            done: false,
        }
    }
}

impl Iterator for ButtonBackupIterator<'_> {
    type Item = OpenDeckResponse;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let new_values = NewValues::new();
        let button_section = match self.button_section_id {
            ButtonSectionId::Type => {
                self.button_section_id = ButtonSectionId::MessageType;
                ButtonSection::Type(self.button.button_type)
            }
            ButtonSectionId::MessageType => {
                self.button_section_id = ButtonSectionId::MidiId;
                ButtonSection::MessageType(self.button.message_type)
            }
            ButtonSectionId::MidiId => {
                self.button_section_id = ButtonSectionId::Value;
                ButtonSection::MidiId(self.button.midi_id)
            }
            ButtonSectionId::Value => {
                self.button_section_id = ButtonSectionId::Channel;
                ButtonSection::Value(self.button.value)
            }
            ButtonSectionId::Channel => {
                self.done = true;
                ButtonSection::Channel(self.button.channel)
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
