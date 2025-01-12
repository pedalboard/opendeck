use crate::button::{Button, ButtonMessageType, ButtonType, ChannelOrAll};

use midi2::{
    buffer::{Buffer, BufferDefault, BufferMut, BufferTryResize},
    channel_voice1::{ChannelVoice1, ControlChange, NoteOn, ProgramChange},
    error::BufferOverflow,
    prelude::*,
    system_common::{ActiveSensing, Continue, Reset, Start, Stop, SystemCommon, TimingClock},
    BytesMessage,
};

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
    pub fn handle<B>(&mut self, action: Action) -> Result<Option<BytesMessage<B>>, BufferOverflow>
    where
        B: Buffer<Unit = u8> + BufferMut + BufferDefault + BufferTryResize,
    {
        let status = self.latch(&action);

        match self.message_type {
            ButtonMessageType::Notes => match status {
                ButtonStatus::On => {
                    let mut m = NoteOn::<B>::try_new()?;
                    m.set_velocity(u7::new(self.value));
                    m.set_note_number(u7::new(self.midi_id));
                    m.set_channel(self.channel());
                    Ok(Some(BytesMessage::<B>::ChannelVoice1(
                        ChannelVoice1::NoteOn(m),
                    )))
                }
                ButtonStatus::Off => {
                    let mut m = NoteOn::<B>::try_new()?;
                    m.set_velocity(u7::MIN);
                    m.set_channel(self.channel());
                    m.set_note_number(u7::new(self.midi_id));
                    Ok(Some(BytesMessage::<B>::ChannelVoice1(
                        ChannelVoice1::NoteOn(m),
                    )))
                }
                ButtonStatus::None => Ok(None),
            },
            ButtonMessageType::ProgramChange => {
                if let Action::Pressed = action {
                    let mut m = ProgramChange::<B>::try_new()?;
                    m.set_channel(self.channel());
                    m.set_program(u7::new(self.midi_id));
                    return Ok(Some(BytesMessage::<B>::ChannelVoice1(
                        ChannelVoice1::ProgramChange(m),
                    )));
                }
                Ok(None)
            }
            ButtonMessageType::ControlChange => {
                if let Action::Pressed = action {
                    let mut m = ControlChange::<B>::try_new()?;
                    m.set_channel(self.channel());
                    m.set_control(u7::new(self.midi_id));
                    m.set_control_data(u7::new(self.value));
                    return Ok(Some(BytesMessage::<B>::ChannelVoice1(
                        ChannelVoice1::ControlChange(m),
                    )));
                }
                Ok(None)
            }
            ButtonMessageType::ControlChangeWithReset => match status {
                ButtonStatus::On => {
                    let mut m = ControlChange::<B>::try_new()?;
                    m.set_channel(self.channel());
                    m.set_control(u7::new(self.midi_id));
                    m.set_control_data(u7::new(self.value));
                    Ok(Some(BytesMessage::<B>::ChannelVoice1(
                        ChannelVoice1::ControlChange(m),
                    )))
                }
                ButtonStatus::Off => {
                    let mut m = ControlChange::<B>::try_new()?;
                    m.set_channel(self.channel());
                    m.set_control(u7::new(self.midi_id));
                    m.set_control_data(u7::new(0));
                    Ok(Some(BytesMessage::<B>::ChannelVoice1(
                        ChannelVoice1::ControlChange(m),
                    )))
                }
                ButtonStatus::None => Ok(None),
            },
            ButtonMessageType::ControlChangeWithValue0 => {
                if let Action::Pressed = action {
                    let mut m = ControlChange::<B>::try_new()?;
                    m.set_channel(self.channel());
                    m.set_control(u7::new(self.midi_id));
                    m.set_control_data(u7::new(0x00));
                    return Ok(Some(BytesMessage::<B>::ChannelVoice1(
                        ChannelVoice1::ControlChange(m),
                    )));
                }
                Ok(None)
            }

            ButtonMessageType::MMCStop => Ok(None),
            ButtonMessageType::MMCPlay => Ok(None),
            ButtonMessageType::MMCRecord => Ok(None),
            ButtonMessageType::MMCPause => Ok(None),

            ButtonMessageType::RealTimeClock => {
                if let Action::Pressed = action {
                    let tc = TimingClock::try_new()?;
                    return Ok(Some(BytesMessage::<B>::SystemCommon(
                        SystemCommon::TimingClock(tc),
                    )));
                }
                Ok(None)
            }

            ButtonMessageType::RealTimeStart => {
                if let Action::Pressed = action {
                    let start = Start::try_new()?;
                    return Ok(Some(BytesMessage::<B>::SystemCommon(SystemCommon::Start(
                        start,
                    ))));
                }
                Ok(None)
            }
            ButtonMessageType::RealTimeContinue => {
                if let Action::Pressed = action {
                    let c = Continue::try_new()?;
                    return Ok(Some(BytesMessage::<B>::SystemCommon(
                        SystemCommon::Continue(c),
                    )));
                }
                Ok(None)
            }
            ButtonMessageType::RealTimeStop => {
                if let Action::Pressed = action {
                    let stop = Stop::try_new()?;
                    return Ok(Some(BytesMessage::<B>::SystemCommon(SystemCommon::Stop(
                        stop,
                    ))));
                }
                Ok(None)
            }
            ButtonMessageType::RealTimeActiveSensing => {
                if let Action::Pressed = action {
                    let s = ActiveSensing::try_new()?;
                    return Ok(Some(BytesMessage::<B>::SystemCommon(
                        SystemCommon::ActiveSensing(s),
                    )));
                }
                Ok(None)
            }
            ButtonMessageType::RealTimeSystemReset => {
                if let Action::Pressed = action {
                    let r = Reset::try_new()?;
                    return Ok(Some(BytesMessage::<B>::SystemCommon(SystemCommon::Reset(
                        r,
                    ))));
                }
                Ok(None)
            }

            ButtonMessageType::ProgramChangeIncr => Ok(None),
            ButtonMessageType::ProgramChangeDecr => Ok(None),
            ButtonMessageType::OpenDeckPresetChange => Ok(None),
            ButtonMessageType::MultiValueIncNote => Ok(None),
            ButtonMessageType::MultiValueDecNote => Ok(None),
            ButtonMessageType::MultiValueIncCC => Ok(None),
            ButtonMessageType::MultiValueDecCC => Ok(None),
            ButtonMessageType::NoteOffOnly => Ok(None),
            ButtonMessageType::ProgramChangeOffsetIncr => Ok(None),
            ButtonMessageType::ProgramChangeOffsetDecr => Ok(None),
            ButtonMessageType::BPMIncr => Ok(None),
            ButtonMessageType::BPMDecr => Ok(None),

            ButtonMessageType::NoMessage => Ok(None),
            ButtonMessageType::Reserved => Ok(None),
        }
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
    fn channel(&self) -> u4 {
        // FIXME: This is a temporary solution to get the code to compile
        u4::new(match self.channel {
            ChannelOrAll::All => 0xF,
            ChannelOrAll::Channel(c) => c,
            ChannelOrAll::None => 0,
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::ChannelOrAll;

    #[test]
    fn test_note_on() {
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::Notes,
            midi_id: 0x03,
            value: 0x7F,
            channel: ChannelOrAll::default(),
            latch_on: false,
        };
        let result = button.handle::<[u8; 3]>(Action::Pressed).unwrap().unwrap();

        let buf = result.data();
        assert_eq!(buf, [0x90, 0x03, 0x7F]);
    }
    #[test]
    fn test_program_change() {
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::ProgramChange,
            midi_id: 0x03,
            value: 0x7F,
            channel: ChannelOrAll::default(),
            latch_on: false,
        };
        let result = button.handle::<[u8; 3]>(Action::Pressed).unwrap().unwrap();
        assert_eq!(result.data(), [0xC0, 0x03]);
    }
    #[test]
    fn test_program_change_release() {
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::ProgramChange,
            midi_id: 0x03,
            value: 0x7F,
            channel: ChannelOrAll::default(),
            latch_on: false,
        };
        let result = button.handle::<[u8; 3]>(Action::Released).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_control_change() {
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::ControlChange,
            midi_id: 0x03,
            value: 0x7F,
            channel: ChannelOrAll::default(),
            latch_on: false,
        };
        let result = button.handle::<[u8; 3]>(Action::Pressed).unwrap().unwrap();
        assert_eq!(result.data(), [0xB0, 0x03, 0x7F]);
    }
    #[test]
    fn test_control_change_release() {
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::ControlChange,
            midi_id: 0x03,
            value: 0x7F,
            channel: ChannelOrAll::default(),
            latch_on: false,
        };
        let result = button.handle::<[u8; 3]>(Action::Released).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_control_change_with_reset() {
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::ControlChangeWithReset,
            midi_id: 0x03,
            value: 0x7F,
            channel: ChannelOrAll::default(),
            latch_on: false,
        };
        let result = button.handle::<[u8; 3]>(Action::Pressed).unwrap().unwrap();
        assert_eq!(result.data(), [0xB0, 0x03, 0x7F]);
    }
    #[test]
    fn test_control_change_with_reset_release() {
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::ControlChangeWithReset,
            midi_id: 0x03,
            value: 0x7F,
            channel: ChannelOrAll::default(),
            latch_on: false,
        };
        let result = button.handle::<[u8; 3]>(Action::Released).unwrap().unwrap();
        assert_eq!(result.data(), [0xB0, 0x03, 0x00]);
    }

    #[test]
    fn test_control_change_with_value0() {
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::ControlChangeWithValue0,
            midi_id: 0x03,
            value: 0x7F,
            channel: ChannelOrAll::default(),
            latch_on: false,
        };
        let result = button.handle::<[u8; 3]>(Action::Pressed).unwrap().unwrap();
        assert_eq!(result.data(), [0xB0, 0x03, 0x00]);
    }

    #[test]
    fn test_no_message() {
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::NoMessage,
            midi_id: 0x03,
            value: 0x7F,
            channel: ChannelOrAll::default(),
            latch_on: false,
        };
        let result = button.handle::<[u8; 3]>(Action::Released).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_realtime_clock() {
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::RealTimeClock,
            midi_id: 0x03,
            value: 0x7F,
            channel: ChannelOrAll::default(),
            latch_on: false,
        };
        let result = button.handle::<[u8; 3]>(Action::Pressed).unwrap().unwrap();
        assert_eq!(result.data(), [0xF8, 0x00]);
    }
    #[test]
    fn test_realtime_start() {
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::RealTimeStart,
            midi_id: 0x03,
            value: 0x7F,
            channel: ChannelOrAll::default(),
            latch_on: false,
        };
        let result = button.handle::<[u8; 3]>(Action::Pressed).unwrap().unwrap();
        assert_eq!(result.data(), [0xFA, 0x00]);
    }
    #[test]
    fn test_realtime_stop() {
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::RealTimeStop,
            midi_id: 0x03,
            value: 0x7F,
            channel: ChannelOrAll::default(),
            latch_on: false,
        };
        let result = button.handle::<[u8; 3]>(Action::Pressed).unwrap().unwrap();
        assert_eq!(result.data(), [0xFC, 0x00]);
    }
    #[test]
    fn test_realtime_continue() {
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::RealTimeContinue,
            midi_id: 0x03,
            value: 0x7F,
            channel: ChannelOrAll::default(),
            latch_on: false,
        };
        let result = button.handle::<[u8; 3]>(Action::Pressed).unwrap().unwrap();
        assert_eq!(result.data(), [0xFB, 0x00]);
    }
    #[test]
    fn test_realtime_active_sensing() {
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::RealTimeActiveSensing,
            midi_id: 0x03,
            value: 0x7F,
            channel: ChannelOrAll::default(),
            latch_on: false,
        };
        let result = button.handle::<[u8; 3]>(Action::Pressed).unwrap().unwrap();
        assert_eq!(result.data(), [0xFE, 0x00]);
    }
    #[test]
    fn test_realtime_reset() {
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::RealTimeSystemReset,
            midi_id: 0x03,
            value: 0x7F,
            channel: ChannelOrAll::default(),
            latch_on: false,
        };
        let result = button.handle::<[u8; 3]>(Action::Pressed).unwrap().unwrap();
        assert_eq!(result.data(), [0xFF, 0x00]);
    }
}
