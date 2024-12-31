use crate::{parser::OpenDeckParseError, MessageStatus, Section};

enum AnalogSectionId {
    Enabled,
    InvertState,
    MessageType,
    MidiIdLSB,
    MidiIdMSB,
    LowerCCLimitLSB,
    LowerCCLimitMSB,
    UpperCCLimitLSB,
    UpperCCLimitMSB,
    Channel,
    LowerADCOffset,
    UpperADCOffset,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AnalogSection {
    Enabled(u16),
    InvertState(u16),
    MessageType(u16),
    MidiIdLSB(u16),
    MidiIdMSB(u16),
    LowerCCLimitLSB(u16),
    LowerCCLimitMSB(u16),
    UpperCCLimitLSB(u16),
    UpperCCLimitMSB(u16),
    Channel(u16),
    LowerADCOffset(u16),
    UpperADCOffset(u16),
}

impl TryFrom<Section> for AnalogSection {
    type Error = OpenDeckParseError;
    fn try_from(value: Section) -> Result<Self, Self::Error> {
        match value {
            x if x.id == AnalogSectionId::Enabled as u8 => Ok(AnalogSection::Enabled(x.value)),
            x if x.id == AnalogSectionId::InvertState as u8 => {
                Ok(AnalogSection::InvertState(x.value))
            }
            x if x.id == AnalogSectionId::MessageType as u8 => {
                Ok(AnalogSection::MessageType(x.value))
            }
            x if x.id == AnalogSectionId::MidiIdLSB as u8 => Ok(AnalogSection::MidiIdLSB(x.value)),
            x if x.id == AnalogSectionId::MidiIdMSB as u8 => Ok(AnalogSection::MidiIdMSB(x.value)),
            x if x.id == AnalogSectionId::LowerCCLimitLSB as u8 => {
                Ok(AnalogSection::LowerCCLimitLSB(x.value))
            }
            x if x.id == AnalogSectionId::LowerCCLimitMSB as u8 => {
                Ok(AnalogSection::LowerCCLimitMSB(x.value))
            }
            x if x.id == AnalogSectionId::UpperCCLimitLSB as u8 => {
                Ok(AnalogSection::UpperCCLimitLSB(x.value))
            }
            x if x.id == AnalogSectionId::UpperCCLimitMSB as u8 => {
                Ok(AnalogSection::UpperCCLimitMSB(x.value))
            }
            x if x.id == AnalogSectionId::Channel as u8 => Ok(AnalogSection::Channel(x.value)),
            x if x.id == AnalogSectionId::LowerADCOffset as u8 => {
                Ok(AnalogSection::LowerADCOffset(x.value))
            }
            x if x.id == AnalogSectionId::UpperADCOffset as u8 => {
                Ok(AnalogSection::UpperADCOffset(x.value))
            }
            _ => Err(OpenDeckParseError::StatusError(MessageStatus::SectionError)),
        }
    }
}

// render

impl From<AnalogSection> for Section {
    fn from(s: AnalogSection) -> Section {
        match s {
            AnalogSection::Enabled(value) => Section {
                id: AnalogSectionId::Enabled as u8,
                value,
            },
            AnalogSection::InvertState(value) => Section {
                id: AnalogSectionId::InvertState as u8,
                value,
            },
            AnalogSection::MessageType(value) => Section {
                id: AnalogSectionId::MessageType as u8,
                value,
            },
            AnalogSection::MidiIdLSB(value) => Section {
                id: AnalogSectionId::MidiIdLSB as u8,
                value,
            },
            AnalogSection::MidiIdMSB(value) => Section {
                id: AnalogSectionId::MidiIdMSB as u8,
                value,
            },
            AnalogSection::LowerCCLimitLSB(value) => Section {
                id: AnalogSectionId::LowerCCLimitLSB as u8,
                value,
            },
            AnalogSection::LowerCCLimitMSB(value) => Section {
                id: AnalogSectionId::LowerCCLimitMSB as u8,
                value,
            },
            AnalogSection::UpperCCLimitLSB(value) => Section {
                id: AnalogSectionId::UpperCCLimitLSB as u8,
                value,
            },
            AnalogSection::UpperCCLimitMSB(value) => Section {
                id: AnalogSectionId::UpperCCLimitMSB as u8,
                value,
            },
            AnalogSection::Channel(value) => Section {
                id: AnalogSectionId::Channel as u8,
                value,
            },
            AnalogSection::LowerADCOffset(value) => Section {
                id: AnalogSectionId::LowerADCOffset as u8,
                value,
            },
            AnalogSection::UpperADCOffset(value) => Section {
                id: AnalogSectionId::UpperADCOffset as u8,
                value,
            },
        }
    }
}
