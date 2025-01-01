use crate::ChannelOrAll;
use int_enum::IntEnum;
use midi_types::Value7;

pub mod parser;
pub mod renderer;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Button {
    button_type: ButtonType,
    value: Value7,
    midi_id: Value7,
    message_type: ButtonMessageType,
    channel: ChannelOrAll,
}

impl Button {
    pub fn new(midi_id: Value7) -> Self {
        Button {
            button_type: ButtonType::default(),
            value: Value7::new(0x01),
            midi_id,
            message_type: ButtonMessageType::default(),
            channel: ChannelOrAll::default(),
        }
    }
    pub fn set(&mut self, section: &ButtonSection) {
        match section {
            ButtonSection::Type(t) => self.button_type = *t,
            ButtonSection::Value(v) => self.value = *v,
            ButtonSection::MidiId(id) => self.midi_id = *id,
            ButtonSection::MessageType(t) => self.message_type = *t,
            ButtonSection::Channel(c) => self.channel = c.clone(),
        }
    }
    pub fn get(&self, section: &ButtonSection) -> u16 {
        match section {
            ButtonSection::Type(_) => self.button_type as u16,
            ButtonSection::MessageType(_) => self.message_type as u16,
            ButtonSection::Value(_) => {
                let v: u8 = self.value.into();
                v as u16
            }
            ButtonSection::MidiId(_) => {
                let v: u8 = self.midi_id.into();
                v as u16
            }
            ButtonSection::Channel(_) => self.channel.clone().into(),
        }
    }
}

impl Default for Button {
    fn default() -> Self {
        Button::new(Value7::new(0x00))
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
    MidiId(Value7),
    Value(Value7),
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
    MultiValueIncNote = 0x12,
    MultiValueDecNote = 0x13,
    MultiValueIncCC = 0x14,
    MultiValueDecCC = 0x15,
    NoteOffOnly = 0x16,
    ControlChangeWithValue0 = 0x17,
    Reserved = 0x18,
    ProgramChangeOffsetIncr = 0x19,
    ProgramChangeOffsetDecr = 0x1A,
    BPMIncr = 0x1B,
    BPMDecr = 0x1C,
}
