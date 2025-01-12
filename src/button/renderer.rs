use crate::{
    button::{ButtonSection, ButtonSectionId},
    Section,
};

impl From<ButtonSection> for Section {
    fn from(s: ButtonSection) -> Section {
        match s {
            ButtonSection::Type(t) => Section {
                id: ButtonSectionId::Type.into(),
                value: t.into(),
            },
            ButtonSection::MessageType(t) => Section {
                id: ButtonSectionId::MessageType.into(),
                value: t.into(),
            },
            ButtonSection::MidiId(v) => Section {
                id: ButtonSectionId::MidiId.into(),
                value: v.into(),
            },
            ButtonSection::Value(v) => Section {
                id: ButtonSectionId::Value.into(),
                value: v.into(),
            },
            ButtonSection::Channel(v) => Section {
                id: ButtonSectionId::Channel.into(),
                value: v.into(),
            },
        }
    }
}
