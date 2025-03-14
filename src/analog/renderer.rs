use crate::{
    analog::{AnalogSection, AnalogSectionId},
    Section,
};

impl From<AnalogSection> for Section {
    fn from(s: AnalogSection) -> Section {
        match s {
            AnalogSection::Enabled(value) => Section {
                id: AnalogSectionId::Enabled.into(),
                value: value as u16,
            },
            AnalogSection::Inverted(value) => Section {
                id: AnalogSectionId::InvertState.into(),
                value: value as u16,
            },
            AnalogSection::MessageType(value) => Section {
                id: AnalogSectionId::MessageType.into(),
                value: value.into(),
            },
            AnalogSection::MidiId(value) => Section {
                id: AnalogSectionId::MidiIdLSB.into(),
                value,
            },
            AnalogSection::LowerCCLimit(value) => Section {
                id: AnalogSectionId::LowerCCLimitLSB.into(),
                value,
            },
            AnalogSection::UpperCCLimit(value) => Section {
                id: AnalogSectionId::UpperCCLimitLSB.into(),
                value,
            },
            AnalogSection::Channel(value) => Section {
                id: AnalogSectionId::Channel.into(),
                value: value.into(),
            },
            AnalogSection::LowerADCOffset(value) => Section {
                id: AnalogSectionId::LowerADCOffset.into(),
                value: value as u16,
            },
            AnalogSection::UpperADCOffset(value) => Section {
                id: AnalogSectionId::UpperADCOffset.into(),
                value: value as u16,
            },
        }
    }
}
