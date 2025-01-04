use crate::{
    encoder::{Accelleration, EncoderMessageType, EncoderSection, EncoderSectionId},
    parser::OpenDeckParseError,
    ChannelOrAll, MessageStatus, Section,
};

impl TryFrom<Section> for EncoderSection {
    type Error = OpenDeckParseError;
    fn try_from(x: Section) -> Result<Self, Self::Error> {
        if let Ok(id) = EncoderSectionId::try_from(x.id) {
            match id {
                EncoderSectionId::InvertState => Ok(EncoderSection::InvertState(x.value > 0)),
                EncoderSectionId::RemoteSync => Ok(EncoderSection::RemoteSync(x.value > 0)),
                EncoderSectionId::Enabled => Ok(EncoderSection::Enabled(x.value > 0)),
                EncoderSectionId::MessageType => EncoderMessageType::try_from(x.value)
                    .map(EncoderSection::MessageType)
                    .map_err(OpenDeckParseError::new_value_err),
                EncoderSectionId::Channel => {
                    Ok(EncoderSection::Channel(ChannelOrAll::from(x.value)))
                }
                EncoderSectionId::Accelleration => Accelleration::try_from(x.value)
                    .map(EncoderSection::Accelleration)
                    .map_err(OpenDeckParseError::new_value_err),
                EncoderSectionId::PulsesPerStep => Ok(EncoderSection::PulsesPerStep(x.value as u8)),
                EncoderSectionId::MidiIdLSB => Ok(EncoderSection::MidiIdLSB(x.value)),
                EncoderSectionId::MidiIdMSB => Ok(EncoderSection::MidiIdMSB(x.value as u8)),
                EncoderSectionId::LowerLimit => Ok(EncoderSection::LowerLimit(x.value)),
                EncoderSectionId::UpperLimit => Ok(EncoderSection::UpperLimit(x.value)),
                EncoderSectionId::RepeatedValue => Ok(EncoderSection::RepeatedValue(x.value)),
                EncoderSectionId::SecondMidiId => Ok(EncoderSection::SecondMidiId(x.value)),
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
        let result = EncoderSection::try_from(Section {
            id: 0x00,
            value: 0x01,
        });
        assert_eq!(result, Ok(EncoderSection::Enabled(true)));
    }

    #[test]
    fn test_message_type_value_error() {
        let result = EncoderSection::try_from(Section {
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

    #[test]
    fn test_acceleration_type_value_error() {
        let result = EncoderSection::try_from(Section {
            id: 0x06,
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
