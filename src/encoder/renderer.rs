use crate::{
    encoder::{EncoderSection, EncoderSectionId},
    Section,
};

impl From<EncoderSection> for Section {
    fn from(s: EncoderSection) -> Section {
        match s {
            EncoderSection::Enabled(value) => Section {
                id: EncoderSectionId::Enabled.into(),
                value: value as u16,
            },
            EncoderSection::RemoteSync(value) => Section {
                id: EncoderSectionId::RemoteSync.into(),
                value: value as u16,
            },
            EncoderSection::InvertState(value) => Section {
                id: EncoderSectionId::InvertState.into(),
                value: value as u16,
            },
            EncoderSection::Channel(value) => Section {
                id: EncoderSectionId::Channel.into(),
                value: value.into(),
            },
            EncoderSection::MessageType(value) => Section {
                id: EncoderSectionId::MessageType.into(),
                value: value as u16,
            },
            EncoderSection::Accelleration(value) => Section {
                id: EncoderSectionId::Accelleration.into(),
                value: value as u16,
            },
            EncoderSection::PulsesPerStep(value) => Section {
                id: EncoderSectionId::PulsesPerStep.into(),
                value: value as u16,
            },
            EncoderSection::MidiIdLSB(v) => Section {
                id: EncoderSectionId::MidiIdLSB.into(),
                value: v.into(),
            },
            EncoderSection::MidiIdMSB(v) => {
                let value: u8 = v.into();
                Section {
                    id: EncoderSectionId::MidiIdMSB.into(),
                    value: value as u16,
                }
            }
            EncoderSection::LowerLimit(v) => Section {
                id: EncoderSectionId::LowerLimit.into(),
                value: v.into(),
            },
            EncoderSection::UpperLimit(v) => Section {
                id: EncoderSectionId::UpperLimit.into(),
                value: v.into(),
            },
            EncoderSection::RepeatedValue(v) => Section {
                id: EncoderSectionId::RepeatedValue.into(),
                value: v.into(),
            },
            EncoderSection::SecondMidiId(v) => Section {
                id: EncoderSectionId::SecondMidiId.into(),
                value: v.into(),
            },
        }
    }
}
