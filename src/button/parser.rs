use crate::{
    button::{ButtonMessageType, ButtonSection, ButtonSectionId, ButtonType},
    parser::OpenDeckParseError,
    ChannelOrAll, MessageStatus, Section,
};
use midi_types::Value7;

impl TryFrom<Section> for ButtonSection {
    type Error = OpenDeckParseError;
    fn try_from(v: Section) -> Result<Self, Self::Error> {
        if let Ok(id) = ButtonSectionId::try_from(v.id) {
            match id {
                ButtonSectionId::MidiId => Ok(ButtonSection::MidiId(Value7::from(v.value as u8))),
                ButtonSectionId::MessageType => {
                    if let Ok(mt) = ButtonMessageType::try_from(v.value) {
                        Ok(ButtonSection::MessageType(mt))
                    } else {
                        Err(OpenDeckParseError::StatusError(
                            MessageStatus::NewValueError,
                        ))
                    }
                }
                ButtonSectionId::Type => {
                    if let Ok(bt) = ButtonType::try_from(v.value) {
                        Ok(ButtonSection::Type(bt))
                    } else {
                        Err(OpenDeckParseError::StatusError(
                            MessageStatus::NewValueError,
                        ))
                    }
                }
                ButtonSectionId::Value => Ok(ButtonSection::Value(Value7::from(v.value as u8))),
                ButtonSectionId::Channel => Ok(ButtonSection::Channel(ChannelOrAll::from(v.value))),
            }
        } else {
            Err(OpenDeckParseError::StatusError(MessageStatus::SectionError))
        }
    }
}
