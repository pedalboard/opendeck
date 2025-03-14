use crate::{
    analog::{AnalogMessageType, AnalogSection, AnalogSectionId},
    parser::OpenDeckParseError,
    ChannelOrAll, MessageStatus, Section,
};

impl TryFrom<Section> for AnalogSection {
    type Error = OpenDeckParseError;
    fn try_from(v: Section) -> Result<Self, Self::Error> {
        if let Ok(id) = AnalogSectionId::try_from(v.id) {
            match id {
                AnalogSectionId::Enabled => Ok(AnalogSection::Enabled(v.value > 0)),
                AnalogSectionId::InvertState => Ok(AnalogSection::Inverted(v.value > 0)),
                AnalogSectionId::MessageType => AnalogMessageType::try_from(v.value)
                    .map(AnalogSection::MessageType)
                    .map_err(OpenDeckParseError::new_value_err),
                AnalogSectionId::MidiIdLSB => Ok(AnalogSection::MidiId(v.value)),
                AnalogSectionId::LowerCCLimitLSB => Ok(AnalogSection::LowerCCLimit(v.value)),
                AnalogSectionId::UpperCCLimitLSB => Ok(AnalogSection::UpperCCLimit(v.value)),
                AnalogSectionId::Channel => Ok(AnalogSection::Channel(ChannelOrAll::from(v.value))),
                AnalogSectionId::LowerADCOffset => Ok(AnalogSection::LowerADCOffset(v.value as u8)),
                AnalogSectionId::UpperADCOffset => Ok(AnalogSection::UpperADCOffset(v.value as u8)),
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
        let value = 1;
        let section = Section { id: 0x03, value };
        let result = AnalogSection::try_from(section);
        assert_eq!(result, Ok(AnalogSection::MidiId(value)));
    }

    #[test]
    fn test_message_type_value_error() {
        let result = AnalogSection::try_from(Section {
            id: 0x02,
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
