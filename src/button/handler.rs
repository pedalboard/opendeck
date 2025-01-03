use crate::button::{Button, ButtonMessageType, ButtonType};
use heapless::Vec;
use midi_types::{MidiMessage, Note, Value7};

pub enum Action {
    Pressed,
    Released,
}

enum ButtonStatus {
    Off,
    On,
    None,
}

impl Button {
    pub fn handle(&mut self, action: Action) -> Vec<MidiMessage, 16> {
        let mut result = Vec::new();
        let value: u8 = self.midi_id.into();
        let status = self.latch(action);

        for ch in self.channel.channels() {
            match self.message_type {
                ButtonMessageType::Notes => match status {
                    ButtonStatus::On => {
                        result
                            .push(MidiMessage::NoteOn(ch, Note::from(value), self.value))
                            .unwrap();
                    }
                    ButtonStatus::Off => {
                        result
                            .push(MidiMessage::NoteOn(ch, Note::from(value), Value7::new(0)))
                            .unwrap();
                    }
                    ButtonStatus::None => {}
                },
                ButtonMessageType::ProgramChange => {}
                ButtonMessageType::ControlChange => {}
                ButtonMessageType::ControlChangeWithReset => {}
                ButtonMessageType::MMCStop => {}
                ButtonMessageType::MMCPlay => {}
                ButtonMessageType::MMCRecord => {}
                ButtonMessageType::MMCPause => {}
                ButtonMessageType::RealTimeClock => {}
                ButtonMessageType::RealTimeStart => {}
                ButtonMessageType::RealTimeContinue => {}

                ButtonMessageType::RealTimeStop => {}
                ButtonMessageType::RealTimeActiveSensing => {}
                ButtonMessageType::RealTimeSystemReset => {}
                ButtonMessageType::ProgramChangeIncr => {}
                ButtonMessageType::ProgramChangeDecr => {}
                ButtonMessageType::NoMessage => {}
                ButtonMessageType::OpenDeckPresetChange => {}
                ButtonMessageType::MultiValueIncNote => {}
                ButtonMessageType::MultiValueDecNote => {}
                ButtonMessageType::MultiValueIncCC => {}
                ButtonMessageType::MultiValueDecCC => {}
                ButtonMessageType::NoteOffOnly => {}
                ButtonMessageType::ControlChangeWithValue0 => {}
                ButtonMessageType::Reserved => {}
                ButtonMessageType::ProgramChangeOffsetIncr => {}
                ButtonMessageType::ProgramChangeOffsetDecr => {}
                ButtonMessageType::BPMIncr => {}
                ButtonMessageType::BPMDecr => {}
            }
        }

        result
    }
    fn latch(&mut self, action: Action) -> ButtonStatus {
        match self.button_type {
            ButtonType::Momentary => match action {
                Action::Pressed => ButtonStatus::On,
                Action::Released => ButtonStatus::Off,
            },
            ButtonType::Latching => {
                if let Action::Pressed = action {
                    self.latch_on = !self.latch_on;
                    if self.latch_on {
                        return ButtonStatus::On;
                    } else {
                        return ButtonStatus::Off;
                    }
                }
                ButtonStatus::None
            }
        }
    }
}
