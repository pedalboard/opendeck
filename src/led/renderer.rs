use crate::{
    led::{LedSection, LedSectionId},
    Section,
};

impl From<LedSection> for Section {
    fn from(s: LedSection) -> Section {
        match s {
            LedSection::BlinkTesting(v) => Section {
                id: LedSectionId::BlinkTesting.into(),
                value: v.into(),
            },
            LedSection::ColorTesting(v) => Section {
                id: LedSectionId::ColorTesting.into(),
                value: v.into(),
            },
            LedSection::ControlType(v) => Section {
                id: LedSectionId::ControlType.into(),
                value: v.into(),
            },
            LedSection::RGBEnabled(v) => Section {
                id: LedSectionId::RGBEnabled.into(),
                value: v.into(),
            },
            LedSection::Global(v) => Section {
                id: LedSectionId::Global.into(),
                value: v,
            },
            LedSection::ActivationId(v) => Section {
                id: LedSectionId::ActivationId.into(),
                value: v.into(),
            },
            LedSection::ActivationValue(v) => Section {
                id: LedSectionId::ActivationValue.into(),
                value: v.into(),
            },
            LedSection::Channel(v) => Section {
                id: LedSectionId::Channel.into(),
                value: v.into(),
            },
        }
    }
}
