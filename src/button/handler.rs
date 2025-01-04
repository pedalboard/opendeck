use crate::button::{Button, ButtonMessageType, ButtonType};
use heapless::Vec;
use midi_types::{Control, MidiMessage, Note, Program, Value7};

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
        let midi_id: u8 = self.midi_id.into();
        let status = self.latch(&action);

        for ch in self.channel.channels() {
            match self.message_type {
                ButtonMessageType::Notes => match status {
                    ButtonStatus::On => {
                        result
                            .push(MidiMessage::NoteOn(ch, Note::from(midi_id), self.value))
                            .unwrap();
                    }
                    ButtonStatus::Off => {
                        result
                            .push(MidiMessage::NoteOn(ch, Note::from(midi_id), Value7::new(0)))
                            .unwrap();
                    }
                    ButtonStatus::None => {}
                },
                ButtonMessageType::ProgramChange => {
                    if let Action::Pressed = action {
                        result
                            .push(MidiMessage::ProgramChange(ch, Program::from(midi_id)))
                            .unwrap();
                    }
                }
                ButtonMessageType::ControlChange => {
                    if let Action::Pressed = action {
                        result
                            .push(MidiMessage::ControlChange(
                                ch,
                                Control::from(midi_id),
                                self.value,
                            ))
                            .unwrap();
                    }
                }
                ButtonMessageType::ControlChangeWithReset => match status {
                    ButtonStatus::On => {
                        result
                            .push(MidiMessage::ControlChange(
                                ch,
                                Control::from(midi_id),
                                self.value,
                            ))
                            .unwrap();
                    }
                    ButtonStatus::Off => {
                        result
                            .push(MidiMessage::ControlChange(
                                ch,
                                Control::from(midi_id),
                                Value7::new(0),
                            ))
                            .unwrap();
                    }
                    ButtonStatus::None => {}
                },
                ButtonMessageType::ControlChangeWithValue0 => {
                    if let Action::Pressed = action {
                        result
                            .push(MidiMessage::ControlChange(
                                ch,
                                Control::from(midi_id),
                                Value7::new(0),
                            ))
                            .unwrap();
                    }
                }

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
                ButtonMessageType::Reserved => {}
                ButtonMessageType::ProgramChangeOffsetIncr => {}
                ButtonMessageType::ProgramChangeOffsetDecr => {}
                ButtonMessageType::BPMIncr => {}
                ButtonMessageType::BPMDecr => {}
            }
        }

        result
    }
    fn latch(&mut self, action: &Action) -> ButtonStatus {
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
#[cfg(test)]
mod tests {

    use midi_types::Channel;

    use super::*;
    use crate::ChannelOrAll;

    #[test]
    fn test_note_on() {
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::Notes,
            midi_id: Value7::new(0x03),
            value: Value7::new(0x7F),
            channel: ChannelOrAll::default(),
            latch_on: false,
        };
        let result = button.handle(Action::Pressed);
        assert_eq!(
            result,
            [MidiMessage::NoteOn(
                Channel::C1,
                Note::from(0x03),
                Value7::new(0x7F)
            )]
        );
    }

    #[test]
    fn test_program_change() {
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::ProgramChange,
            midi_id: Value7::new(0x03),
            value: Value7::new(0x7F),
            channel: ChannelOrAll::default(),
            latch_on: false,
        };
        let result = button.handle(Action::Pressed);
        assert_eq!(
            result,
            [MidiMessage::ProgramChange(Channel::C1, Program::from(0x03))]
        );
    }
    #[test]
    fn test_program_change_release() {
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::ProgramChange,
            midi_id: Value7::new(0x03),
            value: Value7::new(0x7F),
            channel: ChannelOrAll::default(),
            latch_on: false,
        };
        let result = button.handle(Action::Released);
        assert_eq!(result, []);
    }

    #[test]
    fn test_control_change() {
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::ControlChange,
            midi_id: Value7::new(0x03),
            value: Value7::new(0x7F),
            channel: ChannelOrAll::default(),
            latch_on: false,
        };
        let result = button.handle(Action::Pressed);
        assert_eq!(
            result,
            [MidiMessage::ControlChange(
                Channel::C1,
                Control::from(0x03),
                Value7::new(0x7F)
            )]
        );
    }
    #[test]
    fn test_control_change_release() {
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::ControlChange,
            midi_id: Value7::new(0x03),
            value: Value7::new(0x7F),
            channel: ChannelOrAll::default(),
            latch_on: false,
        };
        let result = button.handle(Action::Released);
        assert_eq!(result, []);
    }

    #[test]
    fn test_control_change_with_reset() {
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::ControlChangeWithReset,
            midi_id: Value7::new(0x03),
            value: Value7::new(0x7F),
            channel: ChannelOrAll::default(),
            latch_on: false,
        };
        let result = button.handle(Action::Pressed);
        assert_eq!(
            result,
            [MidiMessage::ControlChange(
                Channel::C1,
                Control::from(0x03),
                Value7::new(0x7F)
            )]
        );
    }
    #[test]
    fn test_control_change_with_reset_release() {
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::ControlChangeWithReset,
            midi_id: Value7::new(0x03),
            value: Value7::new(0x7F),
            channel: ChannelOrAll::default(),
            latch_on: false,
        };
        let result = button.handle(Action::Released);
        assert_eq!(
            result,
            [MidiMessage::ControlChange(
                Channel::C1,
                Control::from(0x03),
                Value7::new(0)
            )]
        );
    }

    #[test]
    fn test_control_change_with_value0() {
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::ControlChangeWithValue0,
            midi_id: Value7::new(0x03),
            value: Value7::new(0x7F),
            channel: ChannelOrAll::default(),
            latch_on: false,
        };
        let result = button.handle(Action::Pressed);
        assert_eq!(
            result,
            [MidiMessage::ControlChange(
                Channel::C1,
                Control::from(3),
                Value7::new(0)
            )]
        );
    }
}
