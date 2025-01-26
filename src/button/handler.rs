use crate::button::{Button, ButtonMessageType, ButtonType, ChannelOrAll};

use midi2::{
    channel_voice1::{ControlChange, NoteOff, NoteOn, ProgramChange},
    error::BufferOverflow,
    prelude::*,
    sysex7::Sysex7,
    system_common::{ActiveSensing, Continue, Reset, Start, Stop, TimingClock},
    BytesMessage,
};

const MAX_MIDI_ID: u8 = 127;
const MAX_VALUE: u8 = 127;

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
    pub fn handle<'a>(
        &mut self,
        action: Action,
        buffer: &'a mut [u8],
    ) -> Result<Option<BytesMessage<&'a mut [u8]>>, BufferOverflow> {
        let status = self.latch(&action);

        match self.message_type {
            ButtonMessageType::Notes => match status {
                ButtonStatus::On => {
                    let mut m = NoteOn::try_new_with_buffer(buffer)?;
                    m.set_velocity(u7::new(self.value));
                    m.set_note_number(u7::new(self.midi_id));
                    m.set_channel(self.channel());
                    Ok(Some(m.into()))
                }
                ButtonStatus::Off => {
                    let mut m = NoteOn::try_new_with_buffer(buffer)?;
                    m.set_velocity(u7::MIN);
                    m.set_channel(self.channel());
                    m.set_note_number(u7::new(self.midi_id));
                    Ok(Some(m.into()))
                }
                ButtonStatus::None => Ok(None),
            },
            ButtonMessageType::NoteOffOnly => {
                if let Action::Pressed = action {
                    let mut m = NoteOff::try_new_with_buffer(buffer)?;
                    m.set_velocity(u7::new(self.value));
                    m.set_note_number(u7::new(self.midi_id));
                    m.set_channel(self.channel());
                    return Ok(Some(m.into()));
                }
                Ok(None)
            }
            ButtonMessageType::ProgramChange => self.program_change(action, buffer),
            ButtonMessageType::ControlChange => {
                if let Action::Pressed = action {
                    let mut m = ControlChange::try_new_with_buffer(buffer)?;
                    m.set_channel(self.channel());
                    m.set_control(u7::new(self.midi_id));
                    m.set_control_data(u7::new(self.value));
                    return Ok(Some(m.into()));
                }
                Ok(None)
            }
            ButtonMessageType::ControlChangeWithReset => match status {
                ButtonStatus::On => {
                    let mut m = ControlChange::try_new_with_buffer(buffer)?;
                    m.set_channel(self.channel());
                    m.set_control(u7::new(self.midi_id));
                    m.set_control_data(u7::new(self.value));
                    Ok(Some(m.into()))
                }
                ButtonStatus::Off => {
                    let mut m = ControlChange::try_new_with_buffer(buffer)?;
                    m.set_channel(self.channel());
                    m.set_control(u7::new(self.midi_id));
                    m.set_control_data(u7::new(0));
                    Ok(Some(m.into()))
                }
                ButtonStatus::None => Ok(None),
            },
            ButtonMessageType::ControlChangeWithValue0 => {
                if let Action::Pressed = action {
                    let mut m = ControlChange::try_new_with_buffer(buffer)?;
                    m.set_channel(self.channel());
                    m.set_control(u7::new(self.midi_id));
                    m.set_control_data(u7::new(0x00));
                    return Ok(Some(m.into()));
                }
                Ok(None)
            }

            ButtonMessageType::MMCStop => {
                if let Action::Pressed = action {
                    let mut m = Sysex7::try_new_with_buffer(buffer)?;
                    let payload: [u8; 4] = [0x7F, self.midi_id, 0x06, 0x01];
                    m.try_set_payload(payload.into_iter().map(u7::new))?;

                    return Ok(Some(m.into()));
                }
                Ok(None)
            }
            ButtonMessageType::MMCPlay => {
                if let Action::Pressed = action {
                    let mut m = Sysex7::try_new_with_buffer(buffer)?;
                    let payload: [u8; 4] = [0x7F, self.midi_id, 0x06, 0x02];
                    m.try_set_payload(payload.into_iter().map(u7::new))?;

                    return Ok(Some(m.into()));
                }
                Ok(None)
            }
            ButtonMessageType::MMCRecord => {
                if let Action::Pressed = action {
                    let mut m = Sysex7::try_new_with_buffer(buffer)?;
                    let payload: [u8; 4] = [0x7F, self.midi_id, 0x06, 0x06];
                    m.try_set_payload(payload.into_iter().map(u7::new))?;

                    return Ok(Some(m.into()));
                }
                Ok(None)
            }
            ButtonMessageType::MMCPause => {
                if let Action::Pressed = action {
                    let mut m = Sysex7::try_new_with_buffer(buffer)?;
                    let payload: [u8; 4] = [0x7F, self.midi_id, 0x06, 0x09];
                    m.try_set_payload(payload.into_iter().map(u7::new))?;

                    return Ok(Some(m.into()));
                }
                Ok(None)
            }

            ButtonMessageType::RealTimeClock => {
                if let Action::Pressed = action {
                    let m = TimingClock::try_new_with_buffer(buffer)?;
                    return Ok(Some(m.into()));
                }
                Ok(None)
            }

            ButtonMessageType::RealTimeStart => {
                if let Action::Pressed = action {
                    let m = Start::try_new_with_buffer(buffer)?;
                    return Ok(Some(m.into()));
                }
                Ok(None)
            }
            ButtonMessageType::RealTimeContinue => {
                if let Action::Pressed = action {
                    let m = Continue::try_new_with_buffer(buffer)?;
                    return Ok(Some(m.into()));
                }
                Ok(None)
            }
            ButtonMessageType::RealTimeStop => {
                if let Action::Pressed = action {
                    let m = Stop::try_new_with_buffer(buffer)?;
                    return Ok(Some(m.into()));
                }
                Ok(None)
            }
            ButtonMessageType::RealTimeActiveSensing => {
                if let Action::Pressed = action {
                    let m = ActiveSensing::try_new_with_buffer(buffer)?;
                    return Ok(Some(m.into()));
                }
                Ok(None)
            }
            ButtonMessageType::RealTimeSystemReset => {
                if let Action::Pressed = action {
                    let m = Reset::try_new_with_buffer(buffer)?;
                    return Ok(Some(m.into()));
                }
                Ok(None)
            }

            ButtonMessageType::ProgramChangeIncr => {
                self.incr_midi_id(&action);
                self.program_change(action, buffer)
            }
            ButtonMessageType::ProgramChangeDecr => {
                self.decr_midi_id(&action);
                self.program_change(action, buffer)
            }
            ButtonMessageType::MultiValueIncResetNote => {
                if let Action::Pressed = action {
                    let mut m = NoteOn::try_new_with_buffer(buffer)?;
                    m.set_channel(self.channel());
                    m.set_note_number(u7::new(self.midi_id));
                    m.set_velocity(self.multi_value_inc_reset());
                    return Ok(Some(m.into()));
                }
                Ok(None)
            }
            ButtonMessageType::MultiValueIncDecNote => {
                if let Action::Pressed = action {
                    let mut m = NoteOn::try_new_with_buffer(buffer)?;
                    m.set_channel(self.channel());
                    m.set_note_number(u7::new(self.midi_id));
                    m.set_velocity(self.multi_value_inc_dec());
                    return Ok(Some(m.into()));
                }
                Ok(None)
            }
            ButtonMessageType::MultiValueIncResetCC => {
                if let Action::Pressed = action {
                    let mut m = ControlChange::try_new_with_buffer(buffer)?;
                    m.set_channel(self.channel());
                    m.set_control(u7::new(self.midi_id));
                    m.set_control_data(self.multi_value_inc_reset());
                    return Ok(Some(m.into()));
                }
                Ok(None)
            }
            ButtonMessageType::MultiValueIncDecCC => {
                if let Action::Pressed = action {
                    let mut m = ControlChange::try_new_with_buffer(buffer)?;
                    m.set_channel(self.channel());
                    m.set_control(u7::new(self.midi_id));
                    m.set_control_data(self.multi_value_inc_dec());
                    return Ok(Some(m.into()));
                }
                Ok(None)
            }

            ButtonMessageType::ProgramChangeOffsetIncr => Ok(None),
            ButtonMessageType::ProgramChangeOffsetDecr => Ok(None),
            ButtonMessageType::BPMIncr => Ok(None),
            ButtonMessageType::BPMDecr => Ok(None),
            ButtonMessageType::OpenDeckPresetChange => Ok(None),

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
                    self.state.latch_on = !self.state.latch_on;
                    if self.state.latch_on {
                        return ButtonStatus::On;
                    } else {
                        return ButtonStatus::Off;
                    }
                }
                ButtonStatus::None
            }
        }
    }
    fn incr_midi_id(&mut self, action: &Action) {
        if let Action::Pressed = action {
            if self.midi_id >= MAX_MIDI_ID {
                self.midi_id = 0
            } else {
                self.midi_id += 1
            }
        }
    }
    fn decr_midi_id(&mut self, action: &Action) {
        if let Action::Pressed = action {
            if self.midi_id == 0 {
                self.midi_id = MAX_MIDI_ID
            } else {
                self.midi_id -= 1
            }
        }
    }
    fn multi_value_inc_reset(&mut self) -> u7 {
        self.state.step += 1;
        let result = self.state.step * self.value;
        if result > MAX_VALUE {
            self.state.step = 1;
            return u7::new(self.value);
        }
        u7::new(result)
    }
    fn multi_value_inc_dec(&mut self) -> u7 {
        if self.state.step_down {
            if self.state.step <= 1 {
                self.state.step = 2;
                self.state.step_down = false;
                let result = self.state.step * self.value;
                return u7::new(result);
            }
            self.state.step -= 1;
            let result = self.state.step * self.value;
            u7::new(result)
        } else {
            // step up
            self.state.step += 1;
            let mut result = self.state.step * self.value;
            if result > MAX_VALUE {
                self.state.step_down = true;
                self.state.step -= 2;
                result = self.state.step * self.value;
            }
            u7::new(result)
        }
    }
    pub fn program_change<'a>(
        &mut self,
        action: Action,
        buffer: &'a mut [u8],
    ) -> Result<Option<BytesMessage<&'a mut [u8]>>, BufferOverflow> {
        if let Action::Pressed = action {
            let mut m = ProgramChange::try_new_with_buffer(buffer)?;
            m.set_channel(self.channel());
            m.set_program(u7::new(self.midi_id));
            return Ok(Some(m.into()));
        }
        Ok(None)
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
    use crate::{button::ButtonState, ChannelOrAll};

    #[test]
    fn test_note_on() {
        let mut message_buffer = [0x00u8; 8];
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::Notes,
            midi_id: 0x03,
            value: 0x7F,
            channel: ChannelOrAll::default(),
            state: ButtonState::default(),
        };
        let result = button
            .handle(Action::Pressed, &mut message_buffer)
            .unwrap()
            .unwrap();

        let buf = result.data();
        assert_eq!(buf, [0x90, 0x03, 0x7F]);
    }
    #[test]
    fn test_program_change() {
        let mut message_buffer = [0x00u8; 8];
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::ProgramChange,
            midi_id: 0x03,
            value: 0x7F,
            channel: ChannelOrAll::default(),
            state: ButtonState::default(),
        };
        let result = button
            .handle(Action::Pressed, &mut message_buffer)
            .unwrap()
            .unwrap();
        assert_eq!(result.data(), [0xC0, 0x03]);
    }
    #[test]
    fn test_program_change_release() {
        let mut message_buffer = [0x00u8; 8];
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::ProgramChange,
            midi_id: 0x03,
            value: 0x7F,
            channel: ChannelOrAll::default(),
            state: ButtonState::default(),
        };
        let result = button
            .handle(Action::Released, &mut message_buffer)
            .unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_control_change() {
        let mut message_buffer = [0x00u8; 8];
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::ControlChange,
            midi_id: 0x03,
            value: 0x7F,
            channel: ChannelOrAll::default(),
            state: ButtonState::default(),
        };
        let result = button
            .handle(Action::Pressed, &mut message_buffer)
            .unwrap()
            .unwrap();
        assert_eq!(result.data(), [0xB0, 0x03, 0x7F]);
    }
    #[test]
    fn test_control_change_release() {
        let mut message_buffer = [0x00u8; 8];
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::ControlChange,
            midi_id: 0x03,
            value: 0x7F,
            channel: ChannelOrAll::default(),
            state: ButtonState::default(),
        };
        let result = button
            .handle(Action::Released, &mut message_buffer)
            .unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_control_change_with_reset() {
        let mut message_buffer = [0x00u8; 8];
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::ControlChangeWithReset,
            midi_id: 0x03,
            value: 0x7F,
            channel: ChannelOrAll::default(),
            state: ButtonState::default(),
        };
        let result = button
            .handle(Action::Pressed, &mut message_buffer)
            .unwrap()
            .unwrap();
        assert_eq!(result.data(), [0xB0, 0x03, 0x7F]);
    }
    #[test]
    fn test_control_change_with_reset_release() {
        let mut message_buffer = [0x00u8; 8];
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::ControlChangeWithReset,
            midi_id: 0x03,
            value: 0x7F,
            channel: ChannelOrAll::default(),
            state: ButtonState::default(),
        };
        let result = button
            .handle(Action::Released, &mut message_buffer)
            .unwrap()
            .unwrap();
        assert_eq!(result.data(), [0xB0, 0x03, 0x00]);
    }

    #[test]
    fn test_control_change_with_value0() {
        let mut message_buffer = [0x00u8; 8];
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::ControlChangeWithValue0,
            midi_id: 0x03,
            value: 0x7F,
            channel: ChannelOrAll::default(),
            state: ButtonState::default(),
        };
        let result = button
            .handle(Action::Pressed, &mut message_buffer)
            .unwrap()
            .unwrap();
        assert_eq!(result.data(), [0xB0, 0x03, 0x00]);
    }

    #[test]
    fn test_no_message() {
        let mut message_buffer = [0x00u8; 8];
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::NoMessage,
            midi_id: 0x03,
            value: 0x7F,
            channel: ChannelOrAll::default(),
            state: ButtonState::default(),
        };
        let result = button
            .handle(Action::Released, &mut message_buffer)
            .unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_realtime_clock() {
        let mut message_buffer = [0x00u8; 8];
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::RealTimeClock,
            midi_id: 0x03,
            value: 0x7F,
            channel: ChannelOrAll::default(),
            state: ButtonState::default(),
        };
        let result = button
            .handle(Action::Pressed, &mut message_buffer)
            .unwrap()
            .unwrap();
        assert_eq!(result.data(), [0xF8]);
    }
    #[test]
    fn test_realtime_start() {
        let mut message_buffer = [0x00u8; 8];
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::RealTimeStart,
            midi_id: 0x03,
            value: 0x7F,
            channel: ChannelOrAll::default(),
            state: ButtonState::default(),
        };
        let result = button
            .handle(Action::Pressed, &mut message_buffer)
            .unwrap()
            .unwrap();
        assert_eq!(result.data(), [0xFA]);
    }
    #[test]
    fn test_realtime_stop() {
        let mut message_buffer = [0x00u8; 8];
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::RealTimeStop,
            midi_id: 0x03,
            value: 0x7F,
            channel: ChannelOrAll::default(),
            state: ButtonState::default(),
        };
        let result = button
            .handle(Action::Pressed, &mut message_buffer)
            .unwrap()
            .unwrap();
        assert_eq!(result.data(), [0xFC]);
    }
    #[test]
    fn test_realtime_continue() {
        let mut message_buffer = [0x00u8; 8];
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::RealTimeContinue,
            midi_id: 0x03,
            value: 0x7F,
            channel: ChannelOrAll::default(),
            state: ButtonState::default(),
        };
        let result = button
            .handle(Action::Pressed, &mut message_buffer)
            .unwrap()
            .unwrap();
        assert_eq!(result.data(), [0xFB]);
    }
    #[test]
    fn test_realtime_active_sensing() {
        let mut message_buffer = [0x00u8; 8];
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::RealTimeActiveSensing,
            midi_id: 0x03,
            value: 0x7F,
            channel: ChannelOrAll::default(),
            state: ButtonState::default(),
        };
        let result = button
            .handle(Action::Pressed, &mut message_buffer)
            .unwrap()
            .unwrap();
        assert_eq!(result.data(), [0xFE]);
    }
    #[test]
    fn test_realtime_reset() {
        let mut message_buffer = [0x00u8; 8];
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::RealTimeSystemReset,
            midi_id: 0x03,
            value: 0x7F,
            channel: ChannelOrAll::default(),
            state: ButtonState::default(),
        };
        let result = button
            .handle(Action::Pressed, &mut message_buffer)
            .unwrap()
            .unwrap();
        assert_eq!(result.data(), [0xFF]);
    }

    #[test]
    fn test_realtime_program_change_incr() {
        let mut message_buffer = [0x00u8; 8];
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::ProgramChangeIncr,
            midi_id: 0x7E,
            value: 0x7F,
            channel: ChannelOrAll::default(),
            state: ButtonState::default(),
        };
        let result_1 = button
            .handle(Action::Pressed, &mut message_buffer)
            .unwrap()
            .unwrap();
        assert_eq!(result_1.data(), [0xC0, 0x7F]);
        let result_1 = button
            .handle(Action::Pressed, &mut message_buffer)
            .unwrap()
            .unwrap();
        assert_eq!(result_1.data(), [0xC0, 0x00]);
    }
    #[test]
    fn test_realtime_program_change_decr() {
        let mut message_buffer = [0x00u8; 8];
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::ProgramChangeDecr,
            midi_id: 0x01,
            value: 0x7F,
            channel: ChannelOrAll::default(),
            state: ButtonState::default(),
        };
        let result_1 = button
            .handle(Action::Pressed, &mut message_buffer)
            .unwrap()
            .unwrap();
        assert_eq!(result_1.data(), [0xC0, 0x00]);
        let result_1 = button
            .handle(Action::Pressed, &mut message_buffer)
            .unwrap()
            .unwrap();
        assert_eq!(result_1.data(), [0xC0, 0x7F]);
    }
    #[test]
    fn test_note_off_only() {
        let mut message_buffer = [0x00u8; 8];
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::NoteOffOnly,
            midi_id: 0x03,
            value: 0x7F,
            channel: ChannelOrAll::default(),
            state: ButtonState::default(),
        };
        let result = button
            .handle(Action::Pressed, &mut message_buffer)
            .unwrap()
            .unwrap();
        assert_eq!(result.data(), [0x80, 0x03, 0x7F]);
    }
    #[test]
    fn test_multi_value_inc_reset_note() {
        let mut message_buffer = [0x00u8; 8];
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::MultiValueIncResetNote,
            midi_id: 0x03,
            value: 50,
            channel: ChannelOrAll::default(),
            state: ButtonState::default(),
        };
        let result_1 = button
            .handle(Action::Pressed, &mut message_buffer)
            .unwrap()
            .unwrap();
        assert_eq!(result_1.data(), [0x90, 0x03, 50]);
        let result_2 = button
            .handle(Action::Pressed, &mut message_buffer)
            .unwrap()
            .unwrap();
        assert_eq!(result_2.data(), [0x90, 0x03, 100]);
        let result_3 = button
            .handle(Action::Pressed, &mut message_buffer)
            .unwrap()
            .unwrap();
        assert_eq!(result_3.data(), [0x90, 0x03, 50]);
        let result_4 = button
            .handle(Action::Pressed, &mut message_buffer)
            .unwrap()
            .unwrap();
        assert_eq!(result_4.data(), [0x90, 0x03, 100]);
        let result_5 = button
            .handle(Action::Pressed, &mut message_buffer)
            .unwrap()
            .unwrap();
        assert_eq!(result_5.data(), [0x90, 0x03, 50]);
    }
    #[test]
    fn test_multi_value_inc_reset_cc() {
        let mut message_buffer = [0x00u8; 8];
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::MultiValueIncResetCC,
            midi_id: 0x03,
            value: 40,
            channel: ChannelOrAll::default(),
            state: ButtonState::default(),
        };
        let result_1 = button
            .handle(Action::Pressed, &mut message_buffer)
            .unwrap()
            .unwrap();
        assert_eq!(result_1.data(), [0xB0, 0x03, 40]);
        let result_2 = button
            .handle(Action::Pressed, &mut message_buffer)
            .unwrap()
            .unwrap();
        assert_eq!(result_2.data(), [0xB0, 0x03, 80]);
        let result_3 = button
            .handle(Action::Pressed, &mut message_buffer)
            .unwrap()
            .unwrap();
        assert_eq!(result_3.data(), [0xB0, 0x03, 120]);
        let result_4 = button
            .handle(Action::Pressed, &mut message_buffer)
            .unwrap()
            .unwrap();
        assert_eq!(result_4.data(), [0xB0, 0x03, 40]);
        let result_5 = button
            .handle(Action::Pressed, &mut message_buffer)
            .unwrap()
            .unwrap();
        assert_eq!(result_5.data(), [0xB0, 0x03, 80]);
    }
    #[test]
    fn test_multi_value_inc_dec_note() {
        let mut message_buffer = [0x00u8; 8];
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::MultiValueIncDecNote,
            midi_id: 0x03,
            value: 50,
            channel: ChannelOrAll::default(),
            state: ButtonState::default(),
        };
        let result_1 = button
            .handle(Action::Pressed, &mut message_buffer)
            .unwrap()
            .unwrap();
        assert_eq!(result_1.data(), [0x90, 0x03, 50]);
        let result_2 = button
            .handle(Action::Pressed, &mut message_buffer)
            .unwrap()
            .unwrap();
        assert_eq!(result_2.data(), [0x90, 0x03, 100]);
        let result_3 = button
            .handle(Action::Pressed, &mut message_buffer)
            .unwrap()
            .unwrap();
        assert_eq!(result_3.data(), [0x90, 0x03, 50]);
        let result_4 = button
            .handle(Action::Pressed, &mut message_buffer)
            .unwrap()
            .unwrap();
        assert_eq!(result_4.data(), [0x90, 0x03, 100]);
        let result_5 = button
            .handle(Action::Pressed, &mut message_buffer)
            .unwrap()
            .unwrap();
        assert_eq!(result_5.data(), [0x90, 0x03, 50]);
    }
    #[test]
    fn test_multi_value_inc_dec_cc() {
        let mut message_buffer = [0x00u8; 8];
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::MultiValueIncDecCC,
            midi_id: 0x03,
            value: 40,
            channel: ChannelOrAll::default(),
            state: ButtonState::default(),
        };
        let result_1 = button
            .handle(Action::Pressed, &mut message_buffer)
            .unwrap()
            .unwrap();
        assert_eq!(result_1.data(), [0xB0, 0x03, 40]);
        let result_2 = button
            .handle(Action::Pressed, &mut message_buffer)
            .unwrap()
            .unwrap();
        assert_eq!(result_2.data(), [0xB0, 0x03, 80]);
        let result_3 = button
            .handle(Action::Pressed, &mut message_buffer)
            .unwrap()
            .unwrap();
        assert_eq!(result_3.data(), [0xB0, 0x03, 120]);
        let result_4 = button
            .handle(Action::Pressed, &mut message_buffer)
            .unwrap()
            .unwrap();
        assert_eq!(result_4.data(), [0xB0, 0x03, 80]);
        let result_5 = button
            .handle(Action::Pressed, &mut message_buffer)
            .unwrap()
            .unwrap();
        assert_eq!(result_5.data(), [0xB0, 0x03, 40]);
        let result_6 = button
            .handle(Action::Pressed, &mut message_buffer)
            .unwrap()
            .unwrap();
        assert_eq!(result_6.data(), [0xB0, 0x03, 80]);
    }
    #[test]
    fn test_mmc_play() {
        let mut message_buffer = [0x00u8; 8];
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::MMCPlay,
            midi_id: 0x03,
            value: 0x7F,
            channel: ChannelOrAll::default(),
            state: ButtonState::default(),
        };
        let result = button
            .handle(Action::Pressed, &mut message_buffer)
            .unwrap()
            .unwrap();
        assert_eq!(result.data(), [0xF0, 0x7F, 0x03, 0x06, 0x02, 0xF7]);
    }
    #[test]
    fn test_mmc_stop() {
        let mut message_buffer = [0x00u8; 8];
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::MMCStop,
            midi_id: 0x04,
            value: 0x7F,
            channel: ChannelOrAll::default(),
            state: ButtonState::default(),
        };
        let result = button
            .handle(Action::Pressed, &mut message_buffer)
            .unwrap()
            .unwrap();
        assert_eq!(result.data(), [0xF0, 0x7F, 0x04, 0x06, 0x01, 0xF7]);
    }
    #[test]
    fn test_mmc_record() {
        let mut message_buffer = [0x00u8; 8];
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::MMCRecord,
            midi_id: 0x05,
            value: 0x7F,
            channel: ChannelOrAll::default(),
            state: ButtonState::default(),
        };
        let result = button
            .handle(Action::Pressed, &mut message_buffer)
            .unwrap()
            .unwrap();
        assert_eq!(result.data(), [0xF0, 0x7F, 0x05, 0x06, 0x06, 0xF7]);
    }
    #[test]
    fn test_mmc_pause() {
        let mut message_buffer = [0x00u8; 8];
        let mut button = Button {
            button_type: ButtonType::Momentary,
            message_type: ButtonMessageType::MMCPause,
            midi_id: 0x05,
            value: 0x7F,
            channel: ChannelOrAll::default(),
            state: ButtonState::default(),
        };
        let result = button
            .handle(Action::Pressed, &mut message_buffer)
            .unwrap()
            .unwrap();
        assert_eq!(result.data(), [0xF0, 0x7F, 0x05, 0x06, 0x09, 0xF7]);
    }
}
