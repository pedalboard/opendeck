use crate::{parser::OpenDeckParseError, ChannelOrAll, MessageStatus, Section};
use midi_types::Value7;

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
