use crate::{parser::OpenDeckParseError, MessageStatus, Section};

pub enum GlobalSectionId {
    Midi,
    Reserved,
    Presets,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PresetIndex {
    Active,
    Preservation,
    ForceValueRefresh,
    EnableMideChange,
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
                let pi = PresetIndex::try_from(x.0)?;
                Ok(GlobalSection::Presets(pi, x.1.value))
            }
            _ => Err(OpenDeckParseError::StatusError(MessageStatus::SectionError)),
        }
    }
}

impl TryFrom<u16> for PresetIndex {
    type Error = OpenDeckParseError;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            // FIXME support more preset values
            x if x == PresetIndex::Active as u16 => Ok(PresetIndex::Active),
            _ => Err(OpenDeckParseError::StatusError(MessageStatus::IndexError)),
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
                index as u16,
                Section {
                    id: GlobalSectionId::Presets as u8,
                    value,
                },
            ),
        }
    }
}
