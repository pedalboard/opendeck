use crate::{
    global::{GlobalSection, GlobalSectionId},
    Section,
};

impl From<GlobalSection> for (u16, Section) {
    fn from(section: GlobalSection) -> (u16, Section) {
        match section {
            GlobalSection::Midi(index, value) => (
                index,
                Section {
                    id: GlobalSectionId::Midi.into(),
                    value,
                },
            ),
            GlobalSection::Presets(index, value) => (
                index.into(),
                Section {
                    id: GlobalSectionId::Presets.into(),
                    value,
                },
            ),
        }
    }
}
