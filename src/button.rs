use crate::{parser::OpenDeckParseError, ChannelOrAll, MessageStatus, Section};
use midi_types::Value7;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Button {
    button_type: ButtonType,
    value: Value7,
    midi_id: Value7,
    message_type: MessageType,
    channel: ChannelOrAll,
}

impl Button {
    pub fn new(midi_id: Value7) -> Self {
        Button {
            button_type: ButtonType::default(),
            value: Value7::new(0x01),
            midi_id,
            message_type: MessageType::default(),
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ButtonType {
    #[default]
    Momentary,
    Latching,
}

enum ButtonSectionId {
    Type,
    MessageType,
    MidiId,
    Value,
    Channel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ButtonSection {
    Type(ButtonType),
    MessageType(MessageType),
    MidiId(Value7),
    Value(Value7),
    Channel(ChannelOrAll),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MessageType {
    #[default]
    Notes,
    ProgramChange,
    ControlChange,
    ControlChangeWithReset,
    MMCStop,
    MMCPlay,
    MMCRecord,
    MMCPause,
    RealTimeClock,
    RealTimeStart,
    RealTimeContinue,
    RealTimeStop,
    RealTimeActiveSensing,
    RealTimeSystemReset,
    ProgramChangeIncr,
    ProgramChangeDecr,
    NoMessage,
    OpenDeckPresetChange,
    MultiValueIncNote,
    MultiValueDecNote,
    MultiValueIncCC,
    MultiValueDecCC,
    NoteOffOnly,
    ControlChangeWithValue0,
    Reserved,
    ProgramChangeOffsetIncr,
    ProgramChangeOffsetDecr,
    BPMIncr,
    BPMDecr,
}

// parsing

impl TryFrom<u16> for MessageType {
    type Error = OpenDeckParseError;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            x if x == MessageType::Notes as u16 => Ok(MessageType::Notes),
            x if x == MessageType::ProgramChange as u16 => Ok(MessageType::ProgramChange),
            x if x == MessageType::ControlChange as u16 => Ok(MessageType::ControlChange),
            x if x == MessageType::ControlChangeWithReset as u16 => {
                Ok(MessageType::ControlChangeWithReset)
            }
            x if x == MessageType::MMCStop as u16 => Ok(MessageType::MMCStop),
            x if x == MessageType::MMCPlay as u16 => Ok(MessageType::MMCPlay),
            x if x == MessageType::MMCRecord as u16 => Ok(MessageType::MMCRecord),
            x if x == MessageType::MMCPause as u16 => Ok(MessageType::MMCPause),
            x if x == MessageType::RealTimeClock as u16 => Ok(MessageType::RealTimeClock),
            x if x == MessageType::RealTimeStart as u16 => Ok(MessageType::RealTimeStart),
            x if x == MessageType::RealTimeContinue as u16 => Ok(MessageType::RealTimeContinue),
            x if x == MessageType::RealTimeStop as u16 => Ok(MessageType::RealTimeStop),
            x if x == MessageType::RealTimeActiveSensing as u16 => {
                Ok(MessageType::RealTimeActiveSensing)
            }
            x if x == MessageType::RealTimeSystemReset as u16 => {
                Ok(MessageType::RealTimeSystemReset)
            }
            x if x == MessageType::ProgramChangeDecr as u16 => Ok(MessageType::ProgramChangeDecr),
            x if x == MessageType::ProgramChangeIncr as u16 => Ok(MessageType::ProgramChangeIncr),
            x if x == MessageType::NoMessage as u16 => Ok(MessageType::NoMessage),
            x if x == MessageType::OpenDeckPresetChange as u16 => {
                Ok(MessageType::OpenDeckPresetChange)
            }
            x if x == MessageType::MultiValueIncNote as u16 => Ok(MessageType::MultiValueIncNote),
            x if x == MessageType::MultiValueDecNote as u16 => Ok(MessageType::MultiValueDecNote),
            x if x == MessageType::MultiValueIncCC as u16 => Ok(MessageType::MultiValueIncCC),
            x if x == MessageType::MultiValueDecCC as u16 => Ok(MessageType::MultiValueDecCC),
            x if x == MessageType::NoteOffOnly as u16 => Ok(MessageType::NoteOffOnly),
            x if x == MessageType::ControlChangeWithValue0 as u16 => {
                Ok(MessageType::ControlChangeWithValue0)
            }
            x if x == MessageType::ProgramChangeOffsetIncr as u16 => {
                Ok(MessageType::ProgramChangeOffsetIncr)
            }
            x if x == MessageType::ProgramChangeOffsetDecr as u16 => {
                Ok(MessageType::ProgramChangeOffsetDecr)
            }
            x if x == MessageType::BPMIncr as u16 => Ok(MessageType::BPMIncr),
            x if x == MessageType::BPMDecr as u16 => Ok(MessageType::BPMDecr),
            _ => Err(OpenDeckParseError::StatusError(
                MessageStatus::NewValueError,
            )),
        }
    }
}

impl TryFrom<u16> for ButtonType {
    type Error = OpenDeckParseError;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            x if x == ButtonType::Momentary as u16 => Ok(ButtonType::Momentary),
            x if x == ButtonType::Latching as u16 => Ok(ButtonType::Latching),
            _ => Err(OpenDeckParseError::StatusError(
                MessageStatus::NewValueError,
            )),
        }
    }
}

impl TryFrom<Section> for ButtonSection {
    type Error = OpenDeckParseError;
    fn try_from(value: Section) -> Result<Self, Self::Error> {
        match value {
            x if x.id == ButtonSectionId::MidiId as u8 => {
                Ok(ButtonSection::MidiId(Value7::from(x.value as u8)))
            }
            x if x.id == ButtonSectionId::MessageType as u8 => {
                let mt = MessageType::try_from(x.value)?;
                Ok(ButtonSection::MessageType(mt))
            }
            x if x.id == ButtonSectionId::Type as u8 => {
                let mt = ButtonType::try_from(x.value)?;
                Ok(ButtonSection::Type(mt))
            }
            x if x.id == ButtonSectionId::Value as u8 => {
                Ok(ButtonSection::Value(Value7::from(x.value as u8)))
            }
            x if x.id == ButtonSectionId::Channel as u8 => {
                Ok(ButtonSection::Channel(ChannelOrAll::from(x.value)))
            }
            _ => Err(OpenDeckParseError::StatusError(MessageStatus::SectionError)),
        }
    }
}

// rendering

impl From<ButtonSection> for Section {
    fn from(s: ButtonSection) -> Section {
        match s {
            ButtonSection::Type(t) => Section {
                id: ButtonSectionId::Type as u8,
                value: t as u16,
            },
            ButtonSection::MessageType(t) => Section {
                id: ButtonSectionId::MessageType as u8,
                value: t as u16,
            },
            ButtonSection::MidiId(v) => {
                let value: u8 = v.into();
                Section {
                    id: ButtonSectionId::MidiId as u8,
                    value: value as u16,
                }
            }
            ButtonSection::Value(v) => {
                let value: u8 = v.into();
                Section {
                    id: ButtonSectionId::Value as u8,
                    value: value as u16,
                }
            }
            ButtonSection::Channel(v) => Section {
                id: ButtonSectionId::Channel as u8,
                value: v.into(),
            },
        }
    }
}
