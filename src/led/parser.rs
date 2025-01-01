use crate::{
    led::{Color, LedSection, LedSectionId},
    parser::OpenDeckParseError,
    ChannelOrAll, MessageStatus, Section,
};
use midi_types::Value7;

use super::ControlType;

impl TryFrom<Section> for LedSection {
    type Error = OpenDeckParseError;
    fn try_from(v: Section) -> Result<Self, Self::Error> {
        if let Ok(id) = LedSectionId::try_from(v.id) {
            match id {
                LedSectionId::ActivationId => {
                    Ok(LedSection::ActivationId(Value7::from(v.value as u8)))
                }
                LedSectionId::ColorTesting => {
                    if let Ok(c) = Color::try_from(v.value) {
                        Ok(LedSection::ColorTesting(c))
                    } else {
                        Err(OpenDeckParseError::StatusError(
                            MessageStatus::NewValueError,
                        ))
                    }
                }
                LedSectionId::ControlType => {
                    if let Ok(c) = ControlType::try_from(v.value) {
                        Ok(LedSection::ControlType(c))
                    } else {
                        Err(OpenDeckParseError::StatusError(
                            MessageStatus::NewValueError,
                        ))
                    }
                }

                LedSectionId::RGBEnabled => Ok(LedSection::RGBEnabled(v.value > 0)),
                LedSectionId::BlinkTesting => Ok(LedSection::RGBEnabled(v.value > 0)),
                LedSectionId::Channel => Ok(LedSection::Channel(ChannelOrAll::from(v.value))),
                LedSectionId::Global => Ok(LedSection::Global(v.value)),
            }
        } else {
            Err(OpenDeckParseError::StatusError(MessageStatus::SectionError))
        }
    }
}
