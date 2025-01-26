use crate::ChannelOrAll;
use int_enum::IntEnum;

pub mod handler;
pub mod parser;
pub mod renderer;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Button {
    value: u8,
    midi_id: u8,
    message_type: ButtonMessageType,
    channel: ChannelOrAll,
    button_type: ButtonType,
    state: ButtonState,
}
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
struct ButtonState {
    // state when letching mode is on
    latch_on: bool,
    // the step for multi value buttons
    step: u8,
}

impl Button {
    pub fn new(midi_id: u8) -> Self {
        Button {
            button_type: ButtonType::default(),
            value: 0x01,
            midi_id,
            message_type: ButtonMessageType::default(),
            channel: ChannelOrAll::default(),
            state: ButtonState::default(),
        }
    }
    pub fn set(&mut self, section: &ButtonSection) {
        match section {
            ButtonSection::Type(t) => self.button_type = *t,
            ButtonSection::Value(v) => self.value = *v,
            ButtonSection::MidiId(id) => self.midi_id = *id,
            ButtonSection::MessageType(t) => self.message_type = *t,
            ButtonSection::Channel(c) => self.channel = *c,
        }
    }
    pub fn get(&self, section: &ButtonSection) -> u16 {
        match section {
            ButtonSection::Type(_) => self.button_type as u16,
            ButtonSection::MessageType(_) => self.message_type as u16,
            ButtonSection::Value(_) => self.value.into(),
            ButtonSection::MidiId(_) => self.midi_id.into(),
            ButtonSection::Channel(_) => self.channel.into(),
        }
    }
}

impl Default for Button {
    fn default() -> Self {
        Button::new(0)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntEnum, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u16)]
pub enum ButtonType {
    #[default]
    Momentary = 0,
    Latching = 1,
}

#[derive(IntEnum)]
#[repr(u8)]
enum ButtonSectionId {
    Type = 0,
    MessageType = 1,
    MidiId = 2,
    Value = 3,
    Channel = 4,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ButtonSection {
    Type(ButtonType),
    MessageType(ButtonMessageType),
    MidiId(u8),
    Value(u8),
    Channel(ChannelOrAll),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntEnum, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u16)]
pub enum ButtonMessageType {
    #[default]
    Notes = 0x00,
    ProgramChange = 0x01,
    ControlChange = 0x02,
    ControlChangeWithReset = 0x03,
    MMCStop = 0x04,
    MMCPlay = 0x05,
    MMCRecord = 0x06,
    MMCPause = 0x07,
    RealTimeClock = 0x08,
    RealTimeStart = 0x09,
    RealTimeContinue = 0x0A,
    RealTimeStop = 0x0B,
    RealTimeActiveSensing = 0x0C,
    RealTimeSystemReset = 0x0D,
    ProgramChangeIncr = 0x0E,
    ProgramChangeDecr = 0x0F,
    NoMessage = 0x10,
    OpenDeckPresetChange = 0x11,
    MultiValueIncResetNote = 0x12,
    MultiValueIncDecNote = 0x13,
    MultiValueIncResetCC = 0x14,
    MultiValueIncDecCC = 0x15,
    NoteOffOnly = 0x16,
    ControlChangeWithValue0 = 0x17,
    Reserved = 0x18,
    ProgramChangeOffsetIncr = 0x19,
    ProgramChangeOffsetDecr = 0x1A,
    BPMIncr = 0x1B,
    BPMDecr = 0x1C,
}
