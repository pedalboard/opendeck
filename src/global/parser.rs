use crate::{
    global::{GlobalSection, GlobalSectionId, PresetIndex},
    parser::OpenDeckParseError,
    MessageStatus, Section,
};

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
