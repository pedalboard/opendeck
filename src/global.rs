use crate::{parser::OpenDeckParseError, MessageStatus, Section};
use int_enum::IntEnum;

pub enum GlobalSectionId {
    Midi,
    Reserved,
    Presets,
}

#[derive(Debug, Clone, PartialEq, IntEnum, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u16)]
pub enum PresetIndex {
    Active = 0,
    Preservation = 1,
    ForceValueRefresh = 2,
    EnableMideChange = 3,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum GlobalSection {
    Midi(u16, u16),
    Presets(PresetIndex, u16),
}

impl TryFrom<(u16, Section)> for GlobalSection {
    type Error = OpenDeckParseError;
    fn try_from(value: (u16, Section)) -> Result<Self, Self::Error> {
        match value {
            x if x.1.id == GlobalSectionId::Midi as u8 => Ok(GlobalSection::Midi(x.0, x.1.value)),
            x if x.1.id == GlobalSectionId::Presets as u8 => {
                if let Ok(pi) = PresetIndex::try_from(x.0) {
                    Ok(GlobalSection::Presets(pi, x.1.value))
                } else {
                    Err(OpenDeckParseError::StatusError(MessageStatus::IndexError))
                }
            }
            _ => Err(OpenDeckParseError::StatusError(MessageStatus::SectionError)),
        }
    }
}

//render

impl From<GlobalSection> for (u16, Section) {
    fn from(section: GlobalSection) -> (u16, Section) {
        match section {
            GlobalSection::Midi(index, value) => (
                index,
                Section {
                    id: GlobalSectionId::Midi as u8,
                    value,
                },
            ),
            GlobalSection::Presets(index, value) => (
                index.into(),
                Section {
                    id: GlobalSectionId::Presets as u8,
                    value,
                },
            ),
        }
    }
}
