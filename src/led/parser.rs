use crate::{
    led::{LedSection, LedSectionId},
    parser::OpenDeckParseError,
    ChannelOrAll, MessageStatus, Section,
};

use super::ControlType;

impl TryFrom<Section> for LedSection {
    type Error = OpenDeckParseError;
    fn try_from(v: Section) -> Result<Self, Self::Error> {
        if let Ok(id) = LedSectionId::try_from(v.id) {
            match id {
                LedSectionId::ActivationId => Ok(LedSection::ActivationId(v.value as u8)),
                LedSectionId::ActivationValue => Ok(LedSection::ActivationValue(v.value as u8)),
                LedSectionId::State => Ok(LedSection::State(v.value > 0)),
                LedSectionId::ControlType => ControlType::try_from(v.value)
                    .map(LedSection::ControlType)
                    .map_err(OpenDeckParseError::new_value_err),
                LedSectionId::Reserved => Ok(LedSection::Reserved(v.value)),
                LedSectionId::BlinkTesting => Ok(LedSection::BlinkTesting(v.value > 0)),
                LedSectionId::Channel => Ok(LedSection::Channel(ChannelOrAll::from(v.value))),
                LedSectionId::Global => Ok(LedSection::Global(v.value)),
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
        let section = Section { id: 4, value: 1 };
        let result = LedSection::try_from(section);
        assert_eq!(result, Ok(LedSection::Reserved(1)));
    }

    #[test]
    fn test_control_type_value_error() {
        let result = LedSection::try_from(Section { id: 5, value: 55 });
        assert_eq!(
            result,
            Err(OpenDeckParseError::StatusError(
                MessageStatus::NewValueError
            ))
        );
    }

    #[test]
    fn test_state_section() {
        let result = LedSection::try_from(Section { id: 0, value: 1 });
        assert_eq!(result, Ok(LedSection::State(true)));
        let result = LedSection::try_from(Section { id: 0, value: 0 });
        assert_eq!(result, Ok(LedSection::State(false)));
    }
}
