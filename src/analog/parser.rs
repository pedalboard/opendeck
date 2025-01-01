use crate::{
    analog::{AnalogMessageType, AnalogSection, AnalogSectionId},
    parser::OpenDeckParseError,
    ChannelOrAll, MessageStatus, Section,
};
use midi_types::Value14;

impl TryFrom<Section> for AnalogSection {
    type Error = OpenDeckParseError;
    fn try_from(v: Section) -> Result<Self, Self::Error> {
        if let Ok(id) = AnalogSectionId::try_from(v.id) {
            match id {
                AnalogSectionId::Enabled => Ok(AnalogSection::Enabled(v.value > 0)),
                AnalogSectionId::InvertState => Ok(AnalogSection::InvertState(v.value > 0)),
                AnalogSectionId::MessageType => {
                    if let Ok(mt) = AnalogMessageType::try_from(v.value) {
                        Ok(AnalogSection::MessageType(mt))
                    } else {
                        Err(OpenDeckParseError::StatusError(
                            MessageStatus::NewValueError,
                        ))
                    }
                }
                AnalogSectionId::MidiIdLSB => Ok(AnalogSection::MidiId(Value14::from(v.value))),
                AnalogSectionId::LowerCCLimitLSB => {
                    Ok(AnalogSection::LowerCCLimit(Value14::from(v.value)))
                }
                AnalogSectionId::UpperCCLimitLSB => {
                    Ok(AnalogSection::UpperCCLimit(Value14::from(v.value)))
                }
                AnalogSectionId::Channel => Ok(AnalogSection::Channel(ChannelOrAll::from(v.value))),
                AnalogSectionId::LowerADCOffset => Ok(AnalogSection::LowerADCOffset(v.value as u8)),
                AnalogSectionId::UpperADCOffset => Ok(AnalogSection::UpperADCOffset(v.value as u8)),
            }
        } else {
            Err(OpenDeckParseError::StatusError(MessageStatus::SectionError))
        }
    }
}
