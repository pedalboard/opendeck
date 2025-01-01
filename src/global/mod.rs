use crate::{parser::OpenDeckParseError, MessageStatus, Section};
use int_enum::IntEnum;

#[derive(IntEnum)]
#[repr(u8)]
pub enum GlobalSectionId {
    Midi = 0,
    // Reserved = 1,
    Presets = 2,
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

// parse

impl TryFrom<(u16, Section)> for GlobalSection {
    type Error = OpenDeckParseError;
    fn try_from(v: (u16, Section)) -> Result<Self, Self::Error> {
        if let Ok(id) = GlobalSectionId::try_from(v.1.id) {
            match id {
                GlobalSectionId::Midi => Ok(GlobalSection::Midi(v.0, v.1.value)),
                GlobalSectionId::Presets => {
                    if let Ok(pi) = PresetIndex::try_from(v.0) {
                        Ok(GlobalSection::Presets(pi, v.1.value))
                    } else {
                        Err(OpenDeckParseError::StatusError(MessageStatus::IndexError))
                    }
                }
            }
        } else {
            Err(OpenDeckParseError::StatusError(MessageStatus::SectionError))
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
                    id: GlobalSectionId::Midi.into(),
                    value,
                },
            ),
            GlobalSection::Presets(index, value) => (
                index.into(),
                Section {
                    id: GlobalSectionId::Presets.into(),
                    value,
                },
            ),
        }
    }
}
