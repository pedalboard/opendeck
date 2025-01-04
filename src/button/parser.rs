use crate::{
    button::{ButtonMessageType, ButtonSection, ButtonSectionId, ButtonType},
    parser::OpenDeckParseError,
    ChannelOrAll, MessageStatus, Section,
};

impl TryFrom<Section> for ButtonSection {
    type Error = OpenDeckParseError;
    fn try_from(v: Section) -> Result<Self, Self::Error> {
        if let Ok(id) = ButtonSectionId::try_from(v.id) {
            match id {
                ButtonSectionId::MidiId => Ok(ButtonSection::MidiId(v.value as u8)),
                ButtonSectionId::MessageType => ButtonMessageType::try_from(v.value)
                    .map(ButtonSection::MessageType)
                    .map_err(OpenDeckParseError::new_value_err),
                ButtonSectionId::Type => ButtonType::try_from(v.value)
                    .map(ButtonSection::Type)
                    .map_err(OpenDeckParseError::new_value_err),
                ButtonSectionId::Value => Ok(ButtonSection::Value(v.value as u8)),
                ButtonSectionId::Channel => Ok(ButtonSection::Channel(ChannelOrAll::from(v.value))),
            }
        } else {
            Err(OpenDeckParseError::StatusError(MessageStatus::SectionError))
        }
    }
}
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_midi_id() {
        let result = ButtonSection::try_from(Section {
            id: 0x02,
            value: 0x01,
        });
        assert_eq!(result, Ok(ButtonSection::MidiId(0x01)));
    }

    #[test]
    fn test_message_type_value_error() {
        let result = ButtonSection::try_from(Section {
            id: 0x01,
            value: 0x20,
        });
        assert_eq!(
            result,
            Err(OpenDeckParseError::StatusError(
                MessageStatus::NewValueError
            ))
        );
    }
}
