use int_enum::IntEnum;

pub mod parser;
pub mod renderer;

#[derive(IntEnum)]
#[repr(u8)]
pub enum GlobalSectionId {
    Midi = 0,
    // Reserved = 1,
    Presets = 2,
}

#[derive(Debug, Clone, PartialEq, IntEnum, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u16)]
pub enum PresetIndex {
    Active = 0,
    Preservation = 1,
    ForceValueRefresh = 2,
    EnableMideChange = 3,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum GlobalSection {
    Midi(u16, u16),
    Presets(PresetIndex, u16),
}
